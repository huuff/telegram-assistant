mod app;
mod domain;
mod infra;

use std::sync::Arc;

use app::output::{
    llm_client::{DynLlmClient, LlmClient},
    repo::ChatRepository,
    trimming::ChatTrimmingService,
};
use askama::Template;
use domain::{prompts::SystemPrompt, user::User};
use infra::output::{
    mem_repo::InMemoryChatRepository, openai_llm_client::OpenaiLlmClient,
    reset_trimming::ChatResettingService,
};
use teloxide::prelude::*;

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

    let llm = DynLlmClient::new_arc(OpenaiLlmClient::new(
        "https://openrouter.ai/api/v1",
        &env.openrouter_token,
    ));
    let chat_repo: Arc<dyn ChatRepository> = Arc::new(InMemoryChatRepository::new());
    let trimming_svc: Arc<dyn ChatTrimmingService> = Arc::new(ChatResettingService::new());

    let bot = Bot::new(env.teloxide_token);

    let schema = Update::filter_message().endpoint(answer);

    tracing::info!(event = "startup", "Starting bot...");
    // TODO: need an error handler to log errs?
    Dispatcher::builder(bot, schema)
        .dependencies(dptree::deps![llm, chat_repo, allowed_users, trimming_svc])
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
    llm: Arc<DynLlmClient<'static>>,
    allowed_users: Arc<Vec<String>>,
    chat_repo: Arc<dyn ChatRepository>,
    trimming_svc: Arc<dyn ChatTrimmingService>,
) -> Result<(), anyhow::Error> {
    use crate::domain::chat::ChatHistory;

    let user = User::try_from(
        msg.from
            .clone()
            .ok_or(anyhow::anyhow!("message without an user"))?,
    )?;

    if !allowed_users.contains(&user.id) {
        bot.send_message(msg.chat.id, "FORBIDDEN").await?;
        tracing::info!(event = "unauthorized", "Rejected message");
        return Ok(());
    }

    let mut chat_history = match chat_repo.find(msg.chat.id.0)? {
        Some(chat_history) => chat_history,
        None => ChatHistory::new(
            // TODO: heresy! two clones! I think the system prompt could take references since it's only supposed to get rendered.
            user.clone(),
            SystemPrompt::builder()
                .user(user.clone())
                .build()
                .render()?,
        ),
    };

    if let Some(text) = msg.text() {
        tracing::info!(event = "received-msg", content = text);
        chat_history.push_user_message(text);
    }

    let llm_response = tokio::select! {
        msg = llm.get_response(&chat_history) => { msg }
        _ = keep_typing(bot.clone(), msg.chat.id) => { unreachable!() }
    }?;

    if let Err(err) = bot
        .send_message(msg.chat.id, &llm_response.content)
        .parse_mode(teloxide::types::ParseMode::Html)
        .await
    {
        tracing::error!(event = "sending-error", msg = llm_response.content, ?err);
        return Err(err.into());
    };

    tracing::info!(event = "sent-message", content = llm_response.content);

    chat_history.push_message(llm_response);

    if chat_history.is_too_long() {
        chat_history = trimming_svc.trim(chat_history);
        tracing::warn!(event = "trimmed-chat", "Chat got too long, trimmed it");
    }

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
