mod tagsdb;
mod tagsearch;
mod utils;

use std::env;
use tagsearch::TagSearch;
use teloxide::{prelude::*, types::ParseMode};

async fn handle_message(message: UpdateWithCx<Message>, tagsearch: TagSearch) {
    if let Some(text) = message.update.text() {
        if let Some(answer) = tagsearch.find_by_text(text) {
            let send_message = message.answer(answer).parse_mode(ParseMode::HTML);
            if let Some(reply) = message.update.reply_to_message() {
                let sent_ans = send_message.reply_to_message_id(reply.id);
                if let Ok(_) = sent_ans.send().await {
                    message.delete_message().send().await.ok();
                }
            } else {
                send_message
                    .reply_to_message_id(message.update.id)
                    .send()
                    .await
                    .ok();
            }
        }
    }
}

async fn run() -> i32 {
    dotenv::dotenv().ok();
    teloxide::enable_logging!();

    let tagsearch = TagSearch::from_env();
    let bot = Bot::from_env();
    log::info!("Starting vim-help bot...");

    teloxide::repl(bot, move |message| {
        let tagsearch = tagsearch.clone();
        async move {
            handle_message(message, tagsearch.clone()).await;
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
