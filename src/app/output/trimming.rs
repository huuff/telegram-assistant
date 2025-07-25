use crate::domain::chat::ChatHistory;

pub trait ChatTrimmingService: Send + Sync {
    fn trim(&self, chat_history: ChatHistory) -> ChatHistory;
}
