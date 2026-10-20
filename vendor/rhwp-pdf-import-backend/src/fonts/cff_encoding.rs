//! CFF (Compact Font Format) encoding parser.
//!
//! Parses the built-in encoding table from CFF font programs and resolves
//! PDF content-stream bytes to glyph IDs.
//!
//! # Two byte → GID resolution paths
//!
//! Simple CFF fonts have two potential sources of byte → glyph mapping
//! information, and PDFs decide between them according to ISO 32000-1
//! §9.6.6:
//!
//! 1. **The PDF font dictionary's `/Encoding`** is authoritative for
//!    simple fonts (Type 1 / TrueType / CFF). It supplies byte → glyph
//!    *name*; the CFF Charset then resolves the name → SID → GID.
//!    [`parse_cff_gid_mapping_with_pdf_encoding`] implements this path.
//!
//! 2. **The CFF font program's own Encoding table** (CFF Tech Note #5176
//!    §12) supplies byte → GID directly. This is the *fallback* when no
//!    PDF-level encoding is supplied (e.g. an `Encoding::Identity` caller).
//!    [`parse_cff_gid_mapping`] implements this path.
//!
//! Subsetter-emitted CFF Encoding tables are frequently sparse —
//! some prepress subsetters commonly emit only `0x20 → space` and
//! `0x41 → A` while the Charset enumerates the full subset — so callers
//! that have the PDF `/Encoding` in hand should always go through the
//! `_with_pdf_encoding` entrypoint. The CFF Encoding path is preserved
//! for backwards compatibility and as a final fallback when the PDF side
//! cannot supply a byte → name resolver.
//!
//! Per PDF spec §9.6.6.2, when no `/BaseEncoding` is specified in an
//! encoding dictionary, the implicit base encoding is the font program's
//! built-in encoding — this module also provides [`parse_cff_encoding`]
//! for that legacy lookup.

use std::collections::HashMap;

/// Standard CFF string IDs (SIDs) 0-390 per CFF specification Table A-2.
/// Only the subset needed for encoding (common glyph names).
fn sid_to_name(sid: u16) -> Option<&'static str> {
    // CFF predefined strings (SID 0-390)
    // Full table from Adobe CFF specification, Appendix A
    static SID_NAMES: &[&str] = &[
        ".notdef",             // 0
        "space",               // 1
        "exclam",              // 2
        "quotedbl",            // 3
        "numbersign",          // 4
        "dollar",              // 5
        "percent",             // 6
        "ampersand",           // 7
        "quoteright",          // 8
        "parenleft",           // 9
        "parenright",          // 10
        "asterisk",            // 11
        "plus",                // 12
        "comma",               // 13
        "hyphen",              // 14
        "period",              // 15
        "slash",               // 16
        "zero",                // 17
        "one",                 // 18
        "two",                 // 19
        "three",               // 20
        "four",                // 21
        "five",                // 22
        "six",                 // 23
        "seven",               // 24
        "eight",               // 25
        "nine",                // 26
        "colon",               // 27
        "semicolon",           // 28
        "less",                // 29
        "equal",               // 30
        "greater",             // 31
        "question",            // 32
        "at",                  // 33
        "A",                   // 34
        "B",                   // 35
        "C",                   // 36
        "D",                   // 37
        "E",                   // 38
        "F",                   // 39
        "G",                   // 40
        "H",                   // 41
        "I",                   // 42
        "J",                   // 43
        "K",                   // 44
        "L",                   // 45
        "M",                   // 46
        "N",                   // 47
        "O",                   // 48
        "P",                   // 49
        "Q",                   // 50
        "R",                   // 51
        "S",                   // 52
        "T",                   // 53
        "U",                   // 54
        "V",                   // 55
        "W",                   // 56
        "X",                   // 57
        "Y",                   // 58
        "Z",                   // 59
        "bracketleft",         // 60
        "backslash",           // 61
        "bracketright",        // 62
        "asciicircum",         // 63
        "underscore",          // 64
        "quoteleft",           // 65
        "a",                   // 66
        "b",                   // 67
        "c",                   // 68
        "d",                   // 69
        "e",                   // 70
        "f",                   // 71
        "g",                   // 72
        "h",                   // 73
        "i",                   // 74
        "j",                   // 75
        "k",                   // 76
        "l",                   // 77
        "m",                   // 78
        "n",                   // 79
        "o",                   // 80
        "p",                   // 81
        "q",                   // 82
        "r",                   // 83
        "s",                   // 84
        "t",                   // 85
        "u",                   // 86
        "v",                   // 87
        "w",                   // 88
        "x",                   // 89
        "y",                   // 90
        "z",                   // 91
        "braceleft",           // 92
        "bar",                 // 93
        "braceright",          // 94
        "asciitilde",          // 95
        "exclamdown",          // 96
        "cent",                // 97
        "sterling",            // 98
        "fraction",            // 99
        "yen",                 // 100
        "florin",              // 101
        "section",             // 102
        "currency",            // 103
        "quotesingle",         // 104
        "quotedblleft",        // 105
        "guillemotleft",       // 106
        "guilsinglleft",       // 107
        "guilsinglright",      // 108
        "fi",                  // 109
        "fl",                  // 110
        "endash",              // 111
        "dagger",              // 112
        "daggerdbl",           // 113
        "periodcentered",      // 114
        "paragraph",           // 115
        "bullet",              // 116
        "quotesinglbase",      // 117
        "quotedblbase",        // 118
        "quotedblright",       // 119
        "guillemotright",      // 120
        "ellipsis",            // 121
        "perthousand",         // 122
        "questiondown",        // 123
        "grave",               // 124
        "acute",               // 125
        "circumflex",          // 126
        "tilde",               // 127
        "macron",              // 128
        "breve",               // 129
        "dotaccent",           // 130
        "dieresis",            // 131
        "ring",                // 132
        "cedilla",             // 133
        "hungarumlaut",        // 134
        "ogonek",              // 135
        "caron",               // 136
        "emdash",              // 137
        "AE",                  // 138
        "ordfeminine",         // 139
        "Lslash",              // 140
        "Oslash",              // 141
        "OE",                  // 142
        "ordmasculine",        // 143
        "ae",                  // 144
        "dotlessi",            // 145
        "lslash",              // 146
        "oslash",              // 147
        "oe",                  // 148
        "germandbls",          // 149
        "onesuperior",         // 150
        "logicalnot",          // 151
        "mu",                  // 152
        "trademark",           // 153
        "Eth",                 // 154
        "onehalf",             // 155
        "plusminus",           // 156
        "Thorn",               // 157
        "onequarter",          // 158
        "divide",              // 159
        "brokenbar",           // 160
        "degree",              // 161
        "thorn",               // 162
        "threequarters",       // 163
        "twosuperior",         // 164
        "registered",          // 165
        "minus",               // 166
        "eth",                 // 167
        "multiply",            // 168
        "threesuperior",       // 169
        "copyright",           // 170
        "Aacute",              // 171
        "Acircumflex",         // 172
        "Adieresis",           // 173
        "Agrave",              // 174
        "Aring",               // 175
        "Atilde",              // 176
        "Ccedilla",            // 177
        "Eacute",              // 178
        "Ecircumflex",         // 179
        "Edieresis",           // 180
        "Egrave",              // 181
        "Iacute",              // 182
        "Icircumflex",         // 183
        "Idieresis",           // 184
        "Igrave",              // 185
        "Ntilde",              // 186
        "Oacute",              // 187
        "Ocircumflex",         // 188
        "Odieresis",           // 189
        "Ograve",              // 190
        "Otilde",              // 191
        "Scaron",              // 192
        "Uacute",              // 193
        "Ucircumflex",         // 194
        "Udieresis",           // 195
        "Ugrave",              // 196
        "Yacute",              // 197
        "Ydieresis",           // 198
        "Zcaron",              // 199
        "aacute",              // 200
        "acircumflex",         // 201
        "adieresis",           // 202
        "agrave",              // 203
        "aring",               // 204
        "atilde",              // 205
        "ccedilla",            // 206
        "eacute",              // 207
        "ecircumflex",         // 208
        "edieresis",           // 209
        "egrave",              // 210
        "iacute",              // 211
        "icircumflex",         // 212
        "idieresis",           // 213
        "igrave",              // 214
        "ntilde",              // 215
        "oacute",              // 216
        "ocircumflex",         // 217
        "odieresis",           // 218
        "ograve",              // 219
        "otilde",              // 220
        "scaron",              // 221
        "uacute",              // 222
        "ucircumflex",         // 223
        "udieresis",           // 224
        "ugrave",              // 225
        "yacute",              // 226
        "ydieresis",           // 227
        "zcaron",              // 228
        "exclamsmall",         // 229
        "Hungarumlautsmall",   // 230
        "dollaroldstyle",      // 231
        "dollarsuperior",      // 232
        "ampersandsmall",      // 233
        "Acutesmall",          // 234
        "parenleftsuperior",   // 235
        "parenrightsuperior",  // 236
        "twodotenleader",      // 237
        "onedotenleader",      // 238
        "zerooldstyle",        // 239
        "oneoldstyle",         // 240
        "twooldstyle",         // 241
        "threeoldstyle",       // 242
        "fouroldstyle",        // 243
        "fiveoldstyle",        // 244
        "sixoldstyle",         // 245
        "sevenoldstyle",       // 246
        "eightoldstyle",       // 247
        "nineoldstyle",        // 248
        "commasuperior",       // 249
        "threequartersemdash", // 250
        "periodsuperior",      // 251
        "questionsmall",       // 252
        "asuperior",           // 253
        "bsuperior",           // 254
        "centsuperior",        // 255
        "dsuperior",           // 256
        "esuperior",           // 257
        "isuperior",           // 258
        "lsuperior",           // 259
        "msuperior",           // 260
        "nsuperior",           // 261
        "osuperior",           // 262
        "rsuperior",           // 263
        "ssuperior",           // 264
        "tsuperior",           // 265
        "ff",                  // 266
        "ffi",                 // 267
        "ffl",                 // 268
        "parenleftinferior",   // 269
        "parenrightinferior",  // 270
        "Circumflexsmall",     // 271
        "hyphensuperior",      // 272
        "Gravesmall",          // 273
        "Asmall",              // 274
        "Bsmall",              // 275
        "Csmall",              // 276
        "Dsmall",              // 277
        "Esmall",              // 278
        "Fsmall",              // 279
        "Gsmall",              // 280
        "Hsmall",              // 281
        "Ismall",              // 282
        "Jsmall",              // 283
        "Ksmall",              // 284
        "Lsmall",              // 285
        "Msmall",              // 286
        "Nsmall",              // 287
        "Osmall",              // 288
        "Psmall",              // 289
        "Qsmall",              // 290
        "Rsmall",              // 291
        "Ssmall",              // 292
        "Tsmall",              // 293
        "Usmall",              // 294
        "Vsmall",              // 295
        "Wsmall",              // 296
        "Xsmall",              // 297
        "Ysmall",              // 298
        "Zsmall",              // 299
        "colonmonetary",       // 300
        "onefitted",           // 301
        "rupiah",              // 302
        "Tildesmall",          // 303
        "exclamdownsmall",     // 304
        "centoldstyle",        // 305
        "Lslashsmall",         // 306
        "Scaronsmall",         // 307
        "Zcaronsmall",         // 308
        "Dieresissmall",       // 309
        "Brevesmall",          // 310
        "Caronsmall",          // 311
        "Dotaccentsmall",      // 312
        "Macronsmall",         // 313
        "figuredash",          // 314
        "hypheninferior",      // 315
        "Ogoneksmall",         // 316
        "Ringsmall",           // 317
        "Cedillasmall",        // 318
        "questiondownsmall",   // 319
        "oneeighth",           // 320
        "threeeighths",        // 321
        "fiveeighths",         // 322
        "seveneighths",        // 323
        "onethird",            // 324
        "twothirds",           // 325
        "zerosuperior",        // 326
        "foursuperior",        // 327
        "fivesuperior",        // 328
        "sixsuperior",         // 329
        "sevensuperior",       // 330
        "eightsuperior",       // 331
        "ninesuperior",        // 332
        "zeroinferior",        // 333
        "oneinferior",         // 334
        "twoinferior",         // 335
        "threeinferior",       // 336
        "fourinferior",        // 337
        "fiveinferior",        // 338
        "sixinferior",         // 339
        "seveninferior",       // 340
        "eightinferior",       // 341
        "nineinferior",        // 342
        "centinferior",        // 343
        "dollarinferior",      // 344
        "periodinferior",      // 345
        "commainferior",       // 346
        "Agravesmall",         // 347
        "Aacutesmall",         // 348
        "Acircumflexsmall",    // 349
        "Atildesmall",         // 350
        "Adieresissmall",      // 351
        "Aringsmall",          // 352
        "AEsmall",             // 353
        "Ccedillasmall",       // 354
        "Egravesmall",         // 355
        "Eacutesmall",         // 356
        "Ecircumflexsmall",    // 357
        "Edieresissmall",      // 358
        "Igravesmall",         // 359
        "Iacutesmall",         // 360
        "Icircumflexsmall",    // 361
        "Idieresissmall",      // 362
        "Ethsmall",            // 363
        "Ntildesmall",         // 364
        "Ogravesmall",         // 365
        "Oacutesmall",         // 366
        "Ocircumflexsmall",    // 367
        "Otildesmall",         // 368
        "Odieresissmall",      // 369
        "OEsmall",             // 370
        "Oslashsmall",         // 371
        "Ugravesmall",         // 372
        "Uacutesmall",         // 373
        "Ucircumflexsmall",    // 374
        "Udieresissmall",      // 375
        "Yacutesmall",         // 376
        "Thornsmall",          // 377
        "Ydieresissmall",      // 378
        "001.000",             // 379
        "001.001",             // 380
        "001.002",             // 381
        "001.003",             // 382
        "Black",               // 383
        "Bold",                // 384
        "Book",                // 385
        "Light",               // 386
        "Medium",              // 387
        "Regular",             // 388
        "Roman",               // 389
        "Semibold",            // 390
    ];

    if (sid as usize) < SID_NAMES.len() {
        Some(SID_NAMES[sid as usize])
    } else {
        None
    }
}

/// Look up the SID for a standard glyph name by searching the predefined SID table.
fn glyph_name_to_sid(name: &str) -> Option<u16> {
    for sid in 0u16..391 {
        if sid_to_name(sid) == Some(name) {
            return Some(sid);
        }
    }
    None
}

/// Parse a CFF INDEX structure, returning byte slices for each entry.
fn parse_index(data: &[u8], offset: usize) -> Option<(Vec<&[u8]>, usize)> {
    if offset + 2 > data.len() {
        return None;
    }
    let count = u16::from_be_bytes([data[offset], data[offset + 1]]) as usize;
    if count == 0 {
        return Some((Vec::new(), offset + 2));
    }

    if offset + 3 > data.len() {
        return None;
    }
    let off_size = data[offset + 2] as usize;
    if off_size == 0 || off_size > 4 {
        return None;
    }

    let offset_array_start = offset + 3;
    let offset_array_len = (count + 1) * off_size;
    if offset_array_start + offset_array_len > data.len() {
        return None;
    }

    // Read offsets
    let mut offsets = Vec::with_capacity(count + 1);
    for i in 0..=count {
        let pos = offset_array_start + i * off_size;
        let mut val: u32 = 0;
        for j in 0..off_size {
            val = (val << 8) | data[pos + j] as u32;
        }
        offsets.push(val as usize);
    }

    let data_start = offset_array_start + offset_array_len;
    let mut entries = Vec::with_capacity(count);
    for i in 0..count {
        let start = data_start + offsets[i] - 1; // CFF offsets are 1-based
        let end = data_start + offsets[i + 1] - 1;
        if start > data.len() || end > data.len() || start > end {
            return None;
        }
        entries.push(&data[start..end]);
    }

    let next_offset = data_start + offsets[count] - 1;
    Some((entries, next_offset))
}

/// Parse a CFF DICT operand (integer or real).
/// Returns (value, bytes consumed).
fn parse_dict_operand(data: &[u8], pos: usize) -> Option<(i32, usize)> {
    if pos >= data.len() {
        return None;
    }
    let b0 = data[pos] as i32;
    match b0 {
        // Integer: 1 byte
        32..=246 => Some((b0 - 139, 1)),
        // Integer: 2 bytes
        247..=250 => {
            if pos + 1 >= data.len() {
                return None;
            }
            let b1 = data[pos + 1] as i32;
            Some(((b0 - 247) * 256 + b1 + 108, 2))
        },
        251..=254 => {
            if pos + 1 >= data.len() {
                return None;
            }
            let b1 = data[pos + 1] as i32;
            Some((-(b0 - 251) * 256 - b1 - 108, 2))
        },
        // Integer: 3 bytes (16-bit)
        28 => {
            if pos + 2 >= data.len() {
                return None;
            }
            let val = i16::from_be_bytes([data[pos + 1], data[pos + 2]]) as i32;
            Some((val, 3))
        },
        // Integer: 5 bytes (32-bit)
        29 => {
            if pos + 4 >= data.len() {
                return None;
            }
            let val =
                i32::from_be_bytes([data[pos + 1], data[pos + 2], data[pos + 3], data[pos + 4]]);
            Some((val, 5))
        },
        // Real number (skip, we only need integers for encoding/charset offsets)
        30 => {
            let mut i = pos + 1;
            while i < data.len() {
                let nibble1 = (data[i] >> 4) & 0x0F;
                let nibble2 = data[i] & 0x0F;
                if nibble1 == 0x0F || nibble2 == 0x0F {
                    return Some((0, i - pos + 1));
                }
                i += 1;
            }
            None
        },
        _ => None,
    }
}

/// Decode a CFF DICT real number (`b0 == 30`) into an `f64`.
///
/// Adobe Technical Note #5176 §4 encodes a real as packed nibbles: `0`-`9` are
/// digits, `0xa` a decimal point, `0xb` `E`, `0xc` `E-`, `0xe` a minus sign and
/// `0xf` terminates. `parse_dict_operand` deliberately discards these — it only
/// needs the integer offsets — so this reads them properly where the value
/// matters.
///
/// Returns `(value, bytes consumed including the 30 marker)`.
fn parse_dict_real(data: &[u8], pos: usize) -> Option<(f64, usize)> {
    if pos >= data.len() || data[pos] != 30 {
        return None;
    }
    let mut text = String::new();
    let mut i = pos + 1;
    while i < data.len() {
        for nib in [(data[i] >> 4) & 0x0F, data[i] & 0x0F] {
            match nib {
                0..=9 => text.push((b'0' + nib) as char),
                0xa => text.push('.'),
                0xb => text.push('E'),
                0xc => text.push_str("E-"),
                0xe => text.push('-'),
                0xf => return text.parse::<f64>().ok().map(|v| (v, i - pos + 1)),
                _ => {}, // 0xd is reserved
            }
        }
        i += 1;
    }
    None
}

/// The uniform glyph-space scale declared by a bare CFF's Top DICT
/// `/FontMatrix` (Adobe TN #5176, operator `12 7`), if it has one and it is a
/// plain uniform scale.
///
/// ISO 32000-1:2008 §9.2.2 (`docs/spec/pdf.md`:8459-8461) fixes glyph space:
/// "The transformation from glyph space to text space shall be defined by the
/// font matrix. For most types of fonts, this matrix shall be predefined to map
/// 1000 units of glyph space to 1 unit of text space." A CFF is free to declare
/// otherwise, and subsetters targeting a 2048-unit em routinely do. The
/// charstring coordinates are then in those units, so assuming 1000 paints
/// every outline at `2048/1000` of its intended size.
///
/// Returns `None` for a skewed, rotated, non-uniform or absent matrix — the
/// caller keeps its 1000-unit default, which is also the CFF default.
pub(crate) fn parse_cff_font_matrix_scale(font_data: &[u8]) -> Option<f64> {
    let cff_data = extract_cff_from_opentype(font_data).unwrap_or(font_data);
    if cff_data.len() < 4 || cff_data[0] != 1 {
        return None;
    }
    let hdr_size = cff_data[2] as usize;
    let (_, after_name) = parse_index(cff_data, hdr_size)?;
    let (top_dicts, _) = parse_index(cff_data, after_name)?;
    let dict = top_dicts.first()?;

    let mut operands: Vec<f64> = Vec::new();
    let mut pos = 0usize;
    while pos < dict.len() {
        let b0 = dict[pos];
        if b0 <= 21 {
            let op = if b0 == 12 {
                pos += 1;
                if pos >= dict.len() {
                    break;
                }
                (12u16 << 8) | dict[pos] as u16
            } else {
                b0 as u16
            };
            // 12 7 = FontMatrix, six operands [a b c d e f].
            if op == 0x0C07 && operands.len() >= 6 {
                let m = &operands[operands.len() - 6..];
                let (a, b, c, d) = (m[0], m[1], m[2], m[3]);
                // Only a plain uniform scale is safe to fold into unitsPerEm.
                if b.abs() < 1e-12 && c.abs() < 1e-12 && a > 0.0 && (a - d).abs() < 1e-12 {
                    return Some(a);
                }
                return None;
            }
            operands.clear();
            pos += 1;
        } else if b0 == 30 {
            let (v, used) = parse_dict_real(dict, pos)?;
            operands.push(v);
            pos += used;
        } else {
            let (v, used) = parse_dict_operand(dict, pos)?;
            operands.push(v as f64);
            pos += used;
        }
    }
    None
}

/// Map CIDs to glyph indices through a CID-keyed CFF's charset.
///
/// ISO 32000-1:2008 §9.7.4.2 (`docs/spec/pdf.md`:18641-18644) splits on whether
/// the embedded program is CID-keyed:
///
/// > The "CFF" font program has a Top DICT that uses CIDFont operators: The
/// > CIDs shall be used to determine the GID value for the glyph procedure
/// > using the charset table in the CFF program.
///
/// and the NOTE at :18646 is explicit that "Although in many fonts the CID
/// value and GID value are the same, the CID and GID values may differ."
/// Only when the Top DICT does **not** use CIDFont operators (:18649-18650)
/// may "the CIDs … be used directly as GID values".
///
/// In a CID-keyed CFF the charset stores a CID per glyph rather than a SID, so
/// inverting it gives the mapping the clause asks for.
///
/// Returns `None` when the font is not CID-keyed — the caller then uses the CID
/// as the GID, which is what the second branch of the clause requires.
pub(crate) fn parse_cff_cid_to_gid(font_data: &[u8]) -> Option<HashMap<u16, u16>> {
    let cff_data = extract_cff_from_opentype(font_data).unwrap_or(font_data);
    if cff_data.len() < 4 || cff_data[0] != 1 {
        return None;
    }
    let hdr_size = cff_data[2] as usize;
    let (_, after_name) = parse_index(cff_data, hdr_size)?;
    let (top_dicts, _) = parse_index(cff_data, after_name)?;
    let dict = top_dicts.first()?;

    // ROS (operator 12 30) is what makes a CFF CID-keyed.
    if !top_dict_has_ros(dict) {
        return None;
    }

    // Returns (charstrings, encoding, charset) — in that order.
    let (charstrings_offset, _encoding_offset, charset_offset) =
        parse_top_dict_with_charstrings(dict);
    if charstrings_offset <= 0 {
        return None;
    }
    let num_glyphs = read_index_count(cff_data, charstrings_offset as usize)? as usize;

    // Charset offsets 0/1/2 name the predefined charsets, none of which is
    // CID-keyed, so a CID-keyed font always carries its own table.
    if charset_offset <= 2 {
        return None;
    }
    let cids = parse_charset(cff_data, charset_offset as usize, num_glyphs)?;

    let mut map = HashMap::with_capacity(cids.len());
    for (gid, &cid) in cids.iter().enumerate() {
        // First writer wins: a well-formed charset lists each CID once, and
        // keeping the lowest GID on a malformed one is stable.
        map.entry(cid).or_insert(gid as u16);
    }
    Some(map)
}

/// Whether a Top DICT carries the ROS operator (12 30), which marks the font
/// CID-keyed per Adobe Technical Note #5176.
fn top_dict_has_ros(dict: &[u8]) -> bool {
    let mut pos = 0usize;
    while pos < dict.len() {
        let b0 = dict[pos];
        if b0 <= 21 {
            if b0 == 12 {
                pos += 1;
                if pos >= dict.len() {
                    return false;
                }
                if dict[pos] == 30 {
                    return true;
                }
            }
            pos += 1;
        } else if b0 == 30 {
            match parse_dict_real(dict, pos) {
                Some((_, used)) => pos += used,
                None => return false,
            }
        } else {
            match parse_dict_operand(dict, pos) {
                Some((_, used)) => pos += used,
                None => return false,
            }
        }
    }
    false
}

/// Parse a CFF Top DICT to extract encoding and charset offsets.
fn parse_top_dict(dict_data: &[u8]) -> (i32, i32) {
    let mut encoding_offset: i32 = 0; // Default: StandardEncoding
    let mut charset_offset: i32 = 0; // Default: ISOAdobe charset

    let mut pos = 0;
    let mut operand_stack: Vec<i32> = Vec::new();

    while pos < dict_data.len() {
        let b0 = dict_data[pos];
        if b0 <= 21 {
            // Operator
            let op = if b0 == 12 {
                pos += 1;
                if pos >= dict_data.len() {
                    break;
                }
                (12u16 << 8) | dict_data[pos] as u16
            } else {
                b0 as u16
            };

            match op {
                16 => {
                    // Encoding (operator 16)
                    if let Some(&val) = operand_stack.last() {
                        encoding_offset = val;
                    }
                },
                15 => {
                    // charset (operator 15)
                    if let Some(&val) = operand_stack.last() {
                        charset_offset = val;
                    }
                },
                _ => {},
            }

            operand_stack.clear();
            pos += 1;
        } else if let Some((val, consumed)) = parse_dict_operand(dict_data, pos) {
            operand_stack.push(val);
            pos += consumed;
        } else {
            pos += 1;
        }
    }

    (encoding_offset, charset_offset)
}

/// Parse the CFF Top DICT and surface (CharStrings, Encoding, charset)
/// offsets. CharStrings (op 17) points at the CharStrings INDEX whose count
/// field is the real `nGlyphs` for the font — needed to parse the full
/// Charset for subsets enumerating >256 glyphs.
///
/// Returns `(charstrings_offset, encoding_offset, charset_offset)`.
fn parse_top_dict_with_charstrings(dict_data: &[u8]) -> (i32, i32, i32) {
    let mut charstrings_offset: i32 = 0;
    let mut encoding_offset: i32 = 0;
    let mut charset_offset: i32 = 0;

    let mut pos = 0;
    let mut operand_stack: Vec<i32> = Vec::new();

    while pos < dict_data.len() {
        let b0 = dict_data[pos];
        if b0 <= 21 {
            let op = if b0 == 12 {
                pos += 1;
                if pos >= dict_data.len() {
                    break;
                }
                (12u16 << 8) | dict_data[pos] as u16
            } else {
                b0 as u16
            };

            match op {
                15 => {
                    if let Some(&val) = operand_stack.last() {
                        charset_offset = val;
                    }
                },
                16 => {
                    if let Some(&val) = operand_stack.last() {
                        encoding_offset = val;
                    }
                },
                17 => {
                    if let Some(&val) = operand_stack.last() {
                        charstrings_offset = val;
                    }
                },
                _ => {},
            }

            operand_stack.clear();
            pos += 1;
        } else if let Some((val, consumed)) = parse_dict_operand(dict_data, pos) {
            operand_stack.push(val);
            pos += consumed;
        } else {
            pos += 1;
        }
    }

    (charstrings_offset, encoding_offset, charset_offset)
}

/// Read the 2-byte big-endian `count` field at the start of a CFF INDEX
/// header (CFF spec §5). For the CharStrings INDEX, this count is `nGlyphs`.
///
/// Returns `None` if the header is truncated.
fn read_index_count(data: &[u8], offset: usize) -> Option<u32> {
    if offset + 2 > data.len() {
        return None;
    }
    Some(u16::from_be_bytes([data[offset], data[offset + 1]]) as u32)
}

/// Parse the CFF charset table.
/// Returns GID → SID mapping (GID 0 is always .notdef).
fn parse_charset(data: &[u8], offset: usize, num_glyphs: usize) -> Option<Vec<u16>> {
    if offset >= data.len() {
        return None;
    }

    let mut sids = Vec::with_capacity(num_glyphs);
    sids.push(0); // GID 0 = .notdef (SID 0)

    let format = data[offset];
    let mut pos = offset + 1;

    match format {
        0 => {
            // Format 0: array of SIDs
            for _ in 1..num_glyphs {
                if pos + 1 >= data.len() {
                    break;
                }
                let sid = u16::from_be_bytes([data[pos], data[pos + 1]]);
                sids.push(sid);
                pos += 2;
            }
        },
        1 => {
            // Format 1: ranges with 1-byte count
            while sids.len() < num_glyphs && pos + 2 < data.len() {
                let first_sid = u16::from_be_bytes([data[pos], data[pos + 1]]);
                let n_left = data[pos + 2] as u16;
                pos += 3;
                for i in 0..=n_left {
                    if sids.len() >= num_glyphs {
                        break;
                    }
                    sids.push(first_sid + i);
                }
            }
        },
        2 => {
            // Format 2: ranges with 2-byte count
            while sids.len() < num_glyphs && pos + 3 < data.len() {
                let first_sid = u16::from_be_bytes([data[pos], data[pos + 1]]);
                let n_left = u16::from_be_bytes([data[pos + 2], data[pos + 3]]);
                pos += 4;
                for i in 0..=n_left {
                    if sids.len() >= num_glyphs {
                        break;
                    }
                    sids.push(first_sid + i);
                }
            }
        },
        _ => return None,
    }

    Some(sids)
}

/// Parse the CFF encoding table.
/// Returns character code → GID mapping.
fn parse_encoding_table(data: &[u8], offset: usize) -> Option<HashMap<u8, u16>> {
    if offset >= data.len() {
        return None;
    }

    let mut code_to_gid = HashMap::new();
    let format = data[offset] & 0x7F; // Bit 7 is supplement flag
    let has_supplement = (data[offset] & 0x80) != 0;
    let mut pos = offset + 1;

    match format {
        0 => {
            // Format 0: array of codes
            if pos >= data.len() {
                return None;
            }
            let n_codes = data[pos] as usize;
            pos += 1;
            for gid in 1..=n_codes {
                if pos >= data.len() {
                    break;
                }
                let code = data[pos];
                code_to_gid.insert(code, gid as u16);
                pos += 1;
            }
        },
        1 => {
            // Format 1: ranges
            if pos >= data.len() {
                return None;
            }
            let n_ranges = data[pos] as usize;
            pos += 1;
            let mut gid: u16 = 1;
            for _ in 0..n_ranges {
                if pos + 1 >= data.len() {
                    break;
                }
                let first = data[pos];
                let n_left = data[pos + 1] as u16;
                pos += 2;
                for i in 0..=n_left {
                    let code = first.wrapping_add(i as u8);
                    code_to_gid.insert(code, gid);
                    gid += 1;
                }
            }
        },
        _ => return None,
    }

    // Handle supplement (additional code → SID mappings)
    if has_supplement && pos < data.len() {
        let n_sups = data[pos] as usize;
        pos += 1;
        for _ in 0..n_sups {
            if pos + 2 >= data.len() {
                break;
            }
            let code = data[pos];
            let sid = u16::from_be_bytes([data[pos + 1], data[pos + 2]]);
            pos += 3;
            // For supplements, we use SID directly as a pseudo-GID
            // The caller will need to handle this via the charset
            code_to_gid.insert(code, sid);
        }
    }

    Some(code_to_gid)
}

/// Resolve a glyph name from a SID, using predefined strings and the
/// String INDEX from the CFF font.
fn resolve_glyph_name<'a>(sid: u16, string_index: &'a [&'a [u8]]) -> Option<String> {
    if sid <= 390 {
        sid_to_name(sid).map(|s| s.to_string())
    } else {
        // Custom string from String INDEX
        let idx = (sid - 391) as usize;
        if idx < string_index.len() {
            std::str::from_utf8(string_index[idx])
                .ok()
                .map(|s| s.to_string())
        } else {
            None
        }
    }
}

/// Extract the CFF table from an OpenType (sfnt) wrapper.
/// Returns the CFF data slice if found, or None if the data isn't an sfnt container.
fn extract_cff_from_opentype(data: &[u8]) -> Option<&[u8]> {
    if data.len() < 12 {
        return None;
    }
    let magic = u32::from_be_bytes([data[0], data[1], data[2], data[3]]);
    // Check for OpenType "OTTO" or TrueType 0x00010000
    if magic != 0x4F54544F && magic != 0x00010000 {
        return None;
    }
    let num_tables = u16::from_be_bytes([data[4], data[5]]) as usize;
    // Table directory starts at offset 12
    let mut pos = 12;
    for _ in 0..num_tables {
        if pos + 16 > data.len() {
            return None;
        }
        let tag = u32::from_be_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]]);
        let offset =
            u32::from_be_bytes([data[pos + 8], data[pos + 9], data[pos + 10], data[pos + 11]])
                as usize;
        let length = u32::from_be_bytes([
            data[pos + 12],
            data[pos + 13],
            data[pos + 14],
            data[pos + 15],
        ]) as usize;
        // CFF tag = 0x43464620 ("CFF ")
        if tag == 0x43464620 && offset + length <= data.len() {
            return Some(&data[offset..offset + length]);
        }
        pos += 16;
    }
    None
}

/// Extract the built-in encoding from a CFF font program.
///
/// Returns a HashMap mapping character codes (u8) to Unicode characters.
/// This implements the CFF encoding → charset → glyph name → Unicode pipeline.
/// Also handles OpenType-wrapped CFF data (FontFile3 with sfnt container).
pub fn parse_cff_encoding(font_data: &[u8]) -> Option<HashMap<u8, char>> {
    if font_data.len() < 4 {
        return None;
    }

    // If data is wrapped in an OpenType container, extract the CFF table
    let cff_data = if font_data[0] != 1 {
        if let Some(cff) = extract_cff_from_opentype(font_data) {
            log::debug!(
                "Extracted CFF table ({} bytes) from OpenType wrapper ({} bytes)",
                cff.len(),
                font_data.len()
            );
            cff
        } else {
            // Not CFF version 1 and not an OpenType wrapper
            log::debug!("CFF version {} not supported (expected 1)", font_data[0]);
            return None;
        }
    } else {
        font_data
    };

    if cff_data.len() < 4 || cff_data[0] != 1 {
        return None;
    }
    let hdr_size = cff_data[2] as usize;

    // Parse Name INDEX
    let (_, after_name) = parse_index(cff_data, hdr_size)?;

    // Parse Top DICT INDEX
    let (top_dicts, after_top_dict) = parse_index(cff_data, after_name)?;
    if top_dicts.is_empty() {
        return None;
    }

    // Parse String INDEX
    let (string_index, _after_string) = parse_index(cff_data, after_top_dict)?;

    // Parse Top DICT to get encoding and charset offsets
    let (encoding_offset, charset_offset) = parse_top_dict(top_dicts[0]);

    if encoding_offset == 1 {
        // ExpertEncoding — rarely used for text
        log::debug!("CFF uses ExpertEncoding (predefined)");
        return None;
    }

    if encoding_offset == 0 {
        // StandardEncoding — for fonts with custom charsets, build a GID-based
        // fallback map. This handles subset fonts where character codes equal
        // GIDs rather than standard encoding positions.
        if charset_offset > 2 {
            log::debug!("CFF StandardEncoding + custom charset; building charset-based map");
            let num_glyphs = 256usize;
            let charset_sids = parse_charset(cff_data, charset_offset as usize, num_glyphs)?;

            let mut encoding_map = HashMap::new();
            for (gid, &sid) in charset_sids.iter().enumerate() {
                if gid == 0 || gid > 255 {
                    continue;
                }
                if let Some(glyph_name) = resolve_glyph_name(sid, &string_index) {
                    if let Some(unicode_char) = super::font_dict::glyph_name_to_unicode(&glyph_name)
                    {
                        encoding_map.insert(gid as u8, unicode_char);
                    }
                }
            }
            if !encoding_map.is_empty() {
                log::debug!(
                    "CFF charset-based fallback: {} character mappings",
                    encoding_map.len()
                );
                return Some(encoding_map);
            }
        }
        log::debug!("CFF uses StandardEncoding (predefined)");
        return None;
    }

    // Custom encoding (encoding_offset > 1): parse it
    let code_to_gid = parse_encoding_table(cff_data, encoding_offset as usize)?;

    let max_gid = code_to_gid.values().max().copied().unwrap_or(0) as usize;
    let num_glyphs = max_gid + 10;

    // Parse charset (GID → SID mapping)
    let charset_sids = if charset_offset == 0 {
        (0..num_glyphs as u16).collect()
    } else if charset_offset == 1 || charset_offset == 2 {
        log::debug!("CFF uses predefined charset {}", charset_offset);
        return None;
    } else {
        parse_charset(cff_data, charset_offset as usize, num_glyphs)?
    };

    // Build the final encoding map: code → Unicode
    let mut encoding_map = HashMap::new();

    for (&code, &gid) in &code_to_gid {
        let sid = if (gid as usize) < charset_sids.len() {
            charset_sids[gid as usize]
        } else {
            continue;
        };

        if let Some(glyph_name) = resolve_glyph_name(sid, &string_index) {
            if let Some(unicode_char) = super::font_dict::glyph_name_to_unicode(&glyph_name) {
                encoding_map.insert(code, unicode_char);
            }
        }
    }

    if encoding_map.is_empty() {
        None
    } else {
        log::debug!("CFF built-in encoding parsed: {} character mappings", encoding_map.len());
        Some(encoding_map)
    }
}

/// Parse a CFF font program and return a byte_code → glyph_id mapping.
/// This allows rendering CFF subset fonts by mapping PDF character codes
/// directly to glyph indices without needing a Unicode cmap.
pub fn parse_cff_gid_mapping(font_data: &[u8]) -> Option<HashMap<u8, u16>> {
    if font_data.len() < 4 {
        return None;
    }

    let cff_data = if font_data[0] != 1 {
        extract_cff_from_opentype(font_data)?
    } else {
        font_data
    };

    if cff_data.len() < 4 || cff_data[0] != 1 {
        return None;
    }
    let hdr_size = cff_data[2] as usize;

    let (_, after_name) = parse_index(cff_data, hdr_size)?;
    let (top_dicts, after_top_dict) = parse_index(cff_data, after_name)?;
    if top_dicts.is_empty() {
        return None;
    }

    let (_string_index, _after_string) = parse_index(cff_data, after_top_dict)?;
    let (encoding_offset, charset_offset) = parse_top_dict(top_dicts[0]);

    if encoding_offset == 0 && charset_offset > 2 {
        // StandardEncoding + custom charset:
        // byte_code → SID (via CFF Standard Encoding) → GID (via charset)
        let num_glyphs = 256usize;
        if let Some(charset_sids) = parse_charset(cff_data, charset_offset as usize, num_glyphs) {
            // Build SID → GID reverse map from charset
            let mut sid_to_gid: HashMap<u16, u16> = HashMap::new();
            for (gid, &sid) in charset_sids.iter().enumerate() {
                if gid > 0 {
                    sid_to_gid.entry(sid).or_insert(gid as u16);
                }
            }
            // CFF Standard Encoding: byte_code → SID
            // Map byte codes through standard encoding to SIDs, then to GIDs
            let mut map = HashMap::new();
            for byte_code in 0u16..256 {
                // Get the glyph name for this byte code using standard encoding
                let glyph_name = super::font_dict::FontInfo::gid_to_standard_glyph_name(byte_code);
                if let Some(name) = glyph_name {
                    // Find the SID for this glyph name
                    if let Some(sid) = glyph_name_to_sid(name) {
                        if let Some(&gid) = sid_to_gid.get(&sid) {
                            map.insert(byte_code as u8, gid);
                        }
                    }
                }
            }
            if !map.is_empty() {
                log::debug!("CFF StandardEncoding→charset GID mapping: {} entries", map.len());
                return Some(map);
            }
        }
        return None;
    }

    if encoding_offset <= 1 {
        return None;
    }

    // Custom encoding: parse byte_code → GID mapping directly
    parse_encoding_table(cff_data, encoding_offset as usize)
}

/// Build the byte → GID map for a simple CFF font using the PDF font
/// dictionary's `/Encoding` as the byte → glyph-name source and the CFF
/// Charset as the glyph-name → GID resolver, per ISO 32000-1 §9.6.6.
///
/// This is the correct resolution model for simple Type 1 / TrueType / CFF
/// fonts. The CFF font program's *own* Encoding table is only authoritative
/// when there is no PDF-level encoding to consult (an Identity case).
///
/// In practice the bug this fixes is prepress-tool-authored subset CFFs
/// whose internal Encoding lists only `0x20 → space` and `0x41 → A` while
/// the Charset enumerates the full subset (e.g. `A B C D E F G I K M N O R
/// S U V X g`). The previous resolution consulted the CFF Encoding directly
/// and silently dropped every non-A content byte to `.notdef`, producing
/// bare-A glyphs on every separation plate.
///
/// Returns `None` when the result is empty so callers can fall through to
/// the legacy [`parse_cff_gid_mapping`] for fonts where the PDF-level
/// encoding genuinely cannot resolve any byte (e.g. `Encoding::Identity`
/// on a CIDFont — though those normally short-circuit before reaching
/// this path).
pub fn parse_cff_gid_mapping_with_pdf_encoding(
    font_data: &[u8],
    pdf_encoding: &crate::fonts::font_dict::Encoding,
    differences: &HashMap<u8, String>,
) -> Option<HashMap<u8, u16>> {
    use crate::fonts::font_dict::Encoding;

    if matches!(pdf_encoding, Encoding::Identity) {
        // Caller has no byte→name mapping to supply — fall through to the
        // CFF Encoding-driven legacy path.
        return parse_cff_gid_mapping(font_data);
    }

    if font_data.len() < 4 {
        return None;
    }
    let cff_data = if font_data[0] != 1 {
        extract_cff_from_opentype(font_data)?
    } else {
        font_data
    };
    if cff_data.len() < 4 || cff_data[0] != 1 {
        return None;
    }
    let hdr_size = cff_data[2] as usize;

    let (_, after_name) = parse_index(cff_data, hdr_size)?;
    let (top_dicts, after_top_dict) = parse_index(cff_data, after_name)?;
    if top_dicts.is_empty() {
        return None;
    }
    let (string_index, _after_string) = parse_index(cff_data, after_top_dict)?;
    let (charstrings_offset, _encoding_offset, charset_offset) =
        parse_top_dict_with_charstrings(top_dicts[0]);

    // §9.6.6 path: build name→GID from the Charset (which always enumerates
    // every subset glyph), then key bytes through the PDF /Encoding +
    // /Differences.
    //
    // nGlyphs comes from the CharStrings INDEX header (CFF spec §9). Simple
    // fonts only address ≤256 codes via /Encoding, but a subset's GID space
    // can exceed 256 — a /Differences entry pointing at a glyph whose CFF
    // Charset GID is >256 must still resolve. Falls back to 256 if the
    // CharStrings offset is missing or the INDEX header is truncated.
    let num_glyphs = if charstrings_offset > 0 {
        read_index_count(cff_data, charstrings_offset as usize)
            .map(|n| n as usize)
            .unwrap_or(256)
    } else {
        256
    };

    let charset_sids = if charset_offset > 2 {
        parse_charset(cff_data, charset_offset as usize, num_glyphs)?
    } else {
        // charset_offset 0 or 1 = ISOAdobe / Expert / ExpertSubset
        // predefined charsets. The CFF Standard Encoding + charset path in
        // `parse_cff_gid_mapping` handles these; defer.
        return parse_cff_gid_mapping(font_data);
    };

    let resolved =
        resolve_bytes_via_pdf_encoding(&charset_sids, &string_index, pdf_encoding, differences);

    if resolved.is_empty() {
        // PDF /Encoding yielded zero hits against the Charset. Fall back to
        // the CFF Encoding-driven path so we never make a working font worse.
        parse_cff_gid_mapping(font_data)
    } else {
        Some(resolved)
    }
}

/// Pure-input helper: given a parsed CFF Charset + String INDEX, build the
/// byte → GID map driven by the PDF font dictionary's `/Encoding` and
/// `/Differences`. Split out of [`parse_cff_gid_mapping_with_pdf_encoding`]
/// so the name-resolution logic can be tested without constructing a
/// custom CFF binary.
fn resolve_bytes_via_pdf_encoding(
    charset_sids: &[u16],
    string_index: &[&[u8]],
    pdf_encoding: &crate::fonts::font_dict::Encoding,
    differences: &HashMap<u8, String>,
) -> HashMap<u8, u16> {
    use crate::fonts::font_dict::{Encoding, FontInfo};

    // Glyph name → GID (lowest GID wins on duplicate names — first occurrence
    // in the Charset reflects the subsetter's primary mapping).
    let mut name_to_gid: HashMap<String, u16> = HashMap::new();
    for (gid, &sid) in charset_sids.iter().enumerate() {
        if gid == 0 {
            continue; // .notdef is implicit and not addressable by name
        }
        if let Some(name) = resolve_glyph_name(sid, string_index) {
            name_to_gid.entry(name).or_insert(gid as u16);
        }
    }

    // Base byte → glyph-name resolver, selected per ISO 32000-1 §9.6.6.1 +
    // Annex D. WinAnsi, MacRoman, and StandardEncoding share ASCII names
    // (0x20-0x7E) but diverge in 0x80-0xFF; the wrong table mis-resolves
    // high bytes for fonts whose /BaseEncoding is non-WinAnsi.
    //
    // `Encoding::Custom(_)` carries the byte→Unicode result of /BaseEncoding
    // + /Differences merge but loses the named base, so we can't tell whether
    // the unmodified bytes came from MacRoman or WinAnsi. The high-byte path
    // for Custom encodings defaults to WinAnsi for backward compatibility;
    // threading /BaseEncoding through `Encoding::Custom` is a separate
    // follow-up.
    let resolve_base_byte = |byte: u8| -> Option<&'static str> {
        match pdf_encoding {
            Encoding::Standard(name) => match name.as_str() {
                "MacRomanEncoding" => mac_roman_byte_to_name(byte),
                "StandardEncoding" => standard_encoding_byte_to_name(byte),
                // WinAnsiEncoding, MacExpertEncoding, PDFDocEncoding, and any
                // unrecognised string fall through to WinAnsi. Mac Expert is
                // a non-text variant; PDF Doc Encoding overlaps WinAnsi.
                _ => FontInfo::gid_to_standard_glyph_name(byte as u16),
            },
            Encoding::Custom(_) => FontInfo::gid_to_standard_glyph_name(byte as u16),
            Encoding::Identity => None, // handled by the outer guard already
        }
    };

    let mut out: HashMap<u8, u16> = HashMap::new();
    for byte_code in 0u16..256 {
        let byte = byte_code as u8;

        // §9.6.6: /Differences entries override the base predefined encoding.
        if let Some(diff_name) = differences.get(&byte) {
            if let Some(&gid) = name_to_gid.get(diff_name) {
                out.insert(byte, gid);
                continue;
            }
        }

        if let Some(name) = resolve_base_byte(byte) {
            if let Some(&gid) = name_to_gid.get(name) {
                out.insert(byte, gid);
            }
        }
    }
    out
}

/// Annex D Table D.2 (MacRomanEncoding) byte → glyph name.
///
/// ASCII range 0x20-0x7E shares glyph names with WinAnsi; 0x80-0xFF is
/// the Mac OS Roman repertoire (which is *not* ISO-8859-1). The Apple-logo
/// glyph at 0xF0 (PUA U+F8FF) has no portable glyph name and returns `None`.
fn mac_roman_byte_to_name(byte: u8) -> Option<&'static str> {
    use crate::fonts::font_dict::FontInfo;
    if (0x20..=0x7E).contains(&byte) {
        return FontInfo::gid_to_standard_glyph_name(byte as u16);
    }
    match byte {
        0x80 => Some("Adieresis"),
        0x81 => Some("Aring"),
        0x82 => Some("Ccedilla"),
        0x83 => Some("Eacute"),
        0x84 => Some("Ntilde"),
        0x85 => Some("Odieresis"),
        0x86 => Some("Udieresis"),
        0x87 => Some("aacute"),
        0x88 => Some("agrave"),
        0x89 => Some("acircumflex"),
        0x8A => Some("adieresis"),
        0x8B => Some("atilde"),
        0x8C => Some("aring"),
        0x8D => Some("ccedilla"),
        0x8E => Some("eacute"),
        0x8F => Some("egrave"),
        0x90 => Some("ecircumflex"),
        0x91 => Some("edieresis"),
        0x92 => Some("iacute"),
        0x93 => Some("igrave"),
        0x94 => Some("icircumflex"),
        0x95 => Some("idieresis"),
        0x96 => Some("ntilde"),
        0x97 => Some("oacute"),
        0x98 => Some("ograve"),
        0x99 => Some("ocircumflex"),
        0x9A => Some("odieresis"),
        0x9B => Some("otilde"),
        0x9C => Some("uacute"),
        0x9D => Some("ugrave"),
        0x9E => Some("ucircumflex"),
        0x9F => Some("udieresis"),
        0xA0 => Some("dagger"),
        0xA1 => Some("degree"),
        0xA2 => Some("cent"),
        0xA3 => Some("sterling"),
        0xA4 => Some("section"),
        0xA5 => Some("bullet"),
        0xA6 => Some("paragraph"),
        0xA7 => Some("germandbls"),
        0xA8 => Some("registered"),
        0xA9 => Some("copyright"),
        0xAA => Some("trademark"),
        0xAB => Some("acute"),
        0xAC => Some("dieresis"),
        0xAD => Some("notequal"),
        0xAE => Some("AE"),
        0xAF => Some("Oslash"),
        0xB0 => Some("infinity"),
        0xB1 => Some("plusminus"),
        0xB2 => Some("lessequal"),
        0xB3 => Some("greaterequal"),
        0xB4 => Some("yen"),
        0xB5 => Some("mu"),
        0xB6 => Some("partialdiff"),
        0xB7 => Some("summation"),
        0xB8 => Some("product"),
        0xB9 => Some("pi"),
        0xBA => Some("integral"),
        0xBB => Some("ordfeminine"),
        0xBC => Some("ordmasculine"),
        0xBD => Some("Omega"),
        0xBE => Some("ae"),
        0xBF => Some("oslash"),
        0xC0 => Some("questiondown"),
        0xC1 => Some("exclamdown"),
        0xC2 => Some("logicalnot"),
        0xC3 => Some("radical"),
        0xC4 => Some("florin"),
        0xC5 => Some("approxequal"),
        0xC6 => Some("Delta"),
        0xC7 => Some("guillemotleft"),
        0xC8 => Some("guillemotright"),
        0xC9 => Some("ellipsis"),
        0xCA => Some("space"), // nonbreakingspace; the canonical glyph name is "space"
        0xCB => Some("Agrave"),
        0xCC => Some("Atilde"),
        0xCD => Some("Otilde"),
        0xCE => Some("OE"),
        0xCF => Some("oe"),
        0xD0 => Some("endash"),
        0xD1 => Some("emdash"),
        0xD2 => Some("quotedblleft"),
        0xD3 => Some("quotedblright"),
        0xD4 => Some("quoteleft"),
        0xD5 => Some("quoteright"),
        0xD6 => Some("divide"),
        0xD7 => Some("lozenge"),
        0xD8 => Some("ydieresis"),
        0xD9 => Some("Ydieresis"),
        0xDA => Some("fraction"),
        0xDB => Some("currency"),
        0xDC => Some("guilsinglleft"),
        0xDD => Some("guilsinglright"),
        0xDE => Some("fi"),
        0xDF => Some("fl"),
        0xE0 => Some("daggerdbl"),
        0xE1 => Some("periodcentered"),
        0xE2 => Some("quotesinglbase"),
        0xE3 => Some("quotedblbase"),
        0xE4 => Some("perthousand"),
        0xE5 => Some("Acircumflex"),
        0xE6 => Some("Ecircumflex"),
        0xE7 => Some("Aacute"),
        0xE8 => Some("Edieresis"),
        0xE9 => Some("Egrave"),
        0xEA => Some("Iacute"),
        0xEB => Some("Icircumflex"),
        0xEC => Some("Idieresis"),
        0xED => Some("Igrave"),
        0xEE => Some("Oacute"),
        0xEF => Some("Ocircumflex"),
        0xF0 => None, // Apple-logo PUA glyph; no portable name
        0xF1 => Some("Ograve"),
        0xF2 => Some("Uacute"),
        0xF3 => Some("Ucircumflex"),
        0xF4 => Some("Ugrave"),
        0xF5 => Some("dotlessi"),
        0xF6 => Some("circumflex"),
        0xF7 => Some("tilde"),
        0xF8 => Some("macron"),
        0xF9 => Some("breve"),
        0xFA => Some("dotaccent"),
        0xFB => Some("ring"),
        0xFC => Some("cedilla"),
        0xFD => Some("hungarumlaut"),
        0xFE => Some("ogonek"),
        0xFF => Some("caron"),
        _ => None,
    }
}

/// Annex D Table D.1 (StandardEncoding, PostScript) byte → glyph name.
///
/// ASCII range 0x20-0x7E shares glyph names with WinAnsi except a handful
/// (0x27 quoteright, 0x60 quoteleft, etc.) — those overlap the existing
/// WinAnsi table's choice of `quoteright` for 0x27. The high-byte range
/// has its own PostScript repertoire (fraction at 0xA4, ligature `fi`/`fl`
/// at 0xAE/0xAF, etc.) which differs sharply from WinAnsi.
///
/// Bytes left unassigned by StandardEncoding return `None`.
fn standard_encoding_byte_to_name(byte: u8) -> Option<&'static str> {
    use crate::fonts::font_dict::FontInfo;
    if (0x20..=0x7E).contains(&byte) {
        return FontInfo::gid_to_standard_glyph_name(byte as u16);
    }
    match byte {
        0xA1 => Some("exclamdown"),
        0xA2 => Some("cent"),
        0xA3 => Some("sterling"),
        0xA4 => Some("fraction"),
        0xA5 => Some("yen"),
        0xA6 => Some("florin"),
        0xA7 => Some("section"),
        0xA8 => Some("currency"),
        0xA9 => Some("quotesingle"),
        0xAA => Some("quotedblleft"),
        0xAB => Some("guillemotleft"),
        0xAC => Some("guilsinglleft"),
        0xAD => Some("guilsinglright"),
        0xAE => Some("fi"),
        0xAF => Some("fl"),
        0xB1 => Some("endash"),
        0xB2 => Some("dagger"),
        0xB3 => Some("daggerdbl"),
        0xB4 => Some("periodcentered"),
        0xB6 => Some("paragraph"),
        0xB7 => Some("bullet"),
        0xB8 => Some("quotesinglbase"),
        0xB9 => Some("quotedblbase"),
        0xBA => Some("quotedblright"),
        0xBB => Some("guillemotright"),
        0xBC => Some("ellipsis"),
        0xBD => Some("perthousand"),
        0xBF => Some("questiondown"),
        0xC1 => Some("grave"),
        0xC2 => Some("acute"),
        0xC3 => Some("circumflex"),
        0xC4 => Some("tilde"),
        0xC5 => Some("macron"),
        0xC6 => Some("breve"),
        0xC7 => Some("dotaccent"),
        0xC8 => Some("dieresis"),
        0xCA => Some("ring"),
        0xCB => Some("cedilla"),
        0xCD => Some("hungarumlaut"),
        0xCE => Some("ogonek"),
        0xCF => Some("caron"),
        0xE1 => Some("AE"),
        0xE3 => Some("ordfeminine"),
        0xE8 => Some("Lslash"),
        0xE9 => Some("Oslash"),
        0xEA => Some("OE"),
        0xEB => Some("ordmasculine"),
        0xF1 => Some("ae"),
        0xF5 => Some("dotlessi"),
        0xF8 => Some("lslash"),
        0xF9 => Some("oslash"),
        0xFA => Some("oe"),
        0xFB => Some("germandbls"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==========================================
    // sid_to_name tests
    // ==========================================

    #[test]
    fn test_sid_to_name_notdef() {
        assert_eq!(sid_to_name(0), Some(".notdef"));
    }

    #[test]
    fn test_sid_to_name_space() {
        assert_eq!(sid_to_name(1), Some("space"));
    }

    #[test]
    fn test_sid_to_name_letters() {
        assert_eq!(sid_to_name(34), Some("A"));
        assert_eq!(sid_to_name(59), Some("Z"));
        assert_eq!(sid_to_name(66), Some("a"));
        assert_eq!(sid_to_name(91), Some("z"));
    }

    #[test]
    fn test_sid_to_name_digits() {
        assert_eq!(sid_to_name(17), Some("zero"));
        assert_eq!(sid_to_name(26), Some("nine"));
    }

    #[test]
    fn test_sid_to_name_punctuation() {
        assert_eq!(sid_to_name(2), Some("exclam"));
        assert_eq!(sid_to_name(15), Some("period"));
        assert_eq!(sid_to_name(13), Some("comma"));
    }

    #[test]
    fn test_sid_to_name_ligatures() {
        assert_eq!(sid_to_name(109), Some("fi"));
        assert_eq!(sid_to_name(110), Some("fl"));
        assert_eq!(sid_to_name(266), Some("ff"));
        assert_eq!(sid_to_name(267), Some("ffi"));
        assert_eq!(sid_to_name(268), Some("ffl"));
    }

    #[test]
    fn test_sid_to_name_accented() {
        assert_eq!(sid_to_name(171), Some("Aacute"));
        assert_eq!(sid_to_name(200), Some("aacute"));
        assert_eq!(sid_to_name(227), Some("ydieresis"));
    }

    #[test]
    fn test_sid_to_name_last_entries() {
        assert_eq!(sid_to_name(388), Some("Regular"));
        assert_eq!(sid_to_name(389), Some("Roman"));
        assert_eq!(sid_to_name(390), Some("Semibold"));
    }

    #[test]
    fn test_sid_to_name_out_of_range() {
        assert_eq!(sid_to_name(391), None);
        assert_eq!(sid_to_name(500), None);
        assert_eq!(sid_to_name(u16::MAX), None);
    }

    // ==========================================
    // parse_index tests
    // ==========================================

    #[test]
    fn test_parse_index_too_short() {
        assert_eq!(parse_index(&[0x00], 0), None);
    }

    #[test]
    fn test_parse_index_empty() {
        // count = 0
        let data = [0x00, 0x00];
        let result = parse_index(&data, 0);
        assert!(result.is_some());
        let (entries, next) = result.unwrap();
        assert!(entries.is_empty());
        assert_eq!(next, 2);
    }

    #[test]
    fn test_parse_index_single_entry() {
        // count=1, off_size=1, offsets=[1, 4] (entry is 3 bytes)
        let data = vec![
            0x00, 0x01, // count = 1
            0x01, // off_size = 1
            0x01, // offset[0] = 1
            0x04, // offset[1] = 4
            b'A', b'B', b'C', // data: "ABC"
        ];
        let result = parse_index(&data, 0);
        assert!(result.is_some());
        let (entries, _next) = result.unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0], b"ABC");
    }

    #[test]
    fn test_parse_index_multiple_entries() {
        // count=2, off_size=1, offsets=[1, 3, 5]
        let data = vec![
            0x00, 0x02, // count = 2
            0x01, // off_size = 1
            0x01, // offset[0] = 1
            0x03, // offset[1] = 3
            0x05, // offset[2] = 5
            b'H', b'i', // entry 0: "Hi"
            b'O', b'K', // entry 1: "OK"
        ];
        let result = parse_index(&data, 0);
        assert!(result.is_some());
        let (entries, _next) = result.unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0], b"Hi");
        assert_eq!(entries[1], b"OK");
    }

    #[test]
    fn test_parse_index_invalid_off_size_zero() {
        let data = vec![0x00, 0x01, 0x00]; // off_size = 0
        assert_eq!(parse_index(&data, 0), None);
    }

    #[test]
    fn test_parse_index_invalid_off_size_too_large() {
        let data = vec![0x00, 0x01, 0x05]; // off_size = 5
        assert_eq!(parse_index(&data, 0), None);
    }

    #[test]
    fn test_parse_index_truncated_offset_array() {
        // count=1, off_size=1, but not enough data for offsets
        let data = vec![0x00, 0x01, 0x01, 0x01]; // missing second offset
        assert_eq!(parse_index(&data, 0), None);
    }

    #[test]
    fn test_parse_index_with_offset() {
        // Parse index starting at offset 3
        let data = vec![
            0xFF, 0xFF, 0xFF, // padding
            0x00, 0x01, // count = 1
            0x01, // off_size = 1
            0x01, // offset[0] = 1
            0x02, // offset[1] = 2
            b'X', // data: "X"
        ];
        let result = parse_index(&data, 3);
        assert!(result.is_some());
        let (entries, _) = result.unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0], b"X");
    }

    #[test]
    fn test_parse_index_off_size_2() {
        // count=1, off_size=2, offsets=[0x0001, 0x0003]
        let data = vec![
            0x00, 0x01, // count = 1
            0x02, // off_size = 2
            0x00, 0x01, // offset[0] = 1
            0x00, 0x03, // offset[1] = 3
            b'A', b'B', // data: "AB"
        ];
        let result = parse_index(&data, 0);
        assert!(result.is_some());
        let (entries, _) = result.unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0], b"AB");
    }

    #[test]
    fn test_parse_index_data_out_of_bounds() {
        // offsets reference data beyond the buffer
        let data = vec![
            0x00, 0x01, // count = 1
            0x01, // off_size = 1
            0x01, // offset[0] = 1
            0xFF, // offset[1] = 255 (way out of bounds)
        ];
        assert_eq!(parse_index(&data, 0), None);
    }

    // ==========================================
    // parse_dict_operand tests
    // ==========================================

    #[test]
    fn test_parse_dict_operand_empty() {
        assert_eq!(parse_dict_operand(&[], 0), None);
    }

    #[test]
    fn test_parse_dict_operand_single_byte_zero() {
        // b0 = 139 => value = 139 - 139 = 0
        assert_eq!(parse_dict_operand(&[139], 0), Some((0, 1)));
    }

    #[test]
    fn test_parse_dict_operand_single_byte_positive() {
        // b0 = 246 => value = 246 - 139 = 107
        assert_eq!(parse_dict_operand(&[246], 0), Some((107, 1)));
    }

    #[test]
    fn test_parse_dict_operand_single_byte_negative() {
        // b0 = 32 => value = 32 - 139 = -107
        assert_eq!(parse_dict_operand(&[32], 0), Some((-107, 1)));
    }

    #[test]
    fn test_parse_dict_operand_two_byte_positive() {
        // b0=247, b1=0 => (247-247)*256 + 0 + 108 = 108
        assert_eq!(parse_dict_operand(&[247, 0], 0), Some((108, 2)));
        // b0=250, b1=255 => (250-247)*256 + 255 + 108 = 1131
        assert_eq!(parse_dict_operand(&[250, 255], 0), Some((1131, 2)));
    }

    #[test]
    fn test_parse_dict_operand_two_byte_negative() {
        // b0=251, b1=0 => -(251-251)*256 - 0 - 108 = -108
        assert_eq!(parse_dict_operand(&[251, 0], 0), Some((-108, 2)));
        // b0=254, b1=255 => -(254-251)*256 - 255 - 108 = -1131
        assert_eq!(parse_dict_operand(&[254, 255], 0), Some((-1131, 2)));
    }

    #[test]
    fn test_parse_dict_operand_two_byte_truncated() {
        // b0=247 but no b1
        assert_eq!(parse_dict_operand(&[247], 0), None);
        assert_eq!(parse_dict_operand(&[251], 0), None);
    }

    #[test]
    fn test_parse_dict_operand_three_byte_int16() {
        // b0=28, then 16-bit signed int
        // 0x00, 0x01 => 1
        assert_eq!(parse_dict_operand(&[28, 0x00, 0x01], 0), Some((1, 3)));
        // 0xFF, 0xFF => -1
        assert_eq!(parse_dict_operand(&[28, 0xFF, 0xFF], 0), Some((-1, 3)));
        // 0x7F, 0xFF => 32767
        assert_eq!(parse_dict_operand(&[28, 0x7F, 0xFF], 0), Some((32767, 3)));
    }

    #[test]
    fn test_parse_dict_operand_three_byte_truncated() {
        assert_eq!(parse_dict_operand(&[28, 0x00], 0), None);
    }

    #[test]
    fn test_parse_dict_operand_five_byte_int32() {
        // b0=29, then 32-bit signed int
        // 0x00, 0x00, 0x00, 0x01 => 1
        assert_eq!(parse_dict_operand(&[29, 0x00, 0x00, 0x00, 0x01], 0), Some((1, 5)));
        // 0xFF, 0xFF, 0xFF, 0xFF => -1
        assert_eq!(parse_dict_operand(&[29, 0xFF, 0xFF, 0xFF, 0xFF], 0), Some((-1, 5)));
    }

    #[test]
    fn test_parse_dict_operand_five_byte_truncated() {
        assert_eq!(parse_dict_operand(&[29, 0x00, 0x00, 0x00], 0), None);
    }

    #[test]
    fn test_parse_dict_operand_real_number() {
        // b0=30, then nibble pairs terminated by 0xF
        // 1.5 would be encoded as: 0x1A 0x5F (1, '.', 5, end)
        // But since we just skip real numbers and return 0...
        let data = [30, 0x1A, 0x5F];
        let result = parse_dict_operand(&data, 0);
        assert!(result.is_some());
        let (val, consumed) = result.unwrap();
        assert_eq!(val, 0); // Real numbers return 0
        assert_eq!(consumed, 3);
    }

    #[test]
    fn test_parse_dict_operand_real_nibble1_end() {
        // nibble1 = 0xF => end marker in high nibble
        let data = [30, 0xF0];
        let result = parse_dict_operand(&data, 0);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), (0, 2));
    }

    #[test]
    fn test_parse_dict_operand_real_unterminated() {
        // Real number that runs off the end without terminator
        let data = [30, 0x12, 0x34];
        assert_eq!(parse_dict_operand(&data, 0), None);
    }

    #[test]
    fn test_parse_dict_operand_unknown_byte() {
        // Byte values 0-21 are operators, not operands => None
        assert_eq!(parse_dict_operand(&[0], 0), None);
        assert_eq!(parse_dict_operand(&[21], 0), None);
        // Byte 255 should be handled by the 251..=254 range => this is 255
        assert_eq!(parse_dict_operand(&[255], 0), None);
    }

    #[test]
    fn test_parse_dict_operand_with_offset() {
        // Parse from a non-zero position
        let data = [0x00, 0x00, 139]; // value 0 at position 2
        assert_eq!(parse_dict_operand(&data, 2), Some((0, 1)));
    }

    // ==========================================
    // parse_top_dict tests
    // ==========================================

    #[test]
    fn test_parse_top_dict_empty() {
        let (enc, charset) = parse_top_dict(&[]);
        assert_eq!(enc, 0);
        assert_eq!(charset, 0);
    }

    #[test]
    fn test_parse_top_dict_encoding_offset() {
        // Push operand 42 (139+42=181), then operator 16 (encoding)
        let data = [181, 16];
        let (enc, charset) = parse_top_dict(&data);
        assert_eq!(enc, 42);
        assert_eq!(charset, 0); // default
    }

    #[test]
    fn test_parse_top_dict_charset_offset() {
        // Push operand 99 (139+99=238), then operator 15 (charset)
        let data = [238, 15];
        let (enc, charset) = parse_top_dict(&data);
        assert_eq!(enc, 0); // default
        assert_eq!(charset, 99);
    }

    #[test]
    fn test_parse_top_dict_both_offsets() {
        // encoding=50 (189), op 16, charset=100 (239), op 15
        let data = [189, 16, 239, 15];
        let (enc, charset) = parse_top_dict(&data);
        assert_eq!(enc, 50);
        assert_eq!(charset, 100);
    }

    #[test]
    fn test_parse_top_dict_two_byte_operator() {
        // b0=12 signals two-byte operator; b0=12, b1=X => operator (12<<8)|X
        // e.g. operator 12, sub 0 => (3072) => not encoding/charset, should be ignored
        let data = [139, 12, 0]; // push 0, then op 12.0
        let (enc, charset) = parse_top_dict(&data);
        assert_eq!(enc, 0);
        assert_eq!(charset, 0);
    }

    #[test]
    fn test_parse_top_dict_unknown_operator() {
        // Unknown operator 17 (not encoding=16 or charset=15)
        let data = [181, 17]; // push 42, operator 17
        let (enc, charset) = parse_top_dict(&data);
        assert_eq!(enc, 0); // not set
        assert_eq!(charset, 0); // not set
    }

    #[test]
    fn test_parse_top_dict_skip_unparseable() {
        // If operand parse fails, we just skip the byte
        // Use byte 255 which is not a valid operand (nor operator)
        let data = [255, 181, 16]; // 255 skipped, then 42 + encoding op
        let (enc, charset) = parse_top_dict(&data);
        assert_eq!(enc, 42);
        assert_eq!(charset, 0);
    }

    // ==========================================
    // parse_charset tests
    // ==========================================

    #[test]
    fn test_parse_charset_out_of_bounds() {
        assert_eq!(parse_charset(&[0x00], 5, 10), None);
    }

    #[test]
    fn test_parse_charset_format0() {
        // format=0, then SIDs for GID 1,2,3...
        let data = vec![
            0x00, // format 0
            0x00, 0x01, // SID 1 (space)
            0x00, 0x22, // SID 34 (A)
            0x00, 0x42, // SID 66 (a)
        ];
        let result = parse_charset(&data, 0, 4);
        assert!(result.is_some());
        let sids = result.unwrap();
        assert_eq!(sids.len(), 4);
        assert_eq!(sids[0], 0); // .notdef
        assert_eq!(sids[1], 1); // space
        assert_eq!(sids[2], 34); // A
        assert_eq!(sids[3], 66); // a
    }

    #[test]
    fn test_parse_charset_format1() {
        // format=1, range: first_sid=34, n_left=2 => SIDs 34,35,36
        let data = vec![
            0x01, // format 1
            0x00, 0x22, // first_sid = 34
            0x02, // n_left = 2
        ];
        let result = parse_charset(&data, 0, 4);
        assert!(result.is_some());
        let sids = result.unwrap();
        assert_eq!(sids.len(), 4);
        assert_eq!(sids[0], 0); // .notdef
        assert_eq!(sids[1], 34); // A
        assert_eq!(sids[2], 35); // B
        assert_eq!(sids[3], 36); // C
    }

    #[test]
    fn test_parse_charset_format2() {
        // format=2, range: first_sid=66, n_left=3 => SIDs 66,67,68,69
        let data = vec![
            0x02, // format 2
            0x00, 0x42, // first_sid = 66
            0x00, 0x03, // n_left = 3
        ];
        let result = parse_charset(&data, 0, 5);
        assert!(result.is_some());
        let sids = result.unwrap();
        assert_eq!(sids.len(), 5);
        assert_eq!(sids[0], 0);
        assert_eq!(sids[1], 66); // a
        assert_eq!(sids[2], 67); // b
        assert_eq!(sids[3], 68); // c
        assert_eq!(sids[4], 69); // d
    }

    #[test]
    fn test_parse_charset_unknown_format() {
        let data = vec![0x03]; // format 3 doesn't exist
        assert_eq!(parse_charset(&data, 0, 2), None);
    }

    #[test]
    fn test_parse_charset_format0_truncated() {
        // format=0 but not enough data for SIDs
        let data = vec![0x00, 0x00]; // only 1 byte of SID data (need 2)
        let result = parse_charset(&data, 0, 3);
        assert!(result.is_some());
        let sids = result.unwrap();
        // Should get .notdef + whatever it could parse
        assert_eq!(sids[0], 0);
    }

    #[test]
    fn test_parse_charset_format1_limits_to_num_glyphs() {
        // Range would give more SIDs than needed
        let data = vec![
            0x01, // format 1
            0x00, 0x01, // first_sid = 1
            0xFF, // n_left = 255 (way more than we need)
        ];
        let result = parse_charset(&data, 0, 3);
        assert!(result.is_some());
        let sids = result.unwrap();
        assert_eq!(sids.len(), 3); // limited to num_glyphs
    }

    // ==========================================
    // parse_encoding_table tests
    // ==========================================

    #[test]
    fn test_parse_encoding_table_out_of_bounds() {
        assert_eq!(parse_encoding_table(&[0x00], 5), None);
    }

    #[test]
    fn test_parse_encoding_table_format0() {
        let data = vec![
            0x00, // format 0 (no supplement)
            0x03, // n_codes = 3
            0x41, // code 0x41 => GID 1
            0x42, // code 0x42 => GID 2
            0x43, // code 0x43 => GID 3
        ];
        let result = parse_encoding_table(&data, 0);
        assert!(result.is_some());
        let map = result.unwrap();
        assert_eq!(map.get(&0x41), Some(&1));
        assert_eq!(map.get(&0x42), Some(&2));
        assert_eq!(map.get(&0x43), Some(&3));
    }

    #[test]
    fn test_parse_encoding_table_format1() {
        let data = vec![
            0x01, // format 1
            0x01, // n_ranges = 1
            0x41, // first = 0x41
            0x02, // n_left = 2 (codes 0x41, 0x42, 0x43)
        ];
        let result = parse_encoding_table(&data, 0);
        assert!(result.is_some());
        let map = result.unwrap();
        assert_eq!(map.get(&0x41), Some(&1));
        assert_eq!(map.get(&0x42), Some(&2));
        assert_eq!(map.get(&0x43), Some(&3));
    }

    #[test]
    fn test_parse_encoding_table_unknown_format() {
        let data = vec![0x02]; // format 2 doesn't exist for encoding
                               // 0x02 & 0x7F = 2
        assert_eq!(parse_encoding_table(&data, 0), None);
    }

    #[test]
    fn test_parse_encoding_table_format0_truncated() {
        let data = vec![
            0x00, // format 0
            0x05, // n_codes = 5 but only 2 bytes of data follow
            0x41, 0x42,
        ];
        let result = parse_encoding_table(&data, 0);
        assert!(result.is_some());
        let map = result.unwrap();
        assert_eq!(map.len(), 2);
    }

    #[test]
    fn test_parse_encoding_table_with_supplement() {
        let data = vec![
            0x80, // format 0 with supplement flag (bit 7 set)
            0x01, // n_codes = 1
            0x41, // code 0x41 => GID 1
            0x01, // n_sups = 1
            0x42, // supplement code = 0x42
            0x00, 0x22, // supplement SID = 34
        ];
        let result = parse_encoding_table(&data, 0);
        assert!(result.is_some());
        let map = result.unwrap();
        assert_eq!(map.get(&0x41), Some(&1));
        assert_eq!(map.get(&0x42), Some(&34)); // supplement
    }

    #[test]
    fn test_parse_encoding_table_format1_truncated() {
        let data = vec![
            0x01, // format 1
            0x02, // n_ranges = 2, but only 1 range of data follows
            0x41, 0x01, // range 1: first=0x41, n_left=1
        ];
        let result = parse_encoding_table(&data, 0);
        assert!(result.is_some());
        let map = result.unwrap();
        assert_eq!(map.len(), 2); // Only first range parsed
    }

    #[test]
    fn test_parse_encoding_table_format0_empty_pos() {
        // format 0, but pos after format byte is at end
        let data = vec![0x00];
        assert_eq!(parse_encoding_table(&data, 0), None);
    }

    #[test]
    fn test_parse_encoding_table_format1_empty_pos() {
        let data = vec![0x01];
        assert_eq!(parse_encoding_table(&data, 0), None);
    }

    // ==========================================
    // resolve_glyph_name tests
    // ==========================================

    #[test]
    fn test_resolve_glyph_name_predefined() {
        let string_index: Vec<&[u8]> = vec![];
        assert_eq!(resolve_glyph_name(0, &string_index), Some(".notdef".to_string()));
        assert_eq!(resolve_glyph_name(1, &string_index), Some("space".to_string()));
        assert_eq!(resolve_glyph_name(34, &string_index), Some("A".to_string()));
        assert_eq!(resolve_glyph_name(390, &string_index), Some("Semibold".to_string()));
    }

    #[test]
    fn test_resolve_glyph_name_custom_string() {
        let custom: Vec<&[u8]> = vec![b"MyGlyph", b"AnotherGlyph"];
        // SID 391 => index 0 in string_index
        assert_eq!(resolve_glyph_name(391, &custom), Some("MyGlyph".to_string()));
        // SID 392 => index 1
        assert_eq!(resolve_glyph_name(392, &custom), Some("AnotherGlyph".to_string()));
    }

    #[test]
    fn test_resolve_glyph_name_custom_out_of_range() {
        let custom: Vec<&[u8]> = vec![b"OnlyOne"];
        assert_eq!(resolve_glyph_name(393, &custom), None); // index 2, but only 1 entry
    }

    #[test]
    fn test_resolve_glyph_name_custom_invalid_utf8() {
        let invalid_utf8: Vec<&[u8]> = vec![&[0xFF, 0xFE]];
        assert_eq!(resolve_glyph_name(391, &invalid_utf8), None);
    }

    // ==========================================
    // extract_cff_from_opentype tests
    // ==========================================

    #[test]
    fn test_extract_cff_from_opentype_too_short() {
        assert_eq!(extract_cff_from_opentype(&[0; 8]), None);
    }

    #[test]
    fn test_extract_cff_from_opentype_not_opentype() {
        let data = vec![0x00; 16];
        assert_eq!(extract_cff_from_opentype(&data), None);
    }

    #[test]
    fn test_extract_cff_from_opentype_otto_no_cff_table() {
        // "OTTO" magic, 0 tables
        let data = vec![
            0x4F, 0x54, 0x54, 0x4F, // "OTTO"
            0x00, 0x00, // num_tables = 0
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // searchRange, entrySelector, rangeShift
        ];
        assert_eq!(extract_cff_from_opentype(&data), None);
    }

    #[test]
    fn test_extract_cff_from_opentype_with_cff_table() {
        let cff_data = b"\x01\x00\x04\x01"; // Minimal CFF header
        let cff_offset: u32 = 28; // 12 (header) + 16 (one table record)
        let cff_length: u32 = cff_data.len() as u32;

        let mut data = vec![
            0x4F, 0x54, 0x54, 0x4F, // "OTTO"
            0x00, 0x01, // num_tables = 1
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // searchRange etc
        ];
        // Table record: tag "CFF ", checksum, offset, length
        data.extend_from_slice(b"CFF "); // tag
        data.extend_from_slice(&[0, 0, 0, 0]); // checksum
        data.extend_from_slice(&cff_offset.to_be_bytes()); // offset
        data.extend_from_slice(&cff_length.to_be_bytes()); // length
        data.extend_from_slice(cff_data);

        let result = extract_cff_from_opentype(&data);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), cff_data);
    }

    #[test]
    fn test_extract_cff_from_opentype_truncated_table_dir() {
        // OTTO with 1 table but data too short
        let data = vec![
            0x4F, 0x54, 0x54, 0x4F, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            // table record starts but is truncated
            b'C', b'F',
        ];
        assert_eq!(extract_cff_from_opentype(&data), None);
    }

    // ==========================================
    // parse_cff_encoding (integration) tests
    // ==========================================

    #[test]
    fn test_parse_cff_encoding_too_short() {
        assert_eq!(parse_cff_encoding(&[0, 1, 2]), None);
    }

    #[test]
    fn test_parse_cff_encoding_wrong_version() {
        // Not version 1, and not an OpenType wrapper
        let data = vec![0x02, 0x00, 0x04, 0x01, 0x00];
        assert_eq!(parse_cff_encoding(&data), None);
    }

    #[test]
    fn test_parse_cff_encoding_version1_too_short_after_check() {
        // Version byte is 1 but too small overall
        let data = vec![0x01, 0x00, 0x04];
        assert_eq!(parse_cff_encoding(&data), None);
    }

    #[test]
    fn test_parse_cff_encoding_expert_encoding() {
        // Build a minimal valid CFF with encoding_offset=1 (ExpertEncoding)
        // This requires: header, name INDEX, top dict INDEX (with encoding=1), string INDEX
        let data = build_minimal_cff(1, 0);
        let result = parse_cff_encoding(&data);
        assert_eq!(result, None); // ExpertEncoding => None
    }

    #[test]
    fn test_parse_cff_encoding_standard_encoding_default_charset() {
        // encoding_offset=0, charset_offset=0 (both defaults)
        let data = build_minimal_cff(0, 0);
        let result = parse_cff_encoding(&data);
        assert_eq!(result, None); // StandardEncoding with default charset => None
    }

    /// Helper: builds a minimal CFF font binary with specified encoding and charset offsets.
    fn build_minimal_cff(encoding_offset: i32, charset_offset: i32) -> Vec<u8> {
        // CFF Header: major=1, minor=0, hdrSize=4, offSize=1
        let mut data = vec![1, 0, 4, 1];

        // Name INDEX: 1 entry "Test"
        append_index(&mut data, &[b"Test"]);

        // Top DICT INDEX: encode encoding_offset and charset_offset
        let top_dict = build_top_dict(encoding_offset, charset_offset);
        append_index(&mut data, &[&top_dict]);

        // String INDEX: empty
        append_index(&mut data, &[]);

        // Global Subr INDEX: empty
        append_index(&mut data, &[]);

        data
    }

    /// Encode a CFF DICT with encoding (op 16) and charset (op 15) operands.
    fn build_top_dict(encoding_offset: i32, charset_offset: i32) -> Vec<u8> {
        let mut dict = Vec::new();
        // Encode encoding_offset as operand, then op 16
        encode_dict_int(&mut dict, encoding_offset);
        dict.push(16); // encoding operator
                       // Encode charset_offset as operand, then op 15
        encode_dict_int(&mut dict, charset_offset);
        dict.push(15); // charset operator
        dict
    }

    /// Encode a CFF integer operand into DICT format.
    fn encode_dict_int(out: &mut Vec<u8>, val: i32) {
        if (-107..=107).contains(&val) {
            out.push((val + 139) as u8);
        } else if (108..=1131).contains(&val) {
            let v = val - 108;
            out.push((v / 256 + 247) as u8);
            out.push((v % 256) as u8);
        } else if (-1131..=-108).contains(&val) {
            let v = -val - 108;
            out.push((v / 256 + 251) as u8);
            out.push((v % 256) as u8);
        } else if (-32768..=32767).contains(&val) {
            out.push(28);
            let bytes = (val as i16).to_be_bytes();
            out.push(bytes[0]);
            out.push(bytes[1]);
        } else {
            out.push(29);
            let bytes = val.to_be_bytes();
            out.extend_from_slice(&bytes);
        }
    }

    /// Append a CFF INDEX to a data vector.
    fn append_index(data: &mut Vec<u8>, entries: &[&[u8]]) {
        let count = entries.len() as u16;
        data.extend_from_slice(&count.to_be_bytes());
        if count == 0 {
            return;
        }
        data.push(1); // off_size = 1

        // Offsets (1-based)
        let mut offset: u8 = 1;
        data.push(offset);
        for entry in entries {
            offset += entry.len() as u8;
            data.push(offset);
        }
        // Data
        for entry in entries {
            data.extend_from_slice(entry);
        }
    }

    // ==========================================
    // glyph_name_to_sid tests
    // ==========================================

    #[test]
    fn test_glyph_name_to_sid_known_names() {
        assert_eq!(glyph_name_to_sid(".notdef"), Some(0));
        assert_eq!(glyph_name_to_sid("space"), Some(1));
        assert_eq!(glyph_name_to_sid("A"), Some(34));
        assert_eq!(glyph_name_to_sid("B"), Some(35));
        assert_eq!(glyph_name_to_sid("Z"), Some(59));
        assert_eq!(glyph_name_to_sid("a"), Some(66));
        assert_eq!(glyph_name_to_sid("z"), Some(91));
        assert_eq!(glyph_name_to_sid("zero"), Some(17));
        assert_eq!(glyph_name_to_sid("nine"), Some(26));
    }

    #[test]
    fn test_glyph_name_to_sid_unknown() {
        assert_eq!(glyph_name_to_sid("nonexistent_glyph_xyz"), None);
        assert_eq!(glyph_name_to_sid(""), None);
    }

    #[test]
    fn test_glyph_name_to_sid_roundtrip() {
        // Verify sid_to_name and glyph_name_to_sid are consistent
        for sid in 0u16..391 {
            if let Some(name) = sid_to_name(sid) {
                assert_eq!(
                    glyph_name_to_sid(name),
                    Some(sid),
                    "Roundtrip failed for SID {} (name '{}')",
                    sid,
                    name
                );
            }
        }
    }

    // ==========================================
    // parse_cff_gid_mapping tests
    // ==========================================

    #[test]
    fn test_parse_cff_gid_mapping_invalid_data() {
        assert!(parse_cff_gid_mapping(&[]).is_none());
        assert!(parse_cff_gid_mapping(&[0, 1, 2]).is_none());
        assert!(parse_cff_gid_mapping(&[2, 0, 4, 2]).is_none()); // wrong version
    }

    // ==========================================
    // resolve_bytes_via_pdf_encoding tests
    //
    // These exercise the name-resolution layer in isolation — the layer
    // that was missing in `parse_cff_gid_mapping` and is the substantive
    // fix here. The CFF binary parser is reused unchanged, so its tests
    // remain authoritative for the parsing path.
    // ==========================================

    use crate::fonts::font_dict::Encoding;

    /// Pin a real-world sparse-CFF subset pattern: charset enumerates
    /// space, A, B, C, O, V, N (SIDs 1, 34, 35, 36, 48, 55, 47) on GIDs
    /// 1..=7. PDF /Encoding is WinAnsiEncoding. The resolver must produce
    /// GIDs for every charset entry — not just for byte 0x41 ("A") as the
    /// sparse CFF Encoding table would have implied.
    #[test]
    fn resolve_via_pdf_encoding_recovers_all_charset_glyphs() {
        // GID order: 0 = .notdef (implicit), 1 = space, 2 = A, 3 = B,
        // 4 = C, 5 = O, 6 = V, 7 = N.
        let charset = [0u16, 1, 34, 35, 36, 48, 55, 47];
        let string_index: Vec<&[u8]> = Vec::new();
        let pdf_enc = Encoding::Standard("WinAnsiEncoding".to_string());
        let differences: HashMap<u8, String> = HashMap::new();

        let map = resolve_bytes_via_pdf_encoding(&charset, &string_index, &pdf_enc, &differences);

        assert_eq!(map.get(&0x20), Some(&1), "0x20 (space) → GID 1");
        assert_eq!(map.get(&0x41), Some(&2), "0x41 (A) → GID 2");
        assert_eq!(map.get(&0x42), Some(&3), "0x42 (B) → GID 3");
        assert_eq!(map.get(&0x43), Some(&4), "0x43 (C) → GID 4");
        assert_eq!(map.get(&0x4f), Some(&5), "0x4f (O) → GID 5");
        assert_eq!(map.get(&0x56), Some(&6), "0x56 (V) → GID 6");
        assert_eq!(map.get(&0x4e), Some(&7), "0x4e (N) → GID 7");

        // Bytes whose glyph name is not in the Charset stay out.
        assert!(!map.contains_key(&0x7e), "0x7e (asciitilde) not in charset");
    }

    /// /Differences entries override the base predefined encoding.
    #[test]
    fn resolve_via_pdf_encoding_honors_differences_array() {
        // Charset includes "bullet" (SID 116) at GID 1.
        let charset = [0u16, 116];
        let string_index: Vec<&[u8]> = Vec::new();
        let pdf_enc = Encoding::Standard("WinAnsiEncoding".to_string());
        let mut differences = HashMap::new();
        // Override byte 0x95 to glyph name "bullet" — WinAnsi's native
        // 0x95 is also "bullet", but we want to pin that the /Differences
        // path is exercised (and would beat a divergent base encoding).
        differences.insert(0x95u8, "bullet".to_string());

        let map = resolve_bytes_via_pdf_encoding(&charset, &string_index, &pdf_enc, &differences);
        assert_eq!(map.get(&0x95), Some(&1));
    }

    /// Identity encoding short-circuits via the outer function; the
    /// helper itself is a no-op for Identity.
    #[test]
    fn resolve_via_pdf_encoding_skips_identity() {
        let charset = [0u16, 34];
        let string_index: Vec<&[u8]> = Vec::new();
        let pdf_enc = Encoding::Identity;
        let differences: HashMap<u8, String> = HashMap::new();
        let map = resolve_bytes_via_pdf_encoding(&charset, &string_index, &pdf_enc, &differences);
        assert!(map.is_empty(), "Identity → no base byte→name resolution");
    }

    /// Custom-string SIDs (>=391) resolved through the String INDEX
    /// land in the name→GID map.
    #[test]
    fn resolve_via_pdf_encoding_resolves_custom_string_sids() {
        // GID 1 is a glyph named "customGlyph" via custom SID 391.
        let charset = [0u16, 391];
        let custom: &[u8] = b"customGlyph";
        let string_index: Vec<&[u8]> = vec![custom];
        let pdf_enc = Encoding::Standard("WinAnsiEncoding".to_string());
        let mut differences = HashMap::new();
        differences.insert(0x21u8, "customGlyph".to_string());

        let map = resolve_bytes_via_pdf_encoding(&charset, &string_index, &pdf_enc, &differences);
        assert_eq!(map.get(&0x21), Some(&1));
    }

    // ==========================================
    // Base-encoding selection (§9.6.6.1 / Annex D)
    //
    // /BaseEncoding picks which byte→glyph-name table resolves high bytes
    // 0x80-0xFF. The three predefined bases diverge:
    //   - WinAnsi 0x80   = "euro"      (Windows-1252)
    //   - MacRoman 0x80  = "Adieresis" (Annex D Table D.2)
    //   - Standard 0xA4  = "fraction"  (Annex D Table D.1; WinAnsi = "currency")
    // Resolving the wrong table for a non-WinAnsi base would mis-route bytes.
    // ==========================================

    /// MacRomanEncoding: byte 0x80 must resolve via "Adieresis", not "euro".
    #[test]
    fn resolve_via_pdf_encoding_uses_mac_roman_table_for_mac_base() {
        // GID 1 = Adieresis (SID 173, predefined)
        // GID 2 = euro (custom SID 391 → string_index[0])
        let charset: Vec<u16> = vec![0, 173, 391];
        let string_index: Vec<&[u8]> = vec![b"euro"];
        let differences: HashMap<u8, String> = HashMap::new();
        let pdf_enc = Encoding::Standard("MacRomanEncoding".to_string());

        let map = resolve_bytes_via_pdf_encoding(&charset, &string_index, &pdf_enc, &differences);
        assert_eq!(map.get(&0x80), Some(&1), "MacRoman 0x80 → Adieresis (GID 1)");
    }

    /// StandardEncoding: byte 0xA4 must resolve via "fraction", not "currency".
    #[test]
    fn resolve_via_pdf_encoding_uses_standard_encoding_table_for_standard_base() {
        // GID 1 = fraction (SID 99), GID 2 = currency (SID 103).
        let charset: Vec<u16> = vec![0, 99, 103];
        let string_index: Vec<&[u8]> = vec![];
        let differences: HashMap<u8, String> = HashMap::new();
        let pdf_enc = Encoding::Standard("StandardEncoding".to_string());

        let map = resolve_bytes_via_pdf_encoding(&charset, &string_index, &pdf_enc, &differences);
        assert_eq!(map.get(&0xA4), Some(&1), "StandardEncoding 0xA4 → fraction (GID 1)");
    }

    /// WinAnsiEncoding regression guard: byte 0x80 still resolves to "euro".
    #[test]
    fn resolve_via_pdf_encoding_uses_winansi_table_for_winansi_base() {
        // Same fixture as the MacRoman test — only the declared base differs.
        let charset: Vec<u16> = vec![0, 173, 391];
        let string_index: Vec<&[u8]> = vec![b"euro"];
        let differences: HashMap<u8, String> = HashMap::new();
        let pdf_enc = Encoding::Standard("WinAnsiEncoding".to_string());

        let map = resolve_bytes_via_pdf_encoding(&charset, &string_index, &pdf_enc, &differences);
        assert_eq!(map.get(&0x80), Some(&2), "WinAnsi 0x80 → euro (GID 2)");
    }

    // ==========================================
    // Charset capacity: full nGlyphs, not capped at 256
    // ==========================================

    /// `parse_charset` already handles arbitrary nGlyphs; the cap lives in the
    /// caller (`parse_cff_gid_mapping_with_pdf_encoding`). This pins the
    /// parser's tolerance for >256 entries so a regression there would
    /// surface independently of the caller refactor.
    #[test]
    fn parse_charset_format0_handles_more_than_256_entries() {
        // Format 0: 1-byte format + 2-byte SID per non-.notdef GID.
        let mut data = vec![0x00u8];
        for gid in 1u16..=299u16 {
            data.extend_from_slice(&gid.to_be_bytes());
        }
        let sids = parse_charset(&data, 0, 300).expect("parse_charset returned None");
        assert_eq!(sids.len(), 300, "300 entries (GID 0 + 299 enumerated)");
        assert_eq!(sids[0], 0, "GID 0 is .notdef (SID 0)");
        assert_eq!(sids[1], 1, "GID 1 → SID 1");
        assert_eq!(sids[256], 256, "GID 256 → SID 256 (past the old 256 cap)");
        assert_eq!(sids[299], 299, "GID 299 → SID 299 (last entry)");
    }

    /// CFF Top DICT must surface the CharStrings INDEX offset (op 17) so
    /// callers can read the real nGlyphs from the INDEX header instead of
    /// guessing 256.
    #[test]
    fn parse_top_dict_surfaces_charstrings_offset() {
        // Minimal top DICT: one operand + op 17 (CharStrings).
        // Operand 1234 encoded as a 3-byte int per CFF DICT encoding:
        //   b0 = 29 (4-byte int marker is 29? Actually 28 = 2-byte, 29 = 4-byte)
        // Use the 2-byte form: b0=28, then 2 BE bytes.
        let mut dict = vec![28u8];
        dict.extend_from_slice(&1234i16.to_be_bytes());
        dict.push(17u8); // op 17 = CharStrings

        let (charstrings_offset, _enc, _charset) = parse_top_dict_with_charstrings(&dict);
        assert_eq!(charstrings_offset, 1234, "Top DICT op 17 → CharStrings offset");
    }

    /// Read the count field of a CFF INDEX header. nGlyphs is the count of
    /// the CharStrings INDEX.
    #[test]
    fn read_index_count_returns_header_count() {
        // INDEX header: 2-byte count, 1-byte offSize, then offsets+data.
        // For this helper we only need the first 2 bytes.
        let data = [0x01u8, 0x2C]; // count = 300 (0x012C)
        assert_eq!(read_index_count(&data, 0), Some(300));
    }

    /// A bare CFF whose Top DICT declares `/FontMatrix [1/2048 0 0 1/2048 0 0]`.
    ///
    /// Built here rather than embedded as a fixture. Layout: header
    /// `01 00 04 01`, Name INDEX ("Test"), Top DICT INDEX carrying six real
    /// operands then operator `12 7`, then an integer and operator `17`
    /// (CharStrings), empty String and GlobalSubr INDEXes, and a CharStrings
    /// INDEX with two trivial glyphs.
    ///
    /// `1E 0A 00 04 88 28 12 FF` is the real-number encoding of 0.00048828125
    /// — nibbles `. 0 0 0 4 8 8 2 8 1 2` terminated by `F`.
    const CFF_2048: &[u8] = &[
        0x01, 0x00, 0x04, 0x01, 0x00, 0x01, 0x01, 0x01, 0x05, 0x54, 0x65, 0x73, 0x74, 0x00, 0x01,
        0x01, 0x01, 0x21, 0x1E, 0x0A, 0x00, 0x04, 0x88, 0x28, 0x12, 0xFF, 0x1E, 0x0A, 0xFF, 0x1E,
        0x0A, 0xFF, 0x1E, 0x0A, 0x00, 0x04, 0x88, 0x28, 0x12, 0xFF, 0x1E, 0x0A, 0xFF, 0x1E, 0x0A,
        0xFF, 0x0C, 0x07, 0xC1, 0x11, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x01, 0x01, 0x06, 0x0B,
        0x1C, 0x04, 0x00, 0x8B, 0x0E, 0x1C, 0x04, 0x00, 0x8B, 0x0E,
    ];

    /// The same font with the `/FontMatrix` operator removed, so the CFF
    /// default of 1/1000 applies and nothing should be folded in.
    fn cff_without_font_matrix() -> Vec<u8> {
        let mut v = CFF_2048.to_vec();
        // Strip the six real operands and the `0C 07` operator from the Top
        // DICT, leaving only the CharStrings offset — then fix the INDEX
        // offset byte that follows the count/offSize header.
        let start = 18; // first byte of the Top DICT data
        let end = 48; // one past the `0C 07`
        v.drain(start..end);
        v[17] = 0x21 - (end - start) as u8;
        v
    }

    /// A subsetter targeting a 2048-unit em declares it, and the glyph-space
    /// scale must come from the font rather than from the 1000-unit
    /// assumption in ISO 32000-1 §9.2.2 ("For **most** types of fonts").
    #[test]
    fn test_font_matrix_of_one_over_2048_is_read() {
        let sx = parse_cff_font_matrix_scale(CFF_2048)
            .expect("the Top DICT declares a FontMatrix, so a scale must come back");
        assert!(
            (1.0 / sx - 2048.0).abs() < 1.0,
            "expected a 2048-unit em, got 1/{sx} = {}",
            1.0 / sx
        );
    }

    /// No `/FontMatrix` means the CFF default, and the caller must keep its own
    /// 1000-unit assumption rather than being handed a value to fold in.
    #[test]
    fn test_absent_font_matrix_yields_nothing_to_fold_in() {
        assert_eq!(
            parse_cff_font_matrix_scale(&cff_without_font_matrix()),
            None,
            "a CFF with no FontMatrix must not override the default em size"
        );
    }

    /// The real-number decoder handles the forms Adobe TN #5176 §4 defines:
    /// a decimal point, a leading minus, and an exponent.
    #[test]
    fn dict_reals_decode() {
        // 0.001 -> nibbles . 0 0 1, terminated
        assert_eq!(parse_dict_real(&[0x1E, 0x0A, 0x00, 0x1F], 0), Some((0.001, 4)));
        // -2.5 -> minus, 2, point, 5
        let (v, _) = parse_dict_real(&[0x1E, 0xE2, 0xA5, 0xFF], 0).expect("negative real");
        assert!((v + 2.5).abs() < 1e-9, "expected -2.5, got {v}");
    }

    /// A minimal **CID-keyed** CFF: the Top DICT carries ROS (operator `12 30`)
    /// and a format-0 charset mapping GID 1 to **CID 100**.
    ///
    /// Built here rather than embedded as a fixture. Layout: header, Name INDEX
    /// ("CIDT"), Top DICT INDEX with ROS / CIDCount / charset / CharStrings,
    /// empty String and GlobalSubr INDEXes, the charset, then a CharStrings
    /// INDEX of two glyphs.
    const CFF_CID_KEYED: &[u8] = &[
        0x01, 0x00, 0x04, 0x01, 0x00, 0x01, 0x01, 0x01, 0x05, 0x43, 0x49, 0x44, 0x54, 0x00, 0x01,
        0x01, 0x01, 0x13, 0x1C, 0x01, 0x87, 0x1C, 0x01, 0x88, 0x8B, 0x0C, 0x1E, 0x1C, 0x03, 0xE8,
        0x0C, 0x22, 0xB3, 0x0F, 0xB6, 0x11, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x64, 0x00, 0x02,
        0x01, 0x01, 0x05, 0x09, 0x1C, 0x03, 0xE8, 0x0E, 0x1C, 0x03, 0xE8, 0x0E,
    ];

    /// §9.7.4.2 (`docs/spec/pdf.md`:18641-18644): when the Top DICT uses
    /// CIDFont operators the CID must be resolved through the charset, and the
    /// NOTE at :18646 warns the CID and GID "may differ". Here CID 100 is
    /// glyph 1, so treating the CID as a GID would ask for glyph 100 of a
    /// two-glyph font and paint nothing.
    #[test]
    fn test_cid_keyed_charset_maps_cid_to_gid() {
        let map = parse_cff_cid_to_gid(CFF_CID_KEYED)
            .expect("the Top DICT carries ROS, so this font is CID-keyed");
        assert_eq!(map.get(&100), Some(&1), "CID 100 is GID 1 in this charset; got {map:?}");
    }

    /// A CFF with no ROS is not CID-keyed, and the same clause (:18649-18650)
    /// then says "the CIDs shall be used directly as GID values" — so the
    /// caller must get nothing to override that with.
    #[test]
    fn test_font_without_ros_yields_no_mapping() {
        // CFF_2048 is the FontMatrix fixture above: a plain, non-CID-keyed font.
        assert_eq!(
            parse_cff_cid_to_gid(CFF_2048),
            None,
            "a font whose Top DICT has no ROS operator must not produce a CID map"
        );
    }
}
