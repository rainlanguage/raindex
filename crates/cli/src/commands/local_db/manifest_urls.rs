use anyhow::{Context, Result};
use clap::Parser;
use raindex_app_settings::yaml::{
    raindex::{RaindexYaml, RaindexYamlValidation},
    YamlParsable,
};
use std::io::{self, Write};
use url::Url;

#[derive(Debug, Clone, Parser)]
#[command(about = "Print local DB remote manifest URLs from settings YAML")]
pub struct ManifestUrls {
    #[clap(
        long,
        help = "Full YAML document that configures local DB remotes",
        value_name = "YAML"
    )]
    pub settings_yaml: String,
}

impl ManifestUrls {
    pub fn execute(self) -> Result<()> {
        let mut stdout = io::stdout();
        self.execute_to(&mut stdout)
    }

    fn execute_to<W: Write>(&self, writer: &mut W) -> Result<()> {
        let urls = manifest_urls_from_settings(&self.settings_yaml)?;

        for url in urls {
            writeln!(writer, "{url}")?;
        }

        Ok(())
    }
}

fn manifest_urls_from_settings(settings_yaml: &str) -> Result<Vec<Url>> {
    let raindex_yaml = RaindexYaml::new(
        vec![settings_yaml.to_string()],
        RaindexYamlValidation {
            local_db_remotes: true,
            ..Default::default()
        },
    )
    .context("failed to parse settings YAML")?;
    let remotes = raindex_yaml
        .get_local_db_remotes()
        .context("failed to parse local-db-remotes from settings YAML")?;

    Ok(remotes
        .into_iter()
        .collect::<std::collections::BTreeMap<_, _>>()
        .into_values()
        .map(|remote| remote.url)
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn render(args: ManifestUrls) -> Result<String> {
        let mut buffer = Vec::new();
        args.execute_to(&mut buffer)?;
        Ok(String::from_utf8(buffer).unwrap())
    }

    #[test]
    fn single_remote_url_prints_one_line() {
        let yaml = r#"
version: 6
local-db-remotes:
  raindex: https://example.com/manifest.yaml
"#;

        let output = render(ManifestUrls {
            settings_yaml: yaml.to_string(),
        })
        .unwrap();

        assert_eq!(output, "https://example.com/manifest.yaml\n");
    }

    #[test]
    fn duplicate_remote_urls_fail_settings_parsing() {
        let yaml = r#"
version: 6
local-db-remotes:
  raindex-a: https://example.com/manifest.yaml
  raindex-b: https://example.com/manifest.yaml
"#;

        let err = render(ManifestUrls {
            settings_yaml: yaml.to_string(),
        })
        .unwrap_err();
        let message = err.to_string();

        assert!(message.contains("failed to parse settings YAML"));
    }

    #[test]
    fn no_remotes_prints_nothing() {
        let yaml = r#"
version: 6
"#;

        let output = render(ManifestUrls {
            settings_yaml: yaml.to_string(),
        })
        .unwrap();

        assert_eq!(output, "");
    }

    #[test]
    fn default_output_is_only_one_url_per_line() {
        let yaml = r#"
version: 6
local-db-remotes:
  raindex-a: https://example.com/a.yaml
  raindex-b: https://example.com/b.yaml
"#;

        let output = render(ManifestUrls {
            settings_yaml: yaml.to_string(),
        })
        .unwrap();

        assert_eq!(
            output,
            "https://example.com/a.yaml\nhttps://example.com/b.yaml\n"
        );
    }

    #[test]
    fn missing_version_fails_before_printing_urls() {
        let yaml = r#"
local-db-remotes:
  raindex: https://example.com/manifest.yaml
"#;

        let err = render(ManifestUrls {
            settings_yaml: yaml.to_string(),
        })
        .unwrap_err();
        let message = err.to_string();

        assert!(message.contains("failed to parse settings YAML"));
    }
}
