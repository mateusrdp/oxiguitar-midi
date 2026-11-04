# oxiguitar-midi

The MIDI plugin for [OxiGuitar](https://github.com/mateusrdp/oxiguitar), a free, open-source guitar tablature and score viewer, editor and player for Windows, macOS and Linux. It is what makes a score sound: it plays every track with a sound set, one of **SoundFont** (`.sf2`, `.sf3`), **DLS** (`.dls`) or **SFZ** (`.sfz`) instruments, the one you pick or else the best your system has.

- Plugin file `midi.oxp`, id `oxiguitar.midi`, shown as **MIDI (SoundFont, SFZ)**. It replaces the former `oxiguitar.soundfont`: its settings and data folder are carried over once, and a score's track settings keep working. A `soundfont.oxp` left in your plugin folder is not used any more and can be deleted.
- Sound sets are plain files. Put yours in OxiGuitar's `soundfonts` folder, or install a recorded set with [oxiguitar-bettersounds-soundfonts](https://github.com/mateusrdp/oxiguitar-bettersounds-soundfonts); the plugin also looks in `/usr/share/oxiguitar/soundfonts` on Linux and reads an `oxiguitar-set.json` beside a set (name, role, licence, credit, and for a set of several patches which is recommended and what each is called) to list it by a short name.
- The settings window (Tools, Plugins, MIDI, Settings) picks a set for clean, overdriven and distorted electric guitars, nylon-string and steel-string guitars, basses, drums and everything else. When it finds only the basic General MIDI set of your system, OxiGuitar tells you once, with a link to the [Better sounds](https://mateusrdp.github.io/oxiguitar/better-sounds.html) page (SFZ sets first).
- The track dialog's section **Sound** lets one track play another row of the settings window (a distorted guitar played clean into an amp model) or a set of its own; Automatic, the default, follows the track's instrument.
- The SFZ player is our own: 256 voices, sfizz's behaviour as the reference (no code taken), samples in WAV, FLAC or Ogg, whole in memory. The SoundFont synthesiser is [rustysynth](https://crates.io/crates/rustysynth) (MIT), used exactly as published.

Listen to it on the manual's [Hear it](https://mateusrdp.github.io/oxiguitar/hear-it.html) page, which has the same songs played with each recommended set.

## Install

Every [release](https://github.com/mateusrdp/oxiguitar-midi/releases) holds a zip per system (Linux x86_64 and ARM64, Windows x86_64 and ARM64, macOS), a `.deb` and an `.rpm` for Linux, a per-user Windows installer (no administrator needed), a macOS `.pkg`, and a `SHA256SUMS` file. The OxiGuitar manual's [install page](https://mateusrdp.github.io/oxiguitar/installing.html) says which one to use and how to keep the plugin updated through your package manager. The plugin needs OxiGuitar 0.11 or later (plugin interface 1.4).

## Build and test

Rust 1.99 or later from [rustup](https://rustup.rs). The plugin SDK comes from OxiGuitar's repository by git tag (`v0.11.0`); nothing of OxiGuitar's is on crates.io.

```sh
cargo test          # unit tests, the contract suite on the built .oxp file, and the real-time checks with an SFZ set
./build-oxp.sh      # dist/midi-VERSION-SYSTEM-ARCH.oxp
```

`.github/workflows/listening.yml` renders OxiGuitar's reference songs with the sets Debian and Ubuntu package and the recommended SFZ sets, for listening by ear (by hand from the Actions tab). CI fetches test material there; the plugin itself downloads and uploads nothing.

## History

This plugin lived in the [OxiGuitar repository](https://github.com/mateusrdp/oxiguitar) (as `plugins/soundfont` and then `plugins/midi`) until its version 0.11.0. Its history came along; an issue number such as `mateusrdp/oxiguitar#123` in an old commit message is in that repository.

## Licence

GPL-3.0-or-later. See LICENSE. Contributions are welcome under the same terms; see [CONTRIBUTING.md](CONTRIBUTING.md).
