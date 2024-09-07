use reqwest::Client;
use serde_json::Value;
use std::error::Error;

pub async fn get_uuid(username: &str) -> Result<String, Box<dyn Error>> {
    let url = format!(
        "https://api.mojang.com/users/profiles/minecraft/{}",
        username
    );
    let client = Client::new();
    let response = client.get(&url).send().await?;
    let body = response.text().await?;
    get_id(&body)
}

fn get_id(all: &str) -> Result<String, Box<dyn Error>> {
    let v: Value = serde_json::from_str(all)?;
    if let Some(id) = v.get("id") {
        if let Some(id_str) = id.as_str() {
            return Ok(id_str.to_string());
        }
    }
    Err("Champ 'id' introuvable ou incorrectement formaté".into())
}
