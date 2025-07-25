use derive_more::Constructor;

use crate::{app::output::trimming::ChatTrimmingService, domain::chat::ChatHistory};

#[derive(Constructor)]
pub struct ChatResettingService {
    system_prompt_factory: Box<dyn Fn() -> String + Send + Sync>,
}

impl ChatTrimmingService for ChatResettingService {
    fn trim(&self, _: ChatHistory) -> ChatHistory {
        ChatHistory::new((self.system_prompt_factory)())
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
