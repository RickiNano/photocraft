//! File › Export puts Export Preferences… in its own group (#13801): a separator after Export As…
//! and one before Save for Web (Legacy)…, with the rest of the submenu unchanged.

use crate::PhotocraftApp;
use crate::menu_catalog::CATALOG;
use crate::menus::menu_items;

/// The submenu's head as the reference application shows it: Export Preferences alone between two separators.
const HEAD: [&str; 6] = ["file.export.quickExportAsPng", "file.export.exportAs", "---", "file.export.exportPreferences", "---", "file.export.saveForWebLegacy"];

#[test]
fn export_preferences_sits_between_two_separators_in_the_catalog() {
    let rows: Vec<_> = CATALOG.iter().filter(|(path, ..)| *path == ["File", "Export"]).collect();
    let ids: Vec<&str> = rows.iter().map(|(_, _, _, id)| *id).collect();
    assert_eq!(ids[..HEAD.len()], HEAD);
    // Labels and shortcuts of the head stay as they were.
    let head: Vec<(&str, Option<&str>)> = rows.iter().take(HEAD.len()).map(|(_, label, shortcut, _)| (*label, *shortcut)).collect();
    assert_eq!(
        head,
        [
            ("Quick Export as PNG", None),
            ("Export As…", Some("Cmd+Alt+Shift+W")),
            ("---", None),
            ("Export Preferences…", None),
            ("---", None),
            ("Save for Web (Legacy)…", Some("Cmd+Alt+Shift+S")),
        ]
    );
    // Nothing after the head moved: the rest of the submenu keeps its order.
    assert_eq!(
        ids[HEAD.len()..],
        [
            "---",
            "file.export.artboardsToFiles",
            "file.export.artboardsToPdf",
            "file.export.layersToFiles",
            "file.export.layerCompsToFiles",
            "---",
            "file.export.colorLookupTables",
            "file.export.dataSetsAsFiles",
            "file.export.pathsToIllustrator",
            "file.export.renderVideo",
        ]
    );
}

#[test]
fn the_built_menu_and_ui_menu_list_show_the_separator() {
    let app = PhotocraftApp::new(photocraft_engine::Session::new(), crate::Services::default());
    let ids: Vec<String> = menu_items(&app).into_iter().filter(|i| i.path == ["File", "Export"]).map(|i| i.id).collect();
    assert_eq!(ids[..HEAD.len()], HEAD);
}
