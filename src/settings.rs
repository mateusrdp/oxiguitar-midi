// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Mateus Rocha de Paula and the OxiGuitar contributors

//! The plugin's settings window: a set for clean, overdriven and distorted
//! electric guitars, nylon and steel-string guitars, basses, drums and everything else,
//! for every file (Mateus, 21 Oct 2026; Guitars split in two on 26 Oct 2026,
//! since an SFZ guitar is one or the other; electric guitars split in three
//! on 3 Nov 2026, #152, as General MIDI names them; acoustic guitars split
//! into nylon and steel strings on 4 Nov 2026), kept in the plugin's
//! own settings as each set's path.
//! Until the user picks, the best General MIDI set found plays; an SFZ
//! instrument plays only where picked.

use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::SystemTime;

use oxiguitar_plugin::audio::PreparedTrack;
use oxiguitar_plugin::serde_json::{Map, Value};
use oxiguitar_plugin::{Error, Host, InstrumentClass, Row, Tab, TabContent, TabEvent, Widget};

use crate::sets::{self, Found, Kind, Origin};

/// The kinds of track the window picks a set for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Role {
    CleanGuitars,
    OverdrivenGuitars,
    DistortedGuitars,
    NylonGuitars,
    SteelGuitars,
    Basses,
    Drums,
    Other,
}

/// The key 0.9's settings kept one pick for both kinds of guitar under.
const GUITARS: &str = "guitars";

/// The key the settings (and a set's `oxiguitar-set.json`) named acoustic
/// guitars under, one row for nylon and steel strings alike, until they were
/// split (4 Nov 2026).
pub(crate) const ACOUSTIC_GUITARS: &str = "acoustic-guitars";

/// The key the settings kept one pick for every electric guitar under,
/// until the three electric rows (#152).
pub(crate) const ELECTRIC_GUITARS: &str = "electric-guitars";

impl Role {
    pub(crate) const ALL: [Role; 8] = [
        Role::CleanGuitars,
        Role::OverdrivenGuitars,
        Role::DistortedGuitars,
        Role::NylonGuitars,
        Role::SteelGuitars,
        Role::Basses,
        Role::Drums,
        Role::Other,
    ];

    /// Its key in the settings and its widget's id.
    pub(crate) fn key(self) -> &'static str {
        match self {
            Role::CleanGuitars => "clean-guitars",
            Role::OverdrivenGuitars => "overdriven-guitars",
            Role::DistortedGuitars => "distorted-guitars",
            Role::NylonGuitars => "nylon-guitars",
            Role::SteelGuitars => "steel-guitars",
            Role::Basses => "basses",
            Role::Drums => "drums",
            Role::Other => "other",
        }
    }

    /// The role a key names (as `oxiguitar-set.json` writes it).
    pub(crate) fn from_key(key: &str) -> Option<Role> {
        Role::ALL.into_iter().find(|r| r.key() == key)
    }

    /// The roles a set's `oxiguitar-set.json` key names: one, the three
    /// electric ones for the key before #152, `electric-guitars`, or both
    /// acoustic ones for the key before they were split, `acoustic-guitars`.
    pub(crate) fn of_key(key: &str) -> Vec<Role> {
        if key == ELECTRIC_GUITARS {
            Role::ALL.into_iter().filter(|r| r.electric()).collect()
        } else if key == ACOUSTIC_GUITARS {
            Role::ALL.into_iter().filter(|r| r.acoustic()).collect()
        } else {
            Role::from_key(key).into_iter().collect()
        }
    }

    pub(crate) fn title(self) -> &'static str {
        match self {
            Role::CleanGuitars => "Clean guitars",
            Role::OverdrivenGuitars => "Overdriven guitars",
            Role::DistortedGuitars => "Distorted guitars",
            Role::NylonGuitars => "Nylon-string guitars",
            Role::SteelGuitars => "Steel-string guitars",
            Role::Basses => "Basses",
            Role::Drums => "Drums",
            Role::Other => "Everything else",
        }
    }

    fn guitar(self) -> bool {
        self.electric() || self.acoustic()
    }

    fn acoustic(self) -> bool {
        matches!(self, Role::NylonGuitars | Role::SteelGuitars)
    }

    fn electric(self) -> bool {
        matches!(
            self,
            Role::CleanGuitars | Role::OverdrivenGuitars | Role::DistortedGuitars
        )
    }

    /// The role of a track: drum kits; basses by their General MIDI
    /// program (32 to 39: acoustic, electric, fretless, slap and synth);
    /// guitars, nylon (24), steel (25), overdriven (29), distorted (30) or
    /// clean (26 jazz, 27 clean, 28 muted, 31 harmonics); then the rest,
    /// the other fretted instruments (banjo, sitar…) among them, so they
    /// keep their General MIDI sounds whatever the guitars play.
    pub(crate) fn of(track: &PreparedTrack) -> Role {
        if track.percussion {
            Role::Drums
        } else if (32..=39).contains(&track.program) {
            Role::Basses
        } else if track.info.class == InstrumentClass::Fretted && track.program == 24 {
            Role::NylonGuitars
        } else if track.info.class == InstrumentClass::Fretted && track.program == 25 {
            Role::SteelGuitars
        } else if track.info.class == InstrumentClass::Fretted && (26..=31).contains(&track.program)
        {
            match track.program {
                29 => Role::OverdrivenGuitars,
                30 => Role::DistortedGuitars,
                _ => Role::CleanGuitars,
            }
        } else {
            Role::Other
        }
    }

    /// What `settings` picks for this role: its own key; for electric
    /// guitars, the one pick they shared before #152; for guitars, 0.9's
    /// one pick for both kinds.
    fn pick(self, settings: &Map<String, Value>) -> Option<&str> {
        settings
            .get(self.key())
            .or_else(|| {
                self.electric()
                    .then(|| settings.get(ELECTRIC_GUITARS))
                    .flatten()
            })
            .or_else(|| {
                self.acoustic()
                    .then(|| settings.get(ACOUSTIC_GUITARS))
                    .flatten()
            })
            .or_else(|| self.guitar().then(|| settings.get(GUITARS)).flatten())
            .and_then(Value::as_str)
    }
}

/// The set `settings` picks for `role` among `found`, if it picks one that
/// is still there; else [`automatic`].
pub(crate) fn chosen<'a>(
    settings: &Map<String, Value>,
    role: Role,
    found: &'a [Found],
) -> Option<&'a Found> {
    role.pick(settings)
        .and_then(|p| {
            found
                .iter()
                .find(|f| f.path.as_os_str() == std::ffi::OsStr::new(p))
        })
        .or_else(|| automatic(role, found))
}

/// What `role` plays where nothing is picked: the best set whose
/// `oxiguitar-set.json` names the role (the sound-set installer's sets do),
/// so better sounds are used as soon as they are installed; else the best
/// General MIDI set. An SFZ instrument that doesn't name its role plays only
/// where picked (Mateus, 26 Oct 2026; 4 Nov 2026 for the sets that do).
/// `found` is best first: the user's before the installed ones, those before
/// the system's, then the larger before the smaller.
pub(crate) fn automatic(role: Role, found: &[Found]) -> Option<&Found> {
    let named = |f: &&Found| f.info.as_ref().is_some_and(|i| i.roles.contains(&role));
    // A set with several patches plays only its recommended one unpicked.
    found
        .iter()
        .find(|f| named(f) && f.info.as_ref().is_some_and(|i| i.plays_unpicked(&f.path)))
        .or_else(|| found.iter().find(named))
        .or_else(|| found.iter().find(|f| f.kind != Kind::Sfz))
}

/// How the window names an SFZ instrument: its set's folder (the first
/// folder under the `soundfonts` folder) and its own name.
fn sfz_label(f: &Found, user: Option<&Path>) -> String {
    let stem = f
        .path
        .file_stem()
        .map_or_else(String::new, |s| s.to_string_lossy().into_owned());
    let set = user
        .and_then(|u| f.path.strip_prefix(u).ok())
        .and_then(|rest| rest.components().next())
        .map(|c| c.as_os_str().to_string_lossy().into_owned())
        .filter(|s| f.path.file_name().is_none_or(|n| n != s.as_str()));
    match set {
        Some(set) => format!("{set}: {stem} (SFZ)"),
        None => format!("{stem} (SFZ)"),
    }
}

/// How the window names a set: by the file's name, or by the name its
/// `oxiguitar-set.json` gives, with the licence and the credit it names.
fn label(f: &Found) -> String {
    let file = f.path.file_name().map_or_else(
        || f.path.display().to_string(),
        |n| n.to_string_lossy().into_owned(),
    );
    let whose = match f.origin {
        Origin::User => "yours",
        Origin::Installed => "installed",
        Origin::System => "the system's",
    };
    let size = f.size.div_ceil(1_000_000);
    let Some(info) = f.info.as_ref() else {
        return format!("{file} ({whose}, {size} MB)");
    };
    if let Some(short) = info.short_label(&f.path) {
        return short;
    }
    let mut label = match &info.name {
        Some(name) => format!("{name}: {file}"),
        None => file,
    };
    let mut notes = vec![whose.to_string(), format!("{size} MB")];
    notes.extend(info.licence.clone());
    notes.extend(info.credit.as_ref().map(|c| format!("by {c}")));
    label.push_str(&format!(" ({})", notes.join(", ")));
    label
}

/// Whether an SFZ file plays as an instrument, as it was when checked.
#[derive(Debug)]
struct Checked {
    path: PathBuf,
    modified: Option<SystemTime>,
    size: u64,
    instrument: bool,
}

/// The settings window.
#[derive(Debug)]
pub(crate) struct Window {
    pub(crate) host: Host,
    /// The SFZ files checked so far.
    checked: Mutex<Vec<Checked>>,
}

impl Window {
    pub(crate) fn new(host: Host) -> Window {
        Window {
            host,
            checked: Mutex::new(Vec::new()),
        }
    }

    /// Every set found, the General MIDI ones first, then the SFZ
    /// instruments that play on their own.
    pub(crate) fn found(&self) -> Vec<Found> {
        let found = crate::playable(&self.host);
        let (sfz, mut sets): (Vec<Found>, Vec<Found>) =
            found.into_iter().partition(|f| f.kind == Kind::Sfz);
        let mut sfz: Vec<Found> = sfz
            .into_iter()
            .filter(|f| self.instrument(&f.path))
            .collect();
        sfz.sort_by(|a, b| a.path.cmp(&b.path));
        sets.extend(sfz);
        sets
    }

    /// Sets relevant for `role`: those whose `info.roles` includes it,
    /// or that have no role info (General MIDI fallbacks).
    pub(crate) fn found_for_role(&self, role: Role) -> Vec<Found> {
        self.found()
            .into_iter()
            .filter(|f| f.info.as_ref().is_none_or(|i| i.roles.contains(&role)))
            .collect()
    }

    /// Whether the SFZ file at `path` is an instrument, not a part others
    /// include: it has regions, and every `$name` it uses is defined
    /// (Karoryfer's sets keep parts that work only when included). Kept
    /// while the file is unchanged.
    fn instrument(&self, path: &Path) -> bool {
        let meta = std::fs::metadata(path).ok();
        let modified = meta.as_ref().and_then(|m| m.modified().ok());
        let size = meta.as_ref().map_or(0, std::fs::Metadata::len);
        if let Ok(checked) = self.checked.lock()
            && let Some(c) = checked
                .iter()
                .find(|c| c.path == path && c.modified == modified && c.size == size)
        {
            return c.instrument;
        }
        let instrument = crate::sfz::text::read(path).is_ok_and(|t| {
            t.items
                .iter()
                .any(|i| matches!(i, crate::sfz::text::Item::Header(h) if h == "region"))
                && !t.problems.iter().any(|p| p.ends_with("isn't defined"))
                && !matches!(t.items.first(), Some(crate::sfz::text::Item::Opcode(..)))
        });
        if let Ok(mut checked) = self.checked.lock() {
            checked.retain(|c| c.path != path);
            checked.push(Checked {
                path: path.to_path_buf(),
                modified,
                size,
                instrument,
            });
        }
        instrument
    }

    pub(crate) fn label(&self, f: &Found) -> String {
        let short = f.info.as_ref().and_then(|i| i.short_label(&f.path));
        if let Some(short) = short {
            short
        } else if f.kind == Kind::Sfz {
            sfz_label(f, crate::user_folder(&self.host).as_deref())
        } else {
            label(f)
        }
    }
}

/// The advice on better sounds at the top of the window (Mateus, 27 Oct
/// 2026: in the settings window and the footer both, with the link, towards
/// SFZ). OxiGuitar downloads nothing, so the link opens the manual's page,
/// which says what to get, where, and what to pick here. Labels don't wrap:
/// one short sentence a row.
fn better_sounds(found: &[Found]) -> Vec<Row> {
    let line = |text: &str| Row {
        widgets: vec![Widget::Label {
            text: text.into(),
            muted: false,
        }],
    };
    let link = |text: &str| Widget::Link {
        text: text.into(),
        url: crate::BETTER_SOUNDS.into(),
    };
    if found.iter().all(|f| f.origin == Origin::System) {
        vec![
            line("Only your system's basic General MIDI set plays now."),
            line("Free recorded SFZ sets sound far better."),
            Row {
                widgets: vec![link(
                    "Better sounds: the sound-set installer, and how to pick sets",
                )],
            },
        ]
    } else {
        vec![Row {
            widgets: vec![
                Widget::Label {
                    text: "The recommended free sets, and how to pick them:".into(),
                    muted: true,
                },
                link("Better sounds"),
            ],
        }]
    }
}

impl Tab for Window {
    fn content(&self) -> TabContent {
        let settings = self.host.settings().unwrap_or_default();
        let mut rows = better_sounds(&self.found());
        for role in Role::ALL {
            let found = self.found_for_role(role);
            let labels: Vec<String> = found.iter().map(|f| self.label(f)).collect();
            let plays =
                automatic(role, &found).map_or_else(|| "none found".to_owned(), |f| self.label(f));
            let mut options = vec![format!("Automatic: {plays}")];
            options.extend(labels.iter().cloned());
            let picked = role
                .pick(&settings)
                .and_then(|p| {
                    found
                        .iter()
                        .position(|f| f.path.as_os_str() == std::ffi::OsStr::new(p))
                })
                .map_or(0, |i| i + 1);
            rows.push(Row {
                widgets: vec![
                    Widget::Label {
                        text: role.title().into(),
                        muted: false,
                    },
                    Widget::Choice {
                        id: role.key().into(),
                        options,
                        selected: picked as u32,
                        enabled: !found.is_empty(),
                    },
                ],
            });
        }
        let folder = crate::user_folder(&self.host).map_or_else(
            || "the soundfonts folder".to_owned(),
            |p| p.display().to_string(),
        );
        // Two rows: labels don't wrap, and the folder's path can be long.
        rows.push(Row {
            widgets: vec![Widget::Label {
                text: "Put .sf2, .sf3 or .dls files, or SFZ instruments' folders, in:".into(),
                muted: true,
            }],
        });
        rows.push(Row {
            widgets: vec![Widget::Label {
                text: folder,
                muted: true,
            }],
        });
        TabContent { rows }
    }

    fn on(&self, widget: &str, event: TabEvent) -> Result<(), Error> {
        let Some(role) = Role::ALL.into_iter().find(|r| r.key() == widget) else {
            return Err(Error::not_found(format!("no widget {widget:?}")));
        };
        let TabEvent::Chosen(i) = event else {
            return Ok(());
        };
        let found = self.found_for_role(role);
        let mut settings = self.host.settings()?;
        // The older shared picks go to every row they stood for, so
        // picking one row leaves the others as they were: the electric one
        // (before #152) to the three electric rows, then 0.9's one for
        // guitars to every guitar row still without a pick.
        if role.guitar() {
            if let Some(old) = settings.remove(ELECTRIC_GUITARS) {
                for r in Role::ALL.into_iter().filter(|r| r.electric()) {
                    settings.entry(r.key()).or_insert_with(|| old.clone());
                }
            }
            if let Some(old) = settings.remove(ACOUSTIC_GUITARS) {
                for r in Role::ALL.into_iter().filter(|r| r.acoustic()) {
                    settings.entry(r.key()).or_insert_with(|| old.clone());
                }
            }
            if let Some(old) = settings.remove(GUITARS) {
                for r in Role::ALL.into_iter().filter(|r| r.guitar()) {
                    settings.entry(r.key()).or_insert_with(|| old.clone());
                }
            }
        }
        match (i as usize).checked_sub(1).and_then(|i| found.get(i)) {
            Some(f) => {
                settings.insert(role.key().into(), f.path.display().to_string().into());
            }
            None => {
                settings.remove(role.key());
            }
        }
        self.host.set_settings(&settings)
    }
}

/// Whether every set in `used` is the system's (all basic, Mateus, 25 Oct
/// 2026: every system set counts as basic, any set of the user's doesn't).
pub(crate) fn only_basic(used: &[&Found]) -> bool {
    !used.is_empty() && used.iter().all(|f| f.origin == sets::Origin::System)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use oxiguitar_plugin::api::message::TrackInfo;

    use super::*;
    use crate::sets::Kind;

    fn track(program: u8, percussion: bool, class: InstrumentClass) -> PreparedTrack {
        PreparedTrack {
            info: TrackInfo {
                index: 0,
                name: String::new(),
                class,
            },
            program,
            percussion,
            tuning: Vec::new(),
            capo: 0,
            settings: "{}".into(),
        }
    }

    #[test]
    fn tracks_fall_into_the_eight_roles() {
        let electric = |p| Role::of(&track(p, false, InstrumentClass::Fretted));
        assert_eq!(
            [26, 27, 28, 29, 30, 31].map(electric),
            [
                Role::CleanGuitars,
                Role::CleanGuitars,
                Role::CleanGuitars,
                Role::OverdrivenGuitars,
                Role::DistortedGuitars,
                Role::CleanGuitars
            ]
        );
        assert_eq!(
            Role::of(&track(24, false, InstrumentClass::Fretted)),
            Role::NylonGuitars
        );
        assert_eq!(
            Role::of(&track(25, false, InstrumentClass::Fretted)),
            Role::SteelGuitars
        );
        assert_eq!(
            Role::of(&track(105, false, InstrumentClass::Fretted)),
            Role::Other
        );
        assert_eq!(
            Role::of(&track(33, false, InstrumentClass::Fretted)),
            Role::Basses
        );
        assert_eq!(
            Role::of(&track(35, false, InstrumentClass::Fretless)),
            Role::Basses
        );
        assert_eq!(
            Role::of(&track(38, false, InstrumentClass::Synth)),
            Role::Basses
        );
        assert_eq!(
            Role::of(&track(0, true, InstrumentClass::Drums)),
            Role::Drums
        );
        assert_eq!(
            Role::of(&track(0, false, InstrumentClass::Keyboard)),
            Role::Other
        );
    }

    fn found(name: &str, origin: Origin) -> Found {
        Found {
            path: PathBuf::from(format!("/sets/{name}")),
            size: 1,
            origin,
            kind: Kind::Sf2,
            info: None,
        }
    }

    #[test]
    fn a_pick_holds_while_its_set_is_there() {
        let sets = [
            found("best.sf2", Origin::User),
            found("other.sf2", Origin::System),
        ];
        let mut s = Map::new();
        assert_eq!(chosen(&s, Role::Drums, &sets).unwrap().path, sets[0].path);
        s.insert("drums".into(), "/sets/other.sf2".into());
        assert_eq!(chosen(&s, Role::Drums, &sets).unwrap().path, sets[1].path);
        assert_eq!(chosen(&s, Role::Basses, &sets).unwrap().path, sets[0].path);
        s.insert("drums".into(), "/sets/gone.sf2".into());
        assert_eq!(chosen(&s, Role::Drums, &sets).unwrap().path, sets[0].path);
        assert!(chosen(&s, Role::Drums, &[]).is_none());
    }

    #[test]
    fn a_set_s_old_acoustic_role_stands_for_nylon_and_steel() {
        assert_eq!(
            Role::of_key("acoustic-guitars"),
            [Role::NylonGuitars, Role::SteelGuitars]
        );
        assert_eq!(Role::of_key("steel-guitars"), [Role::SteelGuitars]);
        assert!(Role::of_key("tubas").is_empty());
    }

    #[test]
    fn sets_that_name_a_role_play_it_without_a_pick() {
        let mut clean = found("clean/guitar.sfz", Origin::Installed);
        clean.kind = Kind::Sfz;
        clean.info = crate::info::Info::parse(r#"{"role": "clean-guitars"}"#);
        let mut distorted = found("dist/guitar.sfz", Origin::Installed);
        distorted.kind = Kind::Sfz;
        distorted.info = crate::info::Info::parse(r#"{"role": "distorted-guitars"}"#);
        let mut bare = found("bare/guitar.sfz", Origin::User);
        bare.kind = Kind::Sfz;
        let sets = [bare, found("best.sf2", Origin::User), clean, distorted];
        let mut s = Map::new();
        let path = |s: &Map<String, Value>, r| chosen(s, r, &sets).unwrap().path.clone();
        // Each row plays the set made for it; an SFZ set that names no role
        // is not used; the rows nobody made a set for play the best GM set.
        assert_eq!(path(&s, Role::CleanGuitars), sets[2].path);
        assert_eq!(path(&s, Role::DistortedGuitars), sets[3].path);
        assert_eq!(path(&s, Role::OverdrivenGuitars), sets[1].path);
        assert_eq!(path(&s, Role::Drums), sets[1].path);
        // A pick still wins, and so does falling back when the pick is gone.
        s.insert("clean-guitars".into(), "/sets/best.sf2".into());
        assert_eq!(path(&s, Role::CleanGuitars), sets[1].path);
        s.insert("clean-guitars".into(), "/sets/gone.sf2".into());
        assert_eq!(path(&s, Role::CleanGuitars), sets[2].path);
    }

    #[test]
    fn the_old_guitars_pick_holds_for_both_kinds_and_sfz_only_where_picked() {
        let mut sfz = found("kit/guitar.sfz", Origin::User);
        sfz.kind = Kind::Sfz;
        let sets = [sfz, found("best.sf2", Origin::User)];
        let mut s = Map::new();
        // An SFZ instrument is never the best found.
        assert_eq!(
            chosen(&s, Role::DistortedGuitars, &sets).unwrap().path,
            sets[1].path
        );
        s.insert("guitars".into(), "/sets/kit/guitar.sfz".into());
        assert_eq!(
            chosen(&s, Role::DistortedGuitars, &sets).unwrap().path,
            sets[0].path
        );
        assert_eq!(
            chosen(&s, Role::SteelGuitars, &sets).unwrap().path,
            sets[0].path
        );
        assert_eq!(chosen(&s, Role::Basses, &sets).unwrap().path, sets[1].path);
        // The one acoustic pick of before the split stands for both rows.
        s.insert("acoustic-guitars".into(), "/sets/best.sf2".into());
        for r in [Role::NylonGuitars, Role::SteelGuitars] {
            assert_eq!(chosen(&s, r, &sets).unwrap().path, sets[1].path, "{r:?}");
        }
        assert_eq!(
            chosen(&s, Role::DistortedGuitars, &sets).unwrap().path,
            sets[0].path
        );
    }

    /// #152: the one electric pick of before stands for the three electric
    /// rows until each has its own, and picking one row keeps it for the
    /// other two.
    #[test]
    fn the_old_electric_pick_holds_for_the_three_electric_rows() {
        let sets = [found("a.sf2", Origin::User), found("b.sf2", Origin::User)];
        let mut s = Map::new();
        s.insert(ELECTRIC_GUITARS.into(), "/sets/b.sf2".into());
        s.insert(GUITARS.into(), "/sets/a.sf2".into());
        for r in [
            Role::CleanGuitars,
            Role::OverdrivenGuitars,
            Role::DistortedGuitars,
        ] {
            assert_eq!(chosen(&s, r, &sets).unwrap().path, sets[1].path, "{r:?}");
        }
        assert_eq!(
            chosen(&s, Role::SteelGuitars, &sets).unwrap().path,
            sets[0].path,
            "0.9's pick still holds for acoustic guitars"
        );
        s.insert("distorted-guitars".into(), "/sets/a.sf2".into());
        assert_eq!(
            chosen(&s, Role::DistortedGuitars, &sets).unwrap().path,
            sets[0].path
        );
        assert_eq!(
            chosen(&s, Role::CleanGuitars, &sets).unwrap().path,
            sets[1].path
        );
    }

    #[test]
    fn sfz_instruments_are_named_by_their_set_and_their_own_name() {
        let mut f = found("x", Origin::User);
        f.kind = Kind::Sfz;
        f.path = PathBuf::from(
            "/home/u/soundfonts/karoryfer.black-and-green-guitars/Programs/01-green_keyswitch.sfz",
        );
        assert_eq!(
            sfz_label(&f, Some(Path::new("/home/u/soundfonts"))),
            "karoryfer.black-and-green-guitars: 01-green_keyswitch (SFZ)"
        );
        f.path = PathBuf::from("/home/u/soundfonts/loose.sfz");
        assert_eq!(
            sfz_label(&f, Some(Path::new("/home/u/soundfonts"))),
            "loose (SFZ)"
        );
    }

    #[test]
    fn only_the_systems_sets_are_basic() {
        let (mine, theirs) = (found("a.sf2", Origin::User), found("b.dls", Origin::System));
        assert!(only_basic(&[&theirs]));
        assert!(!only_basic(&[&theirs, &mine]));
        assert!(!only_basic(&[]));
    }

    #[test]
    fn better_sounds_are_advised_with_the_link() {
        let links = |rows: &[Row]| -> Vec<String> {
            rows.iter()
                .flat_map(|r| &r.widgets)
                .filter_map(|w| match w {
                    Widget::Link { url, .. } => Some(url.clone()),
                    _ => None,
                })
                .collect()
        };
        let basic = better_sounds(&[found("gm.dls", Origin::System)]);
        assert_eq!(basic.len(), 3);
        assert_eq!(links(&basic), [crate::BETTER_SOUNDS]);
        assert_eq!(links(&better_sounds(&[])), [crate::BETTER_SOUNDS]);
        let yours = better_sounds(&[
            found("gm.dls", Origin::System),
            found("FluidR3_GM.sf2", Origin::User),
        ]);
        assert_eq!(yours.len(), 1);
        assert_eq!(links(&yours), [crate::BETTER_SOUNDS]);
        assert!(crate::BETTER_SOUNDS.starts_with("https://"));
    }

    #[test]
    fn a_sets_info_names_it() {
        let mut set = found("guitars/a.sf2", Origin::Installed);
        assert_eq!(label(&set), "a.sf2 (installed, 1 MB)");
        set.info = crate::info::Info::parse(
            r#"{"name": "Black and Green Guitars", "licence": "CC0", "credit": "Karoryfer"}"#,
        );
        // Short: the source and the patch, nothing else.
        assert_eq!(label(&set), "Black and Green Guitars: a");
        set.info = crate::info::Info::parse(
            r#"{"name": "FreePats", "role": "clean-guitars", "licence": "CC0 1.0",
                "credit": "FreePats (Electric Guitar FSBS, clean)"}"#,
        );
        assert_eq!(label(&set), "FreePats: Electric Guitar FSBS (clean)");
    }

    #[test]
    fn only_the_recommended_patch_plays_unpicked() {
        let info = crate::info::Info::parse(
            r#"{"name": "Karoryfer", "role": "basses", "credit": "Karoryfer (Black and Blue Basses, plucked)",
                "recommended": "Programs/Bass.sfz",
                "patches": {"Programs/Bass.sfz": "plucked", "Programs/01-dark.sfz": "dark"}}"#,
        )
        .map(|i| crate::info::Info {
            dir: Some(PathBuf::from("/sets/bass")),
            ..i
        });
        let mut dark = found("bass/Programs/01-dark.sfz", Origin::Installed);
        dark.info = info.clone();
        let mut plucked = found("bass/Programs/Bass.sfz", Origin::Installed);
        plucked.info = info;
        assert_eq!(label(&dark), "Karoryfer: Black and Blue Basses (dark)");
        assert_eq!(
            label(&plucked),
            "Karoryfer: Black and Blue Basses (plucked)"
        );
        let both = [dark, plucked];
        let plays = automatic(Role::Basses, &both).unwrap();
        assert!(plays.path.ends_with("Bass.sfz"));
    }
}
