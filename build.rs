// build.rs

use std::env;
use std::fs::create_dir;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let src_dir = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let out_dir_str = out_dir.to_str().unwrap();

    if !cfg!(target_os = "linux") {
        panic!("Cannot use libbpf-sys on !linux");
    }

    if cfg!(feature = "novendor") {
        let libbpf = pkg_config::Config::new()
            .atleast_version(&format!(
                "{}.{}.{}",
                env!("CARGO_PKG_VERSION_MAJOR"),
                env!("CARGO_PKG_VERSION_MINOR"),
                env!("CARGO_PKG_VERSION_PATCH")
            ))
            .probe("libbpf")
            .unwrap();

        cc::Build::new()
            .file("bindings.c")
            .includes(&libbpf.include_paths)
            .define("__LIBBPF_SYS_NOVENDOR", None)
            .out_dir(out_dir_str)
            .compile("bindings");
    } else {
        let obj_dir = out_dir.join("obj").to_str().unwrap();
        // We want to ignore an AlreadyExists error, and make will probably scream about
        // any other sort of error, so it’s easiest to just ignore all errors here.
        let _ = create_dir(obj_dir);
        let status = Command::new("make")
            .arg("install")
            .env("BUILD_STATIC_ONLY", "y")
            .env("PREFIX", "/")
            .env("LIBDIR", "")
            .env("OBJDIR", obj_dir)
            .env("DESTDIR", out_dir_str)
            .env("CFLAGS", "-g -O2 -Werror -Wall -fPIC")
            .current_dir(src_dir.join("libbpf/src"))
            .status()
            .unwrap();

        assert!(status.success());

        let status = Command::new("make")
            .arg("clean")
            .current_dir(src_dir.join("libbpf/src"))
            .status()
            .unwrap();

        assert!(status.success());

        cc::Build::new()
            .file("bindings.c")
            .include(src_dir.join("libbpf/include"))
            .include(src_dir.join("libbpf/include/uapi"))
            .out_dir(out_dir_str)
            .compile("bindings");

        println!("cargo:rustc-link-search=native={}", out_dir_str);
        println!("cargo:rustc-link-lib=elf");
        println!("cargo:rustc-link-lib=z");
        println!("cargo:rustc-link-lib=static=bpf");
        println!("cargo:include={}/include", out_dir_str);
    }
}
