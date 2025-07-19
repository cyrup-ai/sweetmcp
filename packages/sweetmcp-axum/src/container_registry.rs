use std::{fs, io::Read, path::Path, str::FromStr};

use anyhow::anyhow;
use docker_credential::{CredentialRetrievalError, DockerCredential};
use flate2::read::GzDecoder;
use oci_client::{Client, Reference, manifest, manifest::OciDescriptor, secrets::RegistryAuth};
use serde::{Deserialize, Serialize};
use sigstore::{
    cosign::{ClientBuilder, CosignCapabilities, verify_constraints},
    errors::SigstoreVerifyConstraintsError,
    registry::{Auth, OciReference},
    trust::ManualTrustRoot,
};
use tar::Archive;

// Docker manifest format v2
#[derive(Debug, Serialize, Deserialize)]
struct DockerManifest {
    #[serde(rename = "schemaVersion")]
    schema_version: u32,
    #[serde(rename = "mediaType")]
    media_type: String,
    config: DockerManifestConfig,
    layers: Vec<DockerManifestLayer>,
}

#[derive(Debug, Serialize, Deserialize)]
struct DockerManifestConfig {
    #[serde(rename = "mediaType")]
    media_type: String,
    size: u64,
    digest: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct DockerManifestLayer {
    #[serde(rename = "mediaType")]
    media_type: String,
    size: u64,
    digest: String,
}

fn build_auth(reference: &Reference) -> RegistryAuth {
    let server = reference
        .resolve_registry()
        .strip_suffix('/')
        .unwrap_or_else(|| reference.resolve_registry());

    // if cli.anonymous {
    //     return RegistryAuth::Anonymous;
    // }

    match docker_credential::get_credential(server) {
        Err(CredentialRetrievalError::ConfigNotFound) => RegistryAuth::Anonymous,
        Err(CredentialRetrievalError::NoCredentialConfigured) => RegistryAuth::Anonymous,
        Err(e) => {
            log::info!(
                "Error retrieving docker credentials: {}. Using anonymous auth",
                e
            );
            RegistryAuth::Anonymous
        }
        Ok(DockerCredential::UsernamePassword(username, password)) => {
            log::info!("Found docker credentials");
            RegistryAuth::Basic(username, password)
        }
        Ok(DockerCredential::IdentityToken(_)) => {
            log::info!(
                "Cannot use contents of docker config, identity token not supported. Using anonymous auth"
            );
            RegistryAuth::Anonymous
        }
    }
}

async fn verify_image_signature(image_reference: &str) -> Result<bool, anyhow::Error> {
    log::info!("Verifying signature for {}", image_reference);

    // Create a minimal trust repository
    let repo = ManualTrustRoot::default();
    let auth = &Auth::Anonymous;

    // Create a client builder and build the client
    let client_builder = ClientBuilder::default();

    // Create client with trust repository
    let client_builder = match client_builder.with_trust_repository(&repo) {
        Ok(builder) => builder,
        Err(e) => return Err(anyhow!("Failed to set up trust repository: {}", e)),
    };

    // Build the client
    let mut client = match client_builder.build() {
        Ok(client) => client,
        Err(e) => return Err(anyhow!("Failed to build Sigstore client: {}", e)),
    };

    // Parse the reference
    let image_ref = match OciReference::from_str(image_reference) {
        Ok(reference) => reference,
        Err(e) => return Err(anyhow!("Invalid image reference: {}", e)),
    };

    // Triangulate to find the signature image and source digest
    let (cosign_signature_image, source_image_digest) =
        match client.triangulate(&image_ref, auth).await {
            Ok((sig_image, digest)) => (sig_image, digest),
            Err(e) => {
                log::warn!("Failed to triangulate image: {}", e);
                return Ok(false); // No signatures found
            }
        };

    // Get trusted signature layers
    let signature_layers = match client
        .trusted_signature_layers(auth, &source_image_digest, &cosign_signature_image)
        .await
    {
        Ok(layers) => layers,
        Err(e) => {
            log::warn!("Failed to get trusted signature layers: {}", e);
            return Ok(false);
        }
    };

    if signature_layers.is_empty() {
        log::warn!("No valid signatures found for {}", image_reference);
        return Ok(false);
    }

    // Empty verification constraints means we're just checking for valid signatures
    let verification_constraints = Vec::new();

    // Verify the constraints
    match verify_constraints(&signature_layers, verification_constraints.iter()) {
        Ok(()) => {
            log::info!("Signature verification successful for {}", image_reference);
            Ok(true)
        }
        Err(SigstoreVerifyConstraintsError {
            unsatisfied_constraints,
        }) => {
            log::warn!(
                "Signature verification failed for {}: {:?}",
                image_reference,
                unsatisfied_constraints
            );
            Ok(false)
        }
    }
}

pub async fn pull_and_extract_oci_image(
    image_reference: &str,
    target_file_path: &str,
    local_output_path: &str,
    verify_signature: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if Path::new(local_output_path).exists() {
        log::info!(
            "Plugin {} already cached at: {}. Skipping downloading.",
            image_reference,
            local_output_path
        );
        return Ok(());
    }

    log::info!("Pulling {} ...", image_reference);

    let client_config = oci_client::client::ClientConfig::default();
    let client = Client::new(client_config);

    let reference = Reference::try_from(image_reference)?;
    let auth = build_auth(&reference);

    // Verify the image signature if it's an OCI image and verification is enabled
    if verify_signature {
        log::info!("Signature verification enabled for {}", image_reference);
        match verify_image_signature(image_reference).await {
            Ok(verified) => {
                if !verified {
                    return Err(format!(
                        "No valid signatures found for the image {}",
                        image_reference
                    )
                    .into());
                }
            }
            Err(e) => {
                return Err(format!("Image signature verification failed: {}", e).into());
            }
        }
    } else {
        log::warn!("Signature verification disabled for {}", image_reference);
    }

    // Accept both OCI and Docker manifest types
    let manifest = client
        .pull(
            &reference,
            &auth,
            vec![
                manifest::IMAGE_MANIFEST_MEDIA_TYPE,
                manifest::IMAGE_DOCKER_LAYER_GZIP_MEDIA_TYPE,
            ],
        )
        .await?;

    for layer in manifest.layers.iter() {
        let mut buf = Vec::new();
        let desc = OciDescriptor {
            digest: layer.sha256_digest().clone(),
            media_type: "application/vnd.docker.image.rootfs.diff.tar.gzip".to_string(),
            ..Default::default()
        };
        client.pull_blob(&reference, &desc, &mut buf).await?; // Already fixed, no change needed here

        let gz_extract = GzDecoder::new(&buf[..]);
        let mut archive_extract = Archive::new(gz_extract);

        for entry_result in archive_extract.entries()? {
            match entry_result {
                Ok(mut entry) => {
                    if let Ok(path) = entry.path() {
                        let path_str = path.to_string_lossy();
                        if path_str.ends_with(target_file_path) || path_str.ends_with("plugin.wasm")
                        {
                            if let Some(parent) = Path::new(local_output_path).parent() {
                                fs::create_dir_all(parent)?;
                            }
                            let mut content = Vec::new();
                            entry.read_to_end(&mut content)?;
                            fs::write(local_output_path, content)?;
                            log::info!("Successfully extracted to: {}", local_output_path);
                            return Ok(());
                        }
                    }
                }
                Err(e) => log::info!("Error during extraction: {}", e),
            }
        }
    }

    Err("Target file not found in any layer".into())
}
