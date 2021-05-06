pub mod accent;
pub mod furigana;
pub mod inflection;

use itertools::Itertools;

pub trait JapaneseExt {
    /// Returns true if self is of type ct
    fn is_of_type(&self, ct: CharType) -> bool;

    /// Get the CharType of a character
    fn get_text_type(&self) -> CharType;

    /// Returns true if self contains at least one kana character
    fn has_kana(&self) -> bool;

    /// Returns true if self is entirely written in kana
    fn is_kana(&self) -> bool;

    /// Returns true if inp is entirely written with kanji
    fn is_kanji(&self) -> bool;

    /// Returns true if inp has at least one kanji
    fn has_kanji(&self) -> bool;

    /// Returns true if inp is build with kanji and kana only
    fn is_japanese(&self) -> bool;

    /// Returns true if inp contains japanese characters
    fn has_japanese(&self) -> bool;

    /// Returns true if self is written in katakana
    fn is_katakana(&self) -> bool;

    /// Returns true if self is written in hiragana
    fn is_hiragana(&self) -> bool;

    /// Returns the amount of kanji self has
    fn kanji_count(&self) -> usize;

    /// Returns true if self is a (cjk) symbol
    fn is_symbol(&self) -> bool;

    /// Returns true if self is a (cjk) symbol
    fn has_symbol(&self) -> bool;

    fn to_hiragana(&self) -> String;

    fn is_roman_letter(&self) -> bool;

    /// Returns true if self is a small katakana letter
    fn is_small_katakana(&self) -> bool;

    /// Returns true if self is a small hiragana letter
    fn is_small_hiragana(&self) -> bool;

    /// Returns true if self is a small hiragana letter
    fn is_small_kana(&self) -> bool;
}

impl JapaneseExt for char {
    fn is_katakana(&self) -> bool {
        (*self) >= '\u{30A0}' && (*self) <= '\u{30FF}'
    }

    fn is_hiragana(&self) -> bool {
        (*self) >= '\u{3040}' && (*self) <= '\u{309F}'
    }

    fn is_kana(&self) -> bool {
        self.is_hiragana() || self.is_katakana()
    }

    fn to_hiragana(&self) -> String {
        romaji::RomajiExt::to_hiragana(self.to_string().as_str())
    }

    fn is_roman_letter(&self) -> bool {
        (*self) >= '\u{FF01}' && (*self) <= '\u{FF5A}'
            || ((*self) >= '\u{2000}' && (*self) <= '\u{206F}')
            || ((*self) >= '\u{20000}' && (*self) <= '\u{2A6DF}')
            || (*self) == '\u{2010}'
            || (*self) == '\u{2212}'
    }

    fn is_kanji(&self) -> bool {
        ((*self) >= '\u{3400}' && (*self) <= '\u{4DBF}')
            || ((*self) >= '\u{4E00}' && (*self) <= '\u{9FFF}')
            || ((*self) >= '\u{F900}' && (*self) <= '\u{FAFF}')
            || ((*self) >= '\u{FF10}' && (*self) <= '\u{FF19}')
            || ((*self) >= '\u{20000}' && (*self) <= '\u{2A6DF}')
            || (*self) == '\u{29E8A}'
    }

    fn is_symbol(&self) -> bool {
        ((*self) >= '\u{3000}' && (*self) <= '\u{303F}')
            || ((*self) >= '\u{0370}' && (*self) <= '\u{03FF}')
            || ((*self) >= '\u{25A0}' && (*self) <= '\u{25FF}')
            || ((*self) >= '\u{FF00}' && (*self) <= '\u{FFEF}')
            || (*self) == '\u{002D}'
            || (*self) == '\u{3005}'
            || (*self) == '\u{00D7}'
    }

    fn has_symbol(&self) -> bool {
        self.is_symbol()
    }

    fn has_kana(&self) -> bool {
        return self.is_kana();
    }

    fn has_kanji(&self) -> bool {
        self.is_kanji()
    }

    fn is_of_type(&self, ct: CharType) -> bool {
        self.get_text_type() == ct
    }

    fn get_text_type(&self) -> CharType {
        if self.is_kana() {
            CharType::Kana
        } else if self.is_kanji() || self.is_roman_letter() {
            CharType::Kanji
        } else {
            CharType::Other
        }
    }

    fn is_japanese(&self) -> bool {
        self.is_kana() || self.is_kanji() || self.is_symbol() || self.is_roman_letter()
    }

    fn has_japanese(&self) -> bool {
        self.is_japanese()
    }

    fn kanji_count(&self) -> usize {
        if self.is_kanji() {
            1
        } else {
            0
        }
    }

    fn is_small_hiragana(&self) -> bool {
        *self == '\u{3083}' || *self == '\u{3085}' || *self == '\u{3087}'
    }

    fn is_small_katakana(&self) -> bool {
        *self == '\u{30E3}' || *self == '\u{30E5}' || *self == '\u{30E7}'
    }

    fn is_small_kana(&self) -> bool {
        self.is_small_katakana() || self.is_small_hiragana()
    }
}

impl JapaneseExt for str {
    fn is_of_type(&self, ct: CharType) -> bool {
        self.get_text_type() == ct
    }

    fn get_text_type(&self) -> CharType {
        if self.is_kanji() {
            CharType::Kanji
        } else if self.is_kana() {
            CharType::Kana
        } else {
            CharType::Other
        }
    }

    fn is_hiragana(&self) -> bool {
        !self.chars().into_iter().any(|s| !s.is_hiragana())
    }

    fn is_katakana(&self) -> bool {
        !self.chars().into_iter().any(|s| !s.is_katakana())
    }

    fn is_roman_letter(&self) -> bool {
        !self.chars().into_iter().any(|s| !s.is_roman_letter())
    }

    fn has_kana(&self) -> bool {
        self.chars().into_iter().any(|s| s.is_kana())
    }

    fn has_symbol(&self) -> bool {
        self.chars().into_iter().any(|s| s.is_symbol())
    }

    fn is_kana(&self) -> bool {
        !self.chars().into_iter().any(|s| !s.is_kana())
    }

    fn is_kanji(&self) -> bool {
        !self.chars().into_iter().any(|s| !s.is_kanji())
    }

    fn has_kanji(&self) -> bool {
        self.chars().into_iter().any(|s| s.is_kanji())
    }

    fn is_japanese(&self) -> bool {
        let mut buf = [0; 16];
        !self.chars().into_iter().any(|c| {
            let s = c.encode_utf8(&mut buf);
            !s.is_kana() && !s.is_kanji() && !s.is_symbol() && !s.is_roman_letter()
        })
    }

    fn has_japanese(&self) -> bool {
        let mut buf = [0; 16];
        self.chars().into_iter().any(|c| {
            let s = c.encode_utf8(&mut buf);
            s.is_kana() || s.is_kanji() || s.is_symbol() || s.is_roman_letter()
        })
    }

    fn kanji_count(&self) -> usize {
        self.chars().into_iter().filter(|i| i.is_kanji()).count()
    }

    fn is_symbol(&self) -> bool {
        !self.chars().into_iter().any(|s| !s.is_symbol())
    }

    fn to_hiragana(&self) -> String {
        romaji::RomajiExt::to_hiragana(self)
    }

    fn is_small_katakana(&self) -> bool {
        !self.chars().into_iter().any(|s| !s.is_small_katakana())
    }

    fn is_small_hiragana(&self) -> bool {
        !self.chars().into_iter().any(|s| !s.is_small_hiragana())
    }

    fn is_small_kana(&self) -> bool {
        self.is_small_katakana() || self.is_small_hiragana()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CharType {
    Kana,
    Kanji,
    Other,
}

/// Return all words of chartype ct
pub fn all_words_with_ct(inp: &str, ct: CharType) -> Vec<String> {
    let mut all: Vec<String> = Vec::new();
    let mut curr = String::new();
    let mut iter = inp.chars().into_iter();
    while let Some(c) = iter.next() {
        if c.is_of_type(ct) {
            curr.push(c);
            continue;
        } else {
            if !curr.is_empty() {
                all.push(curr.clone());
            }
            curr.clear();
            iter.take_while_ref(|i| !i.is_of_type(ct)).count();
        }
    }
    if !curr.is_empty() {
        all.push(curr.clone());
    }
    all
}