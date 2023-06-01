// extern crate cbindgen;

// use std::env;

// fn main() {
//     let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

//     cbindgen::Builder::new()
//       .with_crate(crate_dir)
//       .generate()
//       .expect("Unable to generate bindings")
//       .write_to_file("bindings.h");
// }

fn main() {
  built::write_built_file().expect("Failed to acquire build-time information");
}
