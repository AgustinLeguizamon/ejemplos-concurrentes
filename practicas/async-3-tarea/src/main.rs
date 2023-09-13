use async_std::task;
use serde::Deserialize;
use futures::future::join_all;
use lazy_static::lazy_static;

pub type ObservationResponseVec = Vec<ObservationResponse>;

#[derive(Deserialize, Debug)]
pub struct ObservationResponse {
    #[serde(rename = "speciesCode")]
    species_code: String,

    #[serde(rename = "comName")]
    com_name: String,

    #[serde(rename = "sciName")]
    sci_name: String,

    #[serde(rename = "locId")]
    loc_id: String,

    #[serde(rename = "locName")]
    loc_name: String,

    #[serde(rename = "obsDt")]
    obs_dt: String,

    #[serde(rename = "howMany")]
    how_many: i64,

    #[serde(rename = "lat")]
    lat: f64,

    #[serde(rename = "lng")]
    lng: f64,

    #[serde(rename = "obsValid")]
    obs_valid: bool,

    #[serde(rename = "obsReviewed")]
    obs_reviewed: bool,

    #[serde(rename = "locationPrivate")]
    location_private: bool,

    #[serde(rename = "subId")]
    sub_id: String,
}

lazy_static! {
    static ref API_KEY: &'static str = "ekgg6ikp80gb" ;
    static ref REGION_CODE: &'static str = "KZ" ;
}

fn main() {

    println!("Hello, world!");
    println!("async_main = {:?}", task::block_on(async_main()));
}

async fn async_main() -> Result<ObservationResponseVec, reqwest::Error> {
    let client = reqwest::Client::new();
    let url = format!("https://api.ebird.org/v2/data/obs/{}/recent", *REGION_CODE);

    let observations = client
        .get(url)
        .query(&[("maxResults", "20")])
        .header("x-ebirdapitoken", *API_KEY)
        .send()
        .await?
        .json::<ObservationResponseVec>()
        .await?;

    println!("observations = {:?}", observations);
    let species_codes: Vec<String> = observations.iter().map(|o| o.species_code.clone()).collect();
    println!("species_codes = {:?}", species_codes);

    let species_observations_futures = find_observations_by_species_codes(species_codes);
    println!("species_observations_futures = {:?}", species_observations_futures.await);

    Ok(species_observations_futures)
}

async fn find_observations_by_species_codes(species_codes: Vec<String>) -> Vec<Result<ObservationResponseVec, reqwest::Error>> {
    let observations_futures = species_codes.into_iter().map(|code| find_observations_by_species_code(code));
    // TODO: join_all ejecuta los futures de manera concurrente? Si, es la idea de usar un join_all pq si uso un for lo haría de forma secuencial
    return join_all(observations_futures).await;
}

async fn find_observations_by_species_code(code: String) -> Result<ObservationResponseVec, reqwest::Error> {
    let client = reqwest::Client::new();
    return Ok(client
        .get(format!("https://api.ebird.org/v2/data/obs/KZ/recent/{}", code))
        .header("x-ebirdapitoken", "ekgg6ikp80gb")
        .send()
        .await?
        .json::<ObservationResponseVec>()
        .await?);
}
