//! Example showing how to use the Waypoint schemas with Meilisearch

use meilisearch_sdk::client::Client;
use waypoint_schemas::helpers::meilisearch::{
    apply_user_profile_schema, batch_create_user_profiles,
};
use waypoint_schemas::proto::meilisearch::UserProfile;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create Meilisearch client
    let client = Client::new("http://localhost:7700", Some("masterKey"))
        .expect("Failed to create Meilisearch client");

    // Apply the schema to Meilisearch
    apply_user_profile_schema(&client).await?;

    // Create some example user profiles
    let profiles = vec![
        UserProfile {
            id: "1".to_string(),
            fid: 1,
            display_name: Some("Alice".to_string()),
            username: Some("alice".to_string()),
            bio: Some("I love Farcaster!".to_string()),
            twitter: Some("alice_twitter".to_string()),
            updated_at: 1646092800,
            ..Default::default()
        },
        UserProfile {
            id: "2".to_string(),
            fid: 2,
            display_name: Some("Bob".to_string()),
            username: Some("bob".to_string()),
            bio: Some("Web3 enthusiast".to_string()),
            github: Some("bob_github".to_string()),
            updated_at: 1646179200,
            ..Default::default()
        },
    ];

    // Add profiles to Meilisearch
    batch_create_user_profiles(&client, &profiles).await?;

    println!("Successfully set up Meilisearch with example profiles!");

    Ok(())
}
