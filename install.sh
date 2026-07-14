#!/usr/bin/env bash

##################################################################################################
# This script is used to install the TuxMate Companion binary from the latest release on GitHub. #
# It detects the system architecture and downloads the appropriate binary.                       #
##################################################################################################

set -euo pipefail

REPO="TheCodeHeist/tuxmate-companion"
BIN_NAME="tuxmate"

arch() {
  case "$(uname -m)" in
    x86_64) echo amd64 ;;
    aarch64|arm64) echo arm64 ;;
    armv7l) echo armv7 ;;
    *) echo "$(uname -m)" ;;
  esac
}

tmpdir=$(mktemp -d)
cleanup(){ rm -rf "$tmpdir"; }
trap cleanup EXIT

echo "Fetching latest release for $REPO..."
api="https://api.github.com/repos/$REPO/releases/latest"

json="${tmpdir}/release.json"
if ! curl -sSL "$api" -o "$json"; then
  echo "Failed to fetch release info" >&2
  exit 1
fi

ARCH=$(arch)

# Try jq first, fallback to grep/sed
asset_url=""
if command -v jq >/dev/null 2>&1; then
  asset_url=$(jq -r --arg arch "$ARCH" '
    (.assets // [])
    | map(select(((.name // "") | test("linux";"i")) and ((.name // "") | test($arch;"i"))))
    | .[0] // empty
    | .browser_download_url // empty
  ' "$json")
  if [ -z "$asset_url" ]; then
    asset_url=$(jq -r '
      (.assets // [])
      | map(select((.name // "") | test("^tuxmate$";"i")))
      | .[0] // empty
      | .browser_download_url // empty
    ' "$json")
  fi
else
  asset_url=$(grep -oE '"browser_download_url":\s*"[^"]+"' "$json" | sed -E 's/"browser_download_url":\s*"([^"]+)"/\1/' | grep -Ei 'linux' | grep -i "$ARCH" | head -n1 || true)
  if [ -z "$asset_url" ]; then
    asset_url=$(grep -oE '"browser_download_url":\s*"[^"]+"' "$json" | sed -E 's/"browser_download_url":\s*"([^"]+)"/\1/' | grep -Ei 'tuxmate' | head -n1 || true)
  fi
fi

if [ -z "$asset_url" ]; then
  echo "No matching linux/$ARCH asset found in latest release." >&2
  exit 1
fi

echo "Downloading asset: $asset_url"
outfile="$tmpdir/$BIN_NAME"
if ! curl -sSL "$asset_url" -o "$outfile"; then
  echo "Download failed" >&2
  exit 1
fi

chmod +x "$outfile"

install_dir="/usr/local/bin"
if [ ! -w "$install_dir" ]; then
  echo "Installing requires sudo for $install_dir"
  sudo mv "$outfile" "/usr/local/bin/$BIN_NAME"
  sudo chmod +x "/usr/local/bin/$BIN_NAME"
else
  mv "$outfile" "/usr/local/bin/$BIN_NAME"
  chmod +x "/usr/local/bin/$BIN_NAME"
fi

echo "Installed /usr/local/bin/$BIN_NAME"
echo "Done."
