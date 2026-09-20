#!/usr/bin/env sh
# Esprit one-line installer
# Usage: curl -fsSL https://raw.githubusercontent.com/krtvyasingh/Esprit/main/scripts/install.sh | sh
set -eu

REPO="krtvyasingh/Esprit"
VERSION="${ESPRIT_VERSION:-latest}"
BIN_DIR="${ESPRIT_INSTALL_DIR:-/usr/local/bin}"

# ── Detect platform ───────────────────────────────────────────────────────────
OS="$(uname -s)"
ARCH="$(uname -m)"

case "${OS}" in
  Darwin)
    case "${ARCH}" in
      arm64)  TARGET="aarch64-apple-darwin" ;;
      x86_64) TARGET="x86_64-apple-darwin"  ;;
      *) echo "Unsupported macOS arch: ${ARCH}" && exit 1 ;;
    esac
    ;;
  Linux)
    case "${ARCH}" in
      x86_64)  TARGET="x86_64-unknown-linux-musl"  ;;
      aarch64) TARGET="aarch64-unknown-linux-musl"  ;;
      *) echo "Unsupported Linux arch: ${ARCH}" && exit 1 ;;
    esac
    ;;
  *)
    echo "Unsupported OS: ${OS}"
    echo "Please download a binary manually from:"
    echo "  https://github.com/${REPO}/releases"
    exit 1
    ;;
esac

# ── Resolve version ───────────────────────────────────────────────────────────
if [ "${VERSION}" = "latest" ]; then
  VERSION="$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" \
    | grep '"tag_name"' | sed 's/.*"tag_name": *"v\([^"]*\)".*/\1/')"
fi

echo "Installing Esprit v${VERSION} for ${TARGET}…"

# ── Download ──────────────────────────────────────────────────────────────────
URL="https://github.com/${REPO}/releases/download/v${VERSION}/esprit-${TARGET}.tar.gz"
TMPDIR="$(mktemp -d)"
TARBALL="${TMPDIR}/esprit.tar.gz"

if command -v curl > /dev/null 2>&1; then
  curl -fsSL --progress-bar "${URL}" -o "${TARBALL}"
elif command -v wget > /dev/null 2>&1; then
  wget -q --show-progress "${URL}" -O "${TARBALL}"
else
  echo "Error: neither curl nor wget found." && exit 1
fi

# ── Verify SHA-256 ────────────────────────────────────────────────────────────
SHA_URL="${URL}.sha256"
EXPECTED="$(curl -fsSL "${SHA_URL}" | awk '{print $1}')"
if command -v sha256sum > /dev/null 2>&1; then
  ACTUAL="$(sha256sum "${TARBALL}" | awk '{print $1}')"
elif command -v shasum > /dev/null 2>&1; then
  ACTUAL="$(shasum -a 256 "${TARBALL}" | awk '{print $1}')"
else
  echo "Warning: cannot verify SHA-256 (no sha256sum or shasum)."
  ACTUAL="${EXPECTED}"
fi

if [ "${ACTUAL}" != "${EXPECTED}" ]; then
  echo "SHA-256 mismatch! Download may be corrupt."
  echo "  Expected: ${EXPECTED}"
  echo "  Got:      ${ACTUAL}"
  exit 1
fi

# ── Install ───────────────────────────────────────────────────────────────────
tar -xzf "${TARBALL}" -C "${TMPDIR}"
rm -f "${TARBALL}"

if [ -w "${BIN_DIR}" ]; then
  mv "${TMPDIR}/esprit" "${BIN_DIR}/esprit"
else
  sudo mv "${TMPDIR}/esprit" "${BIN_DIR}/esprit"
fi
chmod +x "${BIN_DIR}/esprit"
rm -rf "${TMPDIR}"

echo ""
echo "  ✓ Esprit v${VERSION} installed to ${BIN_DIR}/esprit"
echo ""
echo "  Next steps:"
echo "    esprit init          # Download the default AI model (~390 MB)"
echo "    esprit doctor        # System health check"
echo "    esprit --help        # All commands"
echo ""
