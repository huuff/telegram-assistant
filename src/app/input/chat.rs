use std::sync::Arc;

use derive_more::Constructor;

use crate::{
    app::output::{
        chat_client::{ChatClient, DynChatClient},
        llm_client::DynLlmClient,
        repo::DynChatRepository,
        trimming::{ChatTrimmingService, DynChatTrimmingService},
    },
    domain::{
        chat::{ChatHistory, ChatMessage},
        user::User,
    },
};

#[trait_variant::make(Send)]
#[dynosaur::dynosaur(pub DynChatUseCase = dyn(box) ChatUseCase)]
pub trait ChatUseCase: Send + Sync {
    // MAYBE I think it'd be better if chat IDs were string, that would be portable to many different databases
    async fn reply(
        &self,
        chat_id: i64,
        user: User,
        message: ChatMessage,
    ) -> Result<(), anyhow::Error>;
}

#[derive(Constructor)]
pub struct ChatUseCaseImpl {
    repo: Arc<DynChatRepository<'static>>,
    llm: Arc<DynLlmClient<'static>>,
    chat_client: Arc<DynChatClient<'static>>,
    trimming_svc: Arc<DynChatTrimmingService<'static>>,
}

impl ChatUseCase for ChatUseCaseImpl {
    async fn reply(&self, chat_id: i64, user: User, msg: ChatMessage) -> Result<(), anyhow::Error> {
        use crate::app::{
            output::chat_client::ChatClient as _, output::llm_client::LlmClient as _,
            output::repo::ChatRepository as _,
        };
        use crate::domain::prompts::SystemPrompt;
        // TODO: Askama should be infra
        use askama::Template as _;

        let mut chat_history = match self.repo.find(chat_id).await? {
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

        chat_history.push_message(msg.clone());

        let llm_response = tokio::select! {
            msg = self.llm.get_response(&chat_history) => { msg }
            _ = keep_typing(self.chat_client.clone(), chat_id) => { unreachable!() }
        }?;

        if let Err(err) = self.chat_client.send(chat_id, &llm_response).await {
            // TODO: I don't like this if... is there a crate or pattern to log an error and propagate?
            // or maybe just propagate and log at the top like as for exceptions?
            tracing::error!(event = "sending-error", msg = llm_response.content, ?err);
            return Err(err);
        };

        chat_history.push_message(llm_response);

        if chat_history.is_too_long() {
            chat_history = self.trimming_svc.trim(chat_history).await;
        }

        self.repo.save(chat_id, chat_history).await?;

        Ok(())
    }
}

/// Sends a typing action forever
///
/// Useful to `tokio::select!` this with something else so typing will appear until that's finished
async fn keep_typing(
    chat_client: Arc<DynChatClient<'static>>,
    chat_id: i64,
) -> Result<(), anyhow::Error> {
    loop {
        chat_client.typing(chat_id).await?;
        // TODO: this is mostly adjusted for telegram's typing duration (5s), so this should be in the client implementation
        // so as not to couple this to telegram
        tokio::time::sleep(tokio::time::Duration::from_millis(4500)).await;
    }
}
