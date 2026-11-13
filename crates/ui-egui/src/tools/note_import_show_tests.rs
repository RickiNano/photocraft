//! File › Import › Notes… shows the imported notes on the canvas without opening the Notes panel
//! (#13918).
use super::{PhotocraftApp, json};
use crate::file_dialog::FileDialogAnswer;

/// A `.pcraft` file with `notes` notes, as the file picker hands it over.
fn source_with_notes(notes: usize) -> FileDialogAnswer {
    let mut src = photocraft_doc::Document::new("src", photocraft_doc::Size::new(200, 120), photocraft_color::ColorMode::Rgb, photocraft_color::SampleType::U8);
    for i in 0..notes {
        src.notes.push(photocraft_doc::Note { text: format!("note {i}"), position: [40.0, 30.0], ..Default::default() });
    }
    FileDialogAnswer::Contents("src.pcraft".into(), photocraft_format::save_to_bytes(&src, &Default::default()).unwrap())
}

/// A 200 × 120 document whose file picker answers with `answer` (`None`: Cancel).
fn app(answer: Option<FileDialogAnswer>) -> (PhotocraftApp, egui::Context) {
    let (show, _asked) = crate::file_dialog::fake(vec![answer]);
    let mut app = PhotocraftApp::new(photocraft_engine::Session::new(), crate::Services { file_dialog: Some(show), ..Default::default() });
    app.run("file.new", json!({"width": 200, "height": 120})).unwrap();
    (app, egui::Context::default())
}

/// Run File › Import › Notes… through the menu and let the picker answer.
fn import(app: &mut PhotocraftApp, ctx: &egui::Context) {
    crate::menus::invoke(app, ctx, "file.import.notes", json!({})).unwrap();
    app.poll_file_dialog(ctx, None);
}

fn note_count(app: &PhotocraftApp) -> usize {
    app.session.active().unwrap().doc.notes.len()
}

/// With the panel closed and Show Notes off, an import turns Show Notes on and leaves the panel
/// closed, no note selected and Extras off.
#[test]
fn import_rechecks_show_notes_and_keeps_the_panel_closed() {
    let (mut app, ctx) = app(Some(source_with_notes(1)));
    app.ui.view.show.notes = false;
    app.ui.view.extras = false;
    assert!(!app.ui.analysis.notes && app.ui.analysis.note_selected.is_none());

    import(&mut app, &ctx);

    assert_eq!(note_count(&app), 1);
    assert!(app.ui.view.show.notes, "View › Show › Notes is checked again");
    assert!(!app.ui.view.extras, "the Extras master switch is left alone");
    assert!(!app.ui.analysis.notes, "the Notes panel stays closed");
    assert_eq!(app.ui.analysis.note_selected, None);
}

/// An open Notes panel stays open, and the note it shows stays selected.
#[test]
fn import_keeps_an_open_panel_open() {
    let (mut app, ctx) = app(Some(source_with_notes(2)));
    app.run("notes.add", json!({"x": 10, "y": 10})).unwrap();
    app.ui.analysis.notes = true;
    app.ui.analysis.note_selected = Some(0);
    app.ui.view.show.notes = false;

    import(&mut app, &ctx);

    assert_eq!(note_count(&app), 3);
    assert!(app.ui.view.show.notes && app.ui.view.shows(app.ui.view.show.notes));
    assert!(app.ui.analysis.notes);
    assert_eq!(app.ui.analysis.note_selected, Some(0));
}

/// A failed import (a file without notes) and a cancelled picker change neither Show Notes nor the
/// panel.
#[test]
fn failed_or_cancelled_import_changes_nothing() {
    for answer in [Some(source_with_notes(0)), None] {
        let (mut app, ctx) = app(answer);
        app.ui.view.show.notes = false;

        import(&mut app, &ctx);

        assert_eq!(note_count(&app), 0);
        assert!(!app.ui.view.show.notes && !app.ui.analysis.notes);
        assert_eq!(app.ui.analysis.note_selected, None);
    }
}
