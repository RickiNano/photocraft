//! Export Preferences › PNG › Transparency: on (the default) Quick Export keeps transparent
//! pixels in an RGBA PNG; off it writes an opaque RGB PNG matted on white (#13784).

use super::*;

fn tmp(name: &str) -> String {
    let d = std::env::temp_dir().join(format!("photocraft-quick-png-transparency-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d.to_string_lossy().into_owned()
}

/// A 4×2 document on a transparent background: (1, 0) is 50 % opaque green 0,160,0, (2, 0) is
/// opaque blue, everything else is transparent.
fn transparent_doc() -> Session {
    let mut s = Session::new();
    s.execute("file.new", json!({"width": 4, "height": 2, "depth": 8, "background": "transparent"})).unwrap();
    s.edit("paint", |doc, active| {
        let fmt = doc.pixel_format();
        let mut l = photocraft_doc::Layer::raster("Pixels", fmt);
        let surf = l.surface_mut().unwrap();
        surf.fill_rect(photocraft_geom::Rect::new(1, 0, 2, 1), &photocraft_raster::from_rgba(&fmt, [0.0, 160.0 / 255.0, 0.0, 128.0 / 255.0]));
        surf.fill_rect(photocraft_geom::Rect::new(2, 0, 3, 1), &photocraft_raster::from_rgba(&fmt, [0.0, 0.0, 1.0, 1.0]));
        *active = Some(doc.insert_above(None, l));
        Ok(())
    })
    .unwrap();
    s
}

/// The PNG's IHDR colour type: 2 is RGB, 6 is RGBA.
fn png_color_type(bytes: &[u8]) -> u8 {
    assert!(bytes.starts_with(b"\x89PNG\r\n\x1a\n") && &bytes[12..16] == b"IHDR", "not a PNG");
    bytes[25]
}

fn export(s: &mut Session, path: &str) -> (Vec<u8>, Vec<u8>) {
    s.execute("file.export.quickExport", json!({"path": path})).unwrap();
    let bytes = std::fs::read(path).unwrap();
    let px = photocraft_codecs::decode(&bytes).unwrap().to_rgba8();
    (bytes, px)
}

fn near(got: &[u8], want: [u8; 4]) -> bool {
    got.iter().zip(want).all(|(&g, w)| g.abs_diff(w) <= 1)
}

#[test]
fn transparency_is_on_by_default_and_for_older_preferences() {
    let s = Session::new();
    assert!(s.prefs().export.png_transparency, "PNG Transparency starts on");
    let mut old = s.prefs().to_json();
    old["export"].as_object_mut().unwrap().remove("pngTransparency");
    let restored: crate::prefs::Preferences = serde_json::from_value(old).unwrap();
    assert!(restored.export.png_transparency, "a preferences file without the key loads with Transparency on");
}

#[test]
fn transparency_on_keeps_alpha() {
    let dir = tmp("on");
    let mut s = transparent_doc();
    let (bytes, px) = export(&mut s, &format!("{dir}/on.png"));
    assert_eq!(png_color_type(&bytes), 6, "RGBA PNG");
    assert_eq!(px[3], 0, "a transparent pixel stays transparent");
    assert!(near(&px[4..8], [0, 160, 0, 128]), "50 % green keeps its alpha: {:?}", &px[4..8]);
    assert_eq!(&px[8..12], &[0, 0, 255, 255]);
}

#[test]
fn transparency_off_writes_rgb_over_white() {
    let dir = tmp("off");
    let mut s = transparent_doc();
    let r = s.execute("file.export.exportPreferences", json!({"pngTransparency": false})).unwrap();
    assert_eq!(r["values"]["pngTransparency"], false);
    assert!(!s.prefs().export.png_transparency);
    let (bytes, px) = export(&mut s, &format!("{dir}/off.png"));
    assert_eq!(png_color_type(&bytes), 2, "RGB PNG, no alpha");
    assert_eq!(&px[0..4], &[255, 255, 255, 255], "a transparent pixel becomes white");
    assert!(near(&px[4..8], [127, 207, 127, 255]), "50 % green over white: {:?}", &px[4..8]);
    assert_eq!(&px[8..12], &[0, 0, 255, 255]);

    // Turning it back on restores today's RGBA output.
    s.execute("file.export.exportPreferences", json!({"pngTransparency": true})).unwrap();
    let (bytes, px) = export(&mut s, &format!("{dir}/on-again.png"));
    assert_eq!(png_color_type(&bytes), 6);
    assert_eq!(px[3], 0);
}

#[test]
fn an_opaque_document_writes_rgb_either_way() {
    let dir = tmp("opaque");
    let mut s = Session::new();
    s.execute("file.new", json!({"width": 4, "height": 2, "depth": 8, "background": "white"})).unwrap();
    for on in [true, false] {
        s.execute("file.export.exportPreferences", json!({"pngTransparency": on})).unwrap();
        let (bytes, px) = export(&mut s, &format!("{dir}/opaque-{on}.png"));
        assert_eq!(png_color_type(&bytes), 2, "Transparency {on}");
        assert_eq!(&px[0..4], &[255, 255, 255, 255]);
    }
}

#[test]
fn transparency_off_leaves_gif_alone() {
    let dir = tmp("gif");
    let mut s = transparent_doc();
    s.execute("file.export.exportPreferences", json!({"quickExportFormat": "gif", "pngTransparency": false})).unwrap();
    let (_, px) = export(&mut s, &format!("{dir}/still-transparent.gif"));
    assert_eq!(px[3], 0, "GIF keeps its transparency: the checkbox is PNG's only");
}

#[test]
fn a_non_boolean_transparency_is_refused() {
    let mut s = Session::new();
    for bad in [json!("no"), json!(0), json!(null), json!([true])] {
        assert!(s.execute("file.export.exportPreferences", json!({"pngTransparency": bad})).is_err(), "{bad}");
    }
    assert!(s.prefs().export.png_transparency, "a refused value changes nothing");
}
