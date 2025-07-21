use async_openai::types::{
    ChatCompletionRequestAssistantMessageArgs, ChatCompletionRequestMessage,
    ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestUserMessageArgs,
};

use super::model::{ChatHistory, ChatMessageSender};

impl From<ChatHistory> for Vec<ChatCompletionRequestMessage> {
    fn from(history: ChatHistory) -> Self {
        let mut openai_chat = Vec::with_capacity(1 + history.messages.len());
        openai_chat.push(
            ChatCompletionRequestSystemMessageArgs::default()
                .content(history.system_prompt)
                .build()
                .expect("can this even fail?")
                .into(),
        );

        for msg in history.messages {
            openai_chat.push(match msg.sender {
                ChatMessageSender::User => ChatCompletionRequestUserMessageArgs::default()
                    .content(msg.content)
                    .build()
                    .expect("can this even fail?")
                    .into(),
                ChatMessageSender::Assistant => {
                    ChatCompletionRequestAssistantMessageArgs::default()
                        .content(msg.content)
                        .build()
                        .expect("can this even fail?")
                        .into()
                }
            })
        }

        openai_chat
    }
}

impl ChatHistory {
    pub fn into_openai(self) -> Vec<ChatCompletionRequestMessage> {
        self.into()
    }
}
