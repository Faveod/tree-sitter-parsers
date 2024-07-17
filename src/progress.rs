use std::{borrow::Cow, collections::BTreeMap};

use miette::{miette, Context, IntoDiagnostic, Result};
use tracing_subscriber::fmt::format;

#[derive(Debug, Clone)]
pub struct Screen {
    bars: BTreeMap<String, Handle>,
    multi: indicatif::MultiProgress,
}

impl Default for Screen {
    fn default() -> Self {
        Screen {
            bars: BTreeMap::new(),
            multi: indicatif::MultiProgress::new(),
        }
    }
}

impl Screen {
    pub fn new() -> Self {
        Screen::default()
    }

    pub fn register<S>(&mut self, lang: S, num_tasks: usize) -> Handle
    where
        S: Into<String>,
    {
        let style =
            indicatif::ProgressStyle::with_template("{prefix:.bold.dim} {spinner} {wide_msg}")
                .unwrap()
                .progress_chars("  ");
        let mut bar = indicatif::ProgressBar::new(num_tasks as u64);
        bar.set_prefix(format!("[?/{}]", num_tasks));
        bar.set_style(style);
        bar = self.multi.add(bar);
        let handle = Handle { bar, num_tasks };
        self.bars.insert(lang.into(), handle.clone());
        handle
    }

    pub fn get(&self, language: &str) -> Result<Handle> {
        self.bars
            .get(language)
            .ok_or(miette!(format!("Fetching progress bar for {}", language)))
            .cloned()
    }

    pub fn clear(&self) -> Result<()> {
        self.multi
            .clear()
            .into_diagnostic()
            .context("Clearing the multi-progress bar")
    }
}

#[derive(Debug, Clone)]
pub struct Handle {
    bar: indicatif::ProgressBar,
    num_tasks: usize,
}

impl Handle {
    pub fn tick<S>(&self, delta: Option<u64>, msg: Option<S>)
    where
        S: Into<Cow<'static, str>>,
    {
        if let Some(d) = delta {
            self.bar.inc(d);
            self.bar
                .set_prefix(format!("[{}/{}]", self.bar.position(), self.num_tasks));
        }
        if let Some(m) = msg {
            self.bar.set_message(m);
        }
    }
}
