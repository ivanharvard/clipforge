.DEFAULT_GOAL := help
.PHONY: help setup build run test check fmt fmt-check clippy clean web-bindings web-build web-dev \
	icons package-appimage package-pacman package-msi install uninstall

help: ## Show this help
	@echo "ClipForge — common development tasks"
	@echo
	@awk 'BEGIN {FS = ":.*## "} /^[a-zA-Z_-]+:.*## / {printf "  %-18s %s\n", $$1, $$2}' $(MAKEFILE_LIST)

setup: ## Check that required tools are installed (does not install anything)
	@ok=1; \
	command -v cargo >/dev/null 2>&1 || { echo "missing: cargo/rustc — install via https://rustup.rs"; ok=0; }; \
	command -v ffmpeg >/dev/null 2>&1 || { echo "missing: ffmpeg — install via your package manager (e.g. pacman -S ffmpeg, apt install ffmpeg)"; ok=0; }; \
	command -v ffprobe >/dev/null 2>&1 || { echo "missing: ffprobe — usually bundled with ffmpeg"; ok=0; }; \
	pkg-config --exists mpv 2>/dev/null || { echo "missing: libmpv development headers — install via your package manager (e.g. pacman -S mpv, apt install libmpv-dev)"; ok=0; }; \
	rustup component list --installed 2>/dev/null | grep -q rustfmt || { echo "missing: rustfmt — install via rustup component add rustfmt"; ok=0; }; \
	rustup component list --installed 2>/dev/null | grep -q clippy || { echo "missing: clippy — install via rustup component add clippy"; ok=0; }; \
	command -v rsvg-convert >/dev/null 2>&1 || echo "optional (for 'make icons'): rsvg-convert — install via your package manager"; \
	{ command -v magick >/dev/null 2>&1 || command -v convert >/dev/null 2>&1; } || echo "optional (for 'make icons'): ImageMagick — install via your package manager"; \
	if [ "$$ok" = "1" ]; then echo "All required tools found."; else echo; echo "Install the missing tools above, then re-run 'make setup'."; exit 1; fi

build: ## Build the whole workspace (debug)
	cargo build --workspace

run: ## Run the app (pass CLIP=/path/to/file.mp4 to open a clip on launch)
	cargo run -p clipforge-app $(if $(CLIP),-- $(CLIP))

test: ## Run clipforge-core's test suite (headless, no display/mpv needed)
	cargo test -p clipforge-core

fmt: ## Format the whole workspace
	cargo fmt --all

fmt-check: ## Check formatting without making changes
	cargo fmt --all --check

clippy: ## Lint the whole workspace, warnings are errors
	cargo clippy --workspace --all-targets -- -D warnings

check: fmt-check clippy test ## Run the same checks as CI (fmt, clippy, test)

web-bindings: ## Build the browser Wasm package (requires wasm-pack)
	wasm-pack build crates/clipforge-web-bindings --release --target web \
		--out-dir ../../web/src/generated/clipforge-wasm

web-build: web-bindings ## Build the static web app to dist/web
	npm --prefix web run build

web-dev: web-bindings ## Start the web app development server
	npm --prefix web run dev

clean: ## Remove build artifacts
	cargo clean

icons: ## Regenerate all icon sizes from crates/clipforge-app/icons/src/app-icon.svg
	./scripts/export-icons.sh

package-appimage: ## Build a Linux AppImage (see scripts/build-appimage.sh)
	./scripts/build-appimage.sh

package-pacman: ## Build a pacman package (see scripts/build-pacman.sh)
	./scripts/build-pacman.sh

package-msi: ## Build a Windows MSI (see scripts/build-msi.ps1, run on Windows)
	./scripts/build-msi.ps1

install: ## Build a release binary and install it for the current user (Linux)
	cargo build --release --bin clipforge-app
	install -Dm755 target/release/clipforge-app "$(HOME)/.local/bin/clipforge-app"
	install -Dm644 packaging/linux/appimage/clipforge.desktop \
		"$(HOME)/.local/share/applications/clipforge.desktop"
	@echo "Installed to $(HOME)/.local/bin/clipforge-app — make sure that's on your PATH."

uninstall: ## Remove the user-level install created by 'make install'
	rm -f "$(HOME)/.local/bin/clipforge-app"
	rm -f "$(HOME)/.local/share/applications/clipforge.desktop"
