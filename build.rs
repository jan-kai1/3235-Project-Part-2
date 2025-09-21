use std::env;
use std::path::PathBuf;

fn main() {
    let _out_dir = env::var("OUT_DIR").unwrap();
    let workspace_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    
    let database_enhanced_c: PathBuf = PathBuf::from(&workspace_dir).join("database_enhanced.c");
    
    println!("cargo:rerun-if-changed={}", database_enhanced_c.display());
        
    cc::Build::new()
        .file(&database_enhanced_c)
        .flag("-fno-stack-protector")
        .flag("-fno-delete-null-pointer-checks")
        .flag("-O0")
        .flag("-g")
        .flag("-DDEBUG")
        .compile("database_enhanced");
    
    
    println!("cargo:rustc-link-lib=static=database_enhanced");
}
