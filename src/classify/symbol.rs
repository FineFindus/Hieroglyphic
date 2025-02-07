use base64::Engine;

include!(concat!(env!("OUT_DIR"), "/symbol_table.rs"));

/// Amount of available symbols
pub static SYMBOL_COUNT: usize = SYMBOL_TABLE.len();

// Original code from:
// https://github.com/FineFindus/detexify-rust/blob/311002feb0519f483ef1f9cc8206648286128ff5/src/symbol.rs

/// LateX Symbol
#[derive(Debug, Clone, PartialEq)]
pub struct Symbol {
    /// Command to display the symbol.
    pub command: &'static str,
    /// Package which the symbol belongs to.
    pub package: &'static str,
    /// Font encoding used for the symbol.
    font_encoding: &'static str,
    /// Whether the symbol is available in text mode.
    pub text_mode: bool,
    /// Whether the symbol is available in math mode.
    pub math_mode: bool,
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
                math_mode: false
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
