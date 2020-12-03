use itertools::Itertools;
use once_cell::sync::Lazy;
use percent_encoding::{percent_encode, NON_ALPHANUMERIC};
use regex::Regex;
use teloxide::utils::html;

use crate::{tagsdb::Entry, tagsearch::Flavor};

macro_rules! help_regex_s {
    () => {
        r":h(?:e(?:l(?:p)?)?)?\s+([^\s]+)"
    };
}

pub static HELP_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(help_regex_s!()).expect("failed to compile regex"));

#[allow(dead_code)]
pub static DELETE_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(concat!(r"^[\s\n]*(?:", help_regex_s!(), r"[\s\n]*)+$"))
        .expect("failed to compile regex")
});

static VIM_URL_BASE: &str = "https://vimhelp.org";
static NEOVIM_URL_BASE: &str = "https://neovim.io/doc/user";

fn format_vim_url(entry: &Entry) -> String {
    format!(
        "{}/{}.txt.html#{}",
        VIM_URL_BASE,
        entry.filename,
        percent_encode(entry.topic.as_bytes(), NON_ALPHANUMERIC)
    )
}

fn format_neovim_url(entry: &Entry) -> String {
    format!(
        "{}/{}.html#{}",
        NEOVIM_URL_BASE,
        entry.filename,
        percent_encode(entry.topic.as_bytes(), NON_ALPHANUMERIC)
    )
}

pub fn format_message(links: impl IntoIterator<Item = (Entry, Flavor)>) -> String {
    links
        .into_iter()
        .map(|(entry, flavor)| {
            let url = match flavor {
                Flavor::Vim => format_vim_url(&entry),
                Flavor::NeoVim => format_neovim_url(&entry),
            };
            format!(
                "Found help for {} in {:?} docs:\n{}",
                html::code_inline(&entry.topic),
                flavor,
                url
            )
        })
        .join("\n\n")
}
