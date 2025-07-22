use std::{fs, io, path::Path};

use log::info;
use rustdoc_types::{Crate, Item, ItemEnum, Visibility};

/// Builds the Rustdoc JSON for the specified project
pub fn build_rustdoc<P>(path: P) -> eyre::Result<Crate>
where
    P: AsRef<Path>,
{
    let json_path = configure_rustdoc_builder_from_project_root(path)
        .build_with_captured_output(io::sink(), io::stderr())?;
    info!("Rustdoc build completed at {}", json_path.display());
    let raw_json = fs::read_to_string(json_path)?;
    let cooked_json = serde_json::from_str(&raw_json)?;
    Ok(cooked_json)
}

/// Constructs a [`rustdoc_json::Builder`] from a path to a project root
pub fn configure_rustdoc_builder_from_project_root<P>(
    path: P,
) -> rustdoc_json::Builder
where
    P: AsRef<Path>,
{
    let manifest_path = path.as_ref().join("Cargo.toml");
    rustdoc_json::Builder::default()
        .manifest_path(manifest_path)
        .document_private_items(true)
        .toolchain("nightly")
}

/// Return all functions under `root`
pub fn functions(root: &Crate) -> Vec<Item> {
    root.index
        .values()
        .filter(|&it| matches!(it.inner, ItemEnum::Function(_)))
        .cloned()
        .collect()
}

/// Return all functions under `root`
pub fn functions_with_visibility(
    root: &Crate,
    visibility: Visibility,
) -> Vec<Item> {
    root.index
        .values()
        .filter(|&x| x.visibility == visibility)
        .filter(|&it| matches!(it.inner, ItemEnum::Function(_)))
        .cloned()
        .collect()
}
