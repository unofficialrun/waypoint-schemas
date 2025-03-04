fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Compile the proto files
    prost_build::compile_protos(
        &[
            "proto/meilisearch/user_profile.proto",
            "proto/meilisearch/settings.proto",
            "proto/common/types.proto",
        ],
        &["proto"],
    )?;
    Ok(())
}
