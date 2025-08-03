use crate::domain::chat::ChatHistory;

#[trait_variant::make(Send)]
#[dynosaur::dynosaur(pub DynChatTrimmingService = dyn(box) ChatTrimmingService)]
pub trait ChatTrimmingService: Send + Sync {
    async fn trim(&self, chat_history: ChatHistory) -> ChatHistory;
}
