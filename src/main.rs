extern crate csv;
use reqwest::{self, Response};
use serde::{Deserialize, Serialize};
use std::time::Instant;
use std::{collections::HashMap, error::Error};
use std::{thread, time};

const SETTINGS_CSV_PATH: &str = "./config/destinations.csv";
const RESULT_HIT_KEY: &str = "Hit";
const RESULT_MISS_KEY: &str = "Miss";

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
        if let Some(first_char) = data.name.chars().next() {
            if first_char == '#' {
                println!("skip destination \"{}\"", data.name);
            } else {
                list.push(data);
            }
        }
    }
    Ok(list)
}

#[allow(dead_code)]
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
            // println!(
            //     "{}: {} (expected: {})",
            //     header_cache_key, actual_header_value, expected_header_cache_hit_value
            // );
            actual_header_value == expected_header_cache_hit_value
        }
        None => {
            // println!(
            //     "{}: None (expected: {})",
            //     header_cache_key, expected_header_cache_hit_value,
            // );
            false
        }
    }
}

fn display_result(result_map: HashMap<String, HashMap<&str, Vec<u128>>>) {
    for key in result_map.keys() {
        let mut result_hit_vec = result_map
            .get(key)
            .unwrap()
            .get(RESULT_HIT_KEY)
            .unwrap()
            .clone();
        let mut result_miss_vec = result_map
            .get(key)
            .unwrap()
            .get(RESULT_MISS_KEY)
            .unwrap()
            .clone();

        result_hit_vec.sort();
        result_miss_vec.sort();

        println!("\n======== {} ========", key);

        println!(
            "hit: {} ({}%)",
            result_hit_vec.len(),
            (result_hit_vec.len() * 100 / (result_hit_vec.len() + result_miss_vec.len()))
        );

        println!("========");

        let vec_len = result_hit_vec.len() as f64;
        if vec_len == 0.0 {
            println!("no result");
            continue;
        }

        let index_50 = (vec_len * 0.50).floor() as usize - 1;
        println!(
            "50% ({}): {}ms",
            index_50,
            result_hit_vec.get(index_50).unwrap()
        );
        let index_75 = (vec_len * 0.75).floor() as usize - 1;
        println!(
            "75% ({}): {}ms",
            index_75,
            result_hit_vec.get(index_75).unwrap()
        );
        let index_90 = (vec_len * 0.90).floor() as usize - 1;
        println!(
            "90% ({}): {}ms",
            index_90,
            result_hit_vec.get(index_90).unwrap()
        );
        let index_95 = (vec_len * 0.95).floor() as usize - 1;
        println!(
            "95% ({}): {}ms",
            index_95,
            result_hit_vec.get(index_95).unwrap()
        );
        let index_98 = (vec_len * 0.98).floor() as usize - 1;
        println!(
            "98% ({}): {}ms",
            index_98,
            result_hit_vec.get(index_98).unwrap()
        );
        let index_max = vec_len as usize - 1;
        println!(
            "Max ({}): {}ms",
            index_max,
            result_hit_vec.get(index_max).unwrap()
        );
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // CSVから設定を読み込み
    let destinations = read_settings().unwrap();

    // HTTPクライアントを作成
    let client = reqwest::Client::new();

    // 結果を保存するMap
    let mut result = HashMap::new();

    for destination in destinations {
        println!("======== Send GET request: {destination:?} ========");

        let mut result_for_destination = HashMap::<&str, Vec<u128>>::new();
        result_for_destination.insert(RESULT_HIT_KEY, Vec::new());
        result_for_destination.insert(RESULT_MISS_KEY, Vec::new());

        for _ in 0..50 {
            let request_client = client
                .get(&destination.url)
                .header(reqwest::header::ACCEPT_ENCODING, "gzip, deflate, br")
                .header(reqwest::header::ACCEPT_LANGUAGE, "ja,ja-JP")
                .header(reqwest::header::USER_AGENT, "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36");

            let start = Instant::now();
            // GETリクエストを送信
            let response = request_client.send().await.expect("GET request is failed");
            let duration = start.elapsed();

            thread::sleep(time::Duration::from_millis(50));

            if !response.status().is_success() {
                println!("Request has failed. Status code: {}", response.status());
                break;
            }

            // キャッシュヒットの有無を表示
            let cache_hit = is_cache_hit(
                &response,
                &destination.header_cache_key,
                &destination.header_cache_hit_value,
            );
            println!(
                "URL: {}, Status: {}, Cache hit: {}, time: {}ms",
                destination.url,
                response.status(),
                cache_hit,
                duration.as_millis()
            );

            if cache_hit {
                result_for_destination
                    .get_mut(RESULT_HIT_KEY)
                    .unwrap()
                    .push(duration.as_millis());
            } else {
                result_for_destination
                    .get_mut(RESULT_MISS_KEY)
                    .unwrap()
                    .push(duration.as_millis());
            }
        }
        result.insert(destination.name, result_for_destination);
    }

    display_result(result);

    Ok(())
}
