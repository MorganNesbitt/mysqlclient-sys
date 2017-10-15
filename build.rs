extern crate pkg_config;
extern crate bindgen;

use bindgen::Builder;
use std::env;
use std::path::Path;
use std::process::Command;

fn main() {
    pkg_config::Config::new().statik(true).probe("mysqlclient").unwrap();
    println!("cargo:rerun-if-env-changed=MYSQLCLIENT_LIB_DIR");
    println!("cargo:rerun-if-env-changed=MYSQLCLIENT_LIB_STATIC");
    
    if let Ok(lib_dir) = env::var("MYSQLCLIENT_LIB_DIR") {
        println!("cargo:rustc-link-search=native={}", lib_dir);
    }
    if env::var_os("MYSQLCLIENT_LIB_STATIC").is_some() {
        println!("cargo:rustc-link-lib=static=mysqlclient");
    }
}

fn mysql_config_variable(var_name: &str) -> Option<String> {
    Command::new("mysql_config")
        .arg(format!("--variable={}", var_name))
        .output()
        .into_iter()
        .filter(|output| output.status.success())
        .flat_map(|output| String::from_utf8(output.stdout).ok())
        .map(|output| output.trim().to_string())
        .next()
}

fn generate_bindgen_file() {
    let out_dir = env::var("OUT_DIR").unwrap();
    Builder::default()
        .no_unstable_rust()
        .header("wrapper.h")
        .clang_arg(format!("-I{}", mysql_include_dir()))
        .whitelisted_function("mysql.*")
        .whitelisted_type("MYSQL.*")
        .whitelisted_var("MYSQL.*")
        .generate()
        .expect("Unable to generate bindings for libmysqlclient")
        .write_to_file(Path::new(&out_dir).join("bindings.rs"))
        .expect("Unable to write bindings to file");
}

fn mysql_include_dir() -> String {
    env::var("MYSQLCLIENT_INCLUDE_DIR").ok()
        .or_else(|| pkg_config::get_variable("mysqlclient", "includedir").ok())
        .or_else(|| mysql_config_variable("pkgincludedir"))
        .expect("Unable to locate `mysql.h`")
}
