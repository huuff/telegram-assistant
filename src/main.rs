mod chat;

use std::sync::Arc;

use chat::{ChatHistory, ChatRepository, InMemoryChatRepository};
use teloxide::prelude::*;

const DEFAULT_SYSTEM_PROMPT: &str = "You're a helpful telegram bot. You don't reply with huge walls of text, but try to be concise and to the point. Use the MarkdownV2 telegram format for your messages.";

#[tokio::main]
async fn main() {
    if cfg!(debug_assertions) {
        dotenvy::dotenv().expect("no .env file");
    } else {
        let _ = dotenvy::dotenv();
    }

    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let env = envy::from_env::<Env>().expect("failed to deserialize env vars");

    let allowed_users = env
        .allowed_users()
        .into_iter()
        .map(|it| it.to_owned())
        .collect::<Vec<_>>();
    let allowed_users = Arc::new(allowed_users);

    let config = async_openai::config::OpenAIConfig::new()
        .with_api_key(env.openrouter_token)
        .with_api_base("https://openrouter.ai/api/v1");

    let client = async_openai::Client::with_config(config);
    let chat_repo: Arc<dyn ChatRepository> = Arc::new(InMemoryChatRepository::new());

    let bot = Bot::new(env.teloxide_token);

    let schema = Update::filter_message().endpoint(answer);

    tracing::info!(event = "startup", "Starting bot...");
    Dispatcher::builder(bot, schema)
        .dependencies(dptree::deps![client, chat_repo, allowed_users])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}

fn display_user(user: &teloxide::types::User) -> String {
    format!("{} ({})", user.full_name(), user.id.0)
}

/// Sends a typing action forever
///
/// Useful to `tokio::select!` this with something else so typing will appear until that's finished
async fn keep_typing(bot: Bot, chat_id: teloxide::types::ChatId) -> Result<(), anyhow::Error> {
    loop {
        bot.send_chat_action(chat_id, teloxide::types::ChatAction::Typing)
            .await?;
        tokio::time::sleep(tokio::time::Duration::from_millis(4500)).await;
    }
}

#[tracing::instrument(skip_all, fields(user = msg.from.as_ref().map(display_user)))]
async fn answer(
    bot: Bot,
    msg: Message,
    client: async_openai::Client<async_openai::config::OpenAIConfig>,
    allowed_users: Arc<Vec<String>>,
    chat_repo: Arc<dyn ChatRepository>,
) -> Result<(), anyhow::Error> {
    use async_openai::types::CreateChatCompletionRequestArgs;

    if !msg
        .from
        .as_ref()
        .is_some_and(|user| allowed_users.contains(&user.id.0.to_string()))
    {
        bot.send_message(msg.chat.id, "FORBIDDEN").await?;
        return Ok(());
    }

    let mut chat_history = match chat_repo.find(msg.chat.id.0)? {
        Some(chat_history) => chat_history,
        None => ChatHistory::new(DEFAULT_SYSTEM_PROMPT),
    };

    if let Some(text) = msg.text() {
        tracing::info!(event = "received-msg", content = text);
        chat_history.push_user_message(text);
    }

    let request = CreateChatCompletionRequestArgs::default()
        .model("google/gemini-2.5-pro")
        .messages(chat_history.clone().into_openai())
        .build()?;

    let chat = client.chat();
    let response = tokio::select! {
        msg = chat.create(request) => {msg}
        _ = keep_typing(bot.clone(), msg.chat.id) => {unreachable!()}
    }?;

    let returned_message = response
        .choices
        .first()
        .unwrap()
        .message
        .content
        .clone()
        .unwrap();

    if let Err(err) = bot
        .send_message(msg.chat.id, &returned_message)
        .parse_mode(teloxide::types::ParseMode::MarkdownV2)
        .await
    {
        tracing::error!(event = "sending-error", msg = returned_message, ?err);
        return Err(err.into());
    };

    tracing::info!(event = "sent-message", content = returned_message);

    chat_history.push_assistant_message(returned_message);

    chat_repo.save(msg.chat.id.0, chat_history.clone())?;

    Ok(())
}

#[derive(serde::Deserialize)]
struct Env {
    teloxide_token: String,
    openrouter_token: String,
    allowed_users: Option<String>,
}

impl Env {
    pub fn allowed_users(&self) -> Vec<&str> {
        self.allowed_users
            .as_ref()
            .map(|string| string.split(",").collect())
            .unwrap_or_default()
    }
}
