// SPDX-License-Identifier: LGPL-2.1+

extern crate bindgen;

use std::env;
use std::path::PathBuf;

fn main() {
    // Tell cargo to tell rustc to link the system liblxc
    // shared library.
    println!("cargo:rustc-link-lib=lxc");

    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let bindings = bindgen::Builder::default()
        .whitelist_function("list_all_containers")
        .whitelist_function("lxc_container_new")
        .whitelist_function("lxc_container_put")
        .whitelist_function("lxc_get_version")
        .whitelist_function("lxc_get_global_config_item")
        .whitelist_function("lxc_log_init")
        .whitelist_type("lxc_container")
        .whitelist_type("lxc_log")
        .whitelist_var("LXC_ATTACH_TERMINAL")
        .whitelist_var("LXC_ATTACH_DEFAULT")
        // The input header we would like to generate
        // bindings for.
        .header("wrapper.h")
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
