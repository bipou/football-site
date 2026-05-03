use leptos_i18n_build::{Config, TranslationsInfos};
use std::{error::Error, path::PathBuf};

fn main() -> Result<(), Box<dyn Error>> {
    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-changed=Cargo.toml");

    let out = PathBuf::from(std::env::var_os("OUT_DIR").unwrap()).join("i18n");

    let cfg = Config::new("en")?.add_locale("zh")?;
    let infos = TranslationsInfos::parse(cfg)?;
    infos.emit_diagnostics();
    infos.rerun_if_locales_changed();
    infos.generate_i18n_module(out)?;

    Ok(())
}
