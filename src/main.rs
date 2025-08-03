mod app;
mod domain;
mod infra;

use std::sync::Arc;

use app::{
    input::chat::{ChatUseCase, ChatUseCaseImpl, DynChatUseCase},
    output::{
        chat_client::DynChatClient, llm_client::DynLlmClient, repo::DynChatRepository,
        trimming::DynChatTrimmingService,
    },
};
use domain::{chat::ChatMessage, prompts::SystemPrompt, user::User};
use infra::output::{
    mem_repo::InMemoryChatRepository, openai_llm_client::OpenaiLlmClient,
    reset_trimming::ChatResettingService, telegram::client::TelegramChatClient,
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
    let bot = Bot::new(env.teloxide_token);
    let chat_client = DynChatClient::new_arc(TelegramChatClient::new(bot.clone()));
    let chat_repo = DynChatRepository::new_arc(InMemoryChatRepository::new());
    let trimming_svc = DynChatTrimmingService::new_arc(ChatResettingService::new());
    let chat_use_case = DynChatUseCase::new_arc(ChatUseCaseImpl::new(
        chat_repo.clone(),
        llm.clone(),
        chat_client.clone(),
        trimming_svc.clone(),
    ));

    let schema = Update::filter_message().endpoint(answer);

    tracing::info!(event = "startup", "Starting bot...");
    // TODO: need an error handler to log errs?
    Dispatcher::builder(bot, schema)
        .dependencies(dptree::deps![allowed_users, chat_use_case])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}

fn display_user(user: &teloxide::types::User) -> String {
    format!("{} ({})", user.full_name(), user.id.0)
}

#[tracing::instrument(skip_all, fields(user = msg.from.as_ref().map(display_user)))]
async fn answer(
    bot: Bot,
    msg: Message,
    allowed_users: Arc<Vec<String>>,
    chat_use_case: Arc<DynChatUseCase<'static>>,
) -> Result<(), anyhow::Error> {
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

    let text = msg
        .text()
        .map(|it| it.to_owned())
        .ok_or(anyhow::anyhow!("no text receiver"))?;

    tracing::info!(event = "received-msg", content = text);

    chat_use_case
        .reply(msg.chat.id.0, user, ChatMessage::new_from_user(text))
        .await?;

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
