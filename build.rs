use std::env;
use std::fs;
use std::path::Path;

fn main() {
    let lib_dir = env::var("DEP_DUCKLINGFFI_LIB_DIR")
        .expect("DEP_DUCKLINGFFI_LIB_DIR not set — is duckling_rust a dependency?");

    // Create a temporary directory for fixed libraries
    let out_dir = env::var("OUT_DIR").unwrap();
    let fixed_lib_dir = Path::new(&out_dir).join("fixed_libs");
    if !fixed_lib_dir.exists() {
        fs::create_dir_all(&fixed_lib_dir).expect("Failed to create fixed_libs directory");
    }

    let src_path = Path::new(&lib_dir);
    if src_path.exists() {
        for entry in fs::read_dir(src_path).expect("Failed to read lib_dir") {
            let entry = entry.expect("Failed to read entry");
            let path = entry.path();
            if path.is_file() {
                let name = path.file_name().unwrap().to_string_lossy();
                if name.contains(".so") {
                    let dest_path = fixed_lib_dir.join(path.file_name().unwrap());
                    fs::copy(&path, &dest_path).expect("Failed to copy library");
                    
                    // Clear executable stack if it's a Haskell library or the main library
                    if name.contains("libHS") || name.contains("libducklingffi") {
                        let _ = std::process::Command::new("patchelf")
                            .arg("--clear-execstack")
                            .arg(&dest_path)
                            .output();
                    }
                }
            }
        }
    }

    println!("cargo:rustc-link-search=native={}", fixed_lib_dir.display());
    println!("cargo:rustc-link-arg=-Wl,-rpath,{}", fixed_lib_dir.display());
    
    // Also copy to python/py_duckling_native/libs for local development and preload
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let py_libs_dir = Path::new(&manifest_dir)
        .join("python")
        .join("py_duckling_native")
        .join("libs");

    if !py_libs_dir.exists() {
        fs::create_dir_all(&py_libs_dir).expect("Failed to create py_libs directory");
    }

    // Copy all fixed libs to py_libs_dir
    for entry in fs::read_dir(&fixed_lib_dir).expect("Failed to read fixed_lib_dir") {
        let entry = entry.expect("Failed to read entry");
        let path = entry.path();
        let dest_path = py_libs_dir.join(path.file_name().unwrap());
        fs::copy(&path, &dest_path).expect("Failed to copy to py_libs");
    }
    
    // Rerun if the library directory changes
    println!("cargo:rerun-if-changed={}", lib_dir);
}