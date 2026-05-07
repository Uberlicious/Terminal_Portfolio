# Terminal Portfolio

This is my personal portfolio website.

## Stack:
* Rust
* Askama for html templating
* HTMX
* Tailwind

## Deployment

This project uses GitHub Actions to automatically build and push a Docker image to the GitHub Container Registry (GHCR) on every push to `main`.

### Server Setup

1. **Install Podman & Podman-compose:**
   Ensure your server has Podman and Podman-compose (or Docker and Docker-compose) installed.

2. **Login to GHCR:**
   You need to create a GitHub Personal Access Token (PAT) with `read:packages` permissions and use it to login on your server:
   ```bash
   echo "YOUR_GITHUB_PAT" | podman login ghcr.io -u Uberlicious --password-stdin
   ```

3. **Deploy with Compose:**
   Copy the `compose.yml` file to your server and run:
   ```bash
   export IMAGE_NAME=ghcr.io/uberlicious/terminal_portfolio
   podman-compose pull
   podman-compose up -d
   ```

4. **Enable Auto-updates:**
   To make the server automatically pull and restart when a new image is pushed to GHCR, enable the Podman auto-update timer:
   ```bash
   # Start the timer (checks daily by default)
   systemctl --user enable --now podman-auto-update.timer
   ```
   *Note: This works because of the `io.containers.autoupdate=image` label in `compose.yml`.*

## Compile locally
compile tailwind: `pnpm dev:css` or `pnpm build:css`

run app: `cargo watch -x run`
