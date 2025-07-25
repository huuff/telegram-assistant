use crate::domain::chat::{ChatHistory, ChatMessage};

#[trait_variant::make(Send)]
#[dynosaur::dynosaur(pub DynLlmClient = dyn(box) LlmClient)]
pub trait LlmClient: Send + Sync {
    async fn get_response(&self, history: &ChatHistory) -> Result<ChatMessage, anyhow::Error>;
}
