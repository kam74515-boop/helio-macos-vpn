# Helio

Helio is a macOS proxy and VPN dashboard prototype inspired by Surge's information architecture, with a flatter Material-style visual language.

## Current Scope

- Surge-like activity, overview, process, device, proxy configuration, rules, capture, MITM, rewrite, and settings screens.
- Local-only Vite development server on `localhost`.
- Tauri macOS shell with a bundled `sing-box` sidecar placeholder.

## Development

```bash
npm install
npm run dev
```

## Build

```bash
npm run build
npx tauri build
```

Build artifacts are generated under `src-tauri/target/release/bundle/`.
