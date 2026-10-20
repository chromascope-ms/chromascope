---
title: Chromascope
---

<div class="hero" style="text-align:center" markdown>

![Chromascope](assets/icon.png){ .hero-logo style="display:block;margin:0 auto;width:5.2rem" }

# Chromascope: Analyze more. Click less.

<p class="hero-tagline">AI-assisted LC-MS analysis, across instrument vendors, with traceable results.</p>

<p class="hero-badges">Free <span class="hero-dot">&middot;</span> Open source <span class="hero-dot">&middot;</span> Built for scientists</p>

[Download Chromascope](get-started/download.md){ .md-button .md-button--primary }
[Getting started in ten minutes](get-started/getting-started.md){ .md-button }
[Use it with an AI assistant](mcp/index.md){ .md-button }

</div>

Chromascope is a free, lightweight, easy-to-use, open-source, vendor-agnostic LC-MS workbench designed for targeted analysis. It opens mzML directly and works with any vendor file by converting it automatically with [msconvert](https://proteowizard.sourceforge.io/): download msconvert once and link it in Chromascope ([how](manual/vendor-files.md)). Inspect batches, integrate peaks, build calibration curves, quantify and run QC. Use it like a normal desktop app, script it from the command line, or hand the controls to an AI assistant over [MCP](https://modelcontextprotocol.io). The assistant does the reasoning; Chromascope does the maths, keeps the evidence and records exactly what happened. Every number keeps its units, its settings and the hash of the file it came from.

## Who it is for

Chromascope is for analytical chemists and mass-spectrometry operators who want to explore their data, quantify their batches, and save their analyses reproducibly so they can later see how each result was obtained. No programming is required.

## What it does

- *Explore.* Open mzML files (or vendor files through ProteoWizard), draw TIC, base peak and extracted-ion chromatograms, inspect every spectrum and compare samples side by side.
- *Integrate.* Drag across a peak to get its area from the full-resolution trace, with the baseline and the settings recorded next to the number.
- *Quantify.* Run a method over a batch of samples, review each peak, build weighted calibration curves with internal standards, and evaluate batch QC rules.
- *Deliver.* Keep an append-only project history and export an audit package that another person can verify.
- *Work with an AI.* Over [MCP](mcp/index.md), an assistant such as Claude can open files, extract traces, preview integrations and run a batch. It only does what you allowed when you started the program, and every call is recorded in the window.

<figure markdown>
  ![The Data explorer: six traces of one file, the mass spectrum of the selected scan and the trace settings](assets/data-explorer.png){ loading=lazy }
  <figcaption>The Data explorer: traces of one file, the mass spectrum of the selected scan and the trace settings.</figcaption>
</figure>

<figure markdown>
  ![Quantification and QC, Areas and peak review: every peak of the batch with its confidence, and the integration window of one peak](assets/quantification.png){ loading=lazy }
  <figcaption>Areas and peak review: every peak of the batch with its confidence, and the integration window of one peak.</figcaption>
</figure>

## Where to go next

| I want to... | Go to |
|---|---|
| install the program | [Download and install](get-started/download.md) |
| try it on the example file | [Getting started](get-started/getting-started.md) |
| look things up | [User manual](manual/index.md) |
| connect an AI assistant | [AI assistants (MCP)](mcp/index.md) |
| see what changed | [Changelog](about/changelog.md) |
