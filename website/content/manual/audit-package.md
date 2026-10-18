# Audit package and projects

A *project* keeps the history of your analysis. An *audit package* is a folder you can hand to someone else, who can then check that nothing was changed.

## Projects

Choose **Project** and then **Create project…** to start one from the current workspace, or **Open project…** to continue one. **Save project** stores the workspace and results as a new revision. Revisions are never overwritten: earlier ones stay available, each with the hashes of the source files it used. **Refresh project activity** reloads work that another session saved.

## Review AI proposals

The first section of the tab **Audit package** is **Review AI proposals**. A *proposal* is a change that an AI assistant suggests to one of your results, for example new boundaries for a peak, a review of a calibration result or the acknowledgement of a QC rule. It comes with the assistant's reason and its evidence. *Nothing changes until you approve it.* If you reject it, nothing changes either.

### How a proposal arrives

- *In the window the assistant works with* (`chromascope-mcp`, started by your AI client): the proposal appears in the list by itself, without anyone saving or copying a file. The status bar says *The AI proposed a change* with the link **Review**; the link opens the **Audit package** with that proposal selected. The window does not switch tabs on its own, so it never moves under your hands.
- *In a project*: when the assistant works with a project and the program was started with `--allow-project-writes`, a proposal is also kept in the project. Open the project later, also in the plain `chromascope.exe`, and the proposal is in the list.
- *From a file*: **Load a saved proposal file…** reads a proposal that someone saved with **Save proposal and decisions…**, for example a colleague.

While the list is empty the section explains what a proposal is and how to ask for one. See [Reviewing AI proposals](../mcp/reviewing-proposals.md) for the whole story from the assistant's side.

### What you see

Choose the proposal in the list. You see who proposed it, the reason, the evidence, and the original and the proposed values side by side. For new peak boundaries this is the start, the end and the area before and after. If the result has changed since the assistant made the proposal, the section says so and the proposal can no longer be approved: ask for a new one.

### Approve or reject

Type your own reason in the line below the table. It is required; **Approve** and **Reject** stay grey until you have typed one.

- **Approve** applies the change. For new peak boundaries this is exactly what you would get by typing the boundaries yourself in **Quantification and QC**: the peak is integrated again between them, the result is marked as edited by hand, and the automatic peak is kept, so **Reset to automatic** still brings it back. For a review of a calibration or QC result the shared engine applies it as a new review revision, which you can reverse with **Prepare reversal**.
- **Reject** keeps the proposal in the record and changes no result.

Either way the decision is recorded with your reason next to the assistant's reason and evidence. The assistant cannot approve or reject: the two buttons exist only in the window.

A decided proposal cannot be decided again. To ask for something different, ask the assistant for a new proposal, or use **Prepare revised suggestion** to start from this one. The expert form **Prepare a new review suggestion** lets you write a suggestion yourself.

### Keep the decisions

The decisions are part of the workspace. **Save project** stores them with the project, in a revision that is never overwritten. **Save proposal and decisions…** writes one proposal and its decisions to a file.

Proposals that are still undecided appear as a warning under **1 · Check readiness**, so a package is not exported by accident while a suggestion waits for you.

## Build the audit package

Go to the tab **Audit package**. Choose what goes in with **Include report sections**, give the package a **Report title**, choose where to save it with **Save package to**, then choose **Export audit package**. You get a folder with a readable HTML report, the evidence behind every number and a file of checksums. The option **Portable bundle with verified raw sources** also copies the source files, so the package can be checked elsewhere.

Open warnings (for example peaks that still need review) stop the export until you confirm them.

## Check and recompute

**Reopen project evidence / verify sources** checks that the source files still match their stored hashes. **Reprocess and compare retained evidence** recomputes a stored result from its verified source and compares it with what was saved. A result can only be recomputed with the program version that produced it.

## What the package is, and is not

The report is a review draft. It lists questionable results and the review history next to the good ones, and it states plainly that an AI's explanation is unverified. It is not an approval, a signature or a regulatory release.
