use crate::yaml::{require_string, FieldErrorKind, YamlError, YamlParsableString};
use std::sync::{Arc, RwLock};
use strict_yaml_rust::StrictYaml;

#[derive(Clone, Debug)]
pub struct SpecVersion;

pub const CURRENT_SPEC_VERSION: &str = "5";

impl SpecVersion {
    pub fn current() -> String {
        CURRENT_SPEC_VERSION.to_string()
    }

    pub fn is_current(version: &str) -> bool {
        version == CURRENT_SPEC_VERSION
    }

    pub fn validate(documents: Vec<Arc<RwLock<StrictYaml>>>) -> Result<(), YamlError> {
        let version = Self::parse_from_yaml(documents)?;
        if !Self::is_current(&version) {
            return Err(YamlError::Field {
                kind: FieldErrorKind::InvalidValue {
                    field: "version".to_string(),
                    reason: format!(
                        "spec version mismatch: expected '{}', found '{}'",
                        Self::current(),
                        version
                    ),
                },
                location: "root".to_string(),
            });
        }
        Ok(())
    }
}

impl YamlParsableString for SpecVersion {
    fn parse_from_yaml(documents: Vec<Arc<RwLock<StrictYaml>>>) -> Result<String, YamlError> {
        if documents.is_empty() {
            return Err(YamlError::EmptyFile);
        }

        documents
            .iter()
            .enumerate()
            .try_fold(None, |parsed_version, (index, document)| {
                let location = if index == 0 {
                    "root".to_string()
                } else {
                    format!("document {}", index + 1)
                };
                let version = {
                    let document_read = document.read().map_err(|_| YamlError::ReadLockError)?;
                    require_string(&document_read, Some("version"), Some(location.clone()))?
                };

                match parsed_version {
                    Some(existing_version) if existing_version != version => {
                        Err(YamlError::Field {
                            kind: FieldErrorKind::InvalidValue {
                                field: "version".to_string(),
                                reason: format!(
                                    "spec version mismatch: expected '{}', found '{}'",
                                    existing_version, version
                                ),
                            },
                            location,
                        })
                    }
                    Some(existing_version) => Ok(Some(existing_version)),
                    None => Ok(Some(version)),
                }
            })?
            .ok_or_else(|| YamlError::Field {
                kind: FieldErrorKind::Missing("version".to_string()),
                location: "root".to_string(),
            })
    }

    fn parse_from_yaml_optional(_: Arc<RwLock<StrictYaml>>) -> Result<Option<String>, YamlError> {
        Err(YamlError::InvalidTraitFunction)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::yaml::{tests::get_document, FieldErrorKind};

    #[test]
    fn test_is_current() {
        assert!(SpecVersion::is_current("5"));
        assert!(!SpecVersion::is_current("1"));
    }

    #[test]
    fn test_current() {
        assert_eq!(SpecVersion::current(), "5");
    }

    #[test]
    fn test_parse_from_yaml_missing_spec_version() {
        let yaml = r#"
test: test
"#;

        let error = SpecVersion::parse_from_yaml(vec![get_document(yaml)]).unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::Missing("version".to_string()),
                location: "root".to_string()
            }
        )
    }

    #[test]
    fn test_parse_from_yaml_optional() {
        let yaml = r#"
test: test
"#;
        let error = SpecVersion::parse_from_yaml_optional(get_document(yaml)).unwrap_err();
        assert_eq!(error, YamlError::InvalidTraitFunction);
    }

    #[test]
    fn test_parse_from_yaml_consistent_versions() {
        let documents = vec![
            get_document(
                r#"
version: "3"
"#,
            ),
            get_document(
                r#"
version: "3"
"#,
            ),
        ];

        let version = SpecVersion::parse_from_yaml(documents).unwrap();
        assert_eq!(version, "3");
    }

    #[test]
    fn test_parse_from_yaml_missing_version() {
        let documents = vec![
            get_document(
                r#"
version: "3"
"#,
            ),
            get_document(
                r#"
name: test
"#,
            ),
        ];

        let error = SpecVersion::parse_from_yaml(documents).unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::Missing("version".to_string()),
                location: "document 2".to_string()
            }
        )
    }

    #[test]
    fn test_parse_from_yaml_mismatched_versions() {
        let documents = vec![
            get_document(
                r#"
version: "3"
"#,
            ),
            get_document(
                r#"
version: "2"
"#,
            ),
        ];

        let error = SpecVersion::parse_from_yaml(documents).unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::InvalidValue {
                    field: "version".to_string(),
                    reason: "spec version mismatch: expected '3', found '2'".to_string()
                },
                location: "document 2".to_string()
            }
        )
    }

    #[test]
    fn test_validate_current_version() {
        let yaml = format!(
            r#"
version: {}
"#,
            SpecVersion::current()
        );
        let documents = vec![get_document(&yaml)];
        assert!(SpecVersion::validate(documents).is_ok());
    }

    #[test]
    fn test_validate_current_version_multiple_documents() {
        let yaml = format!(
            r#"
version: {}
"#,
            SpecVersion::current()
        );
        let documents = vec![get_document(&yaml), get_document(&yaml)];
        assert!(SpecVersion::validate(documents).is_ok());
    }

    #[test]
    fn test_validate_missing_version() {
        let yaml = r#"
test: test
"#;
        let documents = vec![get_document(yaml)];
        let error = SpecVersion::validate(documents).unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::Missing("version".to_string()),
                location: "root".to_string()
            }
        )
    }

    #[test]
    fn test_validate_incorrect_version() {
        let yaml = r#"
version: "1"
"#;
        let documents = vec![get_document(yaml)];
        let error = SpecVersion::validate(documents).unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::InvalidValue {
                    field: "version".to_string(),
                    reason: format!(
                        "spec version mismatch: expected '{}', found '1'",
                        SpecVersion::current()
                    )
                },
                location: "root".to_string()
            }
        )
    }

    #[test]
    fn test_validate_mismatched_versions() {
        let yaml1 = format!(
            r#"
version: {}
"#,
            SpecVersion::current()
        );
        let yaml2 = r#"
version: "1"
"#;
        let documents = vec![get_document(&yaml1), get_document(yaml2)];
        let error = SpecVersion::validate(documents).unwrap_err();
        assert_eq!(
            error,
            YamlError::Field {
                kind: FieldErrorKind::InvalidValue {
                    field: "version".to_string(),
                    reason: format!(
                        "spec version mismatch: expected '{}', found '1'",
                        SpecVersion::current()
                    )
                },
                location: "document 2".to_string()
            }
        )
    }
}
