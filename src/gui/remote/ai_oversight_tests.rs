//! What the person sees of the AI in the window: the status bar indicator, the call log and the
//! proposals an AI delivers. Driven through the real MCP protocol like `protocol_tests`, with
//! real egui frames for everything the person looks at and clicks.
use super::protocol_tests::{fixture_method, run_batch, start_request, wait_for_job, Desktop};
use super::*;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

include!("../../../tests/fixtures/targeted_support.rs");

static CLOCK: AtomicU64 = AtomicU64::new(0);
thread_local! {
    /// Every frame of this test, kept only when pictures were asked for.
    static TRAIL: std::cell::RefCell<Vec<egui::FullOutput>> = const { std::cell::RefCell::new(Vec::new()) };
}
/// Saves what the test has drawn so far as `<CHROMASCOPE_PREVIEW_DIR>/<name>.png`, for a look at
/// the layout. Does nothing unless that variable is set.
fn preview(ctx: &egui::Context, name: &str) {
    let Ok(directory) = std::env::var("CHROMASCOPE_PREVIEW_DIR") else {
        return;
    };
    let frames = TRAIL.with(|t| t.borrow().clone());
    super::super::test_render::save(
        ctx,
        frames,
        &std::path::Path::new(&directory).join(format!("{name}.png")),
        egui::vec2(1400.0, 900.0),
    );
}

/// One frame of what these tests look at: the status bar, the Audit package when it is open
/// and the call log window, at a size where nothing needs scrolling.
fn draw(ctx: &egui::Context, app: &mut MzViewerApp, events: Vec<egui::Event>) -> egui::FullOutput {
    ctx.data_mut(|d| d.insert_temp(egui::Id::new("central_workspaces"), true));
    let output = draw_frame(ctx, app, events);
    if std::env::var_os("CHROMASCOPE_PREVIEW_DIR").is_some() {
        TRAIL.with(|t| t.borrow_mut().push(output.clone()));
    }
    output
}
fn draw_frame(
    ctx: &egui::Context,
    app: &mut MzViewerApp,
    events: Vec<egui::Event>,
) -> egui::FullOutput {
    ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1400.0, 2600.0),
            )),
            time: Some(CLOCK.fetch_add(1, Ordering::SeqCst) as f64 * 0.3),
            events,
            ..Default::default()
        },
        |ctx| {
            super::super::workbench::status(app, ctx);
            super::super::delivery::show(app, ctx);
            super::super::ai_status::log_window(app, ctx);
        },
    )
}
fn frame(desktop: &mut Desktop, events: Vec<egui::Event>) -> egui::FullOutput {
    let ctx = desktop.ctx.clone();
    draw(&ctx, &mut desktop.app, events)
}

fn shows(output: &egui::FullOutput, text: &str) -> bool {
    super::super::test_render::text_center(&output.shapes, text).is_some()
}

/// Clicks the control with this text: two frames to lay it out, one to press and release.
fn click_in(ctx: &egui::Context, app: &mut MzViewerApp, label: &str) {
    draw(ctx, app, vec![]);
    let output = draw(ctx, app, vec![]);
    let pos = super::super::test_render::text_center(&output.shapes, label)
        .unwrap_or_else(|| panic!("{label:?} is not on screen"));
    let button = egui::PointerButton::Primary;
    let modifiers = egui::Modifiers::NONE;
    draw(
        ctx,
        app,
        vec![
            egui::Event::PointerMoved(pos),
            egui::Event::PointerButton {
                pos,
                button,
                pressed: true,
                modifiers,
            },
            egui::Event::PointerButton {
                pos,
                button,
                pressed: false,
                modifiers,
            },
        ],
    );
}
fn click(desktop: &mut Desktop, label: &str) {
    let ctx = desktop.ctx.clone();
    click_in(&ctx, &mut desktop.app, label);
}

#[test]
fn the_indicator_appears_only_with_a_bridge_and_counts_the_requests() {
    let mut desktop = Desktop::start(vec![], true, false, false);
    let output = frame(&mut desktop, vec![]);
    assert!(shows(&output, "AI connected · 0 requests ·"));
    assert!(shows(&output, "View log"));
    for tool in ["list_datasets", "gui_state", "list_datasets"] {
        desktop.tool(tool, json!({}));
    }
    let logged = desktop.app.remote.as_ref().unwrap().history.len();
    assert_eq!(logged, 3, "one log entry per request");
    let output = frame(&mut desktop, vec![]);
    assert!(
        shows(&output, "AI connected · 3 requests ·"),
        "the count in the status bar is the number of log entries"
    );
    desktop.tool("list_datasets", json!({}));
    let output = frame(&mut desktop, vec![]);
    assert!(shows(&output, "AI connected · 4 requests ·"));
    // A request that has arrived and is not answered yet shows as work in progress, and the
    // counter returns to zero once every reply has been sent.
    let active = desktop.app.remote.as_ref().unwrap().active.clone();
    assert_eq!(active.load(Ordering::SeqCst), 0);
    active.fetch_add(1, Ordering::SeqCst);
    let output = frame(&mut desktop, vec![]);
    assert!(shows(&output, "AI is working…"));
    assert!(!shows(&output, "AI connected · 4 requests ·"));
    active.fetch_sub(1, Ordering::SeqCst);
    let output = frame(&mut desktop, vec![]);
    assert!(!shows(&output, "AI is working…"));
    // The Quantification workspace has no status bar; its own bar carries the same line.
    let screen = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1280.0, 800.0));
    let ctx = desktop.ctx.clone();
    let bar = |app: &mut MzViewerApp| {
        ctx.run(
            egui::RawInput {
                screen_rect: Some(screen),
                ..Default::default()
            },
            |ctx| super::super::ai_status::bar(app, ctx),
        )
    };
    let output = bar(&mut desktop.app);
    assert!(shows(&output, "AI connected · 4 requests ·") && shows(&output, "View log"));
    desktop.finish();
    let mut plain = MzViewerApp::default();
    let output = bar(&mut plain);
    assert!(!shows(&output, "View log"), "no bar without MCP");

    // chromascope.exe: no bridge, no AI line, no log link, whatever else the window shows.
    let ctx = egui::Context::default();
    assert!(plain.remote.is_none());
    let output = draw(&ctx, &mut plain, vec![]);
    for absent in ["View log", "AI is working…", "AI call log", "Review"] {
        assert!(!shows(&output, absent), "{absent:?} appears without MCP");
    }
    assert!(
        shows(&output, "0 files loaded"),
        "the rest of the status bar is still drawn"
    );
    assert!(!output
        .shapes
        .iter()
        .any(|clipped| format!("{:?}", clipped.shape).contains("AI connected")));
}

#[test]
fn the_call_log_opens_from_the_status_bar_and_survives_workspace_switches() {
    let mut desktop = Desktop::start(vec![], true, true, true);
    desktop.open_fixture();
    desktop.tool("list_datasets", json!({}));
    assert!(!shows(&frame(&mut desktop, vec![]), "AI call log"));
    click(&mut desktop, "View log");
    assert!(desktop.app.remote.as_ref().unwrap().log_open);
    let output = frame(&mut desktop, vec![]);
    for text in [
        "AI call log",
        "--allow-changes: allowed · --allow-exports: allowed · --allow-project-writes: allowed · tool profile: full",
        "Folders the AI may read (--allow-root):",
        "This log is kept only while this window is open. For a permanent record, ask the assistant for session_audit or prepare_analysis_report.",
        "Showing the latest 100 requests.",
    ] {
        assert!(shows(&output, text), "the log window lacks {text:?}");
    }
    // The newest request is first and its details are one click away.
    let sequence = desktop.app.remote.as_ref().unwrap().history.len() - 1;
    assert!(shows(
        &output,
        &format!("{sequence} · list_datasets · Completed")
    ));
    let first_root = desktop.app.remote.as_ref().unwrap().policy.roots()[0]
        .display()
        .to_string();
    assert!(shows(&output, &first_root), "the roots are listed");
    // Whatever workspace the AI shows, the log window stays.
    for workspace in [
        "quantification",
        "audit_package",
        "method_editor",
        "data_explorer",
    ] {
        desktop.tool("show_workspace", json!({"workspace":workspace}));
        assert!(desktop.app.remote.as_ref().unwrap().log_open, "{workspace}");
        let output = frame(&mut desktop, vec![]);
        assert!(shows(&output, "AI call log"), "gone after {workspace}");
        // And it records the switch that was just made.
        let latest = desktop.app.remote.as_ref().unwrap().history.len() - 1;
        assert!(
            shows(&output, &format!("{latest} · show_workspace · Completed")),
            "{workspace}"
        );
    }
    // The person closes it; it stays closed until View log is clicked again.
    desktop.app.remote.as_mut().unwrap().log_open = false;
    assert!(!shows(&frame(&mut desktop, vec![]), "AI call log"));
    desktop.finish();

    // A window started without permissions says so.
    let mut read_only = Desktop::start(vec![], false, false, false);
    read_only.app.remote.as_mut().unwrap().log_open = true;
    // A new window needs a frame to find its size before it shows its contents.
    frame(&mut read_only, vec![]);
    let output = frame(&mut read_only, vec![]);
    assert!(shows(
        &output,
        "--allow-changes: blocked · --allow-exports: blocked · --allow-project-writes: blocked · tool profile: full"
    ));
    read_only.finish();
}

/// The `stable_id` of a dataset, once the window has hashed its source.
fn stable_id(desktop: &mut Desktop, id: u64) -> String {
    for _ in 0..2000 {
        let listed = desktop.tool("list_datasets", json!({}));
        let found = listed["datasets"]
            .as_array()
            .unwrap()
            .iter()
            .find(|d| d["dataset_id"] == id)
            .and_then(|d| d["stable_id"].as_str().map(str::to_owned));
        if let Some(stable) = found {
            return stable;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    panic!("the dataset never got a stable id");
}
/// A batch quantification result in the window and boundaries inside its trace, as the AI
/// would read them from quantification_status and quantification_trace.
struct Batch {
    desktop: Desktop,
    stable: String,
    start: f64,
    end: f64,
    first: Value,
}
fn batch_in_window(changes: bool, project_writes: bool, extra: Vec<PathBuf>) -> Batch {
    let mut desktop = Desktop::start(extra, changes, false, project_writes);
    let id = desktop.open_fixture();
    let method = fixture_method(&mut desktop, id, "Proposals");
    let status = run_batch(&mut desktop, id, &method);
    let stable = stable_id(&mut desktop, id);
    // The AI finds the automatic window wider than the peak and proposes a tighter one.
    let peak = &status["results"][0]["peak"];
    let (a, z) = (
        peak["start"]
            .as_f64()
            .expect("the fixture has an automatic peak"),
        peak["end"].as_f64().unwrap(),
    );
    let width = z - a;
    Batch {
        desktop,
        stable,
        start: a + 0.2 * width,
        end: z - 0.2 * width,
        first: status["results"][0].clone(),
    }
}
impl Batch {
    /// What the AI says to propose the boundaries.
    fn propose(&mut self) -> Value {
        self.desktop.tool(
            "review_quantification",
            json!({
                "result_index": 0,
                "dataset_id": self.stable,
                "start_minutes": self.start,
                "end_minutes": self.end,
                "propose": true,
                "reason": "The automatic window includes the shoulders of the peak",
                "evidence": ["quantification_trace 0: the outer 20 % of the window on each side is baseline"],
            }),
        )
    }
    fn result(&mut self) -> Value {
        self.desktop.tool("quantification_status", json!({}))["results"][0].clone()
    }
}

#[test]
fn an_ai_boundary_proposal_waits_for_the_person_and_applies_only_on_approval() {
    let mut batch = batch_in_window(true, false, vec![]);
    let original = batch.first.clone();
    let reply = batch.propose();
    // The AI is told what happened: a proposal, nothing changed, where the person decides.
    assert_eq!(reply["changed"], false, "{reply}");
    assert_eq!(reply["status"], "awaiting_decision");
    assert!(reply["where"]
        .as_str()
        .unwrap()
        .contains("Review AI proposals"));
    assert_eq!(reply["original"]["start"], original["peak"]["start"]);
    assert_eq!(reply["original"]["area"], original["peak"]["area"]);
    assert_ne!(reply["proposed"]["area"], reply["original"]["area"]);
    batch.desktop.pump();
    // It is in the window's list, undecided, with the dataset it was made for.
    assert_eq!(batch.desktop.app.ai.list().len(), 1);
    let proposal = batch.desktop.app.ai.list()[0].clone();
    assert_eq!(proposal.id.to_string(), reply["proposal_id"]);
    assert!(proposal.events.is_empty() && proposal.applied.is_none());
    assert_eq!(proposal.proposer(), "chromascope-mcp");
    let edit = proposal.peak_edit.clone().unwrap();
    assert_eq!(
        edit.dataset_stable_id.as_deref(),
        Some(batch.stable.as_str())
    );
    assert_eq!(edit.original.start, original["peak"]["start"].as_f64());
    assert_eq!(proposal.evidence.len(), 1);
    // Nothing changed yet, and the window was not switched.
    let unchanged = batch.result();
    assert_eq!(unchanged["peak"], original["peak"]);
    assert_eq!(unchanged["status"], original["status"]);
    assert!(unchanged["boundary_decision"].is_null());
    assert!(!batch.desktop.app.delivery.open && !batch.desktop.app.quant.active);
    // The status bar says so, and the log has the call.
    let output = frame(&mut batch.desktop, vec![]);
    let ctx = batch.desktop.ctx.clone();
    preview(&ctx, "status-bar-notice");
    assert!(shows(&output, "The AI proposed a change ·"), "notice");
    assert!(shows(&output, "Review"));
    assert!(batch
        .desktop
        .app
        .remote
        .as_ref()
        .unwrap()
        .history
        .iter()
        .any(|e| e["request"]["operation"] == "review_quantification"));
    // The person clicks Review: the Audit package opens with that proposal selected.
    click(&mut batch.desktop, "Review");
    assert!(batch.desktop.app.delivery.open && !batch.desktop.app.quant.active);
    assert_eq!(batch.desktop.app.ai.selected_id(), Some(proposal.id));
    assert!(!batch.desktop.app.ai.has_notice());
    frame(&mut batch.desktop, vec![]);
    let output = frame(&mut batch.desktop, vec![]);
    assert!(!shows(&output, "The AI proposed a change ·"));
    for text in [
        "Review AI proposals",
        "Proposed by: chromascope-mcp",
        "Reason: The automatic window includes the shoulders of the peak",
        "Evidence: quantification_trace 0: the outer 20 % of the window on each side is baseline",
        "Approve",
        "Reject",
        "Load a saved proposal file…",
    ] {
        assert!(shows(&output, text), "the section lacks {text:?}");
    }
    preview(&ctx, "review-ai-proposals");
    // Approve needs the person's own reason.
    click(&mut batch.desktop, "Approve");
    assert!(
        batch.desktop.app.ai.list()[0].events.is_empty(),
        "no reason, no decision"
    );
    batch
        .desktop
        .app
        .ai
        .set_review_reason("Boundaries match the visible peak");
    click(&mut batch.desktop, "Approve");
    let decided = batch.desktop.app.ai.list()[0].clone();
    assert_eq!(
        decided.events.len(),
        1,
        "{}",
        batch.desktop.app.ai.message()
    );
    assert_eq!(decided.events[0].decision, "approved");
    assert_eq!(decided.events[0].actor, "desktop-reviewer");
    assert_eq!(
        decided.events[0].reason,
        "Boundaries match the visible peak"
    );
    assert_eq!(
        decided.reason,
        "The automatic window includes the shoulders of the peak"
    );
    // The result has the new boundaries, as after a manual edit, with both reasons recorded.
    let applied = batch.result();
    let proposed = edit.proposed.clone();
    assert!((applied["peak"]["start"].as_f64().unwrap() - proposed.start.unwrap()).abs() < 1e-9);
    assert!((applied["peak"]["end"].as_f64().unwrap() - proposed.end.unwrap()).abs() < 1e-9);
    let area = proposed.area.unwrap();
    assert!((applied["peak"]["area"].as_f64().unwrap() - area).abs() <= 1e-9 * area.abs().max(1.0));
    assert_eq!(applied["status"], "Manual");
    assert_eq!(
        applied["automatic"], original["automatic"],
        "the automatic peak is kept"
    );
    let decision = &applied["boundary_decision"];
    assert_eq!(decision["proposed_by"], "chromascope-mcp");
    assert_eq!(
        decision["proposal_reason"],
        "The automatic window includes the shoulders of the peak"
    );
    assert_eq!(decision["approved_by"], "desktop-reviewer");
    assert_eq!(
        decision["approval_reason"],
        "Boundaries match the visible peak"
    );
    assert_eq!(decision["proposal_id"], reply["proposal_id"]);
    // The same value a person's manual edit gives for those boundaries.
    let manual = batch.desktop.tool(
        "review_quantification",
        json!({"result_index":0,"start_minutes":proposed.start,"end_minutes":proposed.end}),
    );
    assert_eq!(manual["results"][0]["peak"], applied["peak"]);
    assert_eq!(manual["results"][0]["status"], "Manual");
    assert!(
        manual["results"][0]["boundary_decision"].is_null(),
        "a manual edit is not an approved proposal"
    );
    // Reset to automatic still restores the original peak and clears the mark.
    let reset = batch
        .desktop
        .tool("reset_quantification", json!({"result_index":0}));
    assert_eq!(reset["results"][0]["peak"], original["automatic"]);
    assert_eq!(reset["results"][0]["status"], original["automatic_status"]);
    assert!(reset["results"][0]["boundary_decision"].is_null());
    // The decision stays on the proposal, and a decided proposal cannot be decided again.
    click(&mut batch.desktop, "Reject");
    assert_eq!(batch.desktop.app.ai.list()[0].events.len(), 1);
    batch.desktop.finish();
}

#[test]
fn rejecting_a_boundary_proposal_changes_nothing() {
    let mut batch = batch_in_window(true, false, vec![]);
    let original = batch.first.clone();
    batch.propose();
    batch.desktop.pump();
    click(&mut batch.desktop, "Review");
    batch
        .desktop
        .app
        .ai
        .set_review_reason("The tail is part of the peak");
    click(&mut batch.desktop, "Reject");
    let proposal = batch.desktop.app.ai.list()[0].clone();
    assert_eq!(proposal.events.len(), 1);
    assert_eq!(proposal.events[0].decision, "rejected");
    assert_eq!(proposal.events[0].reason, "The tail is part of the peak");
    assert!(proposal.applied.is_none());
    let after = batch.result();
    assert_eq!(after["peak"], original["peak"]);
    assert_eq!(after["status"], original["status"]);
    assert!(after["boundary_decision"].is_null());
    assert!(!shows(
        &frame(&mut batch.desktop, vec![]),
        "The AI proposed a change ·"
    ));
    batch.desktop.finish();
}

#[test]
fn a_boundary_proposal_is_refused_when_the_result_changed_after_it_was_made() {
    let mut batch = batch_in_window(true, false, vec![]);
    batch.propose();
    batch.desktop.pump();
    // The person (or the AI, directly) edits the same result before approving.
    let (start, end) = (batch.start + 0.002, batch.end);
    let manual = batch.desktop.tool(
        "review_quantification",
        json!({"result_index":0,"start_minutes":start,"end_minutes":end}),
    );
    let edited = manual["results"][0].clone();
    click(&mut batch.desktop, "Review");
    // The panel says so before anything is clicked.
    frame(&mut batch.desktop, vec![]);
    let output = frame(&mut batch.desktop, vec![]);
    assert!(
        shows(&output, "The result changed after the AI made this proposal, so it can no longer be approved. Ask for a new proposal."),
        "the stale proposal is flagged"
    );
    batch
        .desktop
        .app
        .ai
        .set_review_reason("I would like to approve it anyway");
    click(&mut batch.desktop, "Approve");
    let proposal = batch.desktop.app.ai.list()[0].clone();
    assert!(
        proposal.events.is_empty(),
        "a stale proposal is not decided"
    );
    assert!(
        batch
            .desktop
            .app
            .ai
            .message()
            .contains("changed after the proposal"),
        "{}",
        batch.desktop.app.ai.message()
    );
    let after = batch.result();
    assert_eq!(
        after["peak"], edited["peak"],
        "the person's edit is untouched"
    );
    assert!(after["boundary_decision"].is_null());
    batch.desktop.finish();
}

#[test]
fn a_boundary_proposal_needs_what_a_person_would_need_and_cannot_be_decided_by_the_ai() {
    let mut batch = batch_in_window(true, false, vec![]);
    let base = json!({"result_index":0,"dataset_id":batch.stable,"start_minutes":batch.start,"end_minutes":batch.end,"propose":true,"reason":"r","evidence":["e"]});
    for (key, wanted) in [
        ("reason", "reason"),
        ("evidence", "evidence"),
        ("start_minutes", "start_minutes"),
        ("end_minutes", "end_minutes"),
    ] {
        let mut arguments = base.clone();
        arguments.as_object_mut().unwrap().remove(key);
        let error = batch.desktop.tool_error("review_quantification", arguments);
        let message = error["message"].as_str().unwrap().to_owned();
        assert!(message.contains(wanted), "{key}: {message}");
    }
    let mut wrong = base.clone();
    wrong["reviewed"] = json!(true);
    let error = batch.desktop.tool_error("review_quantification", wrong);
    assert!(error["message"]
        .as_str()
        .unwrap()
        .contains("a person does that"));
    let mut elsewhere = base.clone();
    elsewhere["dataset_id"] = json!(99);
    let error = batch.desktop.tool_error("review_quantification", elsewhere);
    assert!(
        error["message"]
            .as_str()
            .unwrap()
            .contains("Unknown dataset"),
        "{error}"
    );
    let mut unknown = base.clone();
    unknown["result_index"] = json!(7);
    let error = batch.desktop.tool_error("review_quantification", unknown);
    assert!(error["message"]
        .as_str()
        .unwrap()
        .contains("Unknown result index"));
    let mut backwards = base.clone();
    backwards["start_minutes"] = json!(batch.end);
    backwards["end_minutes"] = json!(batch.start);
    batch.desktop.tool_error("review_quantification", backwards);
    batch.desktop.pump();
    assert!(
        batch.desktop.app.ai.list().is_empty(),
        "refused requests leave no proposal"
    );
    // The AI has no tool that decides on a boundary proposal in the window: the project tools
    // refuse without --allow-project-writes, and the window shows no way for it to click.
    batch.propose();
    batch.desktop.pump();
    let proposal = batch.desktop.app.ai.list()[0].clone();
    let supplied = serde_json::to_value(&proposal).unwrap();
    let directory = env!("CARGO_MANIFEST_DIR");
    let dataset = uuid::Uuid::nil().to_string();
    let error = batch.desktop.tool_error(
        "resolve_review_proposal",
        json!({"directory":directory,"dataset_id":dataset,"proposal":supplied,"approve":true,"actor":"chromascope-mcp","reason":"I approve my own proposal"}),
    );
    assert!(
        error["message"]
            .as_str()
            .unwrap()
            .contains("project writes"),
        "{error}"
    );
    let error = batch.desktop.tool_error(
        "queue_review_proposal",
        json!({"directory":directory,"dataset_id":dataset,"proposal":supplied}),
    );
    assert!(
        error["message"]
            .as_str()
            .unwrap()
            .contains("Project writes"),
        "{error}"
    );
    batch.desktop.pump();
    assert!(batch.desktop.app.ai.list()[0].events.is_empty());
    assert_eq!(batch.result()["peak"], batch.first["peak"]);
    // The decision of the window path needs somebody other than the proposer.
    let again = crate::proposals::resolve_peak_edit(
        &proposal,
        &proposal.before,
        true,
        "chromascope-mcp",
        "self",
    );
    assert!(again.is_err());
    batch.desktop.finish();
}

#[test]
fn proposing_boundaries_needs_no_allow_changes_but_changing_them_does() {
    use crate::mcp::Command;
    let mut read_only = Desktop::start(vec![], false, false, false);
    let bridge = read_only.app.remote.take().unwrap();
    let command = |propose| Command::ReviewQuantification {
        result_index: 0,
        start_minutes: Some(1.0),
        end_minutes: Some(2.0),
        reviewed: false,
        propose,
        dataset_id: None,
        reason: Some("r".into()),
        evidence: vec!["e".into()],
    };
    assert!(read_only
        .app
        .remote_validate(&command(true), &bridge)
        .is_ok());
    let error = read_only
        .app
        .remote_validate(&command(false), &bridge)
        .unwrap_err();
    assert!(error.contains("--allow-changes"), "{error}");
    read_only.app.remote = Some(bridge);
    read_only.finish();
}

/// Opens the project the way chromascope.exe does and waits until it is in the window.
fn open_without_mcp(root: PathBuf) -> (egui::Context, MzViewerApp) {
    let ctx = egui::Context::default();
    let mut plain = MzViewerApp::default();
    super::super::project_workbench::begin_open(&mut plain, root);
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(60);
    while !plain.project.has_project() {
        assert!(
            std::time::Instant::now() < deadline,
            "the project never opened"
        );
        super::super::project_workbench::poll(&mut plain, &ctx);
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    assert!(plain.remote.is_none());
    (ctx, plain)
}

#[test]
fn a_boundary_proposal_saved_in_a_project_is_reviewed_later_in_a_window_without_mcp() {
    let temp = tempfile::tempdir().unwrap();
    let mut batch = batch_in_window(true, true, vec![temp.path().to_owned()]);
    let original = batch.first.clone();
    let reply = batch.propose();
    batch.desktop.pump();
    // With --allow-project-writes the AI can save the project: the proposal goes in with it.
    let file = temp.path().join("review.json");
    batch.desktop.tool("create_project", json!({"path":file}));
    let root = temp.path().join("review_data");
    let stored = crate::project::Project::open(&root).unwrap();
    assert_eq!(stored.review_proposals.len(), 1);
    assert_eq!(
        stored.review_proposals[0].id.to_string(),
        reply["proposal_id"]
    );
    assert!(stored.review_proposals[0].events.is_empty());
    // Even with --allow-project-writes the project tools do not decide on a window result: a
    // boundary proposal is approved in the window that holds the batch.
    batch.desktop.tool("review_quantification", json!({"result_index":0,"dataset_id":batch.stable,"start_minutes":batch.start + 0.002,"end_minutes":batch.end,"propose":true,"reason":"r2","evidence":["e2"]}));
    batch.desktop.pump();
    let second = batch.desktop.app.ai.list()[1].clone();
    let dataset = stored.sources[0].id.0.to_string();
    for (tool, arguments) in [
        (
            "queue_review_proposal",
            json!({"directory":root,"dataset_id":dataset,"proposal":second}),
        ),
        (
            "resolve_review_proposal",
            json!({"directory":root,"dataset_id":dataset,"proposal":second,"approve":true,"actor":"chromascope-mcp","reason":"I approve my own proposal"}),
        ),
    ] {
        let error = batch.desktop.tool_error(tool, arguments);
        let message = error["message"].as_str().unwrap_or_default();
        assert!(
            message.contains("approve it in the window"),
            "{tool}: {error}"
        );
    }
    assert!(
        crate::project::Project::open(&root)
            .unwrap()
            .review_proposals
            .len()
            == 1
    );
    batch.desktop.finish();
    // Later, in chromascope.exe: no MCP, the project is opened, the proposal is waiting.
    let (ctx, mut plain) = open_without_mcp(root);
    assert_eq!(plain.ai.list().len(), 1);
    assert_eq!(plain.ai.review_summary(), (1, 1));
    plain.delivery.open = true;
    draw(&ctx, &mut plain, vec![]);
    let output = draw(&ctx, &mut plain, vec![]);
    assert!(shows(
        &output,
        "Reason: The automatic window includes the shoulders of the peak"
    ));
    assert!(shows(&output, "Approve") && shows(&output, "Reject"));
    assert!(!shows(&output, "No AI proposals yet"));
    assert!(!shows(&output, "View log"));
    // Approving there applies the boundaries to the restored batch.
    plain.ai.set_review_reason("Checked in the saved project");
    click_in(&ctx, &mut plain, "Approve");
    assert_eq!(plain.ai.list()[0].events.len(), 1, "{}", plain.ai.message());
    let applied = super::super::quant::remote_status(&plain);
    assert_eq!(applied["results"][0]["status"], "Manual");
    assert_eq!(applied["results"][0]["automatic"], original["automatic"]);
    assert_eq!(
        applied["results"][0]["boundary_decision"]["approval_reason"],
        "Checked in the saved project"
    );
}

/// A calibration run on the shared engine and shown in the window, and the proposal an AI makes
/// to reject one of its results.
fn targeted_proposal(
    desktop: &mut Desktop,
    source: &str,
    batch: crate::targeted::BatchRequest,
) -> (crate::targeted::BatchResult, Value) {
    let started = desktop.tool(
        "start_analysis",
        json!({"path":source,"request":start_request(crate::domain::Operation::TargetedBatch{batch})}),
    );
    let job = started["job_id"].as_str().unwrap().to_owned();
    wait_for_job(desktop, &job);
    desktop.pump_until(10, |d| d.app.quant.has_targeted_batch());
    let result = desktop.app.quant.latest_targeted().unwrap().clone();
    let reviewed = result
        .results
        .iter()
        .find(|row| row.sample == "raw6")
        .expect("the unknown sample is in the batch");
    let (sample, target) = (reviewed.sample.clone(), reviewed.target.clone());
    let reply = desktop.tool(
        "prepare_review_proposal",
        json!({
            "path": source,
            "request": start_request(crate::domain::Operation::ReviewTargeted {
                expected_revision: result.reviews.len(),
                batch: Box::new(result.clone()),
                sample,
                target,
                accepted: false,
                reason: "Qualifier ratio is outside the range".into(),
            }),
            "reason": "The qualifier ratio of this injection is outside the accepted range",
            "evidence": ["raw6 qualifier ratio from analysis_result"],
        }),
    );
    desktop.pump();
    (result, reply)
}

#[test]
fn an_engine_proposal_reaches_the_window_on_its_own_and_is_applied_only_on_approval() {
    let temp = tempfile::tempdir().unwrap();
    let request = raw_request(temp.path());
    let source = request.samples[0].source.clone();
    let mut desktop = Desktop::start(vec![temp.path().to_owned()], true, false, false);
    let (batch, reply) = targeted_proposal(&mut desktop, &source, request);
    // Delivered without anything being saved or copied by hand; the same proposal the AI got.
    assert_eq!(desktop.app.ai.list().len(), 1);
    let proposal = desktop.app.ai.list()[0].clone();
    assert_eq!(proposal.id.to_string(), reply["id"].as_str().unwrap());
    assert!(proposal.request.is_some() && proposal.peak_edit.is_none());
    assert!(proposal.events.is_empty() && proposal.applied.is_none());
    assert_eq!(proposal.proposer(), "desktop protocol test");
    // The workspace was not switched; the result is not reviewed yet.
    assert!(!desktop.app.delivery.open);
    assert_eq!(
        desktop.app.quant.latest_targeted().unwrap().reviews.len(),
        batch.reviews.len()
    );
    let logged = desktop.app.remote.as_ref().unwrap().history.clone();
    assert!(logged
        .iter()
        .any(|e| e["request"]["operation"] == "show_ai_proposal"));
    assert!(shows(
        &frame(&mut desktop, vec![]),
        "The AI proposed a change ·"
    ));
    click(&mut desktop, "Review");
    assert!(desktop.app.delivery.open);
    frame(&mut desktop, vec![]);
    let output = frame(&mut desktop, vec![]);
    assert!(shows(
        &output,
        "Reason: The qualifier ratio of this injection is outside the accepted range"
    ));
    // The person approves with their own reason; the shared engine applies the review.
    desktop
        .app
        .ai
        .set_review_reason("Ratio confirmed in the raw data");
    click(&mut desktop, "Approve");
    desktop.pump_until(30, |d| !d.app.ai.busy());
    let decided = desktop.app.ai.list()[0].clone();
    assert_eq!(decided.events.len(), 1, "{}", desktop.app.ai.message());
    assert_eq!(decided.events[0].decision, "approved");
    assert_eq!(decided.events[0].reason, "Ratio confirmed in the raw data");
    assert!(decided.applied.is_some());
    assert_eq!(
        desktop.app.quant.latest_targeted().unwrap().reviews.len(),
        batch.reviews.len() + 1,
        "the review is recorded in the batch"
    );
    desktop.finish();
}

#[test]
fn rejecting_an_engine_proposal_and_a_stale_one_change_nothing() {
    let temp = tempfile::tempdir().unwrap();
    let request = raw_request(temp.path());
    let source = request.samples[0].source.clone();
    let mut desktop = Desktop::start(vec![temp.path().to_owned()], true, false, false);
    let (batch, _) = targeted_proposal(&mut desktop, &source, request);
    click(&mut desktop, "Review");
    desktop.app.ai.set_review_reason("Not convinced");
    click(&mut desktop, "Reject");
    desktop.pump_until(30, |d| !d.app.ai.busy());
    let decided = desktop.app.ai.list()[0].clone();
    assert_eq!(decided.events[0].decision, "rejected");
    assert!(decided.applied.is_none());
    assert_eq!(
        desktop.app.quant.latest_targeted().unwrap().reviews.len(),
        batch.reviews.len()
    );
    desktop.finish();

    // A proposal made against evidence that has changed since is refused on approval.
    let temp = tempfile::tempdir().unwrap();
    let request = raw_request(temp.path());
    let source = request.samples[0].source.clone();
    let mut desktop = Desktop::start(vec![temp.path().to_owned()], true, false, false);
    targeted_proposal(&mut desktop, &source, request);
    // The same batch is reviewed in another way first, which changes the evidence.
    let current = desktop.app.quant.latest_targeted().unwrap().clone();
    let row = current.results[0].clone();
    let changed = crate::targeted::review(
        &current,
        current.reviews.len(),
        &row.sample,
        &row.target,
        false,
        "person",
        "Reviewed by hand first",
    )
    .unwrap();
    desktop.app.quant.retain_targeted(changed).unwrap();
    click(&mut desktop, "Review");
    desktop.app.ai.set_review_reason("Approve anyway");
    click(&mut desktop, "Approve");
    desktop.pump_until(30, |d| !d.app.ai.busy());
    assert!(
        desktop.app.ai.list()[0].events.is_empty(),
        "a proposal made against older evidence is not decided: {}",
        desktop.app.ai.message()
    );
    assert!(
        desktop.app.ai.message().contains("changed"),
        "{}",
        desktop.app.ai.message()
    );
    desktop.finish();
}

#[test]
fn an_engine_proposal_queued_in_a_project_is_found_when_the_project_is_opened_without_mcp() {
    let temp = tempfile::tempdir().unwrap();
    let request = raw_request(temp.path());
    let source = std::path::PathBuf::from(&request.samples[0].source);
    let root = temp.path().join("project");
    // A project that holds a calibration, built the way the engine does.
    let mut project = crate::project::Project::create(&root).unwrap();
    let dataset = project.register(&source).unwrap();
    for sample in &request.samples {
        project
            .register(std::path::Path::new(&sample.source))
            .unwrap();
    }
    let executed = crate::engine::execute(
        &source,
        crate::domain::Request {
            version: 1,
            operation_id: Default::default(),
            actor: "test".into(),
            operation: crate::domain::Operation::TargetedBatch { batch: request },
        },
        &crate::jobs::JobControl::default(),
    )
    .unwrap();
    let crate::engine::Output::TargetedQuantification { batch } = &executed.output else {
        panic!("expected a targeted batch");
    };
    let batch = *batch.clone();
    project.add_result(&root, dataset, &executed).unwrap();
    project.commit(&root, 0).unwrap();
    let row = batch.results.iter().find(|r| r.sample == "raw6").unwrap();
    let suggestion = crate::proposals::prepare(
        &source,
        crate::domain::Request {
            version: 1,
            operation_id: Default::default(),
            actor: "agent".into(),
            operation: crate::domain::Operation::ReviewTargeted {
                expected_revision: batch.reviews.len(),
                batch: Box::new(batch.clone()),
                sample: row.sample.clone(),
                target: row.target.clone(),
                accepted: false,
                reason: "ratio".into(),
            },
        },
        Some(project.revision),
        "Qualifier ratio outside the range".into(),
        vec!["raw6 ratio".into()],
        &crate::jobs::JobControl::default(),
    )
    .unwrap();
    // The server runs with --allow-project-writes and an AI queues the proposal in the project.
    let mut desktop = Desktop::start(vec![temp.path().to_owned()], false, false, true);
    let queued = desktop.tool(
        "queue_review_proposal",
        json!({"directory":root,"dataset_id":dataset.0.to_string(),"proposal":suggestion}),
    );
    assert_eq!(queued["effect"], "retained_suggestion", "{queued}");
    desktop.pump();
    // It is stored in the project and, at the same time, in the window the AI works with.
    let stored = crate::project::Project::open(&root).unwrap();
    assert_eq!(stored.review_proposals.len(), 1);
    assert_eq!(desktop.app.ai.list().len(), 1);
    assert_eq!(desktop.app.ai.list()[0].id, stored.review_proposals[0].id);
    assert!(shows(
        &frame(&mut desktop, vec![]),
        "The AI proposed a change ·"
    ));
    desktop.finish();
    // Later: chromascope.exe opens the project and finds the proposal, and can decide on it.
    let (ctx, mut plain) = open_without_mcp(root);
    assert_eq!(plain.ai.list().len(), 1);
    assert_eq!(plain.ai.list()[0].id, stored.review_proposals[0].id);
    assert!(
        plain.quant.has_targeted_batch(),
        "the project's calibration is restored"
    );
    plain.delivery.open = true;
    draw(&ctx, &mut plain, vec![]);
    let output = draw(&ctx, &mut plain, vec![]);
    assert!(shows(&output, "Reason: Qualifier ratio outside the range"));
    assert!(
        !shows(&output, "AI connected"),
        "no AI indicator without MCP"
    );
    plain.ai.set_review_reason("Checked later in the project");
    click_in(&ctx, &mut plain, "Approve");
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(60);
    while plain.ai.busy() {
        assert!(std::time::Instant::now() < deadline);
        super::super::ai_review::poll(&mut plain, &ctx);
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    assert_eq!(plain.ai.list()[0].events.len(), 1, "{}", plain.ai.message());
    assert_eq!(plain.ai.list()[0].events[0].decision, "approved");
    assert_eq!(
        plain.quant.latest_targeted().unwrap().reviews.len(),
        batch.reviews.len() + 1
    );
}

#[test]
fn an_empty_review_list_explains_what_a_proposal_is_and_how_to_get_one() {
    let mut desktop = Desktop::start(vec![], false, false, false);
    desktop.app.delivery.open = true;
    // The section is closed while nothing waits for a decision; open it as the user would.
    desktop.app.ai.reveal_selected();
    frame(&mut desktop, vec![]);
    let output = frame(&mut desktop, vec![]);
    let ctx = desktop.ctx.clone();
    preview(&ctx, "review-ai-proposals-empty");
    for text in [
        "Review AI proposals",
        "No AI proposals yet",
        "What a proposal is: an AI suggests a change to a result (for example new peak boundaries) with its reason and evidence. Nothing changes until you approve it.",
        "How to get one: connect your AI client to chromascope-mcp, then ask for a proposal instead of a direct change, for example:",
        "Where it appears: right here, with Approve and Reject buttons.",
        "With projects: start the server with --allow-project-writes so proposals are also saved in the project and can be reviewed later, also in chromascope.exe.",
        "Load a saved proposal file…",
    ] {
        assert!(shows(&output, text), "the empty state lacks {text:?}");
    }
    assert!(shows(&output, "\u{201c}Check the integration of caffeine in run 3. If the boundaries look wrong, propose new ones with your reason and evidence, but don't apply them.\u{201d}"));
    // In the window an AI is connected to, the sentence for chromascope.exe is not needed.
    let plain_only = "This window has no AI connected: live proposals arrive in the chromascope-mcp window, and saved ones appear here when you open their project.";
    assert!(!shows(&output, plain_only));
    desktop.finish();
    // chromascope.exe says where live proposals arrive.
    let ctx = egui::Context::default();
    let mut plain = MzViewerApp::default();
    plain.delivery.open = true;
    plain.ai.reveal_selected();
    draw(&ctx, &mut plain, vec![]);
    let output = draw(&ctx, &mut plain, vec![]);
    assert!(shows(&output, plain_only));
    assert!(shows(&output, "No AI proposals yet"));
}

#[test]
fn a_client_has_no_way_to_put_a_proposal_in_the_window_except_by_making_one() {
    let mut desktop = Desktop::start(vec![], true, true, true);
    let names: Vec<String> = desktop
        .tools()
        .iter()
        .map(|t| t["name"].as_str().unwrap().to_owned())
        .collect();
    let mut about_proposals: Vec<&str> = names
        .iter()
        .map(String::as_str)
        .filter(|n| n.contains("proposal"))
        .collect();
    about_proposals.sort_unstable();
    assert_eq!(
        about_proposals,
        [
            "prepare_review_proposal",
            "queue_review_proposal",
            "resolve_review_proposal"
        ],
        "tools that deal with proposals"
    );
    // A made-up tool is not served, and the window's list stays empty.
    for invented in ["deliver_proposal", "show_ai_proposal", "add_proposal"] {
        let reply = desktop.call(invented, json!({"proposal":{}}));
        assert!(
            reply["error"].is_object() || reply["result"]["isError"] == true,
            "{invented}: {reply}"
        );
    }
    // A proposal an AI writes itself and hands to the tool that retains one is refused (no
    // project, wrong hash); only what the tools return after the engine made it is delivered, so
    // a refused call shows nothing in the window.
    let forged = json!({"version":1,"id":uuid::Uuid::new_v4(),"request":null,"peak_edit":{"proposer":"x","dataset_stable_id":null,"result_index":0,"sample":"s","analyte":"a","original":{"start":null,"end":null,"area":null},"proposed":{"start":1.0,"end":2.0,"area":3.0}},"expected_project_revision":null,"before":{},"before_sha256":"00","proposed":{},"reason":"r","evidence":["e"],"events":[],"applied":null});
    desktop.tool_error(
        "queue_review_proposal",
        json!({"directory":env!("CARGO_MANIFEST_DIR"),"dataset_id":uuid::Uuid::nil().to_string(),"proposal":forged}),
    );
    desktop.pump();
    assert!(desktop.app.ai.list().is_empty());
    assert!(!desktop.app.ai.has_notice());
    desktop.finish();
}
