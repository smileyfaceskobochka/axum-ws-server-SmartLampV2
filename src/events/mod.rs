// events/mod.rs
use dashmap::DashMap;

pub struct EventBus {
    subscribers: DashMap<String, Vec<Box<dyn Fn(serde_json::Value) + Send + Sync>>>,
}

impl EventBus {
    pub fn new() -> Self {
        Self {
            subscribers: DashMap::new(),
        }
    }

    pub fn publish(&self, event_type: &str, data: serde_json::Value) {
        if let Some(subscribers) = self.subscribers.get(event_type) {
            for callback in subscribers.iter() {
                (callback)(data.clone());
            }
        }
    }

    pub fn subscribe<F: Fn(serde_json::Value) + Send + Sync + 'static>(
        &self,
        event_type: &str,
        callback: F,
    ) {
        self.subscribers
            .entry(event_type.to_string())
            .or_default()
            .push(Box::new(callback));
    }
}