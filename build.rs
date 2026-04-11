fn main() {
    let parser_c = "grammars/groovy/parser.c";
    if std::path::Path::new(parser_c).exists() {
        cc::Build::new()
            .file(parser_c)
            .include("grammars/groovy")
            .flag_if_supported("-Wno-unused-parameter")
            .flag_if_supported("-Wno-unused-variable")
            .flag_if_supported("-Wno-trigraphs")
            .compile("tree-sitter-groovy");
        println!("cargo:rerun-if-changed={}", parser_c);
        println!("cargo:rustc-cfg=feature=\"groovy\"");
    } else {
        eprintln!("cargo:warning=grammars/groovy/parser.c not found — Groovy support disabled");
    }
}
