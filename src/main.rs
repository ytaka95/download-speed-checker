extern crate csv;
use reqwest::{self, Response};
use serde::{Deserialize, Serialize};
use std::error::Error;

const SETTINGS_CSV_PATH: &str = "./settings.csv";

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Destination {
    name: String,
    url: String,
    header_cache_key: String,
    header_cache_hit_value: String,
}

fn read_settings() -> Result<Vec<Destination>, csv::Error> {
    let mut reader =
        csv::Reader::from_path(SETTINGS_CSV_PATH).expect("the setting file is not found");
    let mut list = Vec::<Destination>::new();
    for row in reader.deserialize() {
        let data: Destination = row?;
        list.push(data);
    }
    Ok(list)
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

fn is_cache_hit(
    response: &Response,
    header_cache_key: &String,
    expected_header_cache_hit_value: &String,
) -> bool {
    match response.headers().get(header_cache_key) {
        Some(v) => {
            let actual_header_value = v.to_str().expect("header value cannot be converted to str");
            println!(
                "====\nKey: {}\nExpected value: {}\nActual value: {}\n====",
                header_cache_key, expected_header_cache_hit_value, actual_header_value
            );
            actual_header_value == expected_header_cache_hit_value
        }
        None => {
            println!(
                "====\nKey: {}\nExpected value: {}\n====",
                header_cache_key, expected_header_cache_hit_value,
            );
            false
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // CSVから設定を読み込み
    let destinations = read_settings().unwrap();

    // クライアントを作成
    let client = reqwest::Client::new();

    for destination in destinations {
        println!("======== Send GET request: {destination:?} ========");

        // URLを指定
        let url = destination.url;

        // GETリクエストを送信
        let response = client.get(url).send().await.expect("GET request is failed");

        // ステータスコードを表示
        println!("Status: {}", response.status());

        // キャッシュヒットの有無を表示
        let header_cache_key: String = destination.header_cache_key;
        let header_cache_hit_value: String = destination.header_cache_hit_value;
        let cache_hit = is_cache_hit(&response, &header_cache_key, &header_cache_hit_value);
        println!("cache hit: {}", cache_hit);

        // レスポンスヘッダーを表示
        // display_headers(&response);
    }

    Ok(())
}
