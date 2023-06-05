//use anyhow::*;
use fs_extra::copy_items;
use fs_extra::dir::CopyOptions;
use std::env;

fn main() -> anyhow::Result<()> {
    // This tells cargo to rerun this script if something in /res/ changes.
    println!("cargo:rerun-if-changed=www/images/*");
    println!("cargo:rerun-if-changed=www/media/*");

    let out_dir = env::var("OUT_DIR")?;
    let mut copy_options = CopyOptions::new();
    copy_options.overwrite = true;
    let paths_to_copy = vec!["www/images/", "www/media/"];
    copy_items(&paths_to_copy, out_dir, &copy_options)?;

    Ok(())
}
