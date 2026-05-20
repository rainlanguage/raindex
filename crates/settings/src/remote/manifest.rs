use std::collections::HashMap;

use crate::local_db_manifest::{parse_manifest_doc, LocalDbManifest};
use crate::yaml::{load_yaml, YamlError};
use futures::future::try_join_all;
use thiserror::Error;
use url::Url;

pub type ManifestMap = HashMap<Url, LocalDbManifest>;

#[derive(Error, Debug)]
pub enum FetchManifestError {
    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),
    #[error(transparent)]
    Yaml(#[from] YamlError),
    #[error("manifest versions differ between URLs; {url} returned a conflicting version")]
    InconsistentManifestVersions { url: Url },
}

pub async fn fetch_manifest(url: Url) -> Result<LocalDbManifest, FetchManifestError> {
    let text = reqwest::get(url.to_string()).await?.text().await?;
    let doc = load_yaml(&text)?;
    let manifest = parse_manifest_doc(&doc)?;
    Ok(manifest)
}

pub async fn fetch_multiple_manifests(urls: Vec<Url>) -> Result<ManifestMap, FetchManifestError> {
    let results = try_join_all(urls.into_iter().map(|url| async {
        let manifest = fetch_manifest(url.clone()).await?;
        Ok::<(Url, LocalDbManifest), FetchManifestError>((url, manifest))
    }))
    .await?;

    let manifests_by_url =
        results
            .into_iter()
            .fold(HashMap::new(), |mut acc: ManifestMap, (url, manifest)| {
                acc.insert(url, manifest);
                acc
            });

    Ok(manifests_by_url)
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;
    use alloy::primitives::{address, Bytes};
    use httpmock::MockServer;

    #[tokio::test]
    async fn test_fetch_manifest_happy_path() {
        let server = MockServer::start_async().await;
        let yaml = r#"
manifest-version: 1
db-schema-version: 3
networks:
  mainnet:
    chain-id: 1
    raindexes:
      - address: "0x0000000000000000000000000000000000000001"
        dump-url: "http://example.com/dump1"
        end-block: 123
        end-block-hash: "0x0abc"
        end-block-time-ms: 1000
"#;

        server
            .mock_async(|when, then| {
                when.method("GET").path("/");
                then.status(200)
                    .header("content-type", "application/x-yaml")
                    .body(yaml);
            })
            .await;

        let manifest = fetch_manifest(Url::parse(&server.base_url()).unwrap())
            .await
            .unwrap();

        assert_eq!(manifest.manifest_version, 1);
        assert_eq!(manifest.db_schema_version, 3);
        let net = manifest.networks.get("mainnet").unwrap();
        assert_eq!(net.chain_id, 1);
        assert_eq!(net.raindexes.len(), 1);
        assert_eq!(
            net.raindexes[0].address,
            address!("0x0000000000000000000000000000000000000001")
        );
        assert_eq!(net.raindexes[0].end_block, 123);
        assert_eq!(
            net.raindexes[0].end_block_hash,
            Bytes::from_str("0x0abc").unwrap()
        );
        assert_eq!(net.raindexes[0].end_block_time_ms, 1000);

        // find helper
        let found = manifest.find(1, address!("0x0000000000000000000000000000000000000001"));
        assert!(found.is_some());
    }

    #[tokio::test]
    async fn test_fetch_manifest_unknown_fields_ignored() {
        let server = MockServer::start_async().await;
        let yaml = r#"
manifest-version: 1
db-schema-version: 3
extra-root: ignored
networks:
  goerli:
    chain-id: 5
    extra: ignored
    raindexes:
      - address: "0x0000000000000000000000000000000000000002"
        dump-url: "http://example.com/dump2"
        end-block: 555
        end-block-hash: "0x0def"
        end-block-time-ms: 2000
        extra-raindex: ignored
"#;

        server
            .mock_async(|when, then| {
                when.method("GET").path("/");
                then.status(200).body(yaml);
            })
            .await;

        let manifest = fetch_manifest(Url::parse(&server.base_url()).unwrap())
            .await
            .unwrap();

        assert!(manifest.networks.contains_key("goerli"));
        let net = manifest.networks.get("goerli").unwrap();
        assert_eq!(net.chain_id, 5);
        assert_eq!(net.raindexes.len(), 1);
    }

    #[tokio::test]
    async fn test_fetch_manifest_invalid_yaml() {
        let server = MockServer::start_async().await;
        let yaml = "manifest-version: [\n"; // malformed

        server
            .mock_async(|when, then| {
                when.method("GET").path("/");
                then.status(200).body(yaml);
            })
            .await;

        let err = fetch_manifest(Url::parse(&server.base_url()).unwrap())
            .await
            .unwrap_err();
        match err {
            // Some malformed YAML inputs are surfaced as ScanError by the loader
            FetchManifestError::Yaml(YamlError::ScanError(_)) => {}
            // In certain cases, incomplete structures may parse into BadValue and
            // be reported later as a field error; accept that as invalid YAML too.
            FetchManifestError::Yaml(YamlError::Field { .. }) => {}
            _ => panic!("expected YAML scan or field error"),
        }
    }

    #[tokio::test]
    async fn test_fetch_manifest_invalid_types_and_values() {
        let server = MockServer::start_async().await;
        let yaml = r#"
manifest-version: 1
db-schema-version: 3
networks:
  mainnet:
    chain-id: 1
    raindexes:
      - address: 123 # invalid type
        dump-url: "not-a-url"
        end-block: 0
        end-block-hash: 999 # invalid type
        end-block-time-ms: 0
"#;

        server
            .mock_async(|when, then| {
                when.method("GET").path("/");
                then.status(200).body(yaml);
            })
            .await;

        let err = fetch_manifest(Url::parse(&server.base_url()).unwrap())
            .await
            .unwrap_err();
        match err {
            FetchManifestError::Yaml(YamlError::Field { .. }) => {}
            _ => panic!("expected field error"),
        }
    }

    #[tokio::test]
    async fn test_fetch_manifest_http_error_path() {
        // Use an unsupported scheme to deterministically trigger a reqwest error
        let url = Url::parse("ftp://example.com").unwrap();
        let err = fetch_manifest(url).await.unwrap_err();
        match err {
            FetchManifestError::ReqwestError(_) => {}
            other => panic!("expected reqwest error, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn test_fetch_multiple_manifests_happy_path() {
        let server_one = MockServer::start_async().await;
        let server_two = MockServer::start_async().await;

        let yaml_one = r#"
manifest-version: 1
db-schema-version: 3
networks:
  mainnet:
    chain-id: 1
    raindexes:
      - address: "0x0000000000000000000000000000000000000001"
        dump-url: "http://example.com/dump1"
        end-block: 123
        end-block-hash: "0x0abc"
        end-block-time-ms: 1000
"#;

        let yaml_two = r#"
manifest-version: 1
db-schema-version: 3
networks:
  goerli:
    chain-id: 5
    raindexes:
      - address: "0x0000000000000000000000000000000000000002"
        dump-url: "http://example.com/dump2"
        end-block: 555
        end-block-hash: "0x0def"
        end-block-time-ms: 2000
"#;

        server_one
            .mock_async(|when, then| {
                when.method("GET").path("/");
                then.status(200)
                    .header("content-type", "application/x-yaml")
                    .body(yaml_one);
            })
            .await;

        server_two
            .mock_async(|when, then| {
                when.method("GET").path("/");
                then.status(200)
                    .header("content-type", "application/x-yaml")
                    .body(yaml_two);
            })
            .await;

        let url_one = Url::parse(&server_one.base_url()).unwrap();
        let url_two = Url::parse(&server_two.base_url()).unwrap();

        let manifests = fetch_multiple_manifests(vec![url_one.clone(), url_two.clone()])
            .await
            .unwrap();

        assert_eq!(manifests.len(), 2);
        assert_eq!(manifests.get(&url_one).unwrap().manifest_version, 1);
        assert_eq!(manifests.get(&url_two).unwrap().manifest_version, 1);
    }
}
