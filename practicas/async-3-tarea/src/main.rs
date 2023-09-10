use async_std::task;

fn main() {
    println!("Hello, world!");
    let body = task::block_on(async_main());

    println!("body = {:?}", body);
}

async fn async_main() -> Result<String, reqwest::Error> {
    let client = reqwest::Client::new();
    let api_key = String::from("ekgg6ikp80gb");
    let region_code = String::from("KZ");
    let url = format!("https://api.ebird.org/v2/data/obs/{}/recent", region_code);

    let body = client.get(url).header("x-ebirdapitoken", api_key).send().await?.text().await?;
    Ok(body)
}
