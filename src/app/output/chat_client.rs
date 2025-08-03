use crate::domain::chat::ChatMessage;

// TODO: I find the usage of the chat_id here a bit weird, since that's usually the chat id in the repository
// and not necessarily a valid identifier for the chat app/service...

/// A client for interacting with a chat service or app such as telegram
#[trait_variant::make(Send)]
#[dynosaur::dynosaur(pub DynChatClient = dyn(box) ChatClient)]
pub trait ChatClient: Send + Sync {
    /// Send a message
    async fn send(&self, chat_id: i64, message: &ChatMessage) -> Result<(), anyhow::Error>;
    /// Send a typing action
    ///
    /// This is typically used to show a "user is typing" message. You can use it
    /// to give the user some feedback during reasoning
    async fn typing(&self, chat_id: i64) -> Result<(), anyhow::Error>;
}
