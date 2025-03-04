//! Waypoint Schemas
//!
//! This crate provides schema definitions for Waypoint services.

// Re-export the generated Rust code from protos
pub mod proto {
    pub mod meilisearch {
        include!(concat!(env!("OUT_DIR"), "/waypoint.meilisearch.rs"));
    }

    pub mod common {
        include!(concat!(env!("OUT_DIR"), "/waypoint.common.rs"));
    }
}

// Helper modules
pub mod helpers;

// Example usage:
//
// ```
// use waypoint_schemas::proto::meilisearch::UserProfile;
// use waypoint_schemas::helpers::meilisearch::apply_user_profile_schema;
//
// async fn setup_meilisearch(client: &meilisearch_sdk::client::Client) -> Result<(), Error> {
//     // Apply the schema defined in the waypoint-schemas package
//     apply_user_profile_schema(client).await?;
//     Ok(())
// }
// ```
