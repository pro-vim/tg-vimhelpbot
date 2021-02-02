mod tagsdb;
mod tagsearch;
mod utils;

use tagsearch::TagSearcher;
use teloxide::{
    prelude::*,
    types::{
        InlineQuery, InlineQueryResult, InlineQueryResultArticle, InputMessageContent,
        InputMessageContentText, ParseMode,
    },
};
use utils::{format_inline_answer, format_message, DELETE_REGEX};

async fn handle_message(message: UpdateWithCx<Message>, tagsearcher: TagSearcher) {
    let text = message.update.text();
    let should_delete = text
        .map(|text| DELETE_REGEX.is_match(text))
        .unwrap_or(false);

    let from = if should_delete {
        message.update.from()
    } else {
        None
    };

    let bot_reply = text
        .map(|text| tagsearcher.find_entries_by_text(text))
        .map(|results| format_message(results, from))
        .map(|answer| message.answer(answer).parse_mode(ParseMode::HTML));

    if let Some(bot_reply) = bot_reply {
        let replied = if let Some(reply) = message.update.reply_to_message() {
            bot_reply.reply_to_message_id(reply.id)
        } else if should_delete {
            bot_reply
        } else {
            bot_reply.reply_to_message_id(message.update.id)
        }
        .send()
        .await
        .is_ok();

        if replied && should_delete {
            message.delete_message().send().await.ok();
        }
    }
}

async fn handle_inline_query(query: UpdateWithCx<InlineQuery>, tagsearcher: TagSearcher) {
    let options: Vec<_> = if query.update.query.is_empty() {
        vec![]
    } else {
        tagsearcher
            .search_by_topic(&query.update.query)
            .map(|(entry, flavor)| {
                let text = format_inline_answer(entry.clone(), flavor);
                let content = InputMessageContentText::new(text).parse_mode(ParseMode::HTML);
                InlineQueryResultArticle::new(
                    uuid::Uuid::new_v4().to_string(),
                    format!("`{}` in {} docs", entry.topic, flavor),
                    InputMessageContent::Text(content),
                )
            })
            .map(InlineQueryResult::Article)
            .collect()
    };

    if let Err(err) = query
        .bot
        .answer_inline_query(query.update.id, options)
        .send()
        .await
    {
        log::error!("failed to answer inline query: {}", err)
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    teloxide::enable_logging!();

    let tagsearch = TagSearcher::from_env()
        .map_err(|flavor| anyhow::anyhow!("Failed to load {} database", flavor))?;
    let tagsearch2 = tagsearch.clone();
    let bot = Bot::from_env();
    log::info!("Starting vim-help bot...");

    Dispatcher::new(bot)
        .messages_handler(move |rx: DispatcherHandlerRx<Message>| {
            rx.for_each_concurrent(None, move |message| {
                handle_message(message, tagsearch.clone())
            })
        })
        .inline_queries_handler(move |rx: DispatcherHandlerRx<InlineQuery>| {
            rx.for_each_concurrent(None, move |query: UpdateWithCx<InlineQuery>| {
                handle_inline_query(query, tagsearch2.clone())
            })
        })
        .dispatch()
        .await;

    Ok(())
}
