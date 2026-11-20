# Architecture and strategy

The plugin is an instrument. OxiGuitar hands it each track's notes, and it renders audio on the audio thread, with no memory allocated and no waiting while it plays.

Two engines do the work. SoundFonts and DLS files go through the `rustysynth` library, used as published. SFZ is our own player: 256 voices, with samples in WAV, FLAC or Ogg held whole in memory, using sfizz's behaviour as the reference and none of its code.

The strategy is to make sound with no setup and then make it better with files you drop in a folder. The plugin never downloads anything. Sets are plain files, so any set you already own works.

Each string of a guitar track gets its own MIDI channel where the 16 channels allow, so a bend moves one string and not the whole chord.
