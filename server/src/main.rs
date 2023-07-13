use news_aggregator_server::{feed, NewsAggregator};

#[tokio::main]
async fn main() {
    let mut aggregator = NewsAggregator::new();
    let feed = feed::create_rss_feed("CNN", "http://rss.cnn.com/rss/cnn_latest.rss");
    aggregator.add_feed(feed).await;

    let feed = feed::create_rss_feed("Al Jazeera", "https://www.aljazeera.com/xml/rss/all.xml");
    aggregator.add_feed(feed).await;

    let feed = feed::create_mastodon_feed("Mastodon public", "https://freeradical.zone");
    aggregator.add_feed(feed).await;

    let handle = aggregator.start();
    if let Err(error) = handle.await {
        eprintln!("Error in spawned task: {}", error);
    }
}
