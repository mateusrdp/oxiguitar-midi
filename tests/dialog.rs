// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Mateus Rocha de Paula and the OxiGuitar contributors

//! #19: the Sound Sets dialog (Plugins → MIDI Sound Sets…) changes a draft.
//! A pick is neither saved nor played until OK; Cancel forgets it; OK saves
//! it once, and OxiGuitar hears that the settings changed (so it loads the
//! new sets once). The tool that opens it doesn't edit the score.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use oxiguitar_host::api::tab::{TabContent, TabEvent, Widget};
use oxiguitar_host::{Core, Level, Plugins, SharedSettings, TabId};

/// The plugin as the `.oxp` file we ship (a debug build).
fn midi_oxp() -> PathBuf {
    let crate_folder = Path::new(env!("CARGO_MANIFEST_DIR"));
    oxiguitar_plugin_test::build(crate_folder, env!("CARGO_PKG_NAME"), "midi", false)
        .expect("the .oxp file builds")
}

/// The window, as far as the dialog goes: the dialogs opened and the
/// plugins whose settings changed.
#[derive(Default)]
struct Window {
    dialogs: Mutex<Vec<String>>,
    changed: Mutex<Vec<String>>,
}

impl Core for Window {
    fn notify(&self, _level: Level, _text: String) {}

    fn open_dialog(&self, _tab: &TabId, title: &str) -> Result<(), String> {
        self.dialogs.lock().unwrap().push(title.to_owned());
        Ok(())
    }

    fn settings_changed(&self, id: &str) {
        self.changed.lock().unwrap().push(id.to_owned());
    }
}

/// The options and the selected entry of the row `key`'s list.
fn row(content: &TabContent, key: &str) -> (Vec<String>, u32) {
    content
        .rows
        .iter()
        .flat_map(|r| &r.widgets)
        .find_map(|w| match w {
            Widget::Choice {
                id,
                options,
                selected,
                ..
            } if id == key => Some((options.clone(), *selected)),
            _ => None,
        })
        .unwrap_or_else(|| panic!("no list {key}"))
}

#[test]
fn picks_are_kept_with_ok_and_forgotten_with_cancel() {
    let scratch = std::env::temp_dir().join(format!("oxiguitar-dialog-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&scratch);
    let folder = scratch.join("plugins");
    std::fs::create_dir_all(&folder).unwrap();
    std::fs::copy(midi_oxp(), folder.join("midi.oxp")).unwrap();
    // One SFZ instrument of the user's, so there is something to pick.
    let home = scratch.join("home");
    let kit = home.join("soundfonts").join("Sines");
    std::fs::create_dir_all(&kit).unwrap();
    let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures/sfz");
    std::fs::copy(fixtures.join("sine.ogg"), kit.join("sine.ogg")).unwrap();
    std::fs::write(kit.join("Sine.sfz"), "<region> sample=sine.ogg\n").unwrap();

    let window = Arc::new(Window::default());
    let plugins = Plugins::discover(
        &[folder],
        SharedSettings::default(),
        oxiguitar_host::NO_DISPLAY,
        window.clone(),
    );
    let folders = oxiguitar_host::Folders::resolve(
        oxiguitar_host::System::this(),
        |name| (name == oxiguitar_host::HOME_VARIABLE).then(|| home.clone().into()),
        None,
    )
    .unwrap();
    plugins.use_folders(folders);

    let tools = plugins.tools();
    let tool = tools
        .iter()
        .find(|t| t.spec.id == "sound-sets")
        .expect("the Sound Sets tool");
    assert!(!tool.spec.edits, "it works with no score and while playing");
    let dialog = TabId {
        plugin: "oxiguitar.midi".into(),
        tab: "sound-sets".into(),
    };
    let saved = || plugins.settings().get("oxiguitar.midi");
    let pick = || {
        assert_eq!(plugins.run_tool(*tool).unwrap(), "");
        let (options, selected) = row(&plugins.tab_content(&dialog).unwrap(), "clean-guitars");
        assert_eq!(selected, 0, "each opening starts from what is saved");
        let sine = options
            .iter()
            .position(|o| o.contains("Sines: Sine (SFZ)"))
            .unwrap_or_else(|| panic!("the SFZ instrument is listed: {options:?}"));
        let after = plugins
            .tab_event(&dialog, "clean-guitars", TabEvent::Chosen(sine as u32))
            .unwrap();
        assert_eq!(
            row(&after, "clean-guitars").1,
            sine as u32,
            "the dialog shows it"
        );
    };

    pick();
    assert_eq!(saved(), None, "a pick is a draft");
    plugins.tab_closed(&dialog, false);
    assert_eq!(saved(), None, "Cancel forgets it");
    assert!(window.changed.lock().unwrap().is_empty());

    pick();
    plugins.tab_closed(&dialog, true);
    let kept = saved().expect("OK keeps it");
    assert!(
        kept["clean-guitars"]
            .as_str()
            .is_some_and(|p| p.ends_with("Sine.sfz")),
        "{kept}"
    );
    assert_eq!(*window.changed.lock().unwrap(), ["oxiguitar.midi"]);
    assert_eq!(
        *window.dialogs.lock().unwrap(),
        ["MIDI Sound Sets", "MIDI Sound Sets"]
    );
    let _ = std::fs::remove_dir_all(&scratch);
}
