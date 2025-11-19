use base64::Engine;

use crate::window::MarkupLanguageMode;

include!(concat!(env!("OUT_DIR"), "/symbol_table.rs"));

/// Amount of available symbols
pub static SYMBOL_COUNT: usize = SYMBOL_TABLE.len();

// Original code from:
// https://github.com/FineFindus/detexify-rust/blob/311002feb0519f483ef1f9cc8206648286128ff5/src/symbol.rs

/// LateX Symbol
#[derive(Debug, Clone, PartialEq)]
pub struct Symbol {
    /// Command to display the symbol.
    command: &'static str,
    /// Package which the symbol belongs to.
    package: &'static str,
    /// Font encoding used for the symbol.
    font_encoding: &'static str,
    /// Whether the symbol is available in text mode.
    text_mode: bool,
    /// Whether the symbol is available in math mode.
    math_mode: bool,
    /// Equivalent typst command (if available).
    typst_command: Option<&'static str>,
}

impl Symbol {
    /// Returns the symbol that the `id` specifies.
    pub fn from_id(id: &str) -> Option<Self> {
        SYMBOL_TABLE.get(id).cloned()
    }

    /// Returns the `id` of the symbol.
    pub fn id(&self) -> &'static str {
        let id = format!(
            "{}-{}-{}",
            self.package,
            self.font_encoding,
            self.command.replace('\\', "_")
        );
        let key = base64::prelude::BASE64_STANDARD.encode(id);
        // SAFETY: safe to unwrap, since key must be valid, as it is only possible to get a Symbol
        // from the symbol table
        SYMBOL_TABLE.get_key(&key).unwrap()
    }

    /// Command to use the symbol in the specified markup language.
    pub fn command(&self, language: &MarkupLanguageMode) -> Option<&'static str> {
        match language {
            MarkupLanguageMode::Latex => Some(self.command),
            MarkupLanguageMode::Typst => self.typst_command,
        }
    }

    /// Package which provides the symbol for the specified markup language.
    pub fn package(&self, language: &MarkupLanguageMode) -> Option<&'static str> {
        match language {
            MarkupLanguageMode::Latex => Some(self.package),
            MarkupLanguageMode::Typst => None,
        }
    }

    /// Textual representation of the mode in which the symbol can be used.
    ///
    /// # Examples
    /// ```rust
    /// let symbol = Symbol::from_id("bGF0ZXgyZS1PVDEtX2JpZ2N1cA==").unwrap();
    /// assert_eq!("mathmode", symbol.mode_description(&MarkupLanguage::Latex));
    /// ```
    pub fn mode_description(&self, language: &MarkupLanguageMode) -> Option<&'static str> {
        if matches!(language, MarkupLanguageMode::Typst) {
            return None;
        }

        Some(match (self.math_mode, self.text_mode) {
            (true, true) => "mathmode & textmode",
            (false, true) => "textmode",
            (true, false) => "mathmode",
            (false, false) => {
                // a symbol has to be either math mode or textmode
                unreachable!("Symbol {} is neither math nor textmode", self.id())
            }
        })
    }
}

#[cfg(test)]
mod tests {

    use super::Symbol;
    use crate::classify::symbol::SYMBOL_TABLE;

    #[test]
    fn test_from_id() {
        let symbol = Symbol::from_id("bGF0ZXgyZS1PVDEtX3RleHRhc2NpaWNpcmN1bQ==");

        assert_eq!(
            symbol,
            Some(Symbol {
                command: "\\textasciicircum",
                package: "latex2e",
                font_encoding: "OT1",
                text_mode: true,
                math_mode: false,
                typst_command: Some("\\^"),
            })
        );
    }

    #[test]
    fn test_iterate_symbols() {
        assert_eq!(SYMBOL_TABLE.len(), 1124);
    }

    #[test]
    fn test_id_get_id() {
        for symbol in SYMBOL_TABLE.values() {
            assert_eq!(&Symbol::from_id(symbol.id()).unwrap(), symbol);
        }
    }
}
