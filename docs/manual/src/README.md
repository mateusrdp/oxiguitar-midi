# MIDI plugin

This is the plugin that makes a score sound. It plays every track through a sound set: a SoundFont (`.sf2`, `.sf3`), a DLS file (`.dls`) or an SFZ instrument (`.sfz`). You pick the set, or it uses the best one it finds on your system.

Its name in OxiGuitar is **MIDI (SoundFont, SFZ)**. It replaces the older SoundFont plugin: settings and data are carried over once, and a leftover `soundfont.oxp` can be deleted.

It plays through whatever audio output plugin you have installed. Without one, nothing reaches your speakers.
