use std::path::Path;
use std::fs;
use reqwest::multipart::{Form, Part};
use anyhow::Result;

pub async fn upload_string_to_tmpfile(content: impl ToString, file_path: &str) -> Result<String> {
    let content = content.to_string();

    // Ensure the directory exists
    if let Some(dir) = Path::new(file_path).parent() {
        fs::create_dir_all(dir)?;
    }

    // Write content to file
    fs::write(file_path, content)?;

    // Create a client
    let client = reqwest::Client::new();

    // Get the filename from the path
    let filename = Path::new(file_path)
        .file_name()
        .and_then(|f| f.to_str())
        .unwrap_or("unknown")
        .to_string();

    // Create multipart form with the file
    let file_bytes = fs::read(file_path)?;
    let part = Part::bytes(file_bytes)
        .file_name(filename);

    let form = Form::new().part("file", part);

    // Send the request
    let response = client
        .post("https://tmpfiles.org/api/v1/upload")
        .multipart(form)
        .send()
        .await?;

    // Check response and get URL
    let upload_url = if response.status().is_success() {
        let json_response: serde_json::Value = response.json().await?;

        // Extract the URL from the response
        if let Some(url) = json_response["data"]["url"].as_str() {
            url.to_string()
        }
        else {
            return Err(anyhow::anyhow!("Could not extract URL from response"));
        }
    }
    else {
        return Err(anyhow::anyhow!("Upload failed with status: {}", response.status()));
    };

    // Delete the local file
    fs::remove_file(file_path)?;

    Ok(upload_url)
}