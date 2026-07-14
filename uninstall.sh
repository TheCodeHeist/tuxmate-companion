#!/usr/bin/env bash

##################################################################################################
# This script removes the TuxMate Companion binary installed by install.sh from /usr/local/bin.  #
##################################################################################################

set -euo pipefail

BIN_NAME="tuxmate"
install_dir="/usr/local/bin"
binary_path="$install_dir/$BIN_NAME"

if [ ! -e "$binary_path" ]; then
  echo "TuxMate Companion is not installed at $binary_path"
  exit 0
fi

if [ ! -w "$install_dir" ]; then
  echo "Uninstalling requires sudo for $install_dir"
  sudo rm -f "$binary_path"
else
  rm -f "$binary_path"
fi

echo "Removed $binary_path"
echo "Done."
