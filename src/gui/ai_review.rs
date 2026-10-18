//! Review AI proposals: original and proposed values side by side with the AI's reason and
//! evidence, and a person's approval or rejection with their own reason.
//!
//! A proposal never changes a result until a person approves it here. Two kinds exist: engine
//! proposals (a revision-bound review of a calibration, QC report, chromatogram...) that the
//! shared engine applies, and peak-boundary proposals for the batch quantification table, which
//! this window applies exactly like a manual boundary edit. The AI cannot decide on either: the
//! decision buttons exist only in the window.
use super::MzViewerApp;
use crate::{
    domain::{Operation, Request},
    engine::{Output, Response},
    proposals::Proposal,
};
use eframe::egui;
use serde_json::Value;
type ProposalOutcome = crate::domain::Result<(Proposal, Option<Response>)>;
/// Who a decision made in this window is attributed to.
const REVIEWER: &str = "desktop-reviewer";
#[derive(Default, serde::Serialize, serde::Deserialize)]
pub(super) struct State {
    proposals: Vec<Proposal>,
    selected: usize,
    draft: String,
    reason: String,
    evidence: String,
    review_reason: String,
    message: String,
    #[serde(skip)]
    pending: Option<std::sync::mpsc::Receiver<ProposalOutcome>>,
    /// The newest proposal an AI delivered that nobody has looked at yet (the status bar notice).
    #[serde(skip)]
    notice: Option<uuid::Uuid>,
    /// Open the "Review AI proposals" section on the next frame (the notice's Review button).
    #[serde(skip)]
    reveal: bool,
}
fn current(app: &MzViewerApp, before: &Value) -> Option<Value> {
    let value =
        serde_json::json!({"quant":app.quant,"spectral":app.spectral,"untargeted":app.untargeted});
    crate::proposals::find_current(&value, before)
}
fn diff(before: &Value, after: &Value, path: String, rows: &mut Vec<super::table::Row>) {
    if before == after {
        return;
    }
    match (before, after) {
        (Value::Object(a), Value::Object(b)) => {
            let keys: std::collections::BTreeSet<_> = a.keys().chain(b.keys()).collect();
            for key in keys {
                diff(&before[key], &after[key], format!("{path}/{key}"), rows);
            }
        }
        (Value::Array(a), Value::Array(b)) => {
            for i in 0..a.len().max(b.len()) {
                diff(
                    a.get(i).unwrap_or(&Value::Null),
                    b.get(i).unwrap_or(&Value::Null),
                    format!("{path}/{i}"),
                    rows,
                );
            }
        }
        _ => rows.push(super::table::Row {
            key: path.clone(),
            cells: vec![
                super::table::Cell::text(path),
                super::table::Cell::text(before),
                super::table::Cell::text(after),
            ],
        }),
    }
}
pub(super) fn retain(app: &mut MzViewerApp, response: Response) -> Result<(), String> {
    match &response.output {
        Output::TargetedQuantification { batch } => app.quant.retain_targeted(*batch.clone()),
        Output::QcReport { report } => app.quant.retain_qc(*report.clone()),
        Output::ChromatographicProcessing {
            analyses,
            preview: false,
        } => {
            for analysis in analyses {
                app.quant.retain_chromatography(analysis.clone())?;
            }
            Ok(())
        }
        Output::SpectralSearch { .. } => {
            app.spectral.retain(response);
            Ok(())
        }
        Output::FeatureAnnotations { ledger } => app.untargeted.retain_annotations(*ledger.clone()),
        _ => Err("Unsupported applied review output".into()),
    }
}
pub(super) fn poll(app: &mut MzViewerApp, ctx: &egui::Context) {
    let mut state = std::mem::take(&mut app.ai);
    if let Some(rx) = &state.pending {
        match rx.try_recv() {
            Ok(result) => {
                state.pending = None;
                match result {
                    Ok((proposal, response)) => {
                        if let Some(response) = response {
                            if let Err(error) = retain(app, response) {
                                state.message = error;
                                app.ai = state;
                                return;
                            }
                        }
                        if let Some(index) =
                            state.proposals.iter().position(|old| old.id == proposal.id)
                        {
                            state.proposals[index] = proposal;
                            state.selected = index;
                        } else {
                            state.proposals.push(proposal);
                            state.selected = state.proposals.len() - 1;
                        }
                        state.message =
                            "Decision recorded. Save the project revision to persist this review."
                                .into();
                    }
                    Err(error) => state.message = error.to_string(),
                }
            }
            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                state.pending = None;
                state.message = "Proposal worker disconnected; original evidence retained".into();
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => {
                ctx.request_repaint_after(std::time::Duration::from_millis(100))
            }
        }
    }
    app.ai = state;
}

/// What a proposal is, how to get one and where it appears: shown while the list is empty.
fn empty_state(ui: &mut egui::Ui, mcp_window: bool) {
    ui.add_space(4.0);
    ui.label(egui::RichText::new("No AI proposals yet").strong());
    ui.label("What a proposal is: an AI suggests a change to a result (for example new peak boundaries) with its reason and evidence. Nothing changes until you approve it.");
    ui.label("How to get one: connect your AI client to chromascope-mcp, then ask for a proposal instead of a direct change, for example:");
    ui.indent("proposal_example", |ui| {
        ui.label(egui::RichText::new("\u{201c}Check the integration of caffeine in run 3. If the boundaries look wrong, propose new ones with your reason and evidence, but don't apply them.\u{201d}").italics());
    });
    ui.label("Where it appears: right here, with Approve and Reject buttons.");
    ui.label("With projects: start the server with --allow-project-writes so proposals are also saved in the project and can be reviewed later, also in chromascope.exe.");
    if !mcp_window {
        ui.label("This window has no AI connected: live proposals arrive in the chromascope-mcp window, and saved ones appear here when you open their project.");
    }
    ui.add_space(4.0);
}

/// One line saying what the proposal would change.
fn summary(proposal: &Proposal) -> String {
    match &proposal.peak_edit {
        Some(edit) => format!("{} / {}: peak boundaries", edit.sample, edit.analyte),
        None => match proposal.request.as_ref().map(|r| &r.operation) {
            Some(Operation::ReviewTargeted { sample, target, .. }) => {
                format!("Review of {sample} / {target}")
            }
            Some(Operation::ReviewQc { rule_id, .. }) => format!("QC rule {rule_id}"),
            Some(Operation::ReviseChromatogram { .. }) => "Chromatogram revision".into(),
            Some(Operation::AnnotateSpectrum { .. }) => "Spectrum annotation".into(),
            Some(Operation::ReviewFeatureAnnotation { .. }) => "Feature annotation review".into(),
            _ => "Review".into(),
        },
    }
}
fn minutes(value: Option<f64>) -> String {
    value.map_or("none".into(), |v| format!("{v:.3}"))
}
fn area(value: Option<f64>) -> String {
    value.map_or("none".into(), |v| format!("{v:.4e}"))
}
/// Lines that tell the person what a boundary proposal changes.
fn peak_edit_summary(ui: &mut egui::Ui, edit: &crate::proposals::PeakEdit) {
    ui.label(format!(
        "{} / {}, result {}{}",
        edit.sample,
        edit.analyte,
        edit.result_index,
        edit.dataset_stable_id
            .as_ref()
            .map_or(String::new(), |id| format!(" · dataset {id}"))
    ));
    ui.label(format!(
        "Original: {} – {} min, area {}",
        minutes(edit.original.start),
        minutes(edit.original.end),
        area(edit.original.area)
    ));
    ui.label(format!(
        "Proposed: {} – {} min, area {}",
        minutes(edit.proposed.start),
        minutes(edit.proposed.end),
        area(edit.proposed.area)
    ));
}
/// Whether the result a boundary proposal is about is still what it was (checked about once a
/// second: the check hashes the trace).
fn peak_edit_status(app: &MzViewerApp, ctx: &egui::Context, proposal: &Proposal) -> Option<String> {
    let edit = proposal.peak_edit.as_ref()?;
    if !proposal.events.is_empty() {
        return None;
    }
    let key = egui::Id::new(("peak_edit_status", proposal.id));
    let now = ctx.input(|i| i.time);
    if let Some((at, text)) = ctx.data(|d| d.get_temp::<(f64, Option<String>)>(key)) {
        if now - at < 1.0 {
            return text;
        }
    }
    let text = match super::quant::current_peak_state(&app.quant, edit) {
        None => Some("This result is not in this window's batch (open its project first). It cannot be approved here.".to_owned()),
        Some((_, current)) if current != proposal.before => Some("The result changed after the AI made this proposal, so it can no longer be approved. Ask for a new proposal.".to_owned()),
        Some(_) => None,
    };
    ctx.data_mut(|d| d.insert_temp(key, (now, text.clone())));
    ctx.request_repaint_after(std::time::Duration::from_secs(1));
    text
}
/// A person's decision on a peak-boundary proposal.
fn decide_peak_edit(app: &mut MzViewerApp, state: &mut State, proposal: &Proposal, approve: bool) {
    let Some(edit) = proposal.peak_edit.as_ref() else {
        return;
    };
    let Some((index, current)) = super::quant::current_peak_state(&app.quant, edit) else {
        state.message =
            "The result is not in this window's batch. Open its project before reviewing.".into();
        return;
    };
    let updated = match crate::proposals::resolve_peak_edit(
        proposal,
        &current,
        approve,
        REVIEWER,
        &state.review_reason,
    ) {
        Ok(updated) => updated,
        Err(error) => {
            state.message = error.message;
            return;
        }
    };
    if approve {
        let decision = crate::quant::BoundaryDecision {
            proposal_id: proposal.id.to_string(),
            proposed_by: edit.proposer.clone(),
            proposal_reason: proposal.reason.clone(),
            approved_by: REVIEWER.into(),
            approval_reason: state.review_reason.clone(),
            unix_ms: updated.events.last().map_or(0, |e| e.unix_ms),
        };
        if let Err(error) = super::quant::apply_peak_edit(&mut app.quant, index, edit, decision) {
            state.message = error;
            return;
        }
    }
    if let Some(slot) = state.proposals.iter_mut().find(|p| p.id == proposal.id) {
        *slot = updated;
    }
    state.message = if approve {
        "Approved: the boundaries are applied in Quantification and QC (Reset to automatic restores the automatic peak). Save the batch or the project to keep this.".into()
    } else {
        "Rejected: nothing was changed.".into()
    };
    state.review_reason.clear();
}
pub(super) fn panel(app: &mut MzViewerApp, ui: &mut egui::Ui) {
    let revision = app.project.revision();
    let mcp_window = super::ai_status::connected(app);
    let mut state = std::mem::take(&mut app.ai);
    // Looking at the list answers the status bar notice.
    state.notice = None;
    if state.proposals.is_empty() {
        empty_state(ui, mcp_window);
    } else {
        ui.label("Nothing changes until you approve a proposal. Approving applies the change and records both reasons, the AI's and yours; rejecting changes nothing.");
    }
    ui.horizontal_wrapped(|ui| {
        if ui
            .small_button("Load a saved proposal file…")
            .on_hover_text("A proposal file saved from this window, or sent by a colleague")
            .clicked()
        {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("Proposal JSON", &["json"])
                .pick_file()
            {
                match std::fs::read(path)
                    .map_err(|e| e.to_string())
                    .and_then(|bytes| {
                        serde_json::from_slice::<Proposal>(&bytes).map_err(|e| e.to_string())
                    })
                    .and_then(|p| p.check_shape().map(|()| p).map_err(|e| e.message))
                {
                    Ok(proposal) => {
                        state.proposals.push(proposal);
                        state.selected = state.proposals.len() - 1;
                    }
                    Err(error) => state.message = error,
                }
            }
        }
    });
    if !state.proposals.is_empty() {
        proposal_list(app, ui, &mut state);
    }
    ui.collapsing("Prepare a new review suggestion", |ui| {
        suggestion_form(app, ui, &mut state, revision);
    });
    if state.pending.is_some() {
        ui.spinner();
        ui.label("Verifying shared-engine proposal…");
    }
    ui.label(&state.message);
    app.ai = state;
}
fn proposal_list(app: &mut MzViewerApp, ui: &mut egui::Ui, state: &mut State) {
    state.selected = state.selected.min(state.proposals.len() - 1);
    egui::ComboBox::from_label("Proposal")
        .selected_text(format!(
            "{} · {}",
            state.selected + 1,
            state
                .proposals
                .get(state.selected)
                .map_or(String::new(), summary)
        ))
        .show_ui(ui, |ui| {
            for (index, proposal) in state.proposals.iter().enumerate() {
                let decision = proposal.events.last().map(|e| e.decision.as_str());
                let text = format!(
                    "{} · {} · {}",
                    index + 1,
                    summary(proposal),
                    decision.unwrap_or("awaiting decision")
                );
                ui.selectable_value(&mut state.selected, index, text);
            }
        });
    let Some(proposal) = state.proposals.get(state.selected).cloned() else {
        return;
    };
    ui.label(format!("Proposed by: {}", proposal.proposer()));
    ui.label(format!("Reason: {}", proposal.reason));
    for evidence in &proposal.evidence {
        ui.label(format!("Evidence: {evidence}"));
    }
    if let Some(edit) = &proposal.peak_edit {
        peak_edit_summary(ui, edit);
        if let Some(problem) = peak_edit_status(app, ui.ctx(), &proposal) {
            ui.colored_label(ui.visuals().warn_fg_color, problem);
        }
    }
    let mut rows = vec![];
    diff(
        &proposal.before,
        &proposal.proposed,
        String::new(),
        &mut rows,
    );
    super::table::show(
        ui,
        "proposal_changes",
        &["Field", "Original", "Proposed"],
        &rows,
        None,
    );
    ui.small("All changed fields are available through filtering and scrolling; complete originals and proposals remain in the saved proposal JSON.");
    ui.label("Your reason (required to approve or reject)");
    ui.text_edit_singleline(&mut state.review_reason);
    let decided = !proposal.events.is_empty();
    let can_decide = state.pending.is_none() && !decided && !state.review_reason.trim().is_empty();
    ui.horizontal_wrapped(|ui| {
        let approve = ui
            .add_enabled(can_decide, egui::Button::new("Approve"))
            .clicked();
        let reject = ui
            .add_enabled(can_decide, egui::Button::new("Reject"))
            .clicked();
        if approve || reject {
            decide(app, state, &proposal, approve);
        }
        if proposal.request.is_some()
            && ui
                .add_enabled(
                    proposal.applied.is_some() && !state.review_reason.trim().is_empty(),
                    egui::Button::new("Prepare reversal"),
                )
                .clicked()
        {
            match crate::proposals::reversal(&proposal, REVIEWER, &state.review_reason) {
                Ok(request) => {
                    state.draft = serde_json::to_string_pretty(&request.operation).unwrap();
                    state.reason = state.review_reason.clone();
                    state.evidence = proposal.id.to_string();
                }
                Err(error) => state.message = error.to_string(),
            }
        }
        if let Some(request) = &proposal.request {
            if ui.button("Prepare revised suggestion").clicked() {
                state.draft = serde_json::to_string_pretty(&request.operation).unwrap();
                state.reason = proposal.reason.clone();
                state.evidence = proposal.evidence.join("\n");
            }
        }
        if ui.button("Save proposal and decisions…").clicked() {
            if let Some(path) = rfd::FileDialog::new().save_file() {
                use std::io::Write;
                state.message = serde_json::to_vec_pretty(&proposal)
                    .map_err(|e| e.to_string())
                    .and_then(|bytes| {
                        std::fs::OpenOptions::new()
                            .create_new(true)
                            .write(true)
                            .open(path)
                            .and_then(|mut f| f.write_all(&bytes))
                            .map_err(|e| e.to_string())
                    })
                    .err()
                    .unwrap_or_else(|| "Proposal saved".into());
            }
        }
    });
    if proposal.peak_edit.is_some() && proposal.applied.is_some() {
        ui.small("To undo an approved boundary change, use Reset to automatic on the result in Quantification and QC.");
    }
    for event in &proposal.events {
        ui.label(format!(
            "{} · {} · {}",
            event.decision, event.actor, event.reason
        ));
    }
}
/// A person's click on Approve or Reject.
fn decide(app: &mut MzViewerApp, state: &mut State, proposal: &Proposal, approve: bool) {
    if proposal.peak_edit.is_some() {
        decide_peak_edit(app, state, proposal, approve);
        return;
    }
    let Some(current) = current(app, &proposal.before) else {
        state.message =
            "This result is not in this window. Open its project before reviewing.".into();
        return;
    };
    let reason = state.review_reason.clone();
    let proposal = proposal.clone();
    // The result held in this window is the evidence: it must be exactly what the proposal was
    // made against, whatever revision a project is at.
    let revision = proposal.expected_project_revision;
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let _ = tx.send(crate::proposals::resolve(
            std::path::Path::new("-"),
            &proposal,
            &current,
            revision,
            approve,
            REVIEWER,
            &reason,
            &Default::default(),
        ));
    });
    state.pending = Some(rx);
    state.review_reason.clear();
}
/// The expert form that prepares a new engine suggestion from an operation.
fn suggestion_form(
    app: &mut MzViewerApp,
    ui: &mut egui::Ui,
    state: &mut State,
    revision: Option<u64>,
) {
    if ui
        .add_enabled(
            app.quant.suggestion().is_some(),
            egui::Button::new("Prepare selected concentration review"),
        )
        .clicked()
    {
        if let Some(operation) = app.quant.suggestion() {
            state.draft = serde_json::to_string_pretty(&operation).unwrap();
        }
    }
    ui.label("Revision-bound review operation");
    super::forms::guided::<Operation>(
        ui,
        "Proposed review controls",
        "Routine review fields match the selected concentration. Approval always requires your reason below.",
        &mut state.draft,
    );
    ui.collapsing("Advanced operation JSON — expert exact-operation audit", |ui| {
        ui.small("Routine fields are in the guided controls above. This text is retained so the exact proposed operation can be audited before approval.");
        ui.add(
            egui::TextEdit::multiline(&mut state.draft)
                .code_editor()
                .desired_rows(6)
                .desired_width(f32::INFINITY),
        );
    });
    ui.label("Proposal reason");
    ui.text_edit_singleline(&mut state.reason);
    ui.label("Evidence references (one per line)");
    ui.add(egui::TextEdit::multiline(&mut state.evidence).desired_rows(2));
    if ui
        .add_enabled(
            state.pending.is_none(),
            egui::Button::new("Preview and retain suggestion"),
        )
        .clicked()
    {
        match serde_json::from_str::<Operation>(&state.draft) {
            Ok(operation) => {
                let request = Request {
                    version: 1,
                    operation_id: Default::default(),
                    actor: "desktop-suggestion".into(),
                    operation,
                };
                let reason = state.reason.clone();
                let evidence = state.evidence.lines().map(str::to_owned).collect();
                let (tx, rx) = std::sync::mpsc::channel();
                std::thread::spawn(move || {
                    let _ = tx.send(
                        crate::proposals::prepare(
                            std::path::Path::new("-"),
                            request,
                            revision,
                            reason,
                            evidence,
                            &Default::default(),
                        )
                        .map(|p| (p, None)),
                    );
                });
                state.pending = Some(rx);
            }
            Err(error) => state.message = error.to_string(),
        }
    }
}

impl State {
    /// Puts a proposal an AI made into the review list and raises the status bar notice. Only
    /// the window's own code calls this, with proposals the engine or the window built.
    #[cfg(feature = "mcp")]
    pub(super) fn deliver(&mut self, proposal: Proposal) {
        let id = proposal.id;
        match self.proposals.iter().position(|old| old.id == id) {
            Some(index) => {
                if proposal.events.len() >= self.proposals[index].events.len() {
                    self.proposals[index] = proposal;
                }
                self.selected = index;
            }
            None => {
                self.proposals.push(proposal);
                self.selected = self.proposals.len() - 1;
            }
        }
        self.notice = Some(id);
    }
    /// Whether the status bar should say that the AI proposed a change that still awaits a decision.
    pub(super) fn has_notice(&self) -> bool {
        self.notice.is_some_and(|id| {
            self.proposals
                .iter()
                .any(|p| p.id == id && p.events.is_empty())
        })
    }
    /// Selects the proposal of the notice and clears the notice.
    pub(super) fn take_notice(&mut self) {
        if let Some(id) = self.notice.take() {
            if let Some(index) = self.proposals.iter().position(|p| p.id == id) {
                self.selected = index;
            }
        }
    }
    /// Asks the Audit package to open its "Review AI proposals" section.
    pub(super) fn reveal_selected(&mut self) {
        self.reveal = true;
    }
    /// True once, after [`Self::reveal_selected`].
    pub(super) fn take_reveal(&mut self) -> bool {
        std::mem::take(&mut self.reveal)
    }
    pub(super) fn busy(&self) -> bool {
        self.pending.is_some()
    }
    /// `(total_proposals, awaiting_decision)` — proposals without any
    /// recorded approve/reject event still need scientist review.
    pub(super) fn review_summary(&self) -> (usize, usize) {
        let awaiting = self
            .proposals
            .iter()
            .filter(|p| p.events.is_empty())
            .count();
        (self.proposals.len(), awaiting)
    }
    /// The proposals in the list, for tests.
    #[cfg(all(test, feature = "mcp"))]
    pub(super) fn list(&self) -> &[Proposal] {
        &self.proposals
    }
    /// Types the person's reason, for tests.
    #[cfg(all(test, feature = "mcp"))]
    pub(super) fn set_review_reason(&mut self, text: &str) {
        self.review_reason = text.into();
    }
    /// The line under the review section, for tests.
    #[cfg(all(test, feature = "mcp"))]
    pub(super) fn message(&self) -> &str {
        &self.message
    }
    /// The proposal the review section shows, for tests.
    #[cfg(all(test, feature = "mcp"))]
    pub(super) fn selected_id(&self) -> Option<uuid::Uuid> {
        self.proposals.get(self.selected).map(|p| p.id)
    }
}

impl State {
    pub(super) fn merge_project_reviews(&mut self, proposals: &[Proposal]) {
        for proposal in proposals {
            if let Some(old) = self.proposals.iter_mut().find(|old| old.id == proposal.id) {
                if proposal.events.len() > old.events.len() {
                    *old = proposal.clone();
                }
            } else {
                self.proposals.push(proposal.clone());
            }
        }
    }
}
