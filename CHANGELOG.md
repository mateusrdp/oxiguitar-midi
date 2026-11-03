# Changelog

All notable changes to this project are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and the project
follows [Semantic Versioning](https://semver.org/).

## [Unreleased]

### Changed

- The advice that only basic sounds are installed, and the settings window, point to the sound-set installer (oxiguitar-bettersounds-soundfonts; `oxiguitar-install-sounds` on Linux), the manual's Better sounds page opening there.
- The `.deb` and `.rpm` suggest `oxiguitar-bettersounds-soundfonts`, the installer of the recommended recorded sound sets. The packages are made by oxiguitar-packages' release workflows now.
- The settings window's "Electric guitars" row is three: Clean guitars (General MIDI programs 26 jazz, 27 clean, 28 muted, 31 harmonics), Overdriven guitars (29) and Distorted guitars (30), so clean and distorted guitars in one song get sets of their own (mateusrdp/oxiguitar#152). The one electric pick of before (`electric-guitars` in settings.json) still holds for all three until each has its own; `oxiguitar-set.json`'s `role` takes `clean-guitars`, `overdriven-guitars` and `distorted-guitars`, and `electric-guitars` there stands for all three.

### Added

- The track dialog's section **Sound** (mateusrdp/oxiguitar#152): a track plays **Automatic** (the settings window's row its General MIDI program names; the default), **a row** of the settings window whatever its program says (a distorted guitar played clean into an amp model), or **a set**, exactly. Kept in the song's `.oxg` file. A set picked there that isn't on this computer plays as Automatic, and the footer says so once.

## [0.1.0] - 2026-11-02

First release in a repository of its own (mateusrdp/oxiguitar#165). The
plugin is the one of OxiGuitar 0.11.0, with its history: id
`oxiguitar.midi`, file `midi.oxp`, replacing `oxiguitar.soundfont`.

### Added

- Playing with SoundFont (2 and 3), DLS and SFZ sound sets; the settings
  window; sets in the user's, the system's and `/usr/share/oxiguitar/soundfonts`
  folders; `oxiguitar-set.json` metadata; the `no-sound-sets` advice (plugin
  interface 1.0 to 1.4).
- `build-oxp.sh`, a contract test that checks the built `.oxp` file through
  OxiGuitar's plugin host, and `tests/realtime.rs`, which plays an SFZ
  instrument through the real-time checks.
- `release.yml`: test builds, and on a `v*` tag a draft release with a zip
  per system, `.deb` and `.rpm` packages, a per-user Windows installer and a
  macOS `.pkg`.

### Changed

- The plugin SDK is a git dependency on the `v0.11.0` tag of
  mateusrdp/oxiguitar. The plugin sits at the repository's root.
- The listening packs use OxiGuitar's command line, Guitar Pro plugins and
  reference songs from the `v0.11.0` tag.
