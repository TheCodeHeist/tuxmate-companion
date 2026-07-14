# tuxmate-companion

TuxMate Companion is a small CLI helper for TuxMate (by @abusoww) that streamlines installing and managing apps for the TuxMate platform.

## Key points

- Lightweight command-line interface for app installation and updates
- Focused on simplicity and reproducibility
- Designed to work alongside the upstream TuxMate project

## Quickstart

### Installation (Linux)

For most users, you can install TuxMate Companion directly with a one-liner that fetches and runs the installer script. The repository clone step below is only necessary for contributors.

1. One-line installer (recommended for general users):

   ```bash
   curl -fsSL https://raw.githubusercontent.com/TheCodeHeist/tuxmate-companion/main/install.sh | bash
   ```

   To remove the binary later, run:

   ```bash
   curl -fsSL https://raw.githubusercontent.com/TheCodeHeist/tuxmate-companion/main/uninstall.sh | bash
   ```

2. Developer / contributor workflow (optional):

   Clone the repository if you want to inspect or modify the source

   ```bash
   git clone https://github.com/TheCodeHeist/tuxmate-companion.git
   cd tuxmate-companion
   ```

   Follow the included scripts or README sections for usage.

### Basic usage

- Run the companion CLI (example):

  ```bash
  tuxmate help
  ```

## Contributing

- Issues and pull requests are welcome. Keep changes small and focused.

## License

- **GPL-3.0** License. You are free to use, modify, and distribute this software under the terms of the GNU General Public License version 3.
  In short, go wild with it, follow the license terms, and expect no warranty or liability. See the LICENSE file for details.

For full documentation and advanced usage, see the repository files and upstream TuxMate project.
