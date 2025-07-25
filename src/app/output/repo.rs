use crate::domain::chat::ChatHistory;

// TODO should be async and use dynosaur
pub trait ChatRepository: Send + Sync {
    fn find(&self, id: i64) -> Result<Option<ChatHistory>, anyhow::Error>;

    fn save(&self, id: i64, chat: ChatHistory) -> Result<(), anyhow::Error>;
}
