# rhwp Firefox Extension

Firefox WebExtension build of rhwp's HWP/HWPX viewer and editor.

## Build

The Firefox package reuses the rhwp-studio viewer and copies Firefox-specific
extension files into `rhwp-firefox/dist`.

```bash
cd rhwp-firefox
npm install
npm run build
```

The repository must already have the Rust/WASM package in `pkg/`. If `pkg/` is
missing, build it from the repository root first:

```bash
cp .env.docker.example .env.docker
docker compose --env-file .env.docker run --rm wasm
```

## Load Temporarily

1. Open `about:debugging#/runtime/this-firefox` in Firefox.
2. Click "Load Temporary Add-on...".
3. Select `rhwp-firefox/dist/manifest.json`.

## Firefox Notes

Chrome uses `background.service_worker` for Manifest V3. Firefox does not
support extension background service workers yet, so this build uses
`background.scripts` with ES modules.

Firefox also does not support Chrome's `downloads.onDeterminingFilename` event.
This build opens detected HWP/HWPX downloads with `downloads.onCreated` where
available and intercepts direct HWP/HWPX link clicks from the content script.
