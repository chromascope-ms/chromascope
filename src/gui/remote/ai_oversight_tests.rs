//! What the person sees of the AI in the window: the status bar indicator, the call log and the
//! proposals an AI delivers. Driven through the real MCP protocol like `protocol_tests`, with
//! real egui frames for everything the person looks at and clicks.
use super::protocol_tests::Desktop;
use super::*;

/// One frame of the parts of the window these tests look at.
fn frame(desktop: &mut Desktop, time: f64, events: Vec<egui::Event>) -> egui::FullOutput {
    let ctx = desktop.ctx.clone();
    let app = &mut desktop.app;
    ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1280.0, 800.0),
            )),
            time: Some(time),
            events,
            ..Default::default()
        },
        |ctx| {
            super::super::workbench::status(app, ctx);
            super::super::ai_status::log_window(app, ctx);
        },
    )
}

fn shows(output: &egui::FullOutput, text: &str) -> bool {
    super::super::test_render::text_center(&output.shapes, text).is_some()
}

/// Clicks the control with this text: one frame to lay it out, one to press and release.
fn click(desktop: &mut Desktop, label: &str) {
    let output = frame(desktop, 0.0, vec![]);
    let pos = super::super::test_render::text_center(&output.shapes, label)
        .unwrap_or_else(|| panic!("{label:?} is not on screen"));
    let button = egui::PointerButton::Primary;
    let modifiers = egui::Modifiers::NONE;
    frame(
        desktop,
        0.2,
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

#[test]
fn the_indicator_appears_only_with_a_bridge_and_counts_the_requests() {
    let mut desktop = Desktop::start(vec![], true, false, false);
    let output = frame(&mut desktop, 0.0, vec![]);
    assert!(shows(&output, "AI connected · 0 requests ·"));
    assert!(shows(&output, "View log"));
    for tool in ["list_datasets", "gui_state", "list_datasets"] {
        desktop.tool(tool, json!({}));
    }
    let logged = desktop.app.remote.as_ref().unwrap().history.len();
    assert_eq!(logged, 3, "one log entry per request");
    let output = frame(&mut desktop, 0.4, vec![]);
    assert!(
        shows(&output, "AI connected · 3 requests ·"),
        "the count in the status bar is the number of log entries"
    );
    desktop.tool("list_datasets", json!({}));
    let output = frame(&mut desktop, 0.6, vec![]);
    assert!(shows(&output, "AI connected · 4 requests ·"));
    // A request that has arrived and is not answered yet shows as work in progress, and the
    // counter returns to zero once every reply has been sent.
    let active = desktop.app.remote.as_ref().unwrap().active.clone();
    assert_eq!(active.load(std::sync::atomic::Ordering::SeqCst), 0);
    active.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    let output = frame(&mut desktop, 0.8, vec![]);
    assert!(shows(&output, "AI is working…"));
    assert!(!shows(&output, "AI connected · 4 requests ·"));
    active.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
    let output = frame(&mut desktop, 1.0, vec![]);
    assert!(!shows(&output, "AI is working…"));
    // The Quantification workspace has no status bar; its own bar carries the same line.
    let screen = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1280.0, 800.0));
    let ctx = desktop.ctx.clone();
    let app = &mut desktop.app;
    let bar = |app: &mut MzViewerApp| {
        ctx.run(
            egui::RawInput {
                screen_rect: Some(screen),
                ..Default::default()
            },
            |ctx| super::super::ai_status::bar(app, ctx),
        )
    };
    let output = bar(app);
    assert!(shows(&output, "AI connected · 4 requests ·") && shows(&output, "View log"));
    desktop.finish();
    let mut plain = MzViewerApp::default();
    let output = bar(&mut plain);
    assert!(!shows(&output, "View log"), "no bar without MCP");

    // chromascope.exe: no bridge, no AI line, no log link, whatever else the window shows.
    let ctx = egui::Context::default();
    let mut plain = MzViewerApp::default();
    assert!(plain.remote.is_none());
    let output = ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1280.0, 800.0),
            )),
            ..Default::default()
        },
        |ctx| {
            super::super::workbench::status(&mut plain, ctx);
            super::super::ai_status::log_window(&mut plain, ctx);
        },
    );
    for absent in ["View log", "AI is working…", "AI call log", "Review"] {
        assert!(!shows(&output, absent), "{absent:?} appears without MCP");
    }
    assert!(
        super::super::test_render::text_center(&output.shapes, "0 files loaded").is_some(),
        "the rest of the status bar is still drawn"
    );
    assert!(!output
        .shapes
        .iter()
        .any(|clipped| { format!("{:?}", clipped.shape).contains("AI connected") }));
}

#[test]
fn the_call_log_opens_from_the_status_bar_and_survives_workspace_switches() {
    let mut desktop = Desktop::start(vec![], true, true, true);
    desktop.open_fixture();
    desktop.tool("list_datasets", json!({}));
    assert!(!shows(&frame(&mut desktop, 0.0, vec![]), "AI call log"));
    click(&mut desktop, "View log");
    assert!(desktop.app.remote.as_ref().unwrap().log_open);
    let output = frame(&mut desktop, 0.4, vec![]);
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
        let output = frame(&mut desktop, 0.8, vec![]);
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
    assert!(!shows(&frame(&mut desktop, 1.0, vec![]), "AI call log"));
    desktop.finish();

    // A window started without permissions says so.
    let mut read_only = Desktop::start(vec![], false, false, false);
    read_only.app.remote.as_mut().unwrap().log_open = true;
    // A new window needs a frame to find its size before it shows its contents.
    frame(&mut read_only, 0.0, vec![]);
    let output = frame(&mut read_only, 0.2, vec![]);
    assert!(shows(
        &output,
        "--allow-changes: blocked · --allow-exports: blocked · --allow-project-writes: blocked · tool profile: full"
    ));
    read_only.finish();
}
