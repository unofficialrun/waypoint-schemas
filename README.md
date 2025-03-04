# Schemas for Waypoint

[![Rust](https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Protocol Buffers](https://img.shields.io/badge/Protocol%20Buffers-004D98?style=for-the-badge&logo=protocol-buffers&logoColor=white)](https://developers.google.com/protocol-buffers)
[![Meilisearch](https://img.shields.io/badge/Meilisearch-6D4A9C?style=for-the-badge&logo=meilisearch&logoColor=white)](https://www.meilisearch.com/)
[![License](https://img.shields.io/badge/License-MIT-000000?style=for-the-badge)](https://opensource.org/licenses/MIT)

This crate provides schema definitions for Waypoint services using Protocol Buffers (protobuf).

## Overview

This repo defines a set of common schema formats that can be shared between different Rust applications for use with Waypoint and other use cases. It also ensures consistency and type safety when exchanging data between services.

## Features

- Protocol Buffer definitions for all data models
- Helper functions for working with schemas in Rust
- Serde-compatible data structures
- Integration with Meilisearch SDK

## Schemas

### Meilisearch

- **User Profiles**: Schema for storing and querying Farcaster user profiles
- **Index Settings**: Configuration for Meilisearch indexes

### Common Types

- Error handling
- Pagination structures
- Timestamp formatting
- Farcaster-specific types

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
waypoint-schemas = { git = "https://github.com/unofficialrun/waypoint-schemas.git" }
```

### Example

```rust
use waypoint_schemas::proto::meilisearch::UserProfile;
use waypoint_schemas::helpers::meilisearch::apply_user_profile_schema;

async fn setup_meilisearch(client: &meilisearch_sdk::client::Client) -> Result<(), Error> {
    // Apply the schema defined in the waypoint-schemas package
    apply_user_profile_schema(client).await?;
    Ok(())
}
```

## Development

To build the project:

```bash
cargo build
```

To run tests:

```bash
cargo test
```

## License

MIT
