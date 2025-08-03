use crate::SystemPrompt;
use derive_more::Constructor;

use crate::{app::output::trimming::ChatTrimmingService, domain::chat::ChatHistory};

#[derive(Constructor)]
pub struct ChatResettingService;

impl ChatTrimmingService for ChatResettingService {
    async fn trim(&self, history: ChatHistory) -> ChatHistory {
        ChatHistory::new(
            history.user.clone(),
            // TODO: it'd be cool to make this configurable, perhaps there should be a
            // prompt factory that receives a generic context? (chat/user)
            SystemPrompt::builder().user(history.user).build(),
        )
    }
}

impl ChatHistory {
    pub fn is_too_long(&self) -> bool {
        self.messages
            .iter()
            .map(|msg| msg.content.len())
            .sum::<usize>()
            > 10000
    }
}
