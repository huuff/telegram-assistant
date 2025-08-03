use crate::domain::chat::ChatHistory;

#[trait_variant::make(Send)]
#[dynosaur::dynosaur(pub DynChatRepository = dyn(box) ChatRepository)]
pub trait ChatRepository: Send + Sync {
    async fn find(&self, id: i64) -> Result<Option<ChatHistory>, anyhow::Error>;

    async fn save(&self, id: i64, chat: ChatHistory) -> Result<(), anyhow::Error>;
}
