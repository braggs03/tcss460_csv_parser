use serde_json::Value;
use dotenv::dotenv;
use std::io::{stdin, stdout};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let api_key = std::env::var("API_KEY").expect("API_KEY MUST BE SET.");
    let mut wtr = csv::Writer::from_writer(stdout());
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_reader(stdin());
    for record in rdr.records() {

        let record = record?;
        let title = record.get(3).unwrap();
        let author = record.get(1).unwrap();
        let api_url = format!(
            "https://www.googleapis.com/books/v1/volumes?q=intitle:{}&inauthor:{}&key={}",
            title,
            author,
            api_key
        ).replace(" ", "+");
        let resp = reqwest::get(api_url)
            .await?
            .text()
            .await?;

        let v: Value = serde_json::from_str(resp.as_str())?;
        let industry_identifiers = &v["items"][0]["volumeInfo"]["industryIdentifiers"];
        let mut new_isbn = record.get(0);

        if industry_identifiers.as_array().is_some() {
            for identifier in industry_identifiers.as_array().unwrap() {
                if "ISBN_13".eq(&identifier["type"]) {
                    new_isbn = identifier["identifier"].as_str();
                    break;
                }
            }
            wtr.write_record(
                record.iter().enumerate().map(|(i, v)|
                    if i == 0 { new_isbn } else { Some(v) }
                ).map(|opt| opt.unwrap_or(""))
                    .collect::<Vec<_>>()
            ).expect("Failed to write record.");
        } else {
            wtr.write_record(record.iter()).expect("Failed to write original record.");
        }
    }
    Ok(())
}