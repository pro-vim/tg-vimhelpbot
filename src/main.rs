// lint me harder
#![forbid(non_ascii_idents)]
#![forbid(unsafe_code)]
#![deny(
    future_incompatible,
    keyword_idents,
    elided_lifetimes_in_paths,
    meta_variable_misuse,
    noop_method_call,
    unused_lifetimes,
    unused_qualifications,
    clippy::wildcard_dependencies,
    clippy::debug_assert_with_mut_call,
    clippy::empty_line_after_outer_attr,
    clippy::panic,
    clippy::unwrap_used,
    clippy::redundant_field_names,
    clippy::rest_pat_in_fully_bound_structs,
    clippy::unneeded_field_pattern,
    clippy::useless_let_if_seq
)]
#![warn(clippy::pedantic)]
// not that hard
// that one is just dumb, `.map_or()` is less readable in most cases
#![allow(clippy::map_unwrap_or)]

use std::sync::Arc;

use color_eyre::eyre::{self, eyre};
use teloxide::{
    prelude::*,
    types::{
        InlineQuery, InlineQueryResult, InlineQueryResultArticle, InputMessageContent,
        InputMessageContentText, Me, ParseMode,
    },
};

mod tagsdb;
mod tagsearch;
mod utils;

use crate::{
    tagsearch::{Flavor, TagSearcher},
    utils::{format_inline_answer, format_message, DELETE_REGEX, NEOVIM_REGEX, THANKS_REGEX},
};

async fn handle_message(
    me: Arc<Me>,
    bot: Bot,
    message: Message,
    tagsearcher: Arc<TagSearcher>,
) -> eyre::Result<()> {
    let text = message.text();
    let should_delete = text
        .map(|text| DELETE_REGEX.is_match(text))
        .unwrap_or(false);
    let from = message.from().filter(|_| should_delete);

    if let Some(text) = text {
        let neovim = message
            .chat
            .title()
            .map(|title| NEOVIM_REGEX.is_match(title))
            .unwrap_or(false);
        let preferred_flavor = if neovim { Flavor::NeoVim } else { Flavor::Vim };

        let search_results: Vec<_> = tagsearcher.search_by_text(text, preferred_flavor).collect();

        if search_results.is_empty() {
            if THANKS_REGEX.is_match(text) {
                if let Some(replied_to) = message.reply_to_message() {
                    if let Some(user) = replied_to.from() {
                        if me.user.id == user.id {
                            bot.send_message(message.chat.id, "You're welcome :)")
                                .reply_to_message_id(message.id)
                                .send()
                                .await?;
                        }
                    }
                }
            }
            return Ok(());
        }

        let reply_text = format_message(search_results, from);
        let bot_reply = bot
            .send_message(message.chat.id, reply_text)
            .parse_mode(ParseMode::Html);

        if let Some(reply) = message.reply_to_message() {
            bot_reply.reply_to_message_id(reply.id)
        } else if should_delete {
            bot_reply
        } else {
            bot_reply.reply_to_message_id(message.id)
        }
        .send()
        .await?;

        if should_delete {
            bot.delete_message(message.chat.id, message.id)
                .send()
                .await
                .ok();
        }
    }

    Ok(())
}

async fn handle_inline_query(
    bot: Bot,
    query: InlineQuery,
    tagsearcher: Arc<TagSearcher>,
) -> eyre::Result<()> {
    let options: Vec<_> = if query.query.is_empty() {
        vec![]
    } else {
        tagsearcher
            .search_by_topic(&query.query)
            .map(|(entry, flavor)| {
                let text = format_inline_answer(&entry, flavor);
                let topic = entry.topic;
                let content = InputMessageContentText::new(text).parse_mode(ParseMode::Html);
                InlineQueryResultArticle::new(
                    uuid::Uuid::new_v4().to_string(),
                    format!("`{topic}` in {flavor} docs"),
                    InputMessageContent::Text(content),
                )
            })
            .map(InlineQueryResult::Article)
            .collect()
    };

    bot.answer_inline_query(query.id, options).send().await?;
    Ok(())
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    color_eyre::install()?;
    tracing_subscriber::fmt().init();
    dotenvy::dotenv().ok();

    let bot = Bot::from_env();
    tracing::info!("Starting vim-help bot...");

    let handler = dptree::entry()
        .branch(Update::filter_message().endpoint(handle_message))
        .branch(Update::filter_inline_query().endpoint(handle_inline_query));

    let me = Arc::new(bot.get_me().send().await?);
    let tagsearch = Arc::new(
        TagSearcher::from_env().map_err(|flavor| eyre!("Failed to load {} database", flavor))?,
    );

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![me, tagsearch])
        .build()
        .dispatch()
        .await;

    Ok(())
}
