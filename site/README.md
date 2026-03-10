# LabaClaw Docs Site

This is the standalone Vite frontend for the LabaClaw docs site.

## Commands

```bash
cd site
npm install
npm run dev
```

Build for GitHub Pages:

```bash
cd site
npm run build
```

## Notes

- Canonical reader source: [../docs/README.md](../docs/README.md). The Markdown under `../docs/` is the source of truth; the site renders that material for browser navigation.
- The site is English-only.
- The Vite base path is `/labaclaw/`.
- Build output is generated in `../gh-pages`.
- The docs manifest is regenerated before `dev` and `build`.
- The site should present LabaClaw as a public fork of ZeroClaw, with provenance and sync policy called out explicitly.
