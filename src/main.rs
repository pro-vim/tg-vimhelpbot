mod tagsdb;
use tagsdb::{Entry, TagsDb};

use itertools::Itertools;
use once_cell::sync::Lazy;
use percent_encoding::{percent_encode, NON_ALPHANUMERIC};
use regex::Regex;
use std::{env, sync::Arc};
use teloxide::{prelude::*, types::ParseMode, utils::html};

macro_rules! help_regex_s {
    () => {
        r":he?l?p?\s+([^\s]+)"
    };
}

static HELP_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(help_regex_s!()).expect("regex compiles"));

static DELETE_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(concat!(r"^[\s\n]*(?:", help_regex_s!(), r"[\s\n]*)+$")).expect("regex compiles")
});

static VIM_URL_BASE: &str = "https://vimhelp.org";
static NEOVIM_URL_BASE: &str = "https://neovim.io/doc/user";

#[derive(Clone, Copy, Debug)]
enum Flavor {
    Vim,
    NeoVim,
}

use Flavor::*;

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

fn format_message(links: impl IntoIterator<Item = (Entry, Flavor)>) -> String {
    links
        .into_iter()
        .map(|(entry, flavor)| {
            let url = match flavor {
                Vim => format_vim_url(&entry),
                NeoVim => format_neovim_url(&entry),
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

async fn run() -> i32 {
    dotenv::dotenv().ok();
    teloxide::enable_logging!();

    let vim_db = env::var("VIM_DB_PATH")
        .ok()
        .and_then(|db_path| (TagsDb::read_file(db_path).ok().map(Arc::new)));
    let neovim_db = env::var("NEOVIM_DB_PATH")
        .ok()
        .and_then(|db_path| (TagsDb::read_file(db_path).ok().map(Arc::new)));

    let (vim_db, neovim_db) = if let (Some(v), Some(n)) = (vim_db, neovim_db) {
        (v, n)
    } else {
        log::error!("Either Vim or NeoVim DB was not found");
        log::error!("Vim DB path: {:?}", env::var("VIM_DB_PATH"));
        log::error!("NeoVim DB path: {:?}", env::var("NEO_VIM_DB_PATH"));
        return 1;
    };

    log::info!("Starting vim-help bot...");
    let bot = Bot::from_env();
    teloxide::repl(bot, move |message| {
        let vim_db = vim_db.clone();
        let neovim_db = neovim_db.clone();
        async move {
            if let Some(text) = message.update.text() {
                let links: Vec<_> = HELP_REGEX
                    .captures_iter(text)
                    .filter_map(|capture| {
                        let topic = &capture[1];
                        if let Some(entry) = vim_db.find(topic) {
                            Some((entry, Vim))
                        } else if let Some(entry) = neovim_db.find(topic) {
                            Some((entry, NeoVim))
                        } else {
                            None
                        }
                    })
                    .collect();
                if !links.is_empty() {
                    message
                        .answer(format_message(links))
                        .parse_mode(ParseMode::HTML)
                        .send()
                        .await?;
                    if DELETE_REGEX.is_match(text) {
                        message
                            .delete_message()
                            .send()
                            .await
                            // Ignoring errors if we're not an admin
                            .ok();
                    }
                }
            }
            ResponseResult::<()>::Ok(())
        }
    })
    .await;

    0
}

#[tokio::main]
async fn main() {
    std::process::exit(run().await)
}
