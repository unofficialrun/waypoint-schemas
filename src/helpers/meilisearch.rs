//! Helper functions for working with Meilisearch schemas

use meilisearch_sdk::{client::Client, settings::Settings};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::{error, info};
use chrono::{DateTime, Utc};

use crate::proto::meilisearch::{UserProfile, UserProfileSchema};

/// Error type for Meilisearch operations
#[derive(Error, Debug)]
pub enum MeilisearchSchemaError {
    #[error("Meilisearch client error: {0}")]
    Client(String),

    #[error("Failed to apply schema: {0}")]
    Schema(String),

    #[error("Failed to convert document: {0}")]
    Conversion(String),
}


/// Convert from generated proto type to a Serde-friendly type
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserProfileDocument {
    pub id: String,
    pub fid: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pfp_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bio: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub twitter: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub github: Option<String>,
    // Just make this a String to accept whatever Meilisearch sends us
    #[serde(default)]
    pub updated_at: String,
}

impl From<UserProfile> for UserProfileDocument {
    fn from(proto: UserProfile) -> Self {
        Self {
            id: proto.id,
            fid: proto.fid,
            display_name: proto.display_name,
            username: proto.username,
            pfp_url: proto.pfp_url,
            bio: proto.bio,
            url: proto.url,
            location: proto.location,
            twitter: proto.twitter,
            github: proto.github,
            updated_at: proto.updated_at.to_string(),
        }
    }
}

impl From<UserProfileDocument> for UserProfile {
    fn from(doc: UserProfileDocument) -> Self {
        // Convert the string timestamp to u64 if possible, or default to current time
        let updated_at = doc.updated_at.parse::<u64>()
            .or_else(|_| {
                // Try to parse as ISO datetime with explicit format handling
                if doc.updated_at.contains('T') && doc.updated_at.contains('+') {
                    // Try standard RFC3339 format first
                    DateTime::parse_from_rfc3339(&doc.updated_at)
                        .map(|dt| dt.timestamp() as u64)
                        .or_else(|_| {
                            // Try with various precise formats that might be returned by MeiliSearch
                            DateTime::parse_from_str(&doc.updated_at, "%Y-%m-%dT%H:%M:%S%.f%:z")
                                .or_else(|_| DateTime::parse_from_str(&doc.updated_at, "%Y-%m-%dT%H:%M:%S%.f%z"))
                                .or_else(|_| DateTime::parse_from_str(&doc.updated_at, "%Y-%m-%dT%H:%M:%S%:z"))
                                .or_else(|_| DateTime::parse_from_str(&doc.updated_at, "%Y-%m-%dT%H:%M:%S%z"))
                                .map(|dt| dt.timestamp() as u64)
                        })
                } else {
                    // Not a timestamp format we recognize
                    // Return a generic parse error since we can't construct one directly
                    Err(DateTime::parse_from_rfc3339("invalid").unwrap_err())
                }
            })
            .unwrap_or_else(|_| {
                // Default to current time if parsing fails
                Utc::now().timestamp() as u64
            });

        Self {
            id: doc.id,
            fid: doc.fid,
            display_name: doc.display_name,
            username: doc.username,
            pfp_url: doc.pfp_url,
            bio: doc.bio,
            url: doc.url,
            location: doc.location,
            twitter: doc.twitter,
            github: doc.github,
            updated_at,
        }
    }
}

/// Apply the user profile schema to Meilisearch
pub async fn apply_user_profile_schema(client: &Client) -> Result<(), MeilisearchSchemaError> {
    info!("Applying user profiles schema to Meilisearch");

    // Get the predefined schema
    let schema = get_user_profile_schema();

    // Extract index settings
    let index_settings = schema
        .index
        .ok_or_else(|| MeilisearchSchemaError::Schema("No index settings provided".to_string()))?;

    // Create the index if it doesn't exist
    let index_name = &index_settings.name;
    let primary_key = &index_settings.primary_key;

    // Create index
    match client.create_index(index_name, Some(primary_key)).await {
        Ok(task) => {
            info!(
                "Created index '{}' with primary key '{}', task ID: {}",
                index_name, primary_key, task.task_uid
            );
        }
        Err(e) => {
            // If the error is that the index already exists, that's okay
            if !e.to_string().contains("index_already_exists") {
                return Err(MeilisearchSchemaError::Client(e.to_string()));
            }
        }
    }

    // Configure settings
    let mut settings = Settings::new();

    // Searchable attributes
    if let Some(searchable) = &schema.searchable {
        let attrs: Vec<String> = searchable
            .attributes
            .iter()
            .map(|s| s.to_string())
            .collect();
        settings = settings.with_searchable_attributes(attrs);
    }

    // Ranking rules
    if let Some(ranking) = &schema.ranking {
        let rules: Vec<String> = ranking.rules.iter().map(|s| s.to_string()).collect();
        settings = settings.with_ranking_rules(rules);
    }

    // Distinct attribute
    if !schema.distinct_attribute.is_empty() {
        settings = settings.with_distinct_attribute(Some(schema.distinct_attribute.clone()));
    }

    // Filterable attributes
    if let Some(filterable) = &schema.filterable {
        let attrs: Vec<String> = filterable
            .attributes
            .iter()
            .map(|s| s.to_string())
            .collect();
        settings = settings.with_filterable_attributes(attrs);
    }

    // Sortable attributes
    if let Some(sortable) = &schema.sortable {
        let attrs: Vec<String> = sortable.attributes.iter().map(|s| s.to_string()).collect();
        settings = settings.with_sortable_attributes(attrs);
    }

    // Apply settings
    let index = client.index(index_name);
    match index.set_settings(&settings).await {
        Ok(task) => {
            info!(
                "Applied settings to index '{}', task ID: {}",
                index_name, task.task_uid
            );
            Ok(())
        }
        Err(e) => {
            error!("Failed to apply settings to index '{}': {}", index_name, e);
            Err(MeilisearchSchemaError::Client(e.to_string()))
        }
    }
}

/// Get the predefined user profile schema
pub fn get_user_profile_schema() -> UserProfileSchema {
    let mut schema = UserProfileSchema::default();

    // Index settings
    let index = crate::proto::meilisearch::user_profile_schema::IndexSettings {
        name: "user_profiles".to_string(),
        primary_key: "id".to_string(),
    };

    // Searchable attributes
    let searchable = crate::proto::meilisearch::user_profile_schema::SearchableAttributes {
        attributes: vec![
            "username".to_string(),
            "display_name".to_string(),
            "bio".to_string(),
            "location".to_string(),
            "twitter".to_string(),
            "github".to_string(),
            "fid".to_string(),
        ],
    };

    // Ranking rules
    let ranking = crate::proto::meilisearch::user_profile_schema::RankingRules {
        rules: vec![
            "words".to_string(),
            "typo".to_string(),
            "proximity".to_string(),
            "attribute".to_string(),
            "sort".to_string(),
            "exactness".to_string(),
        ],
    };

    // Filterable attributes
    let filterable = crate::proto::meilisearch::user_profile_schema::FilterableAttributes {
        attributes: vec!["fid".to_string()],
    };

    // Sortable attributes
    let sortable = crate::proto::meilisearch::user_profile_schema::SortableAttributes {
        attributes: vec!["fid".to_string(), "updated_at".to_string()],
    };

    // Set the fields
    schema.index = Some(index);
    schema.searchable = Some(searchable);
    schema.ranking = Some(ranking);
    schema.distinct_attribute = "username".to_string();
    schema.filterable = Some(filterable);
    schema.sortable = Some(sortable);

    schema
}

/// Create a batch of user profiles in Meilisearch
pub async fn batch_create_user_profiles(
    client: &Client,
    profiles: &[UserProfile],
) -> Result<(), MeilisearchSchemaError> {
    // Convert proto profiles to Meilisearch documents
    let documents: Vec<UserProfileDocument> = profiles
        .iter()
        .map(|p| UserProfileDocument::from(p.clone()))
        .collect();

    // Add documents to index
    let index = client.index("user_profiles");
    match index.add_or_update(&documents, Some("id")).await {
        Ok(task) => {
            info!(
                "Added {} user profiles to Meilisearch, task ID: {}",
                documents.len(),
                task.task_uid
            );
            Ok(())
        }
        Err(e) => {
            error!("Failed to add user profiles to Meilisearch: {}", e);
            Err(MeilisearchSchemaError::Client(e.to_string()))
        }
    }
}

/// Search for user profiles
pub async fn search_user_profiles(
    client: &Client,
    query: &str,
    limit: Option<usize>,
    offset: Option<usize>,
    filter: Option<&str>,
) -> Result<Vec<UserProfile>, MeilisearchSchemaError> {
    let index = client.index("user_profiles");

    // Create search query
    let mut search = index.search();
    search.with_query(query);

    if let Some(limit_val) = limit {
        search.with_limit(limit_val);
    }

    if let Some(offset_val) = offset {
        search.with_offset(offset_val);
    }

    if let Some(filter_val) = filter {
        search.with_filter(filter_val);
    }

    // Execute search
    match search.execute::<UserProfileDocument>().await {
        Ok(results) => {
            // Convert documents back to proto messages
            let profiles = results
                .hits
                .into_iter()
                .map(|hit| UserProfile::from(hit.result))
                .collect();

            Ok(profiles)
        }
        Err(e) => {
            error!("Failed to search user profiles: {}", e);
            Err(MeilisearchSchemaError::Client(e.to_string()))
        }
    }
}
