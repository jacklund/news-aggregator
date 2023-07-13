use crate::feed::{Feed, FeedItem};
use megalodon;
use megalodon::megalodon::GetPublicTimelineInputOptions;
use std::time::Duration;
use tokio::sync::mpsc;

pub fn print_terminal(item: &megalodon::entities::status::Status, feed: &Feed) {
    println!("{}: {}", feed.name, item.account.username);
    println!("{}", item.content);
    println!("");
}

pub async fn start(feed: &Feed, sender: mpsc::UnboundedSender<FeedItem>) {
    let feed_name = feed.name.clone();
    let feed_url = feed.url.clone();
    let feed_id = feed.id.clone();
    tokio::spawn(async move {
        let client = megalodon::generator(megalodon::SNS::Mastodon, feed_url, None, None);
        let mut options = GetPublicTimelineInputOptions::default();
        options.limit = Some(20);
        let refresh_seconds = 10;

        loop {
            let res = client
                .get_public_timeline(Some(&options))
                .await
                .expect("error getting instance");
            for item in res.json() {
                options.since_id = Some(item.id.clone());
                let feed_item = FeedItem::from_mastodon(item, &feed_id);
                if let Err(error) = sender.send(feed_item) {
                    eprintln!("Error sending on channel {}: {}", feed_name, error);
                    return;
                }
            }
            println!(
                "{} sleeping, waking in {} seconds...",
                feed_name, refresh_seconds
            );
            tokio::time::sleep(Duration::from_secs(refresh_seconds)).await;
        }
    });
}
