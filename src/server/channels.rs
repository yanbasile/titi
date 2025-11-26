//! Pub/Sub channel system
//!
//! Manages publish/subscribe channels for terminal communication.

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::RwLock;

pub type ConnectionId = u64;

#[derive(Debug, Clone)]
pub struct Message {
    pub channel: String,
    pub content: String,
}

struct Channel {
    name: String,
    subscribers: Vec<ConnectionId>,
    queue: VecDeque<Message>, // FIFO queue for messages
}

impl Channel {
    fn new(name: String) -> Self {
        Self {
            name,
            subscribers: Vec::new(),
            queue: VecDeque::new(),
        }
    }
}

pub struct ChannelManager {
    channels: Arc<RwLock<HashMap<String, Channel>>>,
}

impl ChannelManager {
    pub fn new() -> Self {
        Self {
            channels: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Subscribe a connection to a channel
    pub async fn subscribe(&self, channel_name: &str, conn_id: ConnectionId) {
        let mut channels = self.channels.write().await;
        let channel = channels
            .entry(channel_name.to_string())
            .or_insert_with(|| Channel::new(channel_name.to_string()));

        if !channel.subscribers.contains(&conn_id) {
            channel.subscribers.push(conn_id);
        }
    }

    /// Unsubscribe a connection from a channel
    pub async fn unsubscribe(&self, channel_name: &str, conn_id: ConnectionId) {
        let mut channels = self.channels.write().await;
        if let Some(channel) = channels.get_mut(channel_name) {
            channel.subscribers.retain(|&id| id != conn_id);

            // Remove channel if no subscribers
            if channel.subscribers.is_empty() && channel.queue.is_empty() {
                channels.remove(channel_name);
            }
        }
    }

    /// Unsubscribe a connection from all channels
    pub async fn unsubscribe_all(&self, conn_id: ConnectionId) {
        let mut channels = self.channels.write().await;
        let channel_names: Vec<String> = channels.keys().cloned().collect();

        for channel_name in channel_names {
            if let Some(channel) = channels.get_mut(&channel_name) {
                channel.subscribers.retain(|&id| id != conn_id);

                // Remove channel if no subscribers
                if channel.subscribers.is_empty() && channel.queue.is_empty() {
                    channels.remove(&channel_name);
                }
            }
        }
    }

    /// Publish a message to a channel
    pub async fn publish(&self, channel_name: &str, content: String) -> usize {
        let mut channels = self.channels.write().await;
        let channel = channels
            .entry(channel_name.to_string())
            .or_insert_with(|| Channel::new(channel_name.to_string()));

        let message = Message {
            channel: channel_name.to_string(),
            content,
        };

        // Add to queue
        channel.queue.push_back(message);

        // Return number of subscribers
        channel.subscribers.len()
    }

    /// Pop a message from a channel (FIFO, consume on read)
    pub async fn pop_message(&self, channel_name: &str) -> Option<Message> {
        let mut channels = self.channels.write().await;
        if let Some(channel) = channels.get_mut(channel_name) {
            channel.queue.pop_front()
        } else {
            None
        }
    }

    /// Get all pending messages for a subscriber
    pub async fn get_messages(&self, channel_name: &str, conn_id: ConnectionId) -> Vec<Message> {
        let channels = self.channels.read().await;
        if let Some(channel) = channels.get(channel_name) {
            if channel.subscribers.contains(&conn_id) {
                // Return all messages in queue
                return channel.queue.iter().cloned().collect();
            }
        }
        Vec::new()
    }

    /// Get number of messages in a channel queue
    pub async fn queue_length(&self, channel_name: &str) -> usize {
        let channels = self.channels.read().await;
        channels
            .get(channel_name)
            .map(|c| c.queue.len())
            .unwrap_or(0)
    }

    /// List all channels
    pub async fn list_channels(&self) -> Vec<String> {
        let channels = self.channels.read().await;
        channels.keys().cloned().collect()
    }

    /// Get subscribers for a channel
    pub async fn get_subscribers(&self, channel_name: &str) -> Vec<ConnectionId> {
        let channels = self.channels.read().await;
        channels
            .get(channel_name)
            .map(|c| c.subscribers.clone())
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_subscribe_unsubscribe() {
        let manager = ChannelManager::new();

        manager.subscribe("test-channel", 1).await;
        let subs = manager.get_subscribers("test-channel").await;
        assert_eq!(subs, vec![1]);

        manager.unsubscribe("test-channel", 1).await;
        let subs = manager.get_subscribers("test-channel").await;
        assert_eq!(subs.len(), 0);
    }

    #[tokio::test]
    async fn test_publish_pop() {
        let manager = ChannelManager::new();

        manager.subscribe("test-channel", 1).await;
        manager.publish("test-channel", "Hello".to_string()).await;

        let msg = manager.pop_message("test-channel").await.unwrap();
        assert_eq!(msg.content, "Hello");

        // Queue should be empty now
        assert!(manager.pop_message("test-channel").await.is_none());
    }

    #[tokio::test]
    async fn test_multiple_subscribers() {
        let manager = ChannelManager::new();

        manager.subscribe("test-channel", 1).await;
        manager.subscribe("test-channel", 2).await;
        manager.subscribe("test-channel", 3).await;

        let count = manager.publish("test-channel", "Broadcast".to_string()).await;
        assert_eq!(count, 3); // 3 subscribers

        let subs = manager.get_subscribers("test-channel").await;
        assert_eq!(subs.len(), 3);
    }

    #[tokio::test]
    async fn test_queue_length() {
        let manager = ChannelManager::new();

        manager.publish("test-channel", "msg1".to_string()).await;
        manager.publish("test-channel", "msg2".to_string()).await;
        manager.publish("test-channel", "msg3".to_string()).await;

        assert_eq!(manager.queue_length("test-channel").await, 3);

        manager.pop_message("test-channel").await;
        assert_eq!(manager.queue_length("test-channel").await, 2);
    }

    #[tokio::test]
    async fn test_unsubscribe_all() {
        let manager = ChannelManager::new();

        manager.subscribe("channel1", 1).await;
        manager.subscribe("channel2", 1).await;
        manager.subscribe("channel3", 1).await;

        manager.unsubscribe_all(1).await;

        assert_eq!(manager.get_subscribers("channel1").await.len(), 0);
        assert_eq!(manager.get_subscribers("channel2").await.len(), 0);
        assert_eq!(manager.get_subscribers("channel3").await.len(), 0);
    }
}
