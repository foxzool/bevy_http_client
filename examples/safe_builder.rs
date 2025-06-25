use bevy_http_client::prelude::*;

fn main() {
    println!("Testing safe builder methods...");
    
    // Test successful build
    let client = HttpClient::new()
        .get("https://api.example.com")
        .headers(&[("Accept", "application/json")]);
    
    match client.try_build() {
        Ok(_request) => println!("✅ Successfully built HTTP request"),
        Err(e) => println!("❌ Failed to build: {}", e),
    }
    
    // Test missing method
    let client = HttpClient::new()
        .headers(&[("Accept", "application/json")]);
    
    match client.try_build() {
        Ok(_) => println!("❌ This should have failed"),
        Err(e) => println!("✅ Expected error for missing method: {}", e),
    }
    
    // Test missing URL
    let client = HttpClient::new()
        .get(""); // Empty URL
    
    match client.try_build() {
        Ok(_) => println!("❌ This should have failed"),
        Err(e) => println!("✅ Expected error for missing URL: {}", e),
    }
    
    println!("Safe builder tests completed!");
}