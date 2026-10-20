---
title: Chromascope
---

<div class="hero" markdown>

![Chromascope](assets/icon.png){ .hero-logo width="120" }

# Chromascope

<p class="hero-slogan">Analyze more. Click less.</p>

<p class="hero-tagline">AI-assisted LC-MS analysis, across instrument vendors, with traceable results.</p>

<p class="hero-badges">Free &middot; Open source &middot; Built for scientists</p>

[Download Chromascope](get-started/download.md){ .md-button .md-button--primary }
[Getting started in ten minutes](get-started/getting-started.md){ .md-button }
[Use it with an AI assistant](mcp/index.md){ .md-button }

</div>

Chromascope is a free, lightweight, easy-to-use, open-source, vendor-agnostic LC-MS workbench designed for targeted analysis. Open your LC-MS data (convert vendor files to mzML with [msconvert](https://proteowizard.sourceforge.io/)), inspect batches, integrate peaks, build calibration curves, quantify and run QC. Use it like a normal desktop app, script it from the command line, or hand the controls to an AI assistant over [MCP](https://modelcontextprotocol.io). The assistant does the reasoning; Chromascope does the maths, keeps the evidence and records exactly what happened. Every number keeps its units, its settings and the hash of the file it came from.

## Who it is for

Chromascope is for analytical chemists and mass-spectrometry operators who want to look at their data, quantify a handful of target compounds, and be able to show later how each result was obtained. You do not need to program.

## What it does

- *Explore.* Open mzML files (or vendor files through ProteoWizard), draw TIC, base peak and extracted-ion chromatograms, inspect every spectrum and compare samples side by side.
- *Integrate.* Drag across a peak to get its area from the full-resolution trace, with the baseline and the settings recorded next to the number.
- *Quantify.* Run a method over a batch of samples, review each peak, build weighted calibration curves with internal standards, and evaluate batch QC rules.
- *Deliver.* Keep an append-only project history and export an audit package that another person can verify.
- *Work with an AI.* Over [MCP](mcp/index.md), an assistant such as Claude can open files, extract traces, preview integrations and run a batch. It only does what you allowed when you started the program, and every call is recorded in the window.

## Where to go next

| I want to... | Go to |
|---|---|
| install the program | [Download and install](get-started/download.md) |
| try it on the example file | [Getting started](get-started/getting-started.md) |
| look things up | [User manual](manual/index.md) |
| connect an AI assistant | [AI assistants (MCP)](mcp/index.md) |
| know what it cannot do | [Limitations](manual/limitations.md) |
| see what changed | [Changelog](about/changelog.md) |

!!! note "Not an instrument-validated product"
    Chromascope is tested against authored data and independent numerical references. It has not been validated on instruments, and it does not certify regulatory compliance. Read the [limitations](manual/limitations.md) before you rely on a result.
