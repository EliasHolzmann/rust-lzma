extern crate pkg_config;

use std::process::Command;

use pkg_config::find_library;

fn main() {
    println!("cargo:rerun-if-env-changed=LZMA_VENDORED");

    dbg!(std::env::var_os("LZMA_VENDORED"));
    dbg!(std::env::var_os("PATH"));
    if std::env::var_os("LZMA_VENDORED").is_none() {
        if find_library("liblzma").is_ok() {
            return;
        } else {
            panic!("Could not find liblzma using pkg-config")
        }
    } else {
        let compiler_tool = dbg!(cc::Build::new().get_compiler());

        let out_dir = std::env::var_os("OUT_DIR").map(|s| std::path::PathBuf::from(s).join("lzma-vendored")).expect("OUT_DIR is not set");
        let target = std::env::var("TARGET").ok().expect("TARGET is not set");
        let host = std::env::var("HOST").ok().expect("HOST is not set");

        let source_dir = out_dir.join("source");
        let build_dir = out_dir.join("build");
        let install_dir = out_dir.join("install");

        if source_dir.exists() {
            std::fs::remove_dir_all(&source_dir).unwrap();
        }
        if build_dir.exists() {
            std::fs::remove_dir_all(&build_dir).unwrap();
        }
        if install_dir.exists() {
            std::fs::remove_dir_all(&install_dir).unwrap();
        }

        std::fs::create_dir_all(&source_dir).unwrap();

        let decompress_result = Command::new("tar")
            .current_dir(&std::path::Path::new(
                &std::env::var_os("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set"),
            ))
            .arg("-xf")
            .arg("xz-5.2.5.tar.xz")
            .arg("-C")
            .arg(&source_dir)
            .spawn()
            .expect("Spawn failed")
            .wait()
            .expect("Wait failed");
        assert!(decompress_result.success());

        let configure_result = Command::new("sh")
            .current_dir(
                source_dir.clone().join("xz-5.2.5"),
            )
            .env("CC", dbg!(compiler_tool.path()))
            .env_remove("CROSS_COMPILE")
            .arg("./configure")
            .arg("--disable-xz")
            .arg("--disable-xzdec")
            .arg("--disable-lzmadec")
            .arg("--disable-lzmainfo")
            .arg("--disable-scripts")
            .arg("--disable-lzma-links")
            .spawn()
            .expect("Spawn failed")
            .wait()
            .expect("Wait failed");
        assert!(configure_result.success());

        let make_result = Command::new("make")
            .current_dir(
                source_dir.join("xz-5.2.5"),
            )
            .arg(format!("CFLAGS={}", dbg!(compiler_tool.cflags_env().into_string().unwrap())))
            .spawn()
            .expect("Spawn failed")
            .wait()
            .expect("Wait failed");
        assert!(make_result.success());
    }
}
