# Reviewing AI proposals

An assistant can change a result for you, or it can *propose* the change and leave the decision to you. This page describes the second way: how a proposal gets from the assistant to your screen, what you see, what approving does and what is recorded.

The short version: you ask for a proposal; it appears in your window under **Audit package**, **Review AI proposals**, with the assistant's reason and evidence; you read the original and the proposed values side by side; you click **Approve** or **Reject** and type your own reason. Only an approval changes the result.

## Step by step

1. *Your AI client starts `chromascope-mcp`.* The window that opens is the window the assistant works with. `chromascope.exe`, started from the Start menu or the desktop, is a separate program that no assistant is connected to: it never receives a live proposal. (If you see no *AI connected* line in the status bar, you are in that one; see [MCP troubleshooting](troubleshooting.md#i-see-no-ai-activity-and-no-proposal).)
2. *You ask for a proposal.* For example:

    ```text
    Check the integration of caffeine in run 3. If the boundaries look wrong, propose new ones with your reason and evidence, but don't apply them.
    ```

3. *The assistant inspects the data.* It reads the batch result and the trace behind the peak, with tools such as `quantification_status` and `quantification_trace`. Every call is listed in the AI call log (**View log** in the status bar).
4. *The assistant makes the proposal.* It calls `review_quantification` with `propose` (new peak boundaries of a batch result), or `prepare_review_proposal` (a review of a calibration, a QC rule or a chromatogram). It gives its reason and its evidence. Nothing in your window changes.
5. *The proposal arrives by itself.* Nobody saves a file and nothing is copied by hand. The status bar says *The AI proposed a change* with the link **Review**. The window does not switch tabs on its own, so it never moves while you are working.
6. *You open it.* **Review** opens the **Audit package** with that proposal selected, in the section **Review AI proposals**.
7. *You read it.* The section shows who proposed it, the reason, the evidence, and a table of the original and the proposed values. For new peak boundaries these are the start, the end and the area.
8. *You decide.* Type your own reason, then click **Approve** or **Reject**. The two buttons stay grey until there is a reason.

If the window has a project open and the program was started with `--allow-project-writes`, a proposal can also be stored in the project, so that you can review it later, also in the plain `chromascope.exe`. See [Proposals in a project](#proposals-in-a-project).

## Propose or apply

When you say *propose* the assistant uses one tool; when you say *apply* it uses another. The launch flags decide which of them it may use at all.

| You ask the assistant to | It uses | Flags it needs | Does a result change? |
|---|---|---|---|
| propose new boundaries for a batch result | `review_quantification` with `propose` | none besides `--allow-root` | No, not before you approve. |
| propose a review of a calibration, a QC rule or a chromatogram | `prepare_review_proposal` | none besides `--allow-root`; `--allow-changes` so that the result is in your window | No, not before you approve. |
| keep a proposal in a project | `queue_review_proposal` | `--allow-project-writes` | No. |
| set new boundaries directly | `review_quantification` without `propose` | `--allow-changes` | Yes, at once. |
| approve or reject a proposal that is stored in a project | `resolve_review_proposal` | `--allow-project-writes` | Only a stored proposal, by name, with a reason. |

Approving in the window is your own action and needs no flag. Say *propose* and *don't apply* in your prompt; the assistant's tool descriptions tell it to propose when you ask for a proposal. The built-in prompt `review_integration` works this way.

A proposal needs the result to be in your window. A batch you ran in the tab **Quantification and QC** is there. A calibration or QC result that an assistant calculated is there when the program was started with `--allow-changes`; otherwise open the project that holds it.

## What approving does

*New peak boundaries.* Approving does exactly what you would do by typing the same boundaries in **Quantification and QC**: the peak is integrated again between them with the settings of that compound, the result is marked as edited by hand, and the **automatic** peak is kept. **Reset to automatic** restores it, and removes the note that the boundaries came from a proposal. The review pane of the result shows that note while it applies, with the assistant's reason and your reason.

*A review of a calibration, a QC rule or a chromatogram.* The shared engine applies it as a new review revision of the result, attributed to you, with your reason. **Prepare reversal** prepares the opposite review if you change your mind.

*Rejecting* changes nothing. The proposal stays in the list with your reason.

## What is recorded

For every proposal the record holds, side by side:

- *The assistant's side:* who proposed it, its reason and its evidence, and the original and the proposed values.
- *Your side:* that you approved or rejected it, your name as the window knows it (`desktop-reviewer`), your reason and the time.
- *The result:* an approved boundary proposal leaves the identifier of the proposal, both reasons and the approver on the result itself, and `quantification_status` reports them.

A decided proposal cannot be decided again. The record is saved with the project (**Save project**) and can be written to a file with **Save proposal and decisions…**. Names in the record are labels, not logins.

The AI call log shows that a proposal was delivered. Like the rest of the log it lasts only while the window is open; the proposal itself and its decision are kept with the project.

## Stale proposals

A proposal is made against the result as it was at that moment. If the result changes before you decide (you or the assistant edit the peak, the batch runs again, the method changes), the proposal is *stale*: the section says so, and **Approve** is refused with the message that the result changed after the proposal was made. Nothing is applied. Ask the assistant for a new proposal.

## Proposals in a project

- With `--allow-project-writes` the assistant can store a proposal in a project with `queue_review_proposal`. The proposal also appears in your window at once.
- A boundary proposal belongs to the batch in your window. It is stored when the project is saved or created (`create_project`, `save_project`, or **Save project**), because the project then holds the batch too.
- Open the project later, also in the plain `chromascope.exe`, and the proposals are in **Review AI proposals**, ready to approve or reject there. You do not need an assistant for that.
- **Load a saved proposal file…** loads a proposal that was saved with **Save proposal and decisions…**, for example one a colleague sent you.

## What the assistant cannot do

- It cannot click **Approve** or **Reject**. No tool decides on a proposal in the window.
- It cannot put a proposal in your list in any other way than through the tools that make one. The route into the list is private to the program, and what arrives is what the program itself computed, not text the assistant wrote.
- It cannot mark a peak as reviewed by a person, with or without a proposal.
- With `--allow-project-writes` it can approve a proposal that is stored in a project (`resolve_review_proposal`). Do not give a program that flag if you want every approval to be yours. See [Safety and data](safety.md).

## Next

[Example prompts](example-prompts.md) has ready-to-paste proposal prompts. The user manual describes the section from your side: [Review AI proposals](../manual/audit-package.md#review-ai-proposals).
