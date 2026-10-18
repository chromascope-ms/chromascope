# User manual

This manual is short on purpose. It tells you where things are and how to do the common tasks. Everything that is written in bold is a label you will find in the window.

## The window

At the top you find the menus **File**, **Project** and **View**, then the tabs:

| Tab | What it is for |
|---|---|
| **Data explorer** | Look at the data: file list, chromatograms, spectra, integration, figures. |
| **Quantification and QC** | Measure many samples: methods, peak review, calibration, concentrations and batch QC. |
| **Audit package** | Review what an AI proposed, check a project, then build a report or a portable package from it. |

## The status bar and the AI

If an AI assistant started the program (`chromascope-mcp`, see [AI assistants](../mcp/index.md)), the status bar at the bottom also shows *AI connected · N requests* with the link **View log**, which opens the AI call log. While a request runs it says *AI is working…*. When the assistant proposes a change, the status bar says *The AI proposed a change* with the link **Review**, which opens the **Audit package** at that proposal. The plain `chromascope.exe` has no assistant connected and shows none of this. Proposals that were saved in a project can be reviewed there too: see [Review AI proposals](audit-package.md#review-ai-proposals).

## The tab bar

A button at the right end of the tab bar shows where you are in the analysis, from project to report, and takes you to the next step. **File**, then **About Chromascope…** shows the version, the project links and the licences.

## Pages

- [Data explorer](data-explorer.md): traces, spectra, integration, presets, figures.
- [Quantification and QC](quantification.md): from a method to concentrations and QC.
- [Audit package and projects](audit-package.md): keep a history, build a package.
- [Vendor files](vendor-files.md): open Thermo, SCIEX, Agilent, Waters, Bruker and Shimadzu data.
- [Command line](command-line.md): the same engine as JSON for scripts.
- [Settings](settings.md): theme, size, decimals and the update check.
- [Troubleshooting and FAQ](troubleshooting.md).
- [Limitations](limitations.md): read this before you trust a number.

New here? Do the [ten-minute tutorial](../get-started/getting-started.md) first.
