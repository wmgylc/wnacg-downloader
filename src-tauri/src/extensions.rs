use anyhow::anyhow;
use parking_lot::RwLock;
use scraper::error::SelectorErrorKind;
use tauri::{Manager, State};

use crate::config::Config;

pub trait AnyhowErrorToStringChain {
    fn to_string_chain(&self) -> String;
}

impl AnyhowErrorToStringChain for anyhow::Error {
    fn to_string_chain(&self) -> String {
        use std::fmt::Write;

        self.chain()
            .enumerate()
            .fold(String::new(), |mut output, (i, err)| {
                let _ = writeln!(output, "{i}: {err}");
                output
            })
    }
}

pub trait ToAnyhow<T> {
    fn to_anyhow(self) -> anyhow::Result<T>;
}

impl<T> ToAnyhow<T> for Result<T, SelectorErrorKind<'_>> {
    fn to_anyhow(self) -> anyhow::Result<T> {
        self.map_err(|err| anyhow!(err.to_string()))
    }
}

pub trait AppHandleExt {
    fn get_config(&self) -> State<'_, RwLock<Config>>;
}

impl AppHandleExt for tauri::AppHandle {
    fn get_config(&self) -> State<'_, RwLock<Config>> {
        self.state::<RwLock<Config>>()
    }
}
