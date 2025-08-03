use std::{collections::HashMap, sync::Mutex};

use crate::{app::output::repo::ChatRepository, domain::chat::ChatHistory};

pub struct InMemoryChatRepository {
    store: Mutex<HashMap<i64, ChatHistory>>,
}

impl ChatRepository for InMemoryChatRepository {
    async fn find(&self, id: i64) -> Result<Option<ChatHistory>, anyhow::Error> {
        // TODO: this will break forever if the lock is poisoned right? gotta fix that
        let store = self.store.lock().expect("should never get poisoned");

        Ok(store.get(&id).cloned())
    }

    async fn save(&self, id: i64, chat: ChatHistory) -> Result<(), anyhow::Error> {
        let mut store = self.store.lock().expect("should never get poisoned");

        store.insert(id, chat);

        Ok(())
    }
}

impl InMemoryChatRepository {
    pub fn new() -> Self {
        Self {
            store: Mutex::new(HashMap::default()),
        }
    }
}
