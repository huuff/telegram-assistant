use super::user::User;

nestify::nest! {
    #[derive(Debug, Clone)]*
    pub struct ChatHistory {
        pub user: User,
        pub system_prompt: String,
        pub messages: Vec<pub struct ChatMessage {
            #>[derive(Copy)]
            pub sender: pub enum ChatMessageSender { User, Assistant },
            pub content: String,
        }>
    }
}

impl ChatHistory {
    pub fn new(user: User, system_prompt: impl ToString) -> Self {
        Self {
            user,
            system_prompt: system_prompt.to_string(),
            messages: Vec::default(),
        }
    }

    pub fn push_user_message(&mut self, message: impl ToString) {
        self.messages.push(ChatMessage {
            sender: ChatMessageSender::User,
            content: message.to_string(),
        })
    }

    pub fn push_message(&mut self, message: ChatMessage) {
        self.messages.push(message)
    }
}
