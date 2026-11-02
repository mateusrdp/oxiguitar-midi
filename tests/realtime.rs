// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Mateus Rocha de Paula and the OxiGuitar contributors

//! The plugin plays an SFZ instrument picked in its settings through the
//! contract suite's real-time checks: notes start on time, nothing is
//! allocated and (on Linux) no system call made while it renders. Moved from
//! OxiGuitar's `oxiguitar-plugin-test` tests (#165).

use std::path::{Path, PathBuf};
use std::sync::Arc;

use oxiguitar_host::{Plugins, SharedSettings, Unheard};

/// The plugin as the `.oxp` file we ship (a debug build).
fn midi_oxp() -> PathBuf {
    let crate_folder = Path::new(env!("CARGO_MANIFEST_DIR"));
    oxiguitar_plugin_test::build(crate_folder, env!("CARGO_PKG_NAME"), "midi", false)
        .expect("the .oxp file builds")
}

/// A mono 16-bit WAV file of a 440 Hz sine at half scale, `seconds`
/// long, at 48 kHz.
fn sine_wav(seconds: f64) -> Vec<u8> {
    let rate = 48_000u32;
    let frames = (seconds * f64::from(rate)) as usize;
    let mut data = Vec::with_capacity(frames * 2);
    for i in 0..frames {
        let v = (std::f64::consts::TAU * 440.0 * i as f64 / f64::from(rate)).sin() * 16_384.0;
        data.extend_from_slice(&(v as i16).to_le_bytes());
    }
    let mut fmt = Vec::new();
    fmt.extend_from_slice(&1u16.to_le_bytes());
    fmt.extend_from_slice(&1u16.to_le_bytes());
    fmt.extend_from_slice(&rate.to_le_bytes());
    fmt.extend_from_slice(&(rate * 2).to_le_bytes());
    fmt.extend_from_slice(&2u16.to_le_bytes());
    fmt.extend_from_slice(&16u16.to_le_bytes());
    let mut body = b"WAVE".to_vec();
    for (id, chunk) in [(b"fmt ", fmt), (b"data", data)] {
        body.extend_from_slice(id);
        body.extend_from_slice(&(chunk.len() as u32).to_le_bytes());
        body.extend_from_slice(&chunk);
    }
    let mut file = b"RIFF".to_vec();
    file.extend_from_slice(&(body.len() as u32).to_le_bytes());
    file.extend_from_slice(&body);
    file
}

/// The SoundFont plugin plays an SFZ instrument picked in its settings
/// through the contract suite's real-time checks: notes start on time,
/// nothing is allocated and (on Linux) no system call made while it
/// renders (M4 slice 5).
#[test]
fn our_midi_plugin_plays_sfz_in_real_time() {
    let scratch =
        std::env::temp_dir().join(format!("oxiguitar-sfz-contract-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&scratch);
    let folder = scratch.join("plugins");
    std::fs::create_dir_all(&folder).unwrap();
    std::fs::copy(midi_oxp(), folder.join("midi.oxp")).unwrap();
    let home = scratch.join("home");
    let set = home.join("soundfonts").join("Test Sine");
    std::fs::create_dir_all(set.join("Samples")).unwrap();
    std::fs::write(set.join("Samples/sine.wav"), sine_wav(0.5)).unwrap();
    std::fs::write(
        set.join("sine.sfz"),
        "<control> default_path=Samples/\n\
         <group> ampeg_release=0.01 loop_mode=loop_continuous loop_start=0 loop_end=23999\n\
         <region> sample=sine.wav pitch_keycenter=69 hivel=80\n\
         <region> sample=sine.wav pitch_keycenter=69 lovel=81 seq_length=2 seq_position=1\n\
         <region> sample=sine.wav pitch_keycenter=69 lovel=81 seq_length=2 seq_position=2 volume=-1\n",
    )
    .unwrap();
    let pick = std::fs::canonicalize(set.join("sine.sfz"))
        .unwrap()
        .to_string_lossy()
        .into_owned();
    let settings = SharedSettings::default();
    settings
        .set(
            "oxiguitar.midi",
            serde_json::json!({"guitars": pick, "basses": pick, "drums": pick, "other": pick}),
        )
        .unwrap();
    let plugins = Plugins::discover(
        &[folder],
        settings,
        oxiguitar_host::display_system(),
        Arc::new(Unheard),
    );
    let folders = oxiguitar_host::Folders::resolve(
        oxiguitar_host::System::this(),
        |name| (name == oxiguitar_host::HOME_VARIABLE).then(|| home.clone().into()),
        None,
    )
    .unwrap();
    plugins.use_folders(folders);
    let report = oxiguitar_plugin_test::check_realtime(&plugins, "{}");
    assert!(report.ok(), "{:#?}", report.failed);
    for wanted in ["notes start within", "allocates nothing"] {
        assert!(
            report.passed.iter().any(|p| p.contains(wanted)),
            "{wanted}: {:#?}",
            report.passed
        );
    }
    let _ = std::fs::remove_dir_all(&scratch);
}
