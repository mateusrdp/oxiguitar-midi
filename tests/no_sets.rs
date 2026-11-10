// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Mateus Rocha de Paula and the OxiGuitar contributors

//! With no sound set anywhere (as in OxiGuitar's Flatpak before the user
//! adds one, #163), preparing fails, and the footer says where a set goes,
//! once, with the Better sounds link.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use oxiguitar_host::api::message::{Link, Prepare};
use oxiguitar_host::{Core, Level, Plugins, SharedSettings};

/// The plugin as the `.oxp` file we ship (a debug build).
fn midi_oxp() -> PathBuf {
    let crate_folder = Path::new(env!("CARGO_MANIFEST_DIR"));
    oxiguitar_plugin_test::build(crate_folder, env!("CARGO_PKG_NAME"), "midi", false)
        .expect("the .oxp file builds")
}

/// A footer that keeps what it is told.
#[derive(Default)]
struct Footer(Mutex<Vec<(String, Option<Link>)>>);

impl Core for Footer {
    fn notify(&self, _level: Level, text: String) {
        self.0.lock().unwrap().push((text, None));
    }

    fn notify_with_link(&self, _level: Level, text: String, link: Link) {
        self.0.lock().unwrap().push((text, Some(link)));
    }
}

/// Whether this machine has a set where the plugin looks besides the user's
/// folder: then the test can't make "no set at all".
fn system_has_sets() -> bool {
    [
        "/usr/share/oxiguitar/soundfonts",
        "/usr/share/sounds/sf2",
        "/usr/share/sounds/sf3",
        "/usr/share/soundfonts",
    ]
    .iter()
    .any(|d| std::fs::read_dir(d).is_ok_and(|mut e| e.next().is_some()))
}

#[test]
#[cfg(target_os = "linux")]
fn no_set_says_where_with_the_better_sounds_link() {
    if system_has_sets() {
        eprintln!("this machine has sound sets of its own: nothing to check");
        return;
    }
    let scratch = std::env::temp_dir().join(format!("oxiguitar-no-sets-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&scratch);
    let folder = scratch.join("plugins");
    std::fs::create_dir_all(&folder).unwrap();
    std::fs::copy(midi_oxp(), folder.join("midi.oxp")).unwrap();
    let home = scratch.join("home");
    std::fs::create_dir_all(home.join("soundfonts")).unwrap();
    let footer = Arc::new(Footer::default());
    let plugins = Plugins::discover(
        &[folder],
        SharedSettings::default(),
        oxiguitar_host::display_system(),
        footer.clone(),
    );
    let folders = oxiguitar_host::Folders::resolve(
        oxiguitar_host::System::this(),
        |name| (name == oxiguitar_host::HOME_VARIABLE).then(|| home.clone().into()),
        None,
    )
    .unwrap();
    plugins.use_folders(folders);
    let prepare = || {
        let instrument = plugins
            .instruments()
            .into_iter()
            .next()
            .expect("the MIDI plugin's instrument");
        plugins.prepare(
            instrument,
            Prepare {
                instrument: String::new(),
                rate: 48_000,
                frames: 256,
                tracks: Vec::new(),
            },
        )
    };
    for _ in 0..2 {
        let Err(err) = prepare() else {
            panic!("no set: preparing must fail");
        };
        assert!(err.to_string().contains("no sound set"), "{err}");
    }
    let told = footer.0.lock().unwrap().clone();
    let linked: Vec<_> = told.iter().filter(|(_, l)| l.is_some()).collect();
    assert_eq!(linked.len(), 1, "said once, with its link: {told:#?}");
    let (text, link) = linked[0];
    assert!(text.contains("No sound set found"), "{text}");
    assert!(text.contains("soundfonts"), "names the folder: {text}");
    let link = link.as_ref().unwrap();
    assert_eq!(link.url, oxiguitar_midi_plugin::BETTER_SOUNDS);
    let _ = std::fs::remove_dir_all(&scratch);
}
