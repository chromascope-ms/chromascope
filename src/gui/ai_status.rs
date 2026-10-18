//! The AI call log and the proposal notice in the status bar of the chromascope-mcp window.
//!
//! Only a window that has an MCP bridge (the one an AI client started) shows anything here;
//! chromascope.exe has no AI connected and draws nothing. The log is a floating window, so it
//! stays open whichever workspace the AI switches to.
use super::MzViewerApp;
use eframe::egui;

/// Whether this window is the one an AI client is connected to.
pub(super) fn connected(app: &MzViewerApp) -> bool {
    #[cfg(feature = "mcp")]
    {
        app.remote.is_some()
    }
    #[cfg(not(feature = "mcp"))]
    {
        let _ = app;
        false
    }
}

/// What the status bar says about the AI: the connection with its request count and a link to
/// the log, or "AI is working…" while a request runs, and the notice of a new proposal.
pub(super) fn show(app: &mut MzViewerApp, ui: &mut egui::Ui) {
    #[cfg(feature = "mcp")]
    if let Some(bridge) = app.remote.as_mut() {
        let requests = bridge.history.len();
        if bridge.working() {
            ui.spinner();
            ui.small("AI is working…");
        } else {
            ui.small(format!(
                "AI connected · {requests} {} ·",
                if requests == 1 { "request" } else { "requests" }
            ));
        }
        if ui
            .small_button("View log")
            .on_hover_text("Every request the AI made in this window, with its exact content")
            .clicked()
        {
            bridge.log_open = true;
        }
    }
    notice(app, ui);
}

/// "The AI proposed a change · Review": shown from the moment a proposal arrives until it has
/// been looked at. Review opens the Audit package with that proposal selected.
fn notice(app: &mut MzViewerApp, ui: &mut egui::Ui) {
    if !app.ai.has_notice() {
        return;
    }
    ui.separator();
    ui.small("The AI proposed a change ·");
    if ui
        .small_button("Review")
        .on_hover_text("Open Review AI proposals in the Audit package")
        .clicked()
    {
        app.ai.take_notice();
        super::workflow::go(app, ui.ctx(), super::workflow::Destination::AuditPackage);
        app.ai.reveal_selected();
    }
}

/// The status line of the workspaces that have no status bar of their own (Quantification and
/// QC): the update notice and the AI indicator on one bar, drawn only when there is something.
pub(super) fn bar(app: &mut MzViewerApp, ctx: &egui::Context) {
    let update = super::settings::with_update_prefs(|prefs| app.updates.notice(prefs).is_some());
    if !update && !connected(app) && !app.ai.has_notice() {
        return;
    }
    egui::TopBottomPanel::bottom("workspace_status")
        .frame(super::theme::bar_frame(ctx, 4).fill(ctx.style().visuals.faint_bg_color))
        .show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                super::update::show_notice(&app.updates, ui);
                show(app, ui);
            });
        });
}

/// The floating window behind "View log". Drawn every frame whatever the workspace.
pub(super) fn log_window(app: &mut MzViewerApp, ctx: &egui::Context) {
    #[cfg(feature = "mcp")]
    if let Some(bridge) = app.remote.as_mut() {
        if !bridge.log_open {
            return;
        }
        let mut open = true;
        egui::Window::new("AI call log")
            .id(egui::Id::new("ai_call_log"))
            .open(&mut open)
            .default_width(560.0)
            .default_height(480.0)
            .vscroll(true)
            .show(ctx, |ui| log_contents(bridge, ui));
        bridge.log_open = open;
    }
    #[cfg(not(feature = "mcp"))]
    let _ = (app, ctx);
}

#[cfg(feature = "mcp")]
fn log_contents(bridge: &crate::mcp::GuiBridge, ui: &mut egui::Ui) {
    super::widgets::section_title(ui, "What this window allows");
    let yes_no = |on: bool| if on { "allowed" } else { "blocked" };
    let profile = bridge.profile.lock().map(|p| p.name()).unwrap_or("full");
    ui.label(format!(
        "--allow-changes: {} · --allow-exports: {} · --allow-project-writes: {} · tool profile: {profile}",
        yes_no(bridge.policy.allow_changes),
        yes_no(bridge.policy.allow_exports),
        yes_no(bridge.policy.allow_project_writes),
    ));
    ui.label("Folders the AI may read (--allow-root):");
    for root in bridge.policy.roots() {
        ui.monospace(root.display().to_string());
    }
    ui.add_space(4.0);
    ui.small("This log is kept only while this window is open. For a permanent record, ask the assistant for session_audit or prepare_analysis_report.");
    ui.separator();
    super::widgets::section_title(ui, "Requests");
    ui.label(format!("{} requests so far", bridge.history.len()));
    if bridge.working() {
        ui.horizontal(|ui| {
            ui.spinner();
            ui.label("AI is working…");
        });
    }
    for entry in bridge.history.iter().rev().take(100) {
        let success = entry["success"].as_bool().unwrap_or(false);
        let request = &entry["request"];
        let tool = request["operation"].as_str().unwrap_or("MCP request");
        ui.collapsing(
            format!(
                "{} · {} · {}",
                entry["sequence"],
                tool,
                if success { "Completed" } else { "Failed" }
            ),
            |ui| {
                if let Some(error) = entry["error"].as_str() {
                    ui.colored_label(ui.visuals().error_fg_color, error);
                }
                ui.label("Request details (data, parameters and requested changes)");
                ui.monospace(serde_json::to_string_pretty(request).unwrap_or_default());
            },
        );
    }
    ui.small("Showing the latest 100 requests.");
}
