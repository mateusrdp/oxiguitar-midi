# Credits

Thank you to the people whose work makes OxiGuitar sound.

## Code we use

| Project | Authors | What it does here | Licence |
| --- | --- | --- | --- |
| [OxiGuitar's plugin SDK](https://github.com/mateusrdp/oxiguitar) (`oxiguitar-plugin`, and `oxiguitar-host` and `oxiguitar-plugin-test` for the tests) | Mateus Rocha de Paula and the OxiGuitar contributors | The plugin interface, and OxiGuitar's own host to test against | MIT OR Apache-2.0 |
| [rustysynth](https://github.com/sinshu/rustysynth) | Nobuaki Tanaka (sinshu), a port of his MeltySynth | Plays SoundFonts and DLS sets | MIT |
| [lewton](https://github.com/RustAudio/lewton) | est31 and contributors | Decodes Ogg Vorbis samples (SoundFont 3 and SFZ) | MIT OR Apache-2.0 |
| [Claxon](https://github.com/ruuda/claxon) | Ruud van Asseldonk | Decodes FLAC samples (SFZ) | Apache-2.0 |
| [serde_json](https://github.com/serde-rs/json) | David Tolnay, Erick Tryzelaar and contributors | Tests only | MIT OR Apache-2.0 |
| [Rust](https://www.rust-lang.org/) and its standard library | The Rust Project | Everything else | MIT OR Apache-2.0 |

The SFZ player is our own code.

## References

| Project | What we took |
| --- | --- |
| [sfizz](https://github.com/sfztools/sfizz) (BSD-2-Clause) | How an SFZ player should behave: the parser's rules, each opcode's default, the curves. Reference only; no code taken. |
| [The SFZ format documentation](https://sfzformat.com/) | What each SFZ setting means |
| The General MIDI standard, from the MIDI Manufacturers Association (now the MIDI Association) | The instrument list and the drum map |

## Tools

| Tool | Use |
| --- | --- |
| [mdBook](https://github.com/rust-lang/mdBook) | Builds this manual |
| [cargo-deny](https://github.com/EmbarkStudios/cargo-deny) | Checks the licences of everything we build with |

## Thanks

To the people who record instruments and give the sound sets away, and to the sfizz team, whose careful work on SFZ we learned from.
