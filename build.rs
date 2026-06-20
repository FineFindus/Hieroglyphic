use base64::Engine;
use mitex::CommandSpecItem;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;
use yaml_rust2::{Yaml, YamlLoader};

#[derive(Debug, Clone)]
struct Symbol {
    command: String,
    package: String,
    font_encoding: String,
    // these variables are used in the program, but not in the build process
    #[allow(dead_code)]
    text_mode: bool,
    #[allow(dead_code)]
    math_mode: bool,
    typst_command: Option<String>,
}

impl Symbol {
    fn id(&self) -> String {
        let id = format!(
            "{}-{}-{}",
            self.package,
            self.font_encoding,
            self.command.replace('\\', "_")
        );
        base64::prelude::BASE64_STANDARD.encode(id)
    }
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=symbols.yaml");
    println!("cargo:rerun-if-changed=typst-aliases.yaml");

    let out_dir = std::env::var("OUT_DIR").unwrap();
    let path = Path::new(&out_dir).join("symbol_table.rs");
    let mut file = BufWriter::new(File::create(path).unwrap());

    //TODO: replace this with a simple list?
    let mut map = phf_codegen::Map::new();
    let mut docs = YamlLoader::load_from_str(include_str!("symbols.yaml")).unwrap();
    let doc = docs.pop().unwrap();

    // 'default' values for a symbol, i.e. it has not special table
    let symbol = Symbol {
        command: "".to_string(),
        package: "latex2e".to_string(),
        //TODO: remove? it is not displayed in the UI
        font_encoding: "OT1".to_string(),
        text_mode: true,
        math_mode: false,
        typst_command: None,
    };
    let typst_symbols = get_typst_aliases();
    for data in doc.into_vec().unwrap() {
        generate_symbol(data, symbol.clone(), &mut map, &typst_symbols);
    }

    writeln!(
        &mut file,
        "static SYMBOL_TABLE: phf::Map<&'static str, Symbol> = {};",
        map.build()
    )
    .unwrap();
}

fn get_typst_aliases() -> HashMap<String, String> {
    // load typst to latex mappings from the Typst website (generated in scripts/load_typst_aliases.py)
    let mut docs = YamlLoader::load_from_str(include_str!("typst-aliases.yaml")).unwrap();
    let doc = docs.pop().unwrap();
    let mut commands: HashMap<_, _> = collect_yaml_hash_map(doc);

    // load typst to latex mappings from mitex
    for (latex_cmd, typst_cmd) in mitex_spec_gen::DEFAULT_SPEC.clone().items() {
        if let CommandSpecItem::Cmd(cmd) = typst_cmd {
            let latex_command = format!("\\{latex_cmd}");
            let typst_command = cmd.alias.clone().unwrap_or_else(|| latex_cmd.to_string());

            commands.entry(latex_command).or_insert(typst_command);
        };
    }

    // load typst to latex mappings from Typst's codex crate
    let mut docs = YamlLoader::load_from_str(include_str!("unicode_to_latex.yaml")).unwrap();
    let doc = docs.pop().unwrap();
    let unicode_to_latex = collect_yaml_hash_map(doc);

    let mut typst_to_unicode = HashMap::new();
    collect_codex_symbols(&codex::SYM, &mut typst_to_unicode);

    for (typst_cmd, unicode_symbol) in typst_to_unicode {
        let unicode_symbol_hex = format!("U{:05X}", unicode_symbol as u32);
        if let Some(latex_cmd) = unicode_to_latex.get(&unicode_symbol_hex) {
            commands.entry(latex_cmd.clone()).or_insert(typst_cmd);
        }
    }

    commands
}

fn collect_yaml_hash_map(doc: Yaml) -> HashMap<String, String> {
    doc.as_hash()
        .unwrap()
        .iter()
        .map(|(key, value)| {
            (
                key.as_str().unwrap().to_string(),
                value.as_str().unwrap().to_string(),
            )
        })
        .collect()
}

fn collect_codex_symbols(module: &codex::Module, target_map: &mut HashMap<String, char>) {
    for (_, binding) in module.iter() {
        match binding.def {
            codex::Def::Symbol(symbol) => {
                for (modifiers, value, _) in symbol.variants() {
                    let typst_command = modifiers.as_str();
                    // we only care about the first symbol - for emojis, there sometimes is an additional
                    // emoji variation selector like "\u{fe0f}"
                    let unicode_symbol = value.chars().next().unwrap();
                    target_map.insert(typst_command.to_string(), unicode_symbol);
                }
            }
            codex::Def::Module(module) => {
                collect_codex_symbols(&module, target_map);
            }
        }
    }
}

fn get_equiv_typst_command(
    latex_command: &str,
    typst_symbols: &HashMap<String, String>,
) -> Option<String> {
    let re = regex::Regex::new(r"\\math(cal|frak|scr|ds)\{(\w)\}").unwrap();

    // e.g. \mathfrak{B} -> math.frak(B)
    if let Some((_, [mode, symbol])) = re.captures(latex_command).map(|c| c.extract()) {
        let typst_mode = match mode {
            "ds" => "bb",
            _ => mode,
        };
        return Some(format!("math.{typst_mode}({symbol})"));
    }

    typst_symbols.get(latex_command).cloned()
}

fn generate_symbol(
    data: Yaml,
    mut symbol: Symbol,
    map: &mut phf_codegen::Map<String>,
    typst_symbols: &HashMap<String, String>,
) {
    if let Some(cmd) = data.as_str() {
        cmd.clone_into(&mut symbol.command);
        if let Some(typst_command) = get_equiv_typst_command(cmd, typst_symbols) {
            symbol.typst_command = Some(typst_command);
        }

        assert!(
            !symbol.command.is_empty(),
            "Symbol does not have an associated command"
        );
        map.entry(symbol.id(), format!("{symbol:?}"));
        return;
    }

    // the data should only contain strings and hashes
    assert!(matches!(data, Yaml::Hash(_)));
    let table = data.as_hash().unwrap();

    if let Some(package) = table.get(&Yaml::String("package".to_owned())) {
        symbol.package = package.as_str().unwrap().to_string();
    }

    if let Some(fontenc) = table.get(&Yaml::String("fontenc".to_owned())) {
        symbol.font_encoding = fontenc.as_str().unwrap().to_string();
    }

    for (mode, text_mode, math_mode) in [
        ("bothmodes", true, true),
        ("textmode", true, false),
        ("mathmode", false, true),
    ] {
        symbol.text_mode = text_mode;
        symbol.math_mode = math_mode;
        if let Some(values) = table
            .get(&Yaml::String(mode.to_owned()))
            .cloned()
            .and_then(|v| v.into_vec())
        {
            for mode_data in values {
                generate_symbol(mode_data, symbol.clone(), map, typst_symbols);
            }
        }
    }
}
