# Architecture and strategy

## How it fits into OxiGuitar

The plugin is an *instrument*. OxiGuitar hands it each track's notes, already turned into a timeline of MIDI-like events, and the plugin renders audio on the audio thread with no memory allocated and no waiting while it plays. The audio then goes through the track's effects (the wah, the amp, the equalizer, if you use them), the mixer and an audio output plugin.

Two engines do the work:

- **SoundFonts and DLS files** go through the [rustysynth](https://github.com/sinshu/rustysynth) library, used as published.
- **SFZ** is our own player: 256 voices, with samples in WAV, FLAC or Ogg Vorbis held whole in memory. It follows [sfizz](https://github.com/sfztools/sfizz)'s behaviour as the reference for what each SFZ setting means, and takes none of its code. Ogg Vorbis is decoded with lewton, FLAC with claxon.

Each string of a guitar track gets its own MIDI channel where the 16 channels allow, so a bend moves one string and not the whole chord.

The plugin's **Sound Sets** dialog picks one set for each kind of track (clean, overdriven and distorted electric guitars, nylon and steel guitars, basses, drums, everything else). A track's **Sound** section can follow another row or use a set of its own. An `oxiguitar-set.json` file beside a set gives it a short name, a role, a licence and a credit.

## The strategy

Make sound with no setup, then make it better with files you drop in a folder. The plugin never downloads anything. Sound sets are plain files, so any set you already own works, and a set's licence stays its own.

## Current limitations

- **SFZ coverage.** The SFZ player handles what the sets we recommend use, and much more, but SFZ is a big format. Some advanced opcodes (scripting, some modulation and effect settings) are ignored.
- **Samples load whole.** Big SFZ sets are loaded into memory in full, with no streaming from disk. A huge orchestral library may not fit.
- **No built-in effects from the set.** Reverb and chorus settings in a SoundFont are not applied; OxiGuitar's own effect plugins shape the sound instead.
- **16 channels.** A song with many guitar tracks can run out of channels to give each string its own; strings then share a channel, and a bend on one moves the others on it.
- **No live MIDI.** It plays scores. It doesn't take a MIDI keyboard or guitar as input, and doesn't send MIDI to external synths.

## Left out on purpose

- **Downloading or bundling sound sets.** OxiGuitar never connects to the internet, and every set has its own author and licence. The sound set installer is a separate, optional package that asks first.
- **A synthesiser of our own for SoundFonts.** rustysynth does it well; we use it as published and fix nothing in a fork.
- **sfizz's code.** It is a C++ library; using it would mean bundling native code on every system. We follow its behaviour and write our own Rust.

## Where you can help

- **SFZ opcodes.** If a set you love sounds wrong, tell us which, or add the missing opcode with a test (`src/sfz/`).
- **Streaming samples from disk**, so large libraries fit.
- **Checking sets.** Try free SFZ instruments and report what doesn't play right.

Every change needs its tests and a row on the [Features](features.md) page; the [Testing](testing.md) page says how to run them.
