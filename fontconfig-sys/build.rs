use std::{env, path::PathBuf};

fn main() {
    println!("cargo:rerun-if-env-changed=RUST_FONTCONFIG_DLOPEN");
    println!("cargo:rerun-if-env-changed=FONTCONFIG_DEP_BACKEND");
    println!("cargo:rerun-if-env-changed=TECTONIC_DEP_BACKEND");
    println!("cargo:rustc-check-cfg=cfg(fontconfig_static)");

    let dlopen = std::env::var_os("RUST_FONTCONFIG_DLOPEN").is_some();
    if dlopen {
        println!("cargo:rustc-cfg=feature=\"dlopen\"");
    }
    if !(dlopen || cfg!(feature = "dlopen")) {
        match dep_backend().as_deref() {
            Some("vcpkg") => {
                let library = vcpkg::Config::new()
                    .find_package("fontconfig")
                    .unwrap_or_else(|e| panic!("failed to load fontconfig from vcpkg: {e}"));
                if library.is_static {
                    println!("cargo:rustc-cfg=fontconfig_static");
                }
                emit_include_metadata(library.include_paths);
            }
            Some("pkg-config") | Some("default") | None => {
                let library = pkg_config::Config::new()
                    .probe("fontconfig")
                    .unwrap_or_else(|e| panic!("failed to load fontconfig with pkg-config: {e}"));
                if env::var_os("FONTCONFIG_STATIC").is_some()
                    || env::var_os("PKG_CONFIG_ALL_STATIC").is_some()
                {
                    println!("cargo:rustc-cfg=fontconfig_static");
                }
                emit_include_metadata(library.include_paths);
            }
            Some(other) => {
                panic!("unrecognized Fontconfig dependency backend {other:?}");
            }
        }
    }
}

fn dep_backend() -> Option<String> {
    env::var("FONTCONFIG_DEP_BACKEND")
        .ok()
        .or_else(|| env::var("TECTONIC_DEP_BACKEND").ok())
}

fn emit_include_metadata(paths: Vec<PathBuf>) {
    let include_path = paths
        .into_iter()
        .map(|path| path.to_string_lossy().into_owned())
        .collect::<Vec<_>>()
        .join(";");

    println!("cargo:include={include_path}");
    println!("cargo:include-path={include_path}");
}
