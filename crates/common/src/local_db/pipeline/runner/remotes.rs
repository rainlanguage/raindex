use super::utils::RunnerTarget;
use crate::local_db::LocalDbError;
use flate2::read::GzDecoder;
use itertools::Itertools;
use raindex_app_settings::local_db_manifest::ManifestRaindex;
use raindex_app_settings::raindex::RaindexCfg;
use raindex_app_settings::remote::manifest::{fetch_multiple_manifests, ManifestMap};
use std::collections::HashMap;
use std::io::Read;
use url::Url;

pub(crate) fn collect_manifest_urls(
    raindexes: &HashMap<String, RaindexCfg>,
) -> Result<Vec<Url>, LocalDbError> {
    let urls = raindexes
        .iter()
        .map(|(key, raindex)| {
            raindex
                .local_db_remote
                .as_ref()
                .map(|remote| remote.url.clone())
                .ok_or_else(|| LocalDbError::MissingLocalDbRemote {
                    raindex_key: key.clone(),
                })
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(urls.into_iter().unique().collect())
}

/// Fetches manifests for all distinct remotes referenced by the raindexes.
pub async fn get_manifests(
    raindexes: &HashMap<String, RaindexCfg>,
) -> Result<ManifestMap, LocalDbError> {
    let urls = collect_manifest_urls(raindexes)?;
    if urls.is_empty() {
        return Ok(HashMap::new());
    }

    fetch_multiple_manifests(urls)
        .await
        .map_err(LocalDbError::ManifestFetch)
}

/// Downloads a gzip-compressed SQL dump and returns the decompressed contents.
pub async fn download_and_gunzip(url: &Url) -> Result<String, LocalDbError> {
    let response = reqwest::get(url.clone()).await?;
    if !response.status().is_success() {
        return Err(LocalDbError::HttpStatus {
            status: response.status().as_u16(),
        });
    }

    let bytes = response.bytes().await?.to_vec();
    let mut decoder = GzDecoder::new(bytes.as_slice());
    let mut out = String::new();
    decoder.read_to_string(&mut out)?;
    Ok(out)
}

/// Finds the manifest entry corresponding to the runner target, if present.
pub fn lookup_manifest_entry(
    manifest_map: &ManifestMap,
    target: &RunnerTarget,
) -> Option<ManifestRaindex> {
    manifest_map
        .get(&target.manifest_url)
        .and_then(|manifest| {
            manifest.find(
                target.inputs.raindex_id.chain_id,
                target.inputs.raindex_id.raindex_address,
            )
        })
        .cloned()
}

#[cfg(all(test, not(target_family = "wasm")))]
mod tests {
    use super::*;
    use alloy::primitives::{address, Bytes};
    use flate2::write::GzEncoder;
    use httpmock::prelude::*;
    use raindex_app_settings::local_db_manifest::{
        LocalDbManifest, ManifestNetwork, ManifestRaindex, MANIFEST_VERSION,
    };
    use raindex_app_settings::local_db_remotes::LocalDbRemoteCfg;
    use raindex_app_settings::raindex::RaindexCfg;
    use raindex_app_settings::remote::manifest::{FetchManifestError, ManifestMap};
    use raindex_app_settings::spec_version::SpecVersion;
    use raindex_app_settings::yaml::default_document;
    use std::collections::{HashMap, HashSet};
    use std::io::Write;
    use std::sync::Arc;
    use tokio::runtime::Runtime;
    use url::Url;

    use crate::local_db::pipeline::runner::utils::{
        build_runner_targets, parse_runner_settings, ParsedRunnerSettings, RunnerTarget,
    };
    use crate::local_db::LocalDbError;

    fn sample_settings_yaml() -> String {
        format!(
            r#"
version: {version}
networks:
  network-a:
    rpcs:
      - https://rpc.network/a
    chain-id: 1
  network-b:
    rpcs:
      - https://rpc.network/b
    chain-id: 2
subgraphs:
  network-a: https://subgraph.network/a
  network-b: https://subgraph.network/b
local-db-remotes:
  remote-a: https://remotes.example.com/a.yaml
  remote-b: https://remotes.example.com/b.yaml
local-db-sync:
  network-a:
    batch-size: 10
    max-concurrent-batches: 5
    retry-attempts: 3
    retry-delay-ms: 100
    rate-limit-delay-ms: 50
    finality-depth: 12
    bootstrap-block-threshold: 10000
    sync-interval-ms: 5000
  network-b:
    batch-size: 20
    max-concurrent-batches: 2
    retry-attempts: 4
    retry-delay-ms: 200
    rate-limit-delay-ms: 100
    finality-depth: 24
    bootstrap-block-threshold: 5000
    sync-interval-ms: 5000
raindexes:
  raindex-a:
    address: 0x00000000000000000000000000000000000000a1
    network: network-a
    subgraph: network-a
    local-db-remote: remote-a
    deployment-block: 111
  raindex-b:
    address: 0x00000000000000000000000000000000000000b2
    network: network-b
    subgraph: network-b
    local-db-remote: remote-b
    deployment-block: 222
  raindex-c:
    address: 0x00000000000000000000000000000000000000c3
    network: network-a
    subgraph: network-a
    local-db-remote: remote-a
    deployment-block: 333
"#,
            version = SpecVersion::current()
        )
    }

    fn parsed_settings() -> ParsedRunnerSettings {
        parse_runner_settings(&sample_settings_yaml()).expect("valid sample YAML")
    }

    fn build_runtime() -> Runtime {
        Runtime::new().expect("tokio runtime")
    }

    fn collect_urls_from_raindexes(raindexes: &HashMap<String, RaindexCfg>) -> Vec<Url> {
        collect_manifest_urls(raindexes).expect("urls")
    }

    fn update_remote_url(raindexes: &mut HashMap<String, RaindexCfg>, key: &str, url: &Url) {
        if let Some(raindex) = raindexes.get_mut(key) {
            let remote_key = raindex
                .local_db_remote
                .as_ref()
                .expect("raindex has remote")
                .key
                .clone();
            let remote = LocalDbRemoteCfg {
                document: default_document(),
                key: remote_key,
                url: url.clone(),
            };
            raindex.local_db_remote = Some(Arc::new(remote));
        }
    }

    fn sample_runner_target() -> (RunnerTarget, ManifestMap) {
        let parsed = parsed_settings();
        let targets = build_runner_targets(&parsed.raindexes, &parsed.syncs).unwrap();
        let target = targets
            .into_iter()
            .find(|t| t.raindex_key == "raindex-a")
            .expect("target raindex-a");

        let manifest_entry = ManifestRaindex {
            address: target.inputs.raindex_id.raindex_address,
            dump_url: Url::parse("https://example.com/dump.sql.gz").unwrap(),
            end_block: 123,
            end_block_hash: Bytes::from_static(&[0x01, 0x02, 0x03]),
            end_block_time_ms: 456,
        };

        let manifest = LocalDbManifest {
            manifest_version: MANIFEST_VERSION,
            db_schema_version: 1,
            networks: HashMap::from([(
                "network-a".to_string(),
                ManifestNetwork {
                    chain_id: target.inputs.raindex_id.chain_id,
                    raindexes: vec![manifest_entry.clone()],
                },
            )]),
        };

        let map = HashMap::from([(target.manifest_url.clone(), manifest)]);
        (target, map)
    }

    #[test]
    fn collect_manifest_urls_deduplicates() {
        let parsed = parsed_settings();
        let urls = collect_urls_from_raindexes(&parsed.raindexes);

        assert_eq!(urls.len(), 2);
        let unique: HashSet<Url> = urls.into_iter().collect();
        assert!(
            unique.contains(&Url::parse("https://remotes.example.com/a.yaml").expect("valid url"))
        );
        assert!(
            unique.contains(&Url::parse("https://remotes.example.com/b.yaml").expect("valid url"))
        );
    }

    #[test]
    fn collect_manifest_urls_handles_empty_raindexes() {
        let urls = collect_manifest_urls(&HashMap::new()).expect("empty vector");
        assert!(urls.is_empty());
    }

    #[test]
    fn get_manifests_returns_empty_map_for_no_raindexes() {
        let rt = build_runtime();
        let manifests = rt
            .block_on(get_manifests(&HashMap::new()))
            .expect("empty map");
        assert!(manifests.is_empty());
    }

    #[test]
    fn get_manifests_fetches_unique_urls() {
        let rt = build_runtime();

        let server_one = MockServer::start();
        let server_two = MockServer::start();

        let manifest_one = r#"
manifest-version: 1
db-schema-version: 3
networks:
  mainnet:
    chain-id: 1
    raindexes:
      - address: "0x0000000000000000000000000000000000000001"
        dump-url: "https://example.com/dump1.sql.gz"
        end-block: 100
        end-block-hash: "0x01"
        end-block-time-ms: 1000
"#;

        let manifest_two = r#"
manifest-version: 1
db-schema-version: 3
networks:
  goerli:
    chain-id: 5
    raindexes:
      - address: "0x0000000000000000000000000000000000000002"
        dump-url: "https://example.com/dump2.sql.gz"
        end-block: 200
        end-block-hash: "0x02"
        end-block-time-ms: 2000
"#;

        server_one.mock(|when, then| {
            when.method(GET).path("/");
            then.status(200)
                .header("content-type", "application/x-yaml")
                .body(manifest_one);
        });

        server_two.mock(|when, then| {
            when.method(GET).path("/");
            then.status(200)
                .header("content-type", "application/x-yaml")
                .body(manifest_two);
        });

        let mut parsed = parsed_settings();
        let url_one = Url::parse(&server_one.base_url()).unwrap();
        let url_two = Url::parse(&server_two.base_url()).unwrap();
        update_remote_url(&mut parsed.raindexes, "raindex-a", &url_one);
        update_remote_url(&mut parsed.raindexes, "raindex-c", &url_one);
        update_remote_url(&mut parsed.raindexes, "raindex-b", &url_two);

        let manifests = rt
            .block_on(get_manifests(&parsed.raindexes))
            .expect("manifests fetched");
        assert_eq!(manifests.len(), 2);
        assert!(manifests.contains_key(&url_one));
        assert!(manifests.contains_key(&url_two));
    }

    #[test]
    fn get_manifests_propagates_manifest_error() {
        let rt = build_runtime();
        let server_one = MockServer::start();
        let server_two = MockServer::start();

        let manifest_one = format!(
            r#"
manifest-version: {MANIFEST_VERSION}
db-schema-version: 3
networks: {{}}
"#
        );
        let manifest_two = r#"
manifest-version: 2
db-schema-version: 3
networks: {}
"#;

        server_one.mock(|when, then| {
            when.method(GET).path("/");
            then.status(200)
                .header("content-type", "application/x-yaml")
                .body(&manifest_one);
        });

        server_two.mock(|when, then| {
            when.method(GET).path("/");
            then.status(200)
                .header("content-type", "application/x-yaml")
                .body(manifest_two);
        });

        let mut parsed = parsed_settings();
        let url_one = Url::parse(&server_one.base_url()).unwrap();
        let url_two = Url::parse(&server_two.base_url()).unwrap();
        update_remote_url(&mut parsed.raindexes, "raindex-a", &url_one);
        update_remote_url(&mut parsed.raindexes, "raindex-c", &url_one);
        update_remote_url(&mut parsed.raindexes, "raindex-b", &url_two);

        let err = rt.block_on(get_manifests(&parsed.raindexes)).unwrap_err();
        match err {
            LocalDbError::ManifestFetch(FetchManifestError::Yaml(_)) => {}
            other => panic!("expected YAML manifest error, got {other:?}"),
        }
    }

    #[test]
    fn get_manifests_propagates_network_error() {
        let rt = build_runtime();
        let mut parsed = parsed_settings();
        let unreachable = Url::parse("nosuch://unreachable.example/manifest.yaml").unwrap();
        update_remote_url(&mut parsed.raindexes, "raindex-a", &unreachable);
        update_remote_url(&mut parsed.raindexes, "raindex-b", &unreachable);
        update_remote_url(&mut parsed.raindexes, "raindex-c", &unreachable);

        let err = rt.block_on(get_manifests(&parsed.raindexes)).unwrap_err();
        match err {
            LocalDbError::ManifestFetch(FetchManifestError::ReqwestError(_)) => {}
            other => panic!("expected HTTP manifest error, got {other:?}"),
        }
    }

    #[test]
    fn download_and_gunzip_success() {
        let rt = build_runtime();
        let server = MockServer::start();

        let body_text = "CREATE TABLE test (id INTEGER PRIMARY KEY);";
        let mut encoder = GzEncoder::new(Vec::new(), flate2::Compression::default());
        encoder
            .write_all(body_text.as_bytes())
            .expect("encode gzip");
        let gzipped = encoder.finish().expect("finish gzip");

        server.mock(|when, then| {
            when.method(GET).path("/dump.sql.gz");
            then.status(200)
                .header("content-type", "application/gzip")
                .body(gzipped.clone());
        });

        let url = Url::parse(&format!("{}/dump.sql.gz", server.base_url())).unwrap();
        let contents = rt
            .block_on(download_and_gunzip(&url))
            .expect("decompressed");
        assert_eq!(contents, body_text);
    }

    #[test]
    fn download_and_gunzip_non_success_status() {
        let rt = build_runtime();
        let server = MockServer::start();
        server.mock(|when, then| {
            when.method(GET).path("/dump.sql.gz");
            then.status(404);
        });
        let url = Url::parse(&format!("{}/dump.sql.gz", server.base_url())).unwrap();
        let err = rt.block_on(download_and_gunzip(&url)).unwrap_err();
        match err {
            LocalDbError::HttpStatus { status } => assert_eq!(status, 404),
            other => panic!("expected HttpStatus error, got {other:?}"),
        }
    }

    #[test]
    fn download_and_gunzip_invalid_gzip() {
        let rt = build_runtime();
        let server = MockServer::start();
        server.mock(|when, then| {
            when.method(GET).path("/dump.sql.gz");
            then.status(200).body("not-gzip");
        });
        let url = Url::parse(&format!("{}/dump.sql.gz", server.base_url())).unwrap();
        let err = rt.block_on(download_and_gunzip(&url)).unwrap_err();
        match err {
            LocalDbError::IoError(_) => {}
            other => panic!("expected IoError due to decompression failure, got {other:?}"),
        }
    }

    #[test]
    fn download_and_gunzip_network_error() {
        let rt = build_runtime();
        let url = Url::parse("nosuch://unreachable.example/dump.sql.gz").unwrap();

        let err = rt.block_on(download_and_gunzip(&url)).unwrap_err();
        match err {
            LocalDbError::Http(_) => {}
            other => panic!("expected Http error for unreachable URL, got {other:?}"),
        }
    }

    #[test]
    fn lookup_manifest_entry_found() {
        let (target, manifest_map) = sample_runner_target();
        let entry = lookup_manifest_entry(&manifest_map, &target).expect("entry found");
        assert_eq!(entry.address, target.inputs.raindex_id.raindex_address);
    }

    #[test]
    fn lookup_manifest_entry_missing_address() {
        let (mut target, mut manifest_map) = sample_runner_target();
        target.inputs.raindex_id.raindex_address =
            address!("000000000000000000000000000000000000dead");
        assert!(lookup_manifest_entry(&manifest_map, &target).is_none());

        manifest_map.clear();
        assert!(lookup_manifest_entry(&manifest_map, &target).is_none());
    }

    #[test]
    fn lookup_manifest_entry_chain_id_mismatch() {
        let (target, mut manifest_map) = sample_runner_target();
        if let Some(manifest) = manifest_map.get_mut(&target.manifest_url) {
            if let Some(network) = manifest.networks.get_mut("network-a") {
                network.chain_id = target.inputs.raindex_id.chain_id + 1;
            }
        }

        assert!(lookup_manifest_entry(&manifest_map, &target).is_none());
    }
}
