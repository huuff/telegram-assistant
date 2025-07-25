use crate::{
    app::output::llm_client::LlmClient,
    domain::chat::{ChatHistory, ChatMessage, ChatMessageSender},
};
use async_openai::types::*;

/// An LLM client that uses the OpenAI API
///
/// Note that this doesn't mean that it communicates with an OpenAI model
pub struct OpenaiLlmClient {
    client: async_openai::Client<async_openai::config::OpenAIConfig>,
}

impl LlmClient for OpenaiLlmClient {
    async fn get_response(&self, history: &ChatHistory) -> Result<ChatMessage, anyhow::Error> {
        let mut openai_history =
            Vec::<ChatCompletionRequestMessage>::with_capacity(1 + history.messages.len());
        openai_history.push(
            ChatCompletionRequestSystemMessageArgs::default()
                .content(history.system_prompt.as_str())
                .build()?
                .into(),
        );

        for msg in &history.messages {
            openai_history.push(match msg.sender {
                ChatMessageSender::User => ChatCompletionRequestUserMessageArgs::default()
                    .content(msg.content.as_str())
                    .build()?
                    .into(),
                ChatMessageSender::Assistant => {
                    ChatCompletionRequestAssistantMessageArgs::default()
                        .content(msg.content.as_str())
                        .build()?
                        .into()
                }
            })
        }

        let chat_client = self.client.chat();

        let request = CreateChatCompletionRequestArgs::default()
            .model("google/gemini-2.5-pro")
            .messages(openai_history)
            .build()?;
        let response = chat_client.create(request).await?;
        tracing::info!(event = "received-llm-response", ?response);

        let response_content = response
            .choices
            .into_iter()
            .next()
            .ok_or_else(|| anyhow::anyhow!("no choices in openai response"))?
            .message
            .content
            .ok_or_else(|| anyhow::anyhow!("no content in choice"))?;

        Ok(ChatMessage {
            sender: ChatMessageSender::Assistant,
            content: response_content,
        })
    }
}

impl OpenaiLlmClient {
    pub fn new(base_url: &str, token: &str) -> Self {
        let config = async_openai::config::OpenAIConfig::new()
            .with_api_key(token)
            .with_api_base(base_url);
        Self {
            client: async_openai::Client::with_config(config),
        }
    }
}
