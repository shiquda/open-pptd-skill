#!/usr/bin/env bash
set -euo pipefail

# pptd standalone installer for open-pptd
REPOS=("shiquda/open-pptd-skill")
REPO="${REPOS[0]}"
VERSION="1.4.0"
INSTALL_DIR="${HOME}/.local/bin"
INSTALL_DIR_SPECIFIED=0
FORCE=0

while [ $# -gt 0 ]; do
  case "$1" in
    --force|-f)
      FORCE=1
      shift
      ;;
    --dir=*)
      INSTALL_DIR="${1#*=}"
      INSTALL_DIR_SPECIFIED=1
      shift
      ;;
    --dir)
      INSTALL_DIR="$2"
      INSTALL_DIR_SPECIFIED=1
      shift 2
      ;;
    --version=*)
      VERSION="${1#*=}"
      shift
      ;;
    --version)
      VERSION="$2"
      shift 2
      ;;
    --help|-h)
      echo "Usage: $0 [--force] [--dir <path>] [--version <version>]"
      exit 0
      ;;
    *)
      echo "Unknown option: $1" >&2
      exit 1
      ;;
  esac
done

# Returns 0 (true) if $1 >= $2, 1 otherwise
version_ge() {
  local v1="$1"
  local v2="$2"
  if [ -z "$v1" ] || [ -z "$v2" ]; then
    return 1
  fi
  if [ "$v1" = "$v2" ]; then
    return 0
  fi
  local IFS=.
  local i
  local -a a1=($v1)
  local -a a2=($v2)
  local len=${#a1[@]}
  if [ ${#a2[@]} -gt $len ]; then
    len=${#a2[@]}
  fi
  for ((i=0; i<len; i++)); do
    local n1="${a1[i]:-0}"
    local n2="${a2[i]:-0}"
    n1="${n1%%[^0-9]*}"
    n2="${n2%%[^0-9]*}"
    n1="${n1:-0}"
    n2="${n2:-0}"
    if [ "$n1" -gt "$n2" ]; then
      return 0
    elif [ "$n1" -lt "$n2" ]; then
      return 1
    fi
  done
  return 0
}

probe_version() {
  local bin="$1"
  if [ -x "$bin" ]; then
    "$bin" --version 2>/dev/null | grep -oE '[0-9]+(\.[0-9]+)+' | head -n 1 || true
  fi
}

# If already installed and not forced, verify version
if [ "$FORCE" -eq 0 ]; then
  if [ "$INSTALL_DIR_SPECIFIED" -eq 1 ]; then
    if [ -x "${INSTALL_DIR}/pptd" ]; then
      CUR_VER="$(probe_version "${INSTALL_DIR}/pptd")"
      if [ -n "$CUR_VER" ] && version_ge "$CUR_VER" "$VERSION"; then
        echo "[open-pptd] Success! pptd $CUR_VER is already installed at: ${INSTALL_DIR}/pptd"
        "${INSTALL_DIR}/pptd" --version
        exit 0
      fi
    fi
  else
    if [ -x "${INSTALL_DIR}/pptd" ]; then
      CUR_VER="$(probe_version "${INSTALL_DIR}/pptd")"
      if [ -n "$CUR_VER" ] && version_ge "$CUR_VER" "$VERSION"; then
        echo "[open-pptd] Success! pptd $CUR_VER is already installed at: ${INSTALL_DIR}/pptd"
        "${INSTALL_DIR}/pptd" --version
        exit 0
      fi
    fi
    if command -v pptd >/dev/null 2>&1; then
      PATH_BIN="$(command -v pptd)"
      CUR_VER="$(probe_version "${PATH_BIN}")"
      if [ -n "$CUR_VER" ] && version_ge "$CUR_VER" "$VERSION"; then
        echo "[open-pptd] Success! pptd $CUR_VER is already installed at: ${PATH_BIN}"
        "${PATH_BIN}" --version
        exit 0
      fi
    fi
  fi
fi
mkdir -p "${INSTALL_DIR}"

OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
ARCH="$(uname -m | tr '[:upper:]' '[:lower:]')"

case "${OS}" in
  darwin)
    case "${ARCH}" in
      arm64|aarch64) ARCHIVES=("pptd-${VERSION}-aarch64-macos.tar.gz" "pptd-aarch64-macos.tar.gz") ;;
      x86_64|amd64)   ARCHIVES=("pptd-${VERSION}-x86_64-macos.tar.gz" "pptd-x86_64-macos.tar.gz") ;;
      *) echo "[open-pptd] Unsupported macOS architecture: ${ARCH}" >&2; exit 1 ;;
    esac
    ;;
  linux)
    case "${ARCH}" in
      x86_64|amd64)   ARCHIVES=("pptd-${VERSION}-x86_64-linux.tar.gz" "pptd-x86_64-linux.tar.gz") ;;
      arm64|aarch64) ARCHIVES=("pptd-${VERSION}-aarch64-linux.tar.gz" "pptd-aarch64-linux.tar.gz") ;;
      *) echo "[open-pptd] Unsupported Linux architecture: ${ARCH}" >&2; exit 1 ;;
    esac
    ;;
  *)
    echo "[open-pptd] Unsupported OS: ${OS}. Please build with cargo install --git https://github.com/${REPO}.git" >&2
    exit 1
    ;;
esac

TMPDIR="$(mktemp -d)"
trap 'rm -rf "${TMPDIR}"' EXIT

DOWNLOAD_SUCCESS=0
CHOSEN_ARCHIVE=""

for repo_candidate in "${REPOS[@]}"; do
  for archive_candidate in "${ARCHIVES[@]}"; do
    URL="https://github.com/${repo_candidate}/releases/download/v${VERSION}/${archive_candidate}"
    SUMS_URL="https://github.com/${repo_candidate}/releases/download/v${VERSION}/SHA256SUMS.txt"
    echo "[open-pptd] Attempting to download from ${repo_candidate} (${archive_candidate})..."
    if curl -fSL --connect-timeout 15 --max-time 60 "${URL}" -o "${TMPDIR}/${archive_candidate}" 2>/dev/null; then
      DOWNLOAD_SUCCESS=1
      REPO="${repo_candidate}"
      CHOSEN_ARCHIVE="${archive_candidate}"
      echo "[open-pptd] Download complete from ${repo_candidate}. Verifying checksum..."
      
      if curl -fSL --connect-timeout 10 --max-time 30 "${SUMS_URL}" -o "${TMPDIR}/SHA256SUMS.txt" 2>/dev/null; then
        EXPECTED_SHA="$(grep "${archive_candidate}" "${TMPDIR}/SHA256SUMS.txt" | awk '{print $1}' || true)"
        if [ -n "${EXPECTED_SHA}" ]; then
          if command -v sha256sum >/dev/null 2>&1; then
            ACTUAL_SHA="$(sha256sum "${TMPDIR}/${archive_candidate}" | awk '{print $1}')"
          elif command -v shasum >/dev/null 2>&1; then
            ACTUAL_SHA="$(shasum -a 256 "${TMPDIR}/${archive_candidate}" | awk '{print $1}')"
          else
            ACTUAL_SHA=""
          fi

          if [ -n "${ACTUAL_SHA}" ] && [ "${ACTUAL_SHA}" != "${EXPECTED_SHA}" ]; then
            echo "[open-pptd] Checksum verification failed for ${repo_candidate}!" >&2
            DOWNLOAD_SUCCESS=0
            continue
          else
            echo "[open-pptd] Checksum verified: ${EXPECTED_SHA:0:12}..."
          fi
        fi
      fi
      break 2
    fi
  done
done

if [ "$DOWNLOAD_SUCCESS" -eq 1 ]; then
  tar -xzf "${TMPDIR}/${CHOSEN_ARCHIVE}" -C "${TMPDIR}"
  BIN_FOUND=""
  for b in "pptd" "open-pptd"; do
    if [ -f "${TMPDIR}/${b}" ]; then
      BIN_FOUND="${TMPDIR}/${b}"
      break
    fi
  done

  if [ -n "${BIN_FOUND}" ]; then
    cp "${BIN_FOUND}" "${INSTALL_DIR}/pptd"
    chmod +x "${INSTALL_DIR}/pptd"
    # Also create slideforge alias if not already present
    if [ ! -e "${INSTALL_DIR}/slideforge" ]; then
      ln -sf "pptd" "${INSTALL_DIR}/slideforge" 2>/dev/null || cp "${INSTALL_DIR}/pptd" "${INSTALL_DIR}/slideforge"
    fi
    INSTALLED_VER="$(probe_version "${INSTALL_DIR}/pptd")"
    if [ -z "$INSTALLED_VER" ] || ! version_ge "$INSTALLED_VER" "$VERSION"; then
      echo "[open-pptd] Error: Installed binary at ${INSTALL_DIR}/pptd reported '${INSTALLED_VER:-unknown}', expected >= ${VERSION}" >&2
      exit 1
    fi
    echo "[open-pptd] Success! pptd ${INSTALLED_VER} is ready at: ${INSTALL_DIR}/pptd"
    "${INSTALL_DIR}/pptd" --version

    WHICH_PPTD="$(command -v pptd 2>/dev/null || true)"
    if [ -n "${WHICH_PPTD}" ] && [ "$(cd "$(dirname "${WHICH_PPTD}")" && pwd)/$(basename "${WHICH_PPTD}")" != "$(cd "${INSTALL_DIR}" && pwd)/pptd" ]; then
      echo ""
      echo "!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!"
      echo "[open-pptd] NOTICE: '${INSTALL_DIR}/pptd' is shadowed by '${WHICH_PPTD}' in your PATH."
      echo "[open-pptd] To ensure 'pptd' resolves to the newly installed binary, run:"
      echo "    export PATH=\"${INSTALL_DIR}:\$PATH\""
      echo "!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!"
      echo ""
    elif [[ ":$PATH:" != *":${INSTALL_DIR}:"* ]]; then
      echo ""
      echo "!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!"
      echo "[open-pptd] NOTICE: ${INSTALL_DIR} is not in your current PATH."
      echo "[open-pptd] To invoke 'pptd' directly, run:"
      echo "    export PATH=\"${INSTALL_DIR}:\$PATH\""
      echo "!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!"
      echo ""
    fi
    exit 0
  fi
fi

# Fallback: compile via cargo
echo "[open-pptd] Falling back to building pptd via cargo..." >&2
if command -v cargo >/dev/null 2>&1; then
  if cargo install --git "https://github.com/${REPO}.git" --root "${INSTALL_DIR}/.." --force; then
    INSTALLED_VER="$(probe_version "${INSTALL_DIR}/pptd")"
    if [ -z "$INSTALLED_VER" ] || ! version_ge "$INSTALLED_VER" "$VERSION"; then
      echo "[open-pptd] Error: Cargo compiled binary reported '${INSTALLED_VER:-unknown}', expected >= ${VERSION}" >&2
      exit 1
    fi
    echo "[open-pptd] Success! Cargo build succeeded: pptd ${INSTALLED_VER} is ready at: ${INSTALL_DIR}/pptd"
    "${INSTALL_DIR}/pptd" --version
    exit 0
  fi
fi


# Fallback guide
echo "======================================================================" >&2
echo "PPTD MANUAL INSTALLATION REQUIRED" >&2
echo "======================================================================" >&2
echo "Automatic download and cargo fallback could not complete." >&2
echo "Options to install pptd:" >&2
echo "1. Prebuilt releases: https://github.com/${REPO}/releases" >&2
echo "   Download the archive for ${OS}-${ARCH}, extract 'pptd' into ${INSTALL_DIR}/" >&2
echo "2. Build with Rust: cargo install --git https://github.com/${REPO}.git" >&2
echo "3. Add ${INSTALL_DIR} to your PATH: export PATH=\"${INSTALL_DIR}:\$PATH\"" >&2
echo "======================================================================" >&2
exit 1
