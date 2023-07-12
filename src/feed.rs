use crate::{mastodon, rss};
use ::rss::Item;
use megalodon::entities::status::Status;
use tokio::sync::mpsc;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub enum FeedId {
    Rss(String),
    Mastodon(String),
}

#[derive(Debug, Clone)]
pub enum FeedType {
    Rss,
    Mastodon,
}

#[derive(Debug, Clone)]
pub enum FeedItem {
    Rss { feed_id: String, item: Item },
    Mastodon { feed_id: String, item: Status },
}

impl FeedItem {
    pub fn from_rss(item: Item, feed_id: &str) -> Self {
        Self::Rss {
            feed_id: feed_id.to_string(),
            item,
        }
    }

    pub fn from_mastodon(item: Status, feed_id: &str) -> Self {
        Self::Mastodon {
            feed_id: feed_id.to_string(),
            item,
        }
    }

    pub fn feed_id(&self) -> String {
        match self {
            Self::Rss { feed_id, item: _ } => feed_id.clone(),
            Self::Mastodon { feed_id, item: _ } => feed_id.clone(),
        }
    }

    pub fn print_terminal(&self, feed: &Feed) {
        match self {
            Self::Rss { feed_id: _, item } => rss::print_terminal(&item, feed),
            Self::Mastodon { feed_id: _, item } => mastodon::print_terminal(&item, feed),
        }
    }
}

pub fn create_rss_feed(name: &str, url: &str) -> Feed {
    Feed {
        name: name.to_string(),
        id: Uuid::new_v4().to_string(),
        url: url.to_string(),
        feed_type: FeedType::Rss,
    }
}

pub fn create_mastodon_feed(name: &str, url: &str) -> Feed {
    Feed {
        name: name.to_string(),
        id: Uuid::new_v4().to_string(),
        url: url.to_string(),
        feed_type: FeedType::Mastodon,
    }
}

#[derive(Debug, Clone)]
pub struct Feed {
    pub name: String,
    pub id: String,
    pub url: String,
    pub feed_type: FeedType,
}

impl Feed {
    pub async fn start(&self, sender: mpsc::UnboundedSender<FeedItem>) {
        match self.feed_type {
            FeedType::Rss => rss::start(self, sender).await,
            FeedType::Mastodon => mastodon::start(self, sender).await,
        }
    }
}
