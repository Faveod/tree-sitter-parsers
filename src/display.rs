use std::{borrow::Cow, collections::BTreeMap};

use miette::{miette, Context, IntoDiagnostic, Result};

#[derive(Debug, Clone)]
pub struct Screen {
    bars: BTreeMap<String, Address>,
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

impl Drop for Screen {
    fn drop(&mut self) {
        for (_, handle) in &self.bars {
            handle.bar.finish();
        }
    }
}

impl Screen {
    pub fn new() -> Self {
        Screen::default()
    }

    pub fn register<S>(&mut self, name: S, num_tasks: usize) -> Address
    where
        S: Into<String>,
    {
        let style =
            indicatif::ProgressStyle::with_template("{prefix:.bold.dim} {wide_msg}").unwrap();
        let bar = self
            .multi
            .add(indicatif::ProgressBar::new(num_tasks as u64));
        bar.set_prefix(format!("[?/{}]", num_tasks));
        bar.set_style(style);
        let handle = Address { bar, num_tasks };
        self.bars.insert(name.into(), handle.clone());
        handle
    }

    pub fn get(&self, language: &str) -> Result<Address> {
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
pub struct Address {
    bar: indicatif::ProgressBar,
    num_tasks: usize,
}

impl Address {
    pub fn send<S>(&self, delta: Option<u64>, msg: Option<S>)
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
