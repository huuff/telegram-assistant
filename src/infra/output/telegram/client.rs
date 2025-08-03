use derive_more::Constructor;
use teloxide::{prelude::*, types::*};

use crate::{app::output::chat_client::ChatClient, domain::chat::ChatMessage};

#[derive(Constructor)]
pub struct TelegramChatClient {
    bot: Bot,
}

impl ChatClient for TelegramChatClient {
    async fn send(&self, chat_id: i64, message: &ChatMessage) -> Result<(), anyhow::Error> {
        self.bot
            .send_message(ChatId(chat_id), &message.content)
            .parse_mode(ParseMode::Html)
            .await?;

        Ok(())
    }

    async fn typing(&self, chat_id: i64) -> Result<(), anyhow::Error> {
        self.bot
            .send_chat_action(ChatId(chat_id), ChatAction::Typing)
            .await?;

        Ok(())
    }
}
