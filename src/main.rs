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
use utils::{format_inline_answer, DELETE_REGEX};

async fn handle_message(message: UpdateWithCx<Message>, tagsearcher: TagSearcher) {
    let text = message.update.text();
    let bot_reply = text
        .map(|text| tagsearcher.find_by_text(text))
        .flatten()
        .map(|answer| message.answer(answer).parse_mode(ParseMode::HTML));
    let should_delete = text
        .map(|text| DELETE_REGEX.is_match(text))
        .unwrap_or(false);

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
                InlineQueryResultArticle::new(
                    uuid::Uuid::new_v4().to_string(),
                    format!("`{}` in {} docs", entry.topic, flavor),
                    InputMessageContent::Text(
                        InputMessageContentText::new(format_inline_answer(entry, flavor))
                            .parse_mode(ParseMode::HTML),
                    ),
                )
            })
            .map(InlineQueryResult::Article)
            .collect()
    };

    match query
        .bot
        .answer_inline_query(query.update.id, options)
        .send()
        .await
    {
        Ok(_) => (),
        Err(err) => log::error!("failed to answer inline query: {}", err),
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    teloxide::enable_logging!();

    let tagsearch = TagSearcher::from_env()
        .map_err(|flavor| anyhow::anyhow!("Failed to load {:?} database", flavor))?;
    let tagsearch2 = tagsearch.clone();
    let bot = Bot::from_env();
    log::info!("Starting vim-help bot...");

    Dispatcher::new(bot)
        .messages_handler(move |rx: DispatcherHandlerRx<Message>| async move {
            rx.for_each_concurrent(None, move |message| {
                handle_message(message, tagsearch.clone())
            })
            .await
        })
        .inline_queries_handler(move |rx: DispatcherHandlerRx<InlineQuery>| async move {
            rx.for_each_concurrent(None, |query: UpdateWithCx<InlineQuery>| {
                handle_inline_query(query, tagsearch2.clone())
            })
            .await
        })
        .dispatch()
        .await;

    Ok(())
}
