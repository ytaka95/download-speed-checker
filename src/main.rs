use reqwest::{self, Response};
use std::error::Error;

pub enum HostingService {
    CLOUDFLARE,
    CLOUDFRONT,
    FIREBASE,
    VERCEL,
    NETLIFY,
}

impl HostingService {
    fn cache_header_name(&self) -> &'static str {
        match self {
            HostingService::CLOUDFLARE => "cf-cache-status",
            HostingService::CLOUDFRONT => "x-cache",
            HostingService::FIREBASE => "",
            HostingService::VERCEL => "",
            HostingService::NETLIFY => "",
        }
    }

    fn cache_header_value_hit(&self) -> &'static str {
        match self {
            HostingService::CLOUDFLARE => "HIT",
            HostingService::CLOUDFRONT => "Hit from cloudfront",
            HostingService::FIREBASE => "",
            HostingService::VERCEL => "",
            HostingService::NETLIFY => "",
        }
    }
}

impl std::fmt::Display for HostingService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match *self {
            Self::CLOUDFLARE => "CLOUDFLARE",
            Self::CLOUDFRONT => "CLOUDFRONT",
            Self::FIREBASE => "FIREBASE",
            Self::VERCEL => "VERCEL",
            Self::NETLIFY => "NETLIFY",
        };
        write!(f, "{}", s)
    }
}

fn display_headers(response: &Response) {
    println!("\nResponse Headers:");
    println!("----------------");
    for (name, value) in response.headers() {
        match value.to_str() {
            Ok(v) => println!("{}: {}", name, v),
            Err(_) => println!("{}: <binary value>", name),
        }
    }
}

fn is_cache_hit(response: &Response, hosting_service: &HostingService) -> bool {
    println!(
        "service: {}, header name: {}",
        hosting_service,
        hosting_service.cache_header_name()
    );
    match response
        .headers()
        .get(hosting_service.cache_header_name())
        .expect("header key is not contained")
        .to_str()
    {
        Ok(v) => v == hosting_service.cache_header_value_hit(),
        Err(_) => {
            println!(
                "header {} cannot be obtained",
                hosting_service.cache_header_name()
            );
            false
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // URLを指定
    let url = "https://www.example.com";

    // クライアントを作成
    let client = reqwest::Client::new();

    // GETリクエストを送信
    let response = client.get(url).send().await.expect("GET request is failed");

    // ステータスコードを表示
    println!("Status: {}", response.status());

    // キャッシュヒットの有無を表示
    let cache_hit = is_cache_hit(&response, &HostingService::CLOUDFRONT);
    println!("cache hit: {}", cache_hit);

    // レスポンスヘッダーを表示
    display_headers(&response);

    Ok(())
}
