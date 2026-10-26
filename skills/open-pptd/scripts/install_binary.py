#!/usr/bin/env python3
"""Standalone installer for the `pptd` native PPTD compiler binary.

Downloads a pinned pre-compiled binary from GitHub Releases for macOS (arm64 & x86_64),
Linux (x86_64 & aarch64), or Windows (x86_64), and refuses to install it unless the
archive SHA-256 matches the release's `SHA256SUMS.txt`. When the pinned archive or its
checksum is unavailable, the installer falls back to `cargo install --tag v<version>`
so the source side is pinned to the same version as the binaries.
"""

from __future__ import annotations

import argparse
import hashlib
import io
import os
import platform
import re
import shutil
import stat
import subprocess
import sys
import tarfile
import urllib.error
import urllib.request
import zipfile
from pathlib import Path
from typing import Optional, Tuple

REPO = "shiquda/open-pptd-skill"
REPO_CANDIDATES = ["shiquda/open-pptd-skill"]
VERSION = "1.4.2"


def detect_target(version: str = VERSION) -> Optional[Tuple[str, list[str]]]:
    """Return (target_triple, candidate_archive_names)."""
    system = platform.system().lower()
    machine = platform.machine().lower()

    if system == "darwin":
        if machine in ("arm64", "aarch64"):
            return (
                "aarch64-apple-darwin",
                [f"pptd-{version}-aarch64-macos.tar.gz", "pptd-aarch64-macos.tar.gz"],
            )
        elif machine in ("x86_64", "amd64"):
            return (
                "x86_64-apple-darwin",
                [f"pptd-{version}-x86_64-macos.tar.gz", "pptd-x86_64-macos.tar.gz"],
            )
    elif system == "linux":
        if machine in ("x86_64", "amd64"):
            return (
                "x86_64-unknown-linux-gnu",
                [f"pptd-{version}-x86_64-linux.tar.gz", "pptd-x86_64-linux.tar.gz"],
            )
        elif machine in ("aarch64", "arm64"):
            return (
                "aarch64-unknown-linux-gnu",
                [f"pptd-{version}-aarch64-linux.tar.gz", "pptd-aarch64-linux.tar.gz"],
            )
    elif system == "windows":
        if machine in ("x86_64", "amd64"):
            return (
                "x86_64-pc-windows-msvc",
                [f"pptd-{version}-x86_64-windows.zip", "pptd-x86_64-windows.zip"],
            )

    return None


def parse_version(v_str: str) -> Optional[tuple[int, ...]]:
    """Extract semantic version digits from version output string (e.g. 'pptd 1.3.5' -> (1, 3, 5))."""
    if not v_str:
        return None
    m = re.search(r"(\d+(?:\.\d+)+)", v_str)
    if not m:
        return None
    try:
        return tuple(int(part) for part in m.group(1).split("."))
    except ValueError:
        return None


def compare_versions(v1: str, v2: str) -> Optional[int]:
    """Compare two version strings.

    Returns:
        1 if v1 > v2
        0 if v1 == v2
        -1 if v1 < v2
        None if either version is unparseable
    """
    t1 = parse_version(v1)
    t2 = parse_version(v2)
    if t1 is None or t2 is None:
        return None

    max_len = max(len(t1), len(t2))
    t1_padded = t1 + (0,) * (max_len - len(t1))
    t2_padded = t2 + (0,) * (max_len - len(t2))

    if t1_padded > t2_padded:
        return 1
    elif t1_padded < t2_padded:
        return -1
    else:
        return 0


def is_version_outdated(current: str, target: str) -> bool:
    """Return True if current version is older than target, or unparseable."""
    cmp = compare_versions(current, target)
    if cmp is None:
        return True
    return cmp < 0


def probe_binary_version(bin_path: Path) -> Optional[str]:
    """Run `<bin_path> --version` and return its stripped stdout, or None on error."""
    try:
        proc = subprocess.run(
            [str(bin_path), "--version"],
            capture_output=True,
            text=True,
            timeout=5,
        )
        if proc.returncode == 0:
            return proc.stdout.strip()
    except Exception:
        pass
    return None


def find_pptd(min_version: Optional[str] = None) -> Optional[Path]:
    """Find existing pptd executable in PATH or standard user directories that meets min_version."""
    names = ["pptd", "open-pptd", "slideforge"]
    candidates = []

    for bin_name in names:
        found = shutil.which(bin_name)
        if found:
            p = Path(found).resolve()
            if p not in candidates:
                candidates.append(p)

    home = Path.home()
    for name in names:
        candidates.extend([
            home / ".local" / "bin" / name,
            home / ".cargo" / "bin" / name,
            Path(f"/usr/local/bin/{name}"),
            Path(f"/opt/homebrew/bin/{name}"),
            Path(__file__).resolve().parent.parent / "bin" / name,
            Path.cwd() / "target" / "release" / name,
            Path.cwd() / "open-pptd-skill" / "target" / "release" / name,
        ])
    if platform.system().lower() == "windows":
        for name in names:
            candidates.extend([
                home / "AppData" / "Local" / "Programs" / "pptd" / f"{name}.exe",
                home / ".cargo" / "bin" / f"{name}.exe",
                home / ".local" / "bin" / f"{name}.exe",
                Path.cwd() / "target" / "release" / f"{name}.exe",
            ])

    for c in candidates:
        if c.is_file() and (platform.system().lower() == "windows" or os.access(c, os.X_OK)):
            p = c.resolve()
            if min_version:
                v_out = probe_binary_version(p)
                if not v_out or is_version_outdated(v_out, min_version):
                    continue
            return p

    return None


find_slideforge = find_pptd

def fetch_checksums(version: str) -> dict[str, str]:
    """Fetch SHA256SUMS.txt from candidate GitHub releases; empty when unavailable."""
    for repo in REPO_CANDIDATES:
        sums_url = f"https://github.com/{repo}/releases/download/v{version}/SHA256SUMS.txt"
        req = urllib.request.Request(
            sums_url,
            headers={"User-Agent": "open-pptd-installer/1.0"},
        )
        try:
            with urllib.request.urlopen(req, timeout=10) as resp:
                text = resp.read().decode("utf-8", errors="replace")
                checksums = {}
                for line in text.splitlines():
                    parts = line.strip().split()
                    if len(parts) >= 2:
                        h, name = parts[0], parts[1].lstrip("*")
                        checksums[name] = h
                if checksums:
                    return checksums
        except Exception:
            continue
    return {}


def install_from_release(dest_dir: Path, version: str = VERSION) -> Path:
    target_info = detect_target(version)
    if not target_info:
        raise RuntimeError(
            f"Unsupported operating system or CPU architecture: system='{platform.system()}', machine='{platform.machine()}'.\n"
            f"Please compile pptd from source using: cargo install --git https://github.com/{REPO}.git"
        )

    target, candidate_archives = target_info
    data = None
    last_err = None
    chosen_archive = None

    for repo in REPO_CANDIDATES:
        for archive_name in candidate_archives:
            url = f"https://github.com/{repo}/releases/download/v{version}/{archive_name}"
            req = urllib.request.Request(
                url,
                headers={"User-Agent": "open-pptd-installer/1.0"},
            )
            try:
                print(f"[open-pptd] Attempting to download prebuilt binary from {repo} ({archive_name})...")
                with urllib.request.urlopen(req, timeout=30) as resp:
                    data = resp.read()
                    chosen_archive = archive_name
                    print(f"[open-pptd] Successfully downloaded from {url}")
                    break
            except Exception as err:
                last_err = err
                continue
        if data is not None:
            break

    if data is None or chosen_archive is None:
        raise RuntimeError(
            f"Failed to download prebuilt binary for v{version} from candidate repositories {REPO_CANDIDATES}: {last_err}"
        )

    # SHA256 verification is mandatory: an archive we cannot verify is never installed.
    expected_sums = fetch_checksums(version)
    expected_sha = expected_sums.get(chosen_archive)
    if not expected_sha:
        raise RuntimeError(
            f"v{version} has no SHA256SUMS.txt entry for '{chosen_archive}'; refusing to install an unverified binary. "
            f"Install from pinned source instead: cargo install --git https://github.com/{REPO}.git --tag v{version}"
        )
    actual_sha = hashlib.sha256(data).hexdigest()
    if actual_sha.lower() != expected_sha.lower():
        raise RuntimeError(
            f"SHA256 checksum mismatch for {chosen_archive}!\n"
            f"  Expected: {expected_sha}\n"
            f"  Actual:   {actual_sha}\n"
            f"The downloaded archive may be corrupted or compromised."
        )
    print(f"[open-pptd] SHA256 verified for {chosen_archive}: {actual_sha}")

    dest_dir.mkdir(parents=True, exist_ok=True)
    is_win = platform.system().lower() == "windows"
    target_bin_name = "pptd.exe" if is_win else "pptd"
    dest_path = dest_dir / target_bin_name

    extracted = False
    valid_names = {"pptd", "pptd.exe", "open-pptd", "open-pptd.exe"}
    if chosen_archive.endswith(".tar.gz") or chosen_archive.endswith(".tgz"):
        with tarfile.open(fileobj=io.BytesIO(data), mode="r:gz") as tar:
            for member in tar.getmembers():
                fname = Path(member.name).name
                if fname in valid_names:
                    f = tar.extractfile(member)
                    if f:
                        dest_path.write_bytes(f.read())
                        extracted = True
                        break
    elif chosen_archive.endswith(".zip"):
        with zipfile.ZipFile(io.BytesIO(data)) as z:
            for name in z.namelist():
                fname = Path(name).name
                if fname in valid_names:
                    dest_path.write_bytes(z.read(name))
                    extracted = True
                    break

    if not extracted or not dest_path.is_file():
        raise RuntimeError(f"Downloaded archive '{chosen_archive}' did not contain a valid compiler executable.")

    dest_path.chmod(dest_path.stat().st_mode | stat.S_IXUSR | stat.S_IXGRP | stat.S_IXOTH)

    # Create open-pptd symlink/copy if needed
    open_pptd_bin = dest_dir / ("open-pptd.exe" if is_win else "open-pptd")
    if not open_pptd_bin.exists():
        try:
            open_pptd_bin.symlink_to(dest_path.name)
        except Exception:
            try:
                shutil.copy2(dest_path, open_pptd_bin)
            except Exception:
                pass
    print(f"[open-pptd] Installed pptd v{version} from {chosen_archive} (SHA256 {actual_sha}) to: {dest_path}")
    return dest_path


def install_from_cargo(dest_dir: Path, version: str = VERSION) -> Path:
    print("[open-pptd] Attempting fallback compilation via cargo...", file=sys.stderr)
    cargo_bin = shutil.which("cargo")
    if not cargo_bin:
        raise RuntimeError("`cargo` is not installed on this machine, cannot build from source.")

    dest_dir.mkdir(parents=True, exist_ok=True)
    install_root = dest_dir.parent if dest_dir.name == "bin" else dest_dir
    cmd = [
        "cargo",
        "install",
        "--git",
        f"https://github.com/{REPO}.git",
        "--tag",
        f"v{version}",
        "--root",
        str(install_root),
        "--force",
    ]
    print(f"[open-pptd] Running: {' '.join(cmd)}")
    subprocess.run(cmd, check=True)

    bin_name = "pptd.exe" if platform.system().lower() == "windows" else "pptd"
    installed = (install_root / "bin" / bin_name) if dest_dir.name == "bin" else (dest_dir / bin_name)
    if not installed.is_file():
        # Fallback check
        installed = (install_root / "bin" / "open-pptd") if dest_dir.name == "bin" else (dest_dir / "open-pptd")

    if not installed.is_file():
        raise RuntimeError(f"Cargo build finished but binary not found at {installed}")

    installed.chmod(installed.stat().st_mode | stat.S_IXUSR | stat.S_IXGRP | stat.S_IXOTH)
    print(f"[open-pptd] Successfully built and installed pptd via cargo: {installed}")
    return installed


def print_manual_install_instructions(dest_dir: Path) -> None:
    bin_name = "pptd.exe" if platform.system().lower() == "windows" else "pptd"
    target_path = dest_dir / bin_name
    print("\n" + "=" * 70, file=sys.stderr)
    print("PPTD MANUAL INSTALLATION GUIDE", file=sys.stderr)
    print("=" * 70, file=sys.stderr)
    print("Automatic installation could not be completed. You can install pptd manually:", file=sys.stderr)
    print(f"\n1. Precompiled binary from GitHub Releases:\n   https://github.com/{REPO}/releases", file=sys.stderr)
    print(f"   Download the archive for your OS, extract '{bin_name}', and place it at:\n   {target_path}", file=sys.stderr)
    print(f"\n2. Or compile with Rust / Cargo:\n   cargo install --git https://github.com/{REPO}.git", file=sys.stderr)
    print(f"\n3. Ensure {dest_dir} is in your system PATH:\n   export PATH=\"{dest_dir}:$PATH\"", file=sys.stderr)
    print("=" * 70 + "\n", file=sys.stderr)


def ensure_pptd(force_reinstall: bool = False, install_dir: Optional[Path] = None, version: str = VERSION) -> Path:
    bin_name = "pptd.exe" if platform.system().lower() == "windows" else "pptd"

    if not force_reinstall:
        if install_dir is not None:
            target_bin = install_dir / bin_name
            if target_bin.is_file() and (platform.system().lower() == "windows" or os.access(target_bin, os.X_OK)):
                v_out = probe_binary_version(target_bin)
                if v_out and not is_version_outdated(v_out, version):
                    return target_bin.resolve()
        else:
            existing = find_pptd(min_version=version)
            if existing:
                return existing

    if install_dir is None:
        install_dir = Path.home() / ".local" / "bin"

    errors = []
    # 1. Try download prebuilt release
    try:
        installed = install_from_release(install_dir, version=version)
        v_out = probe_binary_version(installed)
        if not v_out or is_version_outdated(v_out, version):
            raise RuntimeError(f"Installed binary reported '{v_out}', expected version {version}")
        return installed
    except Exception as e:
        errors.append(f"Release download: {e}")
        print(f"[open-pptd] Precompiled binary download failed: {e}", file=sys.stderr)

    # 2. Try cargo fallback
    try:
        installed = install_from_cargo(install_dir, version=version)
        v_out = probe_binary_version(installed)
        if not v_out or is_version_outdated(v_out, version):
            raise RuntimeError(f"Cargo compiled binary reported '{v_out}', expected version {version}")
        return installed
    except Exception as e:
        errors.append(f"Cargo build: {e}")
        print(f"[open-pptd] Cargo build fallback failed: {e}", file=sys.stderr)

    # 3. Print instructions and fail cleanly
    print_manual_install_instructions(install_dir)
    raise RuntimeError(
        "Could not automatically install pptd:\n  - " + "\n  - ".join(errors)
    )


ensure_slideforge = ensure_pptd


def main() -> int:
    parser = argparse.ArgumentParser(description="Install pptd native PPTD compiler binary")
    parser.add_argument(
        "--dir",
        type=Path,
        default=Path.home() / ".local" / "bin",
        help="Destination directory (default: ~/.local/bin)",
    )
    parser.add_argument(
        "--force",
        action="store_true",
        help="Force re-download even if already installed",
    )
    parser.add_argument(
        "--version",
        type=str,
        default=VERSION,
        help=f"Specific pptd version to install (default: {VERSION})",
    )
    args = parser.parse_args()

    try:
        bin_path = ensure_pptd(force_reinstall=args.force, install_dir=args.dir, version=args.version)
        version_str = probe_binary_version(bin_path) or "unknown"
        if is_version_outdated(version_str, args.version):
            print(
                f"[open-pptd] Error: Resolved binary at {bin_path} reports '{version_str}', which is older than target version {args.version}",
                file=sys.stderr,
            )
            return 1

        print(f"[open-pptd] Success! {version_str} is ready at: {bin_path}")

        path_env = os.environ.get("PATH", "")
        path_dirs = [Path(p).resolve() for p in path_env.split(os.pathsep) if p.strip()]
        bin_dir = bin_path.parent.resolve()
        which_pptd = shutil.which("pptd")
        if which_pptd and Path(which_pptd).resolve() != bin_path.resolve():
            print("\n" + "!" * 68)
            print(f"[open-pptd] NOTICE: '{bin_path}' is shadowed by '{which_pptd}' in your PATH.")
            print(f"[open-pptd] To ensure 'pptd' resolves to the newly installed binary, run:")
            print(f"    export PATH=\"{bin_dir}:$PATH\"")
            print("!" * 68 + "\n")
        elif bin_dir not in path_dirs:
            print("\n" + "!" * 68)
            print(f"[open-pptd] NOTICE: '{bin_dir}' is not in your current PATH environment.")
            print(f"[open-pptd] To invoke 'pptd' directly from anywhere, run:")
            print(f"    export PATH=\"{bin_dir}:$PATH\"")
            print(f"[open-pptd] Or add that line to your ~/.bashrc / ~/.zshrc.")
            print(f"[open-pptd] You can also invoke the compiler using its full path:")
            print(f"    {bin_path} --help")
            print("!" * 68 + "\n")
        return 0
    except Exception as err:
        print(f"[open-pptd] Installation error: {err}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
