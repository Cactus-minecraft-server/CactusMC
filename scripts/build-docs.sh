#!/usr/bin/env bash

# Filename: build-docs
# Author: Urpagin
# Date: 2025-08-14
# Description: Build the static documentations for a Rust project.
# Script requirements:
#     The script must be into a Rust project with cargo and mdbook installed.
#     The script must be into a one-level subdirectory from the root of the project,
#     e.g., /docs/build-docs and not /build-docs

### INIT

# NOTICE: do not move this script elsewhere than inside /scripts
# / being the root of the project.
SK_DIR=$(temp=$( realpath "$0"  ) && dirname "$temp") || {
  printf 'realpath failed\n' >&2; exit 1;
}
readonly SK_DIR

# Shown in the super-HTML.
# THE SAME AS CARGO! AS INSIDE THE CARGO.TOML
readonly PROJECT_NAME='Cactus'

# The directory where all the docs will be put, ready to be served.
# Add this into your .gitignore!!
readonly DOCS_OUT="${SK_DIR}/../target/custom_docs"

# Where the mdbook directory is.
readonly MDBOOK_DIR="${SK_DIR}/../docs"


# mkdbook output dir
readonly MDBOOK_OUT_DIR="${DOCS_OUT}/mdbook"
# cargo doc output dir
readonly CARGODOC_OUT_DIR="${DOCS_OUT}/cargo-doc"


# Create the output directory if not already.
mkdir -p "$DOCS_OUT"
if [[ -d "$DOCS_OUT" ]]; then
  rm -rf "${DOCS_OUT:?}"/*
else
  echo 'Error: DOCS_OUT does not exist' >&2
  exit 1
fi

### BUILDING

echo 'Building MdBook...'
mdbook build "$MDBOOK_DIR"
mv "${MDBOOK_DIR}/book" "$MDBOOK_OUT_DIR"


echo 'Building Cargo doc...'
cargo doc --no-deps --all-features --release --jobs "$(nproc)" --target-dir "$CARGODOC_OUT_DIR"

### OUTPUT LAYOUT

HTML_PATH=$(realpath "${DOCS_OUT}/index.html") || {
  printf 'realpath failed to make super HTML path\n' >&2
  exit 1
}
readonly HTML_PATH


# A 'super-HTML' webpage to link to both docs.
cat > "${HTML_PATH}" <<EOF
<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1" />
  <title>Project Documentation – ${PROJECT_NAME}</title>
  <!-- Change the two href values below to match your actual directory names if needed.
       Each target directory should contain its own index.html. -->
  <style>
    :root {
      --bg: #0f1116;
      --bg-soft: #12151d;
      --fg: #e6e9ef;
      --muted: #aab0bc;
      --card: #171a22cc;
      --btn: #2b7cff;
      --btn-hover: #1e66ff;
      --ring: #9abaff;
    }
    @media (prefers-color-scheme: light) {
      :root {
        --bg: #f6f7fb;
        --bg-soft: #eef1f8;
        --fg: #0e1222;
        --muted: #4b5568;
        --card: #ffffffcc;
        --btn: #2563eb;
        --btn-hover: #1d4ed8;
        --ring: #93c5fd;
      }
    }
    * { box-sizing: border-box; }
    html, body {
      height: 100%;
      margin: 0;
      color: var(--fg);
      background: radial-gradient(1200px 800px at 20% 0%, var(--bg-soft), var(--bg));
      font-family: system-ui, -apple-system, Segoe UI, Roboto, Ubuntu, Cantarell, "Noto Sans", Arial, "Apple Color Emoji", "Segoe UI Emoji", "Segoe UI Symbol", "Noto Color Emoji", sans-serif;
      line-height: 1.4;
    }
    .wrap {
      min-height: 100%;
      display: grid;
      place-items: center;
      padding: 2rem;
    }
    .card {
      width: 100%;
      max-width: 720px;
      background: var(--card);
      backdrop-filter: blur(8px);
      border: 1px solid rgba(255,255,255,0.08);
      border-radius: 18px;
      box-shadow: 0 10px 30px rgba(0,0,0,0.25);
      padding: 2rem;
    }
    h1 {
      margin: 0 0 0.25rem;
      font-size: clamp(1.6rem, 2.2vw + 1rem, 2.4rem);
      letter-spacing: -0.02em;
    }
    p.lead {
      margin: 0 0 1.5rem;
      color: var(--muted);
      font-size: 0.98rem;
    }
    .grid {
      display: grid;
      grid-template-columns: repeat(2, minmax(0, 1fr));
      gap: 1rem;
    }
    @media (max-width: 600px) { .grid { grid-template-columns: 1fr; } }
    a.btn {
      display: flex;
      align-items: center;
      gap: 0.75rem;
      width: 100%;
      padding: 1rem 1.1rem;
      text-decoration: none;
      color: white;
      background: var(--btn);
      border-radius: 14px;
      font-weight: 600;
      justify-content: center;
      border: 1px solid rgba(255,255,255,0.12);
      transition: transform .06s ease, box-shadow .2s ease, background .2s ease;
      box-shadow: 0 6px 18px rgba(37,99,235,0.28);
    }
    a.btn:hover { background: var(--btn-hover); transform: translateY(-1px); }
    a.btn:active { transform: translateY(0); }
    a.btn:focus-visible {
      outline: none;
      box-shadow: 0 0 0 4px var(--ring);
    }
    .btn small {
      display: block;
      font-weight: 500;
      opacity: .85;
    }
    .btn .icon {
      font-size: 1.25rem;
      line-height: 1;
    }
    footer {
      margin-top: 1.25rem;
      font-size: 0.85rem;
      color: var(--muted);
      text-align: center;
    }
    code { font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, "Liberation Mono", monospace; }
  </style>
</head>
<body>
  <main class="wrap">
    <section class="card" role="region" aria-labelledby="title">
      <h1 id="title">Project Documentation</h1>
      <p class="lead">Choose a documentation set. Each link opens the <code>index.html</code> inside that directory.</p>

      <div class="grid">
        <!-- MDBook output directory (e.g., "./book/"). Update href if your directory name differs. -->
        <a class="btn" href="./mdbook/index.html" aria-label="Open MDBook documentation">
          <span class="icon">📚</span>
          <span>
            MDBook
            <small>Modular book-style docs</small>
          </span>
        </a>

        <!-- Cargo/Rustdoc output directory (you might name it "./rustdoc/" or similar). -->
        <a class="btn" href="./cargo-doc/doc/${PROJECT_NAME}/index.html" aria-label="Open CargoDoc (rustdoc) documentation">
          <span class="icon">🦀</span>
          <span>
            CargoDoc
            <small>API docs generated by rustdoc</small>
          </span>
        </a>
      </div>

    </section>
  </main>
</body>
</html>
EOF

printf '\n\nDocs built! Visit them at %s\n' "${HTML_PATH}"