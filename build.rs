use base64::Engine;
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
    println!("cargo:rerun-if-changed=symbols.yaml");
    println!("cargo:rustc-link-search=native=/app/lib");

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
    };
    for data in doc.into_vec().unwrap() {
        generate_symbol(data, symbol.clone(), &mut map);
    }

    writeln!(
        &mut file,
        "static SYMBOL_TABLE: phf::Map<&'static str, Symbol> = {};",
        map.build()
    )
    .unwrap();
}

fn generate_symbol(data: Yaml, mut symbol: Symbol, map: &mut phf_codegen::Map<String>) {
    if let Some(cmd) = data.as_str() {
        cmd.clone_into(&mut symbol.command);
        assert!(
            !symbol.command.is_empty(),
            "Symbol does not have an associated command"
        );
        map.entry(symbol.id(), format!("{:?}", symbol));
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
                generate_symbol(mode_data, symbol.clone(), map);
            }
        }
    }
}
