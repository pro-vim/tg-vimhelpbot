use itertools::Itertools;
use once_cell::sync::Lazy;
use regex::Regex;
use teloxide::{types::User, utils::html};

use crate::{tagsdb::Entry, tagsearch::Flavor};

macro_rules! help_regex_s {
    () => {
        r":h(?:e(?:l(?:p)?)?)?\s+([^\s]+)"
    };
}

pub static HELP_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(help_regex_s!()).expect("failed to compile regex"));

pub static DELETE_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(concat!(r"^[\s\n]*(?:", help_regex_s!(), r"[\s\n]*)+$"))
        .expect("failed to compile regex")
});

pub fn format_message(
    links: impl IntoIterator<Item = (Entry, Flavor)>,
    user: Option<&User>,
) -> String {
    links
        .into_iter()
        .enumerate()
        .map(|(index, (entry, flavor))| {
            let prefix = match index {
                0 => match user {
                    Some(user) => format!("{} found help", html::user_mention_or_link(user)),
                    None => "Found help".to_string(),
                },
                _ => "... and".to_string(),
            };
            format!(
                "{} for {} in {} docs:\n{}",
                prefix,
                html::code_inline(&entry.topic),
                flavor,
                flavor.format_url(&entry),
            )
        })
        .join("\n\n")
}

pub fn format_inline_answer(entry: Entry, flavor: Flavor) -> String {
    format!(
        "Help for {} in {} docs:\n{}",
        html::code_inline(&entry.topic),
        flavor,
        flavor.format_url(&entry)
    )
}
