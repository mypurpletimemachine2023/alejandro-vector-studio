# ACA Vector Studio — ALEJANDRO Creative Academy

Local source fork of https://github.com/storytold/vectorcraft at `5a9a14fdbd5f566be5adb19a45decbbef1c22270`, on `codex/academy-purple-gold`.

The Academy edition keeps upstream engine, file formats, tools, undo, import/export and contributor notices. Purple/gold UI, Academy titles and original crown/A icons are fork changes. `LICENSE-MIT`, `LICENSE-APACHE`, `NOTICE` and upstream asset attribution remain intact. Upstream ArtCraft trademarks are not used as Academy branding.

Browser build: `cd apps/vectorcraft-web` then `trunk build --release`. Requirements: Rust >=1.95, wasm32-unknown-unknown, Trunk, wasm-bindgen and Binaryen. Build outcome is recorded separately; source changes alone are not a runnable browser build.

## Manual public build

`.github/workflows/academy-web.yml` runs only by explicit workflow dispatch on standard free public Ubuntu runners. Inherited automatic workflows are retained under `.github/upstream-workflows-disabled/`. The build pins Rust1.95, Trunk0.22.1 and wasm-bindgen0.2.129, verifies unchanged license texts and Cargo.lock, and uploads the real `dist/web` application with `ACA_BUILD_INFO.json`. No secrets, paid runners, automated publishing or game source/assets are included.
