#!/usr/bin/env python3
"""Offline image export and multi-page preview montage for PPTD projects.

Uses the native pptd compiler + headless document renderer to generate
high-resolution PNG images of every slide and a unified grid overview,
without requiring any browser automation, token authentication, or cloud services.
"""

from __future__ import annotations

import argparse
import os
import re
import shutil
import subprocess
import sys
import tempfile
import time
from pathlib import Path
from typing import Optional, Sequence

from export_pptx import export_pptx


def build_contact_sheet(image_paths: list[Path], output_path: Path, cols: int = 5) -> Optional[Path]:
    """Generate a multi-page overview contact sheet grid using Pillow."""
    if not image_paths:
        return None
    try:
        from PIL import Image
    except ImportError:
        print("[open-pptd-skill] PIL/Pillow not installed; skipping overview contact sheet generation.", file=sys.stderr)
        return None

    try:
        first_im = Image.open(image_paths[0])
        w, h = first_im.size
        thumb_w = 400
        thumb_h = max(1, int(thumb_w * h / w))
        pad = 20

        n = len(image_paths)
        actual_cols = min(cols, n)
        rows = (n + actual_cols - 1) // actual_cols

        total_w = actual_cols * thumb_w + (actual_cols + 1) * pad
        total_h = rows * thumb_h + (rows + 1) * pad

        grid = Image.new("RGB", (total_w, total_h), "#EFECE1")

        for idx, img_p in enumerate(image_paths):
            im = Image.open(img_p).convert("RGB")
            im_resized = im.resize((thumb_w, thumb_h), Image.Resampling.LANCZOS)
            r = idx // actual_cols
            c = idx % actual_cols
            x = pad + c * (thumb_w + pad)
            y = pad + r * (thumb_h + pad)
            grid.paste(im_resized, (x, y))

        output_path.parent.mkdir(parents=True, exist_ok=True)
        grid.save(output_path, "JPEG", quality=90)
        print(f"[open-pptd-skill] Generated contact sheet: {output_path}")
        return output_path
    except Exception as e:
        print(f"[open-pptd-skill] Failed to generate contact sheet: {e}", file=sys.stderr)
        return None


build_montage = build_contact_sheet
def export_images(
    source: Path,
    out_dir: Optional[Path] = None,
    dpi: int = 150,
    force: bool = True,
    pptd_bin: Optional[Path] = None,
) -> Path:
    source = source.expanduser().resolve()
    if not source.exists():
        raise FileNotFoundError(f"Source does not exist: {source}")
    if out_dir is None:
        out_dir = (source.parent if source.is_file() else source) / "qa-images"
    out_dir = out_dir.expanduser().resolve()
    out_dir.mkdir(parents=True, exist_ok=True)

    tmp_dir_obj = tempfile.TemporaryDirectory(prefix="pptd_img_")
    try:
        tmp_dir = Path(tmp_dir_obj.name)
        tmp_pptx = tmp_dir / "render.pptx"

        # Isolate deck directory inside tmp_dir so .sync.hash is never written to user's deck directory
        from export_pptx import find_manifest
        manifest = find_manifest(source)
        source_dir = manifest.parent.resolve()
        tmp_deck_dir = tmp_dir / "deck"
        shutil.copytree(source_dir, tmp_deck_dir, ignore=shutil.ignore_patterns(".*", "*.pptx"))
        isolated_manifest = tmp_deck_dir / manifest.name

        export_pptx(isolated_manifest, output=tmp_pptx, force=force, pptd_bin=pptd_bin)
        soffice = shutil.which("soffice") or shutil.which("libreoffice")
        if not soffice and sys.platform == "darwin":
            mac_soffice = Path("/Applications/LibreOffice.app/Contents/MacOS/soffice")
            if mac_soffice.is_file():
                soffice = str(mac_soffice)

        pdftoppm = shutil.which("pdftoppm")

        if not soffice or not pdftoppm:
            print("[open-pptd-skill] Image rendering requires `soffice` (LibreOffice) and `pdftoppm` (poppler-utils).", file=sys.stderr)
            print("[open-pptd-skill] PPTX was built successfully, but PNG rasterization is unavailable in this environment.", file=sys.stderr)
            return out_dir

        # Convert PPTX to PDF inside temp dir
        subprocess.run(
            [soffice, "--headless", "--convert-to", "pdf", str(tmp_pptx), "--outdir", str(tmp_dir)],
            check=True,
            capture_output=True,
        )
        tmp_pdf = tmp_dir / "render.pdf"

        # Rasterize PDF to slide-NN.png directly into out_dir
        print(f"[open-pptd-skill] Rasterizing slides at {dpi} DPI...")
        prefix = out_dir / "slide"
        subprocess.run(
            [pdftoppm, "-png", "-r", str(dpi), str(tmp_pdf), str(prefix)],
            check=True,
            capture_output=True,
        )

        raw_slides = sorted(
            out_dir.glob("slide-*.png"),
            key=lambda p: int(re.search(r"\d+", p.stem).group(0)) if re.search(r"\d+", p.stem) else 0,
        )
        # Normalize filenames to slide-01.png, slide-02.png (matching slide-NN.png contract)
        pad_width = max(2, len(str(len(raw_slides))))
        rendered_pages: list[Path] = []
        for idx, p in enumerate(raw_slides, 1):
            target_name = out_dir / f"slide-{idx:0{pad_width}d}.png"
            if p != target_name:
                p.rename(target_name)
            rendered_pages.append(target_name)

        print(f"[open-pptd-skill] Successfully exported {len(rendered_pages)} slide images to: {out_dir}")

        # Build overview contact sheet
        contact_sheet_path = out_dir / "contact-sheet.jpg"
        build_contact_sheet(rendered_pages, contact_sheet_path)

    finally:
        try:
            tmp_dir_obj.cleanup()
        except Exception:
            pass
        if os.path.exists(tmp_dir_obj.name):
            shutil.rmtree(tmp_dir_obj.name, ignore_errors=True)

    return out_dir


def main(argv: Optional[Sequence[str]] = None) -> int:
    parser = argparse.ArgumentParser(description="Export a PPTD project to PNG images and contact sheet overview")
    parser.add_argument("source", type=Path, help="Path to deck.pptd manifest or project directory")
    parser.add_argument("-o", "--output", type=Path, default=None, help="Output directory for images (default: qa-images)")
    parser.add_argument("--dpi", type=int, default=150, help="Image DPI (default: 150)")
    parser.add_argument("--force", action="store_true", default=True, help="Force overwrite bypassing .sync.hash protection")
    parser.add_argument("--compiler", type=Path, default=None, help="Explicit path to pptd binary")
    args = parser.parse_args(argv)

    try:
        export_images(args.source, out_dir=args.output, dpi=args.dpi, force=args.force, pptd_bin=args.compiler)
        return 0
    except Exception as err:
        print(f"[open-pptd-skill] Error: {err}", file=sys.stderr)
        return 1

if __name__ == "__main__":
    sys.exit(main())
