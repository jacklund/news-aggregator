use crate::feed::{Feed, FeedItem};
use anyhow::Result;
use reqwest;
use rss;
use std::collections::HashSet;
use std::time::Duration;
use tokio::sync::mpsc;

pub fn print_terminal(item: &rss::Item, feed: &Feed) {
    println!("{}: {}", feed.name, item.title.clone().unwrap());
    if let Some(summary) = item.description.clone() {
        println!("{}", summary);
    }
    println!("");
}

async fn refresh_channel(feed_url: &str) -> Result<rss::Channel> {
    let content = reqwest::get(feed_url).await?.bytes().await?;
    Ok(rss::Channel::read_from(&content[..])?)
}

pub async fn start(feed: &Feed, sender: mpsc::UnboundedSender<FeedItem>) {
    let name = feed.name.clone();
    let feed_url = feed.url.clone();
    let feed_id = feed.id.clone();
    let mut guids = HashSet::<String>::new();
    tokio::spawn(async move {
        let mut refresh_minutes: Option<u64> = None;
        loop {
            println!("Refreshing channel {}", name);
            let channel = match refresh_channel(&feed_url).await {
                Ok(channel) => channel,
                Err(error) => {
                    eprintln!("Got error refreshing RSS channel {}: {}", feed_url, error);
                    break;
                }
            };
            if refresh_minutes.is_none() {
                refresh_minutes = match channel.ttl.clone() {
                    Some(ttl) => Some(ttl.parse::<u64>().unwrap()),
                    None => Some(5),
                };
            }
            let mut new_guids: HashSet<String> = HashSet::new();
            for item in channel.items() {
                new_guids.insert(item.guid.clone().unwrap().value);
                if !guids.contains(&item.guid.clone().unwrap().value) {
                    let feed_item = FeedItem::from_rss(item.clone(), &feed_id);
                    if let Err(error) = sender.send(feed_item) {
                        eprintln!("Error sending on channel {}: {}", name, error);
                        return;
                    }
                }
            }
            guids = new_guids;
            println!(
                "{} sleeping, waking in {} minutes...",
                name,
                refresh_minutes.unwrap(),
            );
            tokio::time::sleep(Duration::from_secs(refresh_minutes.unwrap() * 60)).await;
        }
    });
}
