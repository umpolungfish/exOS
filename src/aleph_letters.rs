//! ℵ-OS λ_ℵ type system — 22 Hebrew letter encodings.
//!
//! Index:  D  T  R  P  F  K  G Ga Ph  H  S  Om   Glyph
pub const LETTERS: [LetterDef; 22] = [
    LetterDef { name: "aleph",  glyph: 'א', t: [0,3,0,3,2,2,2,0,1,3,0,2] },
    LetterDef { name: "bet",    glyph: 'ב', t: [1,3,1,2,1,1,1,0,0,1,1,1] },
    LetterDef { name: "gimel",  glyph: 'ג', t: [0,2,3,0,0,0,0,2,0,0,0,0] },
    LetterDef { name: "dalet",  glyph: 'ד', t: [0,1,3,0,0,0,0,2,0,0,0,0] },
    LetterDef { name: "hei",    glyph: 'ה', t: [3,4,2,3,2,2,2,3,1,3,2,2] },
    LetterDef { name: "vav",    glyph: 'ו', t: [0,0,3,4,0,2,1,0,1,1,0,0] },
    LetterDef { name: "zayin",  glyph: 'ז', t: [0,0,3,0,0,0,0,2,0,0,0,0] },
    LetterDef { name: "chet",   glyph: 'ח', t: [1,3,1,2,1,1,1,0,0,1,1,1] },
    LetterDef { name: "tet",    glyph: 'ט', t: [1,1,3,0,0,2,1,2,0,1,0,0] },
    LetterDef { name: "yod",    glyph: 'י', t: [0,3,0,3,2,2,2,0,0,1,0,0] },
    LetterDef { name: "kaf",    glyph: 'כ', t: [1,3,1,2,1,1,1,0,0,1,1,1] },
    LetterDef { name: "lamed",  glyph: 'ל', t: [2,0,3,0,0,1,0,2,1,2,2,0] },
    LetterDef { name: "mem",    glyph: 'מ', t: [1,1,2,4,2,2,2,3,1,2,1,2] },
    LetterDef { name: "nun",    glyph: 'נ', t: [0,0,3,0,0,0,0,2,0,0,0,0] },
    LetterDef { name: "samech", glyph: 'ס', t: [1,3,1,3,1,1,1,0,0,1,1,1] },
    LetterDef { name: "ayin",   glyph: 'ע', t: [3,4,2,2,2,2,2,3,1,2,2,2] },
    LetterDef { name: "pei",    glyph: 'פ', t: [0,0,3,0,0,0,0,3,0,1,2,0] },
    LetterDef { name: "tzadi",  glyph: 'צ', t: [0,1,3,0,0,0,0,2,0,0,0,0] },
    LetterDef { name: "kuf",    glyph: 'ק', t: [1,3,1,3,1,2,1,0,1,2,1,1] },
    LetterDef { name: "resh",   glyph: 'ר', t: [0,3,3,0,0,1,0,0,0,1,0,0] },
    LetterDef { name: "shin",   glyph: 'ש', t: [1,2,2,4,2,2,2,3,1,3,1,2] },
    LetterDef { name: "tav",    glyph: 'ת', t: [1,3,1,3,1,2,1,0,1,3,1,2] },
];
