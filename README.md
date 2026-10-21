<p align="center">
  <img src="website/content/assets/icon.png" alt="Chromascope" width="120">
</p>

<h1 align="center">Chromascope</h1>

<p align="center"><strong>Analyze more. Click less.</strong></p>

<p align="center">
  AI-assisted LC-MS analysis, across instrument vendors, with traceable results.<br>
  Free &middot; Open source &middot; Built for scientists
</p>

<p align="center">
  <a href="https://github.com/chromascope-ms/chromascope/actions/workflows/CI.yml"><img src="https://github.com/chromascope-ms/chromascope/actions/workflows/CI.yml/badge.svg" alt="CI"></a>
  <a href="https://github.com/chromascope-ms/chromascope/releases/latest"><img src="https://img.shields.io/github/v/release/chromascope-ms/chromascope?include_prereleases" alt="Latest release"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/licence-GPL--3.0-blue" alt="GPL-3.0 licence"></a>
  <img src="https://img.shields.io/badge/Rust-1.88%2B-orange" alt="Rust 1.88 or newer">
</p>

<p align="center">
  <a href="https://github.com/chromascope-ms/chromascope/releases/latest">Download</a> &nbsp;&middot;&nbsp;
  <a href="https://chromascope-ms.github.io/chromascope/">Documentation</a> &nbsp;&middot;&nbsp;
  <a href="https://chromascope-ms.github.io/chromascope/get-started/getting-started/">Getting started</a> &nbsp;&middot;&nbsp;
  <a href="https://chromascope-ms.github.io/chromascope/mcp/">MCP guide</a>
</p>

---

Chromascope is a free, lightweight, easy-to-use, open-source, vendor-agnostic LC-MS workbench designed for targeted analysis. It opens mzML directly and works with any vendor file by converting it automatically with [msconvert](https://proteowizard.sourceforge.io/): download msconvert once and link it in Chromascope ([how](https://chromascope-ms.github.io/chromascope/manual/vendor-files/)). Inspect batches, integrate peaks, build calibration curves, quantify and run QC. Use it like a normal desktop app, script it from the command line, or hand the controls to an AI assistant over [MCP](https://modelcontextprotocol.io). The assistant does the reasoning; Chromascope does the maths, keeps the evidence and records exactly what happened.

<p align="center">
  <img src="assets/screenshots/data-explorer.png" alt="The Data explorer: six traces of one file, the mass spectrum of the selected scan and the trace settings" width="49%">
  <img src="assets/screenshots/quantification.png" alt="Quantification and QC, Areas and peak review: every peak of the batch with its confidence, and the integration window of one peak" width="49%">
</p>

https://github.com/user-attachments/assets/cfbe286f-962d-41b3-ad6f-ffa3f33bebf3

<sub>Video not playing? Open [assets/demo.mp4](assets/demo.mp4) directly. It shows an AI client opening a file, extracting an XIC and previewing an integration in the Chromascope window; nothing is changed permanently.</sub>

## Documentation

- **Website and user manual: <https://chromascope-ms.github.io/chromascope/>**
- **[MCP guide](https://chromascope-ms.github.io/chromascope/mcp/)**: set up an AI assistant, what it can do, ready-to-paste prompts, safety.
- [Getting started](https://chromascope-ms.github.io/chromascope/get-started/getting-started/): a ten-minute tutorial with the example file in `test_file/`.
- [CHANGELOG.md](CHANGELOG.md) and [examples/README.md](examples/README.md).

## Install

1. Download the installer for your system from the [releases page](https://github.com/chromascope-ms/chromascope/releases/latest): `.msi` or portable `.zip` (Windows), `.dmg` (macOS), `.deb`, `.rpm`, `.AppImage` or `.tar.gz` (Linux).
2. The builds are not code-signed. On Windows choose *More info*, *Run anyway* in SmartScreen; on macOS use *System Settings*, *Privacy & Security*, *Open Anyway*.
3. Start Chromascope, choose **File**, **Open data…**, and follow the [tutorial](https://chromascope-ms.github.io/chromascope/get-started/getting-started/).

Details for every system, the checksums and building from source are on the [download page](https://chromascope-ms.github.io/chromascope/get-started/download/). The desktop program checks GitHub once a day for a newer version; see [Settings](https://chromascope-ms.github.io/chromascope/manual/settings/#update-check) to turn that off.

## What is in the package

| Program | What it is |
|---|---|
| `chromascope` | The desktop app |
| `chromascope-cli` | The same analysis engine as JSON on the command line |
| `chromascope-mcp` | The desktop app as an MCP server: an AI assistant works in a visible window (74 tools) |
| `chromascope-engine-mcp` | A window-less MCP server with the 14 calculation tools, for scripts and servers |

One engine sits behind all four, so a number from the window is the number from the CLI and from the assistant. Raw files are never modified, and results keep their units, settings, software versions and source hashes.

An assistant works in the window that its client started (`chromascope-mcp`). Its status bar shows how many requests the assistant made, with a link to the call log. When you ask it to *propose* instead of *apply* a change, for example new peak boundaries, the proposal appears in the window under **Audit package**, **Review AI proposals**, with its reason and evidence; only your **Approve** changes a result. Proposals saved in a project can be reviewed later in the plain `chromascope.exe`. See the [guide to reviewing AI proposals](https://chromascope-ms.github.io/chromascope/mcp/reviewing-proposals/).

## Development

```sh
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- -D warnings
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked
cargo test --locked --features mcp
cargo test --locked --no-default-features --features mcp-headless
cargo test --locked --all-features
```

The package needs Rust 1.88 or newer. On Ubuntu the window also needs `libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libxkbcommon-dev`. The website is in `website/` (MkDocs Material); build it from the repository root with `pip install -r website/requirements.txt` and `mkdocs build --strict -f website/mkdocs.yml`. See [contributing](.github/CONTRIBUTING.md): a change to the window or to the MCP tools updates the manual and the MCP guide in the same commit.

```mermaid
flowchart LR
  GUI[egui desktop] --> Engine[Shared analytical engine]
  CLI[JSON CLI] --> Engine
  DMCP[Desktop MCP server] --> GUI
  DMCP --> Engine
  HMCP[Headless MCP server] --> Engine
  Engine --> Rust[Rust kernels / mzdata]
  Engine --> Project[Immutable revisions and provenance]
  Project --> Delivery[JSON / CSV / TSV / SVG / HTML drafts]
```

## Citation

If Chromascope helps with work you publish, please cite it and name the version you used (**File**, **About Chromascope…**). The metadata are in [CITATION.cff](CITATION.cff), which GitHub shows under "Cite this repository".

[![DOI](https://zenodo.org/badge/DOI/10.5281/zenodo.23542311.svg)](https://doi.org/10.5281/zenodo.23542311)

The DOI above is the concept DOI: it always resolves to the latest archived version. Each release also has its own version DOI on [Zenodo](https://doi.org/10.5281/zenodo.23542311).

Plain text:

> Cseresznye, A. Chromascope: an open-source, AI-ready LC-MS workbench, version 0.3.0. Zenodo. https://doi.org/10.5281/zenodo.23542311

BibTeX:

```bibtex
@software{cseresznye_chromascope,
  author  = {Cseresznye, Adam},
  title   = {Chromascope: an open-source, AI-ready LC-MS workbench},
  version = {0.3.0},
  year    = {2026},
  doi     = {10.5281/zenodo.23542311},
  url     = {https://github.com/chromascope-ms/chromascope},
  license = {GPL-3.0-only}
}
```

## Licence

Licensed under [GPL-3.0-only](LICENSE). [THIRD_PARTY_LICENSES.txt](THIRD_PARTY_LICENSES.txt) lists the notice of every dependency and of the embedded Inter and JetBrains Mono fonts.
