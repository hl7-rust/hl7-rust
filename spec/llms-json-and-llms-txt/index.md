# llms.json and llms.txt

Create AI guidance helper files at the repo root:

- `llms.json` -> JSON
- `llms.txt` -> markdown text

Purpose: Provide AI tools with a clean, curated map of its most important content.

Help large language models (LLMs) read, understand, and cite a site's documentation or resources without getting bogged down 

File size:  < 40k bytes.

## Website copies use different links, on purpose

The workspace-root `llms.txt`/`llms.json` use repo-relative links (e.g.
`README.md`), which only resolve inside the git checkout. Serving that
exact text from `*.github.io/llms.txt` would ship dead links, so the
copies under `*.github.io/static/` are not byte-identical: each entry
points at wherever it actually resolves from the site's own domain — a
site page (`/docs/...`, `/crates/<slug>/`, ...) where one covers the same
content, a `github.com/hl7-rust/hl7-rust` blob URL otherwise.
