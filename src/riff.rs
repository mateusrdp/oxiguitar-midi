// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Mateus Rocha de Paula and the OxiGuitar contributors

//! Reading RIFF files (SoundFont and DLS are both RIFF): chunks and lists,
//! every length checked, nothing read past the end.

use oxiguitar_plugin::{Error, ErrorKind};

/// A chunk: its id, its data, and where its header starts in the list's
/// body.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Chunk<'a> {
    pub(crate) id: [u8; 4],
    pub(crate) data: &'a [u8],
    pub(crate) at: usize,
}

impl<'a> Chunk<'a> {
    /// The list's type and its chunks, when this is a `LIST` (or the file's
    /// `RIFF`) chunk.
    pub(crate) fn list(&self) -> Option<([u8; 4], &'a [u8])> {
        if (&self.id == b"LIST" || &self.id == b"RIFF") && self.data.len() >= 4 {
            let mut kind = [0; 4];
            kind.copy_from_slice(&self.data[..4]);
            Some((kind, &self.data[4..]))
        } else {
            None
        }
    }
}

/// A little-endian number at `at` in `data`, if it is all there.
pub(crate) fn u16_at(data: &[u8], at: usize) -> Option<u16> {
    Some(u16::from_le_bytes(data.get(at..at + 2)?.try_into().ok()?))
}

/// A little-endian number at `at` in `data`, if it is all there.
pub(crate) fn u32_at(data: &[u8], at: usize) -> Option<u32> {
    Some(u32::from_le_bytes(data.get(at..at + 4)?.try_into().ok()?))
}

/// A little-endian signed number at `at` in `data`, if it is all there.
pub(crate) fn i32_at(data: &[u8], at: usize) -> Option<i32> {
    Some(i32::from_le_bytes(data.get(at..at + 4)?.try_into().ok()?))
}

/// An error naming what is wrong with the file.
pub(crate) fn invalid(what: impl Into<String>) -> Error {
    Error::new(ErrorKind::Invalid, what.into())
}

/// The chunks of a list's body, in order. RIFF pads a chunk of odd length
/// with a byte, but some writers (MuseScore's SF3 files) don't: the pad is
/// taken only when a plausible chunk follows it.
pub(crate) fn chunks(mut body: &[u8]) -> Result<Vec<Chunk<'_>>, Error> {
    let mut out = Vec::new();
    let mut at = 0;
    while body.len() >= 8 {
        let mut id = [0; 4];
        id.copy_from_slice(&body[..4]);
        let size = u32_at(body, 4).unwrap_or(0) as usize;
        let data = body
            .get(8..8 + size)
            .ok_or_else(|| invalid(format!("a chunk ({}) runs past the end", show(&id))))?;
        out.push(Chunk { id, data, at });
        let plausible = |at: usize| {
            body.get(at..at + 8).is_some_and(|h| {
                h[..4].iter().all(|c| c.is_ascii_graphic() || *c == b' ')
                    && u32_at(h, 4).is_some_and(|n| at + 8 + n as usize <= body.len())
            })
        };
        let next = if size % 2 == 1 && !plausible(8 + size + 1) && plausible(8 + size) {
            8 + size
        } else {
            8 + size + size % 2
        };
        body = body.get(next..).unwrap_or(&[]);
        at += next;
    }
    Ok(out)
}

/// The file's form and its chunks: `RIFF` then the form's type.
pub(crate) fn form(file: &[u8]) -> Result<([u8; 4], Vec<Chunk<'_>>), Error> {
    if file.len() < 12 || &file[..4] != b"RIFF" {
        return Err(invalid("it isn't a RIFF file"));
    }
    let size = u32_at(file, 4).unwrap_or(0) as usize;
    // A size past the end is taken as the end: some writers get it wrong.
    let end = (8 + size).min(file.len());
    let mut kind = [0; 4];
    kind.copy_from_slice(&file[8..12]);
    Ok((kind, chunks(&file[12..end])?))
}

/// The first chunk `id` of `chunks`.
pub(crate) fn find<'a>(chunks: &[Chunk<'a>], id: &[u8; 4]) -> Option<&'a [u8]> {
    chunks.iter().find(|c| &c.id == id).map(|c| c.data)
}

/// The body of the first list of type `kind` in `chunks`.
pub(crate) fn find_list<'a>(chunks: &[Chunk<'a>], kind: &[u8; 4]) -> Option<&'a [u8]> {
    chunks
        .iter()
        .filter_map(Chunk::list)
        .find(|(k, _)| k == kind)
        .map(|(_, body)| body)
}

/// A chunk id as text, for messages.
pub(crate) fn show(id: &[u8; 4]) -> String {
    id.iter()
        .map(|&c| if c.is_ascii_graphic() { c as char } else { '?' })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn chunk(id: &[u8; 4], data: &[u8]) -> Vec<u8> {
        let mut out = id.to_vec();
        out.extend_from_slice(&(data.len() as u32).to_le_bytes());
        out.extend_from_slice(data);
        out
    }

    fn riff(kind: &[u8; 4], body: &[u8]) -> Vec<u8> {
        let mut inner = kind.to_vec();
        inner.extend_from_slice(body);
        chunk(b"RIFF", &inner)
    }

    /// Chunks in order, an odd one's pad byte skipped; lists found by type.
    #[test]
    fn a_file_s_chunks_and_lists_are_read_in_order() {
        let mut body = chunk(b"abc ", b"odd");
        body.push(0);
        let mut list = b"INFO".to_vec();
        list.extend(chunk(b"INAM", b"name"));
        body.extend(chunk(b"LIST", &list));
        let file = riff(b"sfbk", &body);
        let (kind, top) = form(&file).unwrap();
        assert_eq!(&kind, b"sfbk");
        assert_eq!(top.len(), 2);
        assert_eq!(find(&top, b"abc "), Some(&b"odd"[..]));
        let info = find_list(&top, b"INFO").unwrap();
        assert_eq!(find(&chunks(info).unwrap(), b"INAM"), Some(&b"name"[..]));
        assert_eq!(find_list(&top, b"pdta"), None);
    }

    /// Some writers (MuseScore's SF3 files) leave out an odd chunk's pad:
    /// the next chunk is still found.
    #[test]
    fn a_missing_pad_byte_is_forgiven() {
        let mut body = chunk(b"abc ", b"odd");
        body.extend(chunk(b"next", b"ok"));
        let read = chunks(&body).unwrap();
        assert_eq!(read.len(), 2);
        assert_eq!(&read[1].id, b"next");
        assert_eq!(read[1].data, b"ok");
    }

    /// Not RIFF, too short, or a chunk running past the end: errors, never
    /// a panic. A RIFF size past the end is taken as the end.
    #[test]
    fn damaged_files_are_errors() {
        assert!(form(b"RIFX\0\0\0\0sfbk").is_err());
        assert!(form(b"RIFF").is_err());
        let mut cut = chunk(b"data", b"0123456789");
        cut.truncate(12);
        assert!(chunks(&cut).is_err());
        let mut long = riff(b"DLS ", &chunk(b"vers", b"12345678"));
        long[4..8].copy_from_slice(&u32::MAX.to_le_bytes());
        let (_, read) = form(&long).unwrap();
        assert_eq!(find(&read, b"vers"), Some(&b"12345678"[..]));
        assert_eq!(show(b"a\0b "), "a?b?");
    }

    /// Numbers read only when all their bytes are there.
    #[test]
    fn numbers_are_read_little_endian_and_only_when_whole() {
        let data = [0x34, 0x12, 0xff, 0xff, 0xff, 0x7f];
        assert_eq!(u16_at(&data, 0), Some(0x1234));
        assert_eq!(u32_at(&data, 2), Some(0x7fff_ffff));
        assert_eq!(
            i32_at(&data, 0),
            Some(i32::from_le_bytes([0x34, 0x12, 0xff, 0xff]))
        );
        assert_eq!(u16_at(&data, 5), None);
        assert_eq!(u32_at(&data, 3), None);
    }
}
