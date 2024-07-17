use std::env;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

use cargo_metadata::MetadataCommand;

const TARGETS: &[(&str, &str)] = &[
    ("aarch64-apple-darwin", "macos-arm64"),
    ("aarch64-unknown-linux-gnu", "linux-arm64"),
    ("arm-unknown-linux-gnueabi", "linux-arm"),
    ("i686-unknown-linux-gnu", "linux-x86"),
    ("x86_64-apple-darwin", "macos-x64"),
    ("x86_64-unknown-linux-gnu", "linux-x64"),
];

const fn platform_for_target(target: &str) -> &str {
    let mut i = 0;
    while i < TARGETS.len() {
        if const_str::equal!(TARGETS[i].0, target) {
            return TARGETS[i].1;
        }
        i += 1;
    }
    target
}

fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let build_target = env::var_os("TARGET").unwrap();
    let metadata = MetadataCommand::new().exec().unwrap();
    let meta = metadata
        .root_package()
        .unwrap()
        .metadata
        .as_object()
        .unwrap();
    write_tree_sitter_consts(meta, build_target, &out_dir);
    write_tsp_consts(meta, out_dir);
}

fn write_tsp_consts(
    meta: &serde_json::Map<String, serde_json::Value>,
    out_dir: std::ffi::OsString,
) {
    let root = PathBuf::from(file!());
    let tsp_bin_build_dir = root.parent().unwrap().join("src").canonicalize().unwrap();
    let tsp = meta.get("tsp").unwrap();
    let tsp_build_dir = tsp.get("build-dir").unwrap().as_str().unwrap();
    let tsp_fresh = tsp.get("fresh").unwrap().as_bool().unwrap();
    let tsp_from = tsp.get("from").unwrap().as_str().unwrap();
    let tsp_out = tsp.get("out").unwrap().as_str().unwrap();
    let tsp_ref = tsp.get("ref").unwrap().as_str().unwrap();
    let tsp_show_config = tsp.get("show-config").unwrap().as_bool().unwrap();
    let tsp_static = tsp.get("static").unwrap().as_bool().unwrap();
    let tsp_consts = Path::new(&out_dir).join("tsp_consts.rs");
    fs::write(
        tsp_consts,
        format!(
            r#"
            pub const TSP_BIN_BUILD_DIR: &str = "{}/";

            pub const TSP_BUILD_DIR: &str = "{}";
            pub const TSP_FRESH: bool = {};
            pub const TSP_FROM: &str = "{}";
            pub const TSP_OUT: &str = "{}";
            pub const TSP_REF: &str = "{}";
            pub const TSP_SHOW_CONFIG: bool = {};
            pub const TSP_STATIC: bool = {};
            "#,
            tsp_bin_build_dir.to_str().unwrap(),
            tsp_build_dir,
            tsp_fresh,
            tsp_from,
            tsp_out,
            tsp_ref,
            tsp_show_config,
            tsp_static,
        ),
    )
    .unwrap();
}

fn write_tree_sitter_consts(
    meta: &serde_json::Map<String, serde_json::Value>,
    build_target: std::ffi::OsString,
    out_dir: &std::ffi::OsString,
) {
    let tree_sitter = meta.get("tree-sitter").unwrap();
    let tree_sitter_version = tree_sitter.get("version").unwrap().as_str().unwrap();
    let tree_sitter_repo = tree_sitter.get("repo").unwrap().as_str().unwrap();
    let tree_sitter_platform = platform_for_target(build_target.to_str().unwrap());
    let tree_sitter_consts = Path::new(out_dir).join("tree_sitter_consts.rs");
    fs::write(
        tree_sitter_consts,
        format!(
            r#"
            pub const TREE_SITTER_PLATFORM: &str = "{}";
            pub const TREE_SITTER_REPO: &str = "{}";
            pub const TREE_SITTER_VERSION: &str = "{}";
            "#,
            tree_sitter_platform, tree_sitter_repo, tree_sitter_version,
        ),
    )
    .unwrap();
}
