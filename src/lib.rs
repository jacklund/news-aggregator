use feed::Feed;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

pub mod feed;
pub mod mastodon;
pub mod rss;

pub struct NewsAggregator {
    sender: mpsc::UnboundedSender<feed::FeedItem>,
    receiver: Option<mpsc::UnboundedReceiver<feed::FeedItem>>,
    feeds: Arc<RwLock<HashMap<String, feed::Feed>>>,
}

impl NewsAggregator {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();
        Self {
            sender,
            receiver: Some(receiver),
            feeds: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn add_feed(&mut self, feed: Feed) {
        self.feeds
            .write()
            .unwrap()
            .insert(feed.id.clone(), feed.clone());
        feed.start(self.sender.clone()).await;
    }

    pub fn start(&mut self) -> JoinHandle<()> {
        let mut receiver = self.receiver.take().unwrap();
        let feeds = self.feeds.clone();
        tokio::spawn(async move {
            while let Some(feed_item) = receiver.recv().await {
                match feeds.read().unwrap().get(&feed_item.feed_id()) {
                    Some(feed) => {
                        feed_item.print_terminal(&feed);
                    }
                    None => {
                        eprintln!("Unknown feed ID {}", feed_item.feed_id());
                    }
                }
            }
        })
    }
}
