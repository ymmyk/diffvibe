# DiffVibe - Visual Diff & Merge Tool
# Run `just` to see available commands

# Default recipe - show help
default:
    @just --list

# Install dependencies
install:
    pnpm install

# Run the app in development mode
dev:
    pnpm tauri dev

# Build the app for production
build:
    pnpm tauri build

# Build debug version (faster compilation)
build-debug:
    pnpm tauri build --debug

# Run Rust tests
test:
    cd src-tauri && cargo test

# Run frontend type checking
check-frontend:
    pnpm check

# Check Rust code for errors without building
check:
    cd src-tauri && cargo check

# Check everything (Rust + frontend)
check-all: check check-frontend

# Format Rust code
fmt:
    cd src-tauri && cargo fmt

# Format frontend code
fmt-frontend:
    pnpm exec prettier --write "src/**/*.{ts,svelte,css,html}"

# Lint Rust code
lint:
    cd src-tauri && cargo clippy

# Clean build artifacts
clean:
    cd src-tauri && cargo clean
    rm -rf node_modules
    rm -rf build
    rm -rf .svelte-kit

# Update dependencies
update:
    pnpm update
    cd src-tauri && cargo update

# Open the project in VS Code
code:
    code .

# Attach to or create a zellij session
attach:
    zellij attach diffvibe --create

# Tag and push a release (usage: just release 0.2.0)
release version:
    @echo "Updating version to {{version}}..."
    sed -i '' 's/"version": "[^"]*"/"version": "{{version}}"/' src-tauri/tauri.conf.json
    sed -i '' 's/"version": "[^"]*"/"version": "{{version}}"/' package.json
    sed -i '' 's/^version = "[^"]*"/version = "{{version}}"/' src-tauri/Cargo.toml
    git add src-tauri/tauri.conf.json package.json src-tauri/Cargo.toml
    git commit -m "Release v{{version}}"
    git tag "v{{version}}"
    git push origin main --tags
    @echo "Release v{{version}} tagged and pushed!"

# Configure git to use DiffVibe as difftool
git-difftool-config:
    @echo "Run these commands to configure git to use DiffVibe:"
    @echo ""
    @echo "git config --global diff.tool diffvibe"
    @echo "git config --global difftool.diffvibe.cmd 'diffvibe \"$LOCAL\" \"$REMOTE\"'"
    @echo "git config --global difftool.prompt false"

# Configure git to use DiffVibe as mergetool
git-mergetool-config:
    @echo "Run these commands to configure git to use DiffVibe as mergetool:"
    @echo ""
    @echo "git config --global merge.tool diffvibe"
    @echo "git config --global mergetool.diffvibe.cmd 'diffvibe \"$LOCAL\" \"$BASE\" \"$REMOTE\" --output \"$MERGED\"'"
    @echo "git config --global mergetool.diffvibe.trustExitCode true"
