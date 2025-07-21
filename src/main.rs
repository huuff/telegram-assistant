mod chat;

use std::sync::Arc;

use chat::{ChatHistory, ChatRepository, InMemoryChatRepository};
use teloxide::prelude::*;

const DEFAULT_SYSTEM_PROMPT: &str = "You're a helpful telegram bot. You don't reply with huge walls of text, but try to be concise and to the point";

#[tokio::main]
async fn main() {
    println!("Starting echo bot...");

    if cfg!(debug_assertions) {
        dotenvy::dotenv().expect("no .env file");
    } else {
        let _ = dotenvy::dotenv();
    }

    let env = envy::from_env::<Env>().expect("failed to deserialize env vars");

    let config = async_openai::config::OpenAIConfig::new()
        .with_api_key(env.openrouter_token)
        .with_api_base("https://openrouter.ai/api/v1");

    let client = async_openai::Client::with_config(config);
    let chat_repo: Arc<dyn ChatRepository> = Arc::new(InMemoryChatRepository::new());

    let bot = Bot::new(env.teloxide_token);

    let schema = Update::filter_message().endpoint(answer);

    Dispatcher::builder(bot, schema)
        .dependencies(dptree::deps![client, chat_repo])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}

async fn answer(
    bot: Bot,
    msg: Message,
    client: async_openai::Client<async_openai::config::OpenAIConfig>,
    chat_repo: Arc<dyn ChatRepository>,
) -> Result<(), anyhow::Error> {
    use async_openai::types::CreateChatCompletionRequestArgs;

    let mut chat_history = match chat_repo.find(msg.chat.id.0)? {
        Some(chat_history) => chat_history,
        None => ChatHistory::new(DEFAULT_SYSTEM_PROMPT),
    };

    if let Some(text) = msg.text() {
        chat_history.push_user_message(text);
    }

    let request = CreateChatCompletionRequestArgs::default()
        .model("google/gemini-2.5-pro")
        .messages(chat_history.clone().into_openai())
        .build()?;

    bot.send_chat_action(msg.chat.id, teloxide::types::ChatAction::Typing)
        .await?;
    let chat_completion = client.chat().create(request).await?;

    let returned_message = chat_completion
        .choices
        .first()
        .unwrap()
        .message
        .content
        .clone()
        .unwrap();
    bot.send_message(msg.chat.id, &returned_message)
        .send()
        .await?;

    chat_history.push_assistant_message(returned_message);

    chat_repo.save(msg.chat.id.0, chat_history.clone())?;

    Ok(())
}

#[derive(serde::Deserialize)]
struct Env {
    teloxide_token: String,
    openrouter_token: String,
}
