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
    let mut text = links
        .into_iter()
        .map(|(entry, flavor)| {
            format!(
                "{} docs for {}:\n{}",
                flavor,
                html::code_inline(&entry.topic),
                flavor.format_url(&entry),
            )
        })
        .join("\n\n");
    if let Some(user) = user {
        text += format!("\n\n[summoned by {}]", html::user_mention_or_link(user)).as_str();
    }
    text
}

pub fn format_inline_answer(entry: Entry, flavor: Flavor) -> String {
    format!(
        "Help for {} in {} docs:\n{}",
        html::code_inline(&entry.topic),
        flavor,
        flavor.format_url(&entry)
    )
}
