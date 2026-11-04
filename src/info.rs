// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Mateus Rocha de Paula and the OxiGuitar contributors

//! What a set's folder says about it: `oxiguitar-set.json`, written by the
//! sound-set installer beside each set it installs (#146, #165) and free
//! for anyone to write for a set of their own.
//!
//! ```json
//! {
//!   "name": "Black and Green Guitars (Karoryfer)",
//!   "role": "clean-guitars",
//!   "licence": "CC0",
//!   "credit": "Karoryfer Samples",
//!   "source": "https://github.com/sfzinstruments/karoryfer.black-and-green-guitars"
//! }
//! ```
//!
//! A set that holds several instruments (patches) may also say which one
//! plays where nothing is picked and what to call each, by path from the
//! set's folder; the installer writes these when asked to install them all:
//!
//! ```json
//! { "recommended": "Programs/Bass.sfz",
//!   "patches": { "Programs/Bass.sfz": "plucked", "Programs/01-dark.sfz": "dark" } }
//! ```
//!
//! Every key is optional and unknown keys are left alone. The file sits in
//! the set's folder, at any depth below the `soundfonts` folder (so every
//! instrument of an SFZ set, in its `Programs` folder, finds its set's);
//! the nearest one wins. The `soundfonts` folder itself is never a set.
//! Nothing here is fetched: the file is read, and the `source` shown as
//! text, never opened.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use oxiguitar_plugin::serde_json::{self, Value};

use crate::settings::Role;

/// The file's name.
pub(crate) const FILE: &str = "oxiguitar-set.json";

/// The most such a file may hold; a bigger one is not read.
const MAX_BYTES: u64 = 64 * 1024;

/// What the file says.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Info {
    /// What to call the set.
    pub(crate) name: Option<String>,
    /// The kinds of track it is for: one role, or the three electric ones
    /// for `electric-guitars`, the key before #152.
    pub(crate) roles: Vec<Role>,
    /// Its licence, as its authors name it ("CC0", "CC BY 4.0").
    pub(crate) licence: Option<String>,
    /// Who made it, to be credited.
    pub(crate) credit: Option<String>,
    /// Where it comes from, as text.
    pub(crate) source: Option<String>,
    /// The patch that plays where nothing is picked, by path from the set's
    /// folder; every other patch plays only where picked.
    pub(crate) recommended: Option<String>,
    /// What to call each patch, by path from the set's folder.
    pub(crate) patches: BTreeMap<String, String>,
    /// The set's folder, where the file was found (not in the file).
    pub(crate) dir: Option<PathBuf>,
}

impl Info {
    /// Reads the file's text. `None` when it isn't a JSON object or says
    /// nothing usable.
    pub(crate) fn parse(text: &str) -> Option<Info> {
        let Value::Object(map) = serde_json::from_str::<Value>(text).ok()? else {
            return None;
        };
        let string = |key: &str| {
            map.get(key)
                .and_then(Value::as_str)
                .map(|s| s.trim().chars().take(200).collect::<String>())
                .filter(|s| !s.is_empty())
        };
        let info = Info {
            name: string("name"),
            roles: string("role").map_or_else(Vec::new, |r| Role::of_key(&r)),
            // Both spellings, as the installer's author may use either.
            licence: string("licence").or_else(|| string("license")),
            credit: string("credit"),
            source: string("source"),
            recommended: string("recommended"),
            patches: map
                .get("patches")
                .and_then(Value::as_object)
                .map(|o| {
                    o.iter()
                        .take(1000)
                        .filter_map(|(k, v)| {
                            let name = v.as_str()?.trim().chars().take(200).collect::<String>();
                            (!name.is_empty()).then(|| (k.clone(), name))
                        })
                        .collect()
                })
                .unwrap_or_default(),
            dir: None,
        };
        (info != Info::default()).then_some(info)
    }

    /// The path of `file` from the set's folder.
    fn relative<'a>(&self, file: &'a Path) -> Option<&'a Path> {
        file.strip_prefix(self.dir.as_deref()?).ok()
    }

    /// Whether `file`, a patch of this set, plays where nothing is picked:
    /// the recommended one, or any when the set names none.
    pub(crate) fn plays_unpicked(&self, file: &Path) -> bool {
        match (&self.recommended, self.relative(file)) {
            (Some(rec), Some(rel)) => rel == Path::new(rec),
            _ => true,
        }
    }

    /// How the settings window names `file`, a patch of this set: the
    /// source and the patch, "FreePats: Electric Guitar FSBS (clean)". The
    /// source is the set's name, the patch's product and variant come from
    /// the innermost parentheses of its credit ("FreePats (Electric Guitar
    /// FSBS, clean)"), the variant overridden by the name `patches` gives
    /// the file. `None` when the set gives nothing to build that from.
    pub(crate) fn short_label(&self, file: &Path) -> Option<String> {
        let name = self.name.as_deref()?;
        let (product, variant) =
            self.credit
                .as_deref()
                .and_then(innermost)
                .map_or((None, None), |inner| match inner.split_once(',') {
                    Some((p, v)) => (Some(p.trim()), Some(v.trim())),
                    None => (Some(inner.trim()), None),
                });
        let named = self
            .relative(file)
            .and_then(|rel| self.patches.get(rel.to_str()?))
            .map(String::as_str);
        let stem = file.file_stem()?.to_string_lossy();
        let product = product.filter(|p| !p.is_empty()).unwrap_or(&stem);
        Some(match named.or(variant).filter(|v| !v.is_empty()) {
            Some(variant) => format!("{name}: {product} ({variant})"),
            None => format!("{name}: {product}"),
        })
    }
}

/// The text of the last innermost pair of parentheses in `text`, if any.
fn innermost(text: &str) -> Option<&str> {
    let close = text.rfind(')')?;
    let open = text[..close].rfind('(')?;
    Some(&text[open + 1..close])
}

/// The info of the set at `set`, a file found under `root` (the folder the
/// search began in): the nearest `oxiguitar-set.json` in the folders from
/// the file's own up to, but not including, `root`.
pub(crate) fn of(set: &Path, root: &Path) -> Option<Info> {
    let mut dir = set.parent()?;
    while dir != root && dir.starts_with(root) {
        let file = dir.join(FILE);
        if let Ok(meta) = std::fs::metadata(&file)
            && meta.is_file()
            && meta.len() <= MAX_BYTES
            && let Ok(text) = std::fs::read_to_string(&file)
        {
            // A file that can't be read as info ends the search: it is the
            // nearest, and the folder above it is another set's.
            return Info::parse(&text).map(|info| Info {
                dir: Some(dir.to_path_buf()),
                ..info
            });
        }
        dir = dir.parent()?;
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_file_is_read_leniently() {
        let info = Info::parse(
            r#"{"name": " Black and Green Guitars (Karoryfer) ", "role": "distorted-guitars",
                "license": "CC0", "credit": "Karoryfer Samples", "extra": [1, 2]}"#,
        )
        .unwrap();
        assert_eq!(
            info.name.as_deref(),
            Some("Black and Green Guitars (Karoryfer)")
        );
        assert_eq!(info.roles, [Role::DistortedGuitars]);
        // The key before #152 stands for the three electric rows.
        let old = Info::parse(r#"{"role": "electric-guitars"}"#).unwrap();
        assert_eq!(
            old.roles,
            [
                Role::CleanGuitars,
                Role::OverdrivenGuitars,
                Role::DistortedGuitars
            ]
        );
        assert_eq!(info.licence.as_deref(), Some("CC0"));
        assert_eq!(info.credit.as_deref(), Some("Karoryfer Samples"));
        assert_eq!(info.source, None);
        // The British spelling wins when both are there.
        let both = Info::parse(r#"{"licence": "CC BY 4.0", "license": "CC0"}"#).unwrap();
        assert_eq!(both.licence.as_deref(), Some("CC BY 4.0"));
        // An unknown role, a wrong type, or nothing: ignored.
        let odd = Info::parse(r#"{"name": "X", "role": "tubas", "credit": 5}"#).unwrap();
        assert_eq!((odd.roles, odd.credit), (Vec::new(), None));
        for text in ["", "[]", "{", "{}", r#"{"name": "  "}"#, r#"{"name": 5}"#] {
            assert_eq!(Info::parse(text), None, "{text:?}");
        }
    }

    #[test]
    fn patches_are_named_and_one_is_recommended() {
        let text = r#"{"name": "Karoryfer", "credit": "Karoryfer (Black and Blue Basses, plucked)",
            "recommended": "Programs/Bass.sfz", "patches": {"Programs/01-dark.sfz": "dark", "x": 5}}"#;
        let info = Info {
            dir: Some(PathBuf::from("/sf/bass")),
            ..Info::parse(text).unwrap()
        };
        assert_eq!(info.patches.len(), 1);
        let dark = Path::new("/sf/bass/Programs/01-dark.sfz");
        let plucked = Path::new("/sf/bass/Programs/Bass.sfz");
        assert!(info.plays_unpicked(plucked) && !info.plays_unpicked(dark));
        assert_eq!(
            info.short_label(dark).as_deref(),
            Some("Karoryfer: Black and Blue Basses (dark)")
        );
        // The credit's own variant is the one the file is not named for.
        assert_eq!(
            info.short_label(plucked).as_deref(),
            Some("Karoryfer: Black and Blue Basses (plucked)")
        );
        // No name, nothing to build from; no parentheses, the file's name.
        assert_eq!(
            Info::parse(r#"{"role": "basses"}"#)
                .unwrap()
                .short_label(plucked),
            None
        );
        let plain = Info::parse(r#"{"name": "Mine", "credit": "Me"}"#).unwrap();
        assert_eq!(plain.short_label(plucked).as_deref(), Some("Mine: Bass"));
        assert!(plain.plays_unpicked(plucked));
    }

    #[test]
    fn the_nearest_file_below_the_root_wins() {
        let dir = std::env::temp_dir().join(format!("oxiguitar-midi-info-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let root = dir.join("soundfonts");
        let set = root.join("Black and Green Guitars (Karoryfer)");
        std::fs::create_dir_all(set.join("Programs/inner")).unwrap();
        std::fs::write(set.join(FILE), r#"{"name": "Whole set", "role": "basses"}"#).unwrap();
        std::fs::write(
            set.join("Programs/inner").join(FILE),
            r#"{"name": "Inner part"}"#,
        )
        .unwrap();
        // The root's own file labels no set.
        std::fs::write(root.join(FILE), r#"{"name": "Everything"}"#).unwrap();
        std::fs::write(root.join("loose.sf2"), b"").unwrap();

        let name = |p: &Path| of(p, &root).and_then(|i| i.name);
        assert_eq!(
            name(&set.join("Programs/01.sfz")).as_deref(),
            Some("Whole set")
        );
        assert_eq!(
            name(&set.join("Programs/inner/02.sfz")).as_deref(),
            Some("Inner part")
        );
        assert_eq!(name(&root.join("loose.sf2")), None);
        // A file that says nothing usable still ends the search there.
        std::fs::write(set.join("Programs/inner").join(FILE), "oops").unwrap();
        assert_eq!(name(&set.join("Programs/inner/02.sfz")), None);
        // Outside the root nothing is looked at.
        assert_eq!(of(&dir.join("elsewhere/x.sf2"), &root), None);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
