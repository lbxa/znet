use std::collections::{HashSet, VecDeque};

use reqwest::{self, Url};
use scraper::{Html, Selector};

fn resolve_url(link: &str, base_url: &str) -> String {
    let base_url = Url::parse(base_url).unwrap();
    base_url.join(link).unwrap().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_url() {
        assert_eq!(
            resolve_url("/path", "https://example.com"),
            "https://example.com/path"
        );
    }
}

async fn crawl(seed_urls: Vec<String>, max_depth: usize) {
    let mut to_visit = VecDeque::from(
        seed_urls
            .into_iter()
            .map(|url| (url, 0))
            .collect::<Vec<_>>(),
    );
    let mut visited: HashSet<String> = HashSet::new();

    while let Some((url, depth)) = to_visit.pop_front() {
        if visited.contains(&url) || depth >= max_depth {
            continue;
        }
        visited.insert(url.clone());

        match reqwest::get(&url).await {
            Ok(response) => {
                let body = response.text().await.unwrap();
                let document = Html::parse_document(&body);
                let selector = Selector::parse("a").unwrap();

                for element in document.select(&selector) {
                    if let Some(link) = element.value().attr("href") {
                        let absolute_link = resolve_url(link, &url);
                        if !visited.contains(&absolute_link) {
                            to_visit.push_back((absolute_link, depth + 1));
                        }
                    }
                }

                println!("Crawled: {} (depth: {})", url, depth);
            }
            Err(e) => println!("Failed to fetch {}: {:?}", url, e),
        }
    }
}

#[tokio::main(flavor = "multi_thread", worker_threads = 6)]
async fn main() {
    let contents = std::fs::read_to_string("seed.txt").unwrap();
    let seed_urls = contents.lines().map(|line| line.to_string()).collect();
    crawl(seed_urls, 10).await;
}
