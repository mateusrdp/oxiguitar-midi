// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Mateus Rocha de Paula and the OxiGuitar contributors

//! The track dialog's section "Sound" (mateusrdp/oxiguitar#152, Mateus, 3
//! Nov 2026): what one track plays, over the settings window's rows. Kept
//! in the song's sidecar (`FILE.oxg`) as this plugin's settings for the
//! track:
//!
//! - nothing: **Automatic**, the settings window's row for the kind of
//!   track its General MIDI program names (a distorted guitar the
//!   Distorted guitars row);
//! - `{"sound": "role:clean-guitars"}`: **a row** of the settings window,
//!   whatever the program says (a distorted guitar played clean into an
//!   amp model, a bass played as a distorted guitar); it means the same on
//!   any computer;
//! - `{"sound": "/path/to/set.sfz"}`: **a set**, exactly, by its path. Where
//!   the song is opened without that set, the track plays as Automatic and
//!   the footer says so once.

use std::path::{Path, PathBuf};

use oxiguitar_plugin::serde_json::{self, Map, Value};
use oxiguitar_plugin::{Error, Row, TabContent, TabEvent, TrackInfo, TrackSection, Widget};

use crate::sets::Found;
use crate::settings::{Role, Window};

/// The section's id in the manifest.
pub(crate) const SECTION: &str = "sound";

/// The key of the track's settings, and the choice's id.
const KEY: &str = "sound";

/// How a row of the settings window is saved.
const ROLE: &str = "role:";

/// What a track picks.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Pick {
    Automatic,
    Role(Role),
    Set(PathBuf),
}

impl Pick {
    /// From the track's settings for this plugin; anything unknown is
    /// Automatic.
    pub(crate) fn of(settings: &Map<String, Value>) -> Pick {
        match settings.get(KEY).and_then(Value::as_str) {
            None | Some("") => Pick::Automatic,
            Some(s) => match s.strip_prefix(ROLE) {
                Some(key) => Role::from_key(key).map_or(Pick::Automatic, Pick::Role),
                None => Pick::Set(PathBuf::from(s)),
            },
        }
    }

    /// From the settings as the host hands them, a JSON object as text.
    pub(crate) fn of_text(settings: &str) -> Pick {
        match serde_json::from_str::<Value>(settings) {
            Ok(Value::Object(map)) => Pick::of(&map),
            _ => Pick::Automatic,
        }
    }

    /// Writes it into the track's settings.
    fn save(&self, settings: &mut Map<String, Value>) {
        match self {
            Pick::Automatic => {
                settings.remove(KEY);
            }
            Pick::Role(r) => {
                settings.insert(KEY.into(), format!("{ROLE}{}", r.key()).into());
            }
            Pick::Set(p) => {
                settings.insert(KEY.into(), p.display().to_string().into());
            }
        }
    }
}

/// The set a track with `pick` plays among `found` (`row` gives the
/// settings window's set for a role), and whether its own set is missing.
pub(crate) fn set_for<'a>(
    pick: &Pick,
    automatic: Role,
    found: &'a [Found],
    row: impl Fn(Role) -> Option<&'a Found>,
) -> (Option<&'a Found>, bool) {
    match pick {
        Pick::Automatic => (row(automatic), false),
        Pick::Role(r) => (row(*r), false),
        Pick::Set(p) => match found.iter().find(|f| f.path == *p) {
            Some(f) => (Some(f), false),
            None => (row(automatic), true),
        },
    }
}

/// The choice's options and the one picked: Automatic, each row, then
/// each set `labels` names (in `paths`' order); a set that isn't here
/// comes last, picked.
fn choices(pick: &Pick, paths: &[&Path], labels: &[String]) -> (Vec<String>, u32) {
    let mut options = vec!["Automatic (the row its instrument names)".to_owned()];
    options.extend(Role::ALL.map(|r| format!("Like {}", r.title().to_lowercase())));
    options.extend(labels.iter().cloned());
    let rows = 1 + Role::ALL.len();
    let picked = match pick {
        Pick::Automatic => Some(0),
        Pick::Role(r) => Role::ALL.iter().position(|x| x == r).map(|i| i + 1),
        Pick::Set(p) => paths
            .iter()
            .position(|f| *f == p.as_path())
            .map(|i| i + rows),
    };
    let picked = picked.unwrap_or_else(|| {
        let name = match pick {
            Pick::Set(p) => p
                .file_name()
                .map_or_else(|| p.display().to_string(), |n| n.to_string_lossy().into()),
            _ => String::new(),
        };
        options.push(format!("{name} (not here: plays as Automatic)"));
        options.len() - 1
    });
    (options, picked as u32)
}

/// What option `i` of [`choices`] saves; `None` for the missing set's
/// line, which changes nothing.
fn chosen(i: usize, paths: &[&Path]) -> Option<Pick> {
    let rows = 1 + Role::ALL.len();
    if i == 0 {
        Some(Pick::Automatic)
    } else if i < rows {
        Role::ALL.get(i - 1).copied().map(Pick::Role)
    } else {
        paths.get(i - rows).map(|p| Pick::Set(p.to_path_buf()))
    }
}

impl TrackSection for Window {
    fn content(&self, _track: &TrackInfo, settings: &Map<String, Value>) -> TabContent {
        let found = self.found();
        let paths: Vec<&Path> = found.iter().map(|f| f.path.as_path()).collect();
        let labels: Vec<String> = found.iter().map(|f| self.label(f)).collect();
        let (options, selected) = choices(&Pick::of(settings), &paths, &labels);
        TabContent {
            rows: vec![
                Row {
                    widgets: vec![
                        Widget::Label {
                            text: "Sound".into(),
                            muted: false,
                        },
                        Widget::Choice {
                            id: KEY.into(),
                            options,
                            selected,
                            enabled: true,
                        },
                    ],
                },
                Row {
                    widgets: vec![Widget::Label {
                        text: "The rows are the plugin's settings window's.".into(),
                        muted: true,
                    }],
                },
            ],
        }
    }

    fn on(
        &self,
        _track: &TrackInfo,
        settings: &mut Map<String, Value>,
        widget: &str,
        event: TabEvent,
    ) -> Result<(), Error> {
        if widget != KEY {
            return Err(Error::not_found(format!("no widget {widget:?}")));
        }
        let TabEvent::Chosen(i) = event else {
            return Ok(());
        };
        let found = self.found();
        let paths: Vec<&Path> = found.iter().map(|f| f.path.as_path()).collect();
        if let Some(pick) = chosen(i as usize, &paths) {
            pick.save(settings);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sets::{Kind, Origin};

    fn map(json: &str) -> Map<String, Value> {
        match serde_json::from_str(json).unwrap() {
            Value::Object(m) => m,
            _ => unreachable!(),
        }
    }

    #[test]
    fn a_tracks_pick_is_read_and_saved() {
        assert_eq!(Pick::of(&map("{}")), Pick::Automatic);
        assert_eq!(
            Pick::of(&map(r#"{"sound": "role:clean-guitars"}"#)),
            Pick::Role(Role::CleanGuitars)
        );
        assert_eq!(
            Pick::of(&map(r#"{"sound": "role:tubas"}"#)),
            Pick::Automatic
        );
        assert_eq!(
            Pick::of_text(r#"{"sound": "/sets/a.sfz"}"#),
            Pick::Set("/sets/a.sfz".into())
        );
        assert_eq!(Pick::of_text("not json"), Pick::Automatic);
        let mut m = map(r#"{"other": 1}"#);
        Pick::Role(Role::Basses).save(&mut m);
        assert_eq!(Pick::of(&m), Pick::Role(Role::Basses));
        Pick::Automatic.save(&mut m);
        assert_eq!(m, map(r#"{"other": 1}"#), "the rest is kept");
    }

    #[test]
    fn the_choice_lists_automatic_the_rows_and_the_sets() {
        let (a, b) = (Path::new("/sets/a.sf2"), Path::new("/sets/b.sfz"));
        let labels = ["A".to_owned(), "B".to_owned()];
        let (options, picked) = choices(&Pick::Automatic, &[a, b], &labels);
        assert_eq!(options.len(), 1 + 7 + 2);
        assert_eq!(options[1], "Like clean guitars");
        assert_eq!(options[7], "Like everything else");
        assert_eq!(picked, 0);
        let (_, picked) = choices(&Pick::Role(Role::DistortedGuitars), &[a, b], &labels);
        assert_eq!(
            chosen(picked as usize, &[a, b]),
            Some(Pick::Role(Role::DistortedGuitars))
        );
        let (_, picked) = choices(&Pick::Set(b.into()), &[a, b], &labels);
        assert_eq!(picked, 9);
        assert_eq!(chosen(9, &[a, b]), Some(Pick::Set(b.into())));
        // A set that isn't here is shown, picked, and choosing it again
        // changes nothing.
        let gone = Pick::Set("/elsewhere/kit.sfz".into());
        let (options, picked) = choices(&gone, &[a, b], &labels);
        assert_eq!(
            options.last().unwrap(),
            "kit.sfz (not here: plays as Automatic)"
        );
        assert_eq!(picked as usize, options.len() - 1);
        assert_eq!(chosen(picked as usize, &[a, b]), None);
    }

    #[test]
    fn a_track_plays_its_pick_or_its_row() {
        let set = |name: &str| Found {
            path: PathBuf::from(format!("/sets/{name}")),
            kind: Kind::Sf2,
            origin: Origin::User,
            size: 0,
            info: None,
        };
        let found = [set("clean.sf2"), set("dist.sf2"), set("mine.sfz")];
        let row = |r: Role| {
            Some(match r {
                Role::CleanGuitars => &found[0],
                _ => &found[1],
            })
        };
        let auto = Role::DistortedGuitars;
        let path = |f: Option<&Found>| f.map(|f| f.path.clone());
        let (f, missing) = set_for(&Pick::Automatic, auto, &found, row);
        assert_eq!((path(f), missing), (Some(found[1].path.clone()), false));
        let (f, _) = set_for(&Pick::Role(Role::CleanGuitars), auto, &found, row);
        assert_eq!(path(f), Some(found[0].path.clone()), "clean into an amp");
        let (f, _) = set_for(&Pick::Set(found[2].path.clone()), auto, &found, row);
        assert_eq!(path(f), Some(found[2].path.clone()));
        let (f, missing) = set_for(&Pick::Set("/gone.sfz".into()), auto, &found, row);
        assert_eq!((path(f), missing), (Some(found[1].path.clone()), true));
    }
}
