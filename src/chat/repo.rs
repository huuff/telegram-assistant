use super::model::ChatHistory;

use std::{collections::HashMap, sync::Mutex};

// TODO should be async and use dynosaur
pub trait ChatRepository: Send + Sync {
    fn find(&self, id: i64) -> Result<Option<ChatHistory>, anyhow::Error>;

    fn save(&self, id: i64, chat: ChatHistory) -> Result<(), anyhow::Error>;
}

pub struct InMemoryChatRepository {
    store: Mutex<HashMap<i64, ChatHistory>>,
}

impl ChatRepository for InMemoryChatRepository {
    fn find(&self, id: i64) -> Result<Option<ChatHistory>, anyhow::Error> {
        // TODO: this will break forever if the lock is poisoned right? gotta fix that
        let store = self.store.lock().expect("should never get poisoned");

        Ok(store.get(&id).cloned())
    }

    fn save(&self, id: i64, chat: ChatHistory) -> Result<(), anyhow::Error> {
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
