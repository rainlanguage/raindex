use alloy::primitives::hex::{decode, FromHexError};
use rain_metadata::{Error as RainMetadataError, KnownMagic, RainMetaDocumentV1Item};
use std::string::FromUtf8Error;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TryDecodeRainlangSourceError {
    #[error(transparent)]
    FromHexError(#[from] FromHexError),
    #[error(transparent)]
    FromUtf8Error(#[from] FromUtf8Error),
    #[error("Meta bytes do not start with RainlangSourceV1 Magic")]
    MissingRainlangSourceV1,
    #[error(transparent)]
    RainMetadataError(#[from] RainMetadataError),
    #[error("Rainlang Source does not match rainlang")]
    RainlangSourceMismatch,
}

pub trait TryDecodeRainlangSource {
    fn try_decode_rainlangsource(&self) -> Result<String, TryDecodeRainlangSourceError>;
}

impl TryDecodeRainlangSource for String {
    fn try_decode_rainlangsource(&self) -> Result<String, TryDecodeRainlangSourceError> {
        // Ensure meta has expected magic prefix
        let meta_bytes = decode(self)?;
        if !meta_bytes
            .clone()
            .starts_with(&KnownMagic::RainMetaDocumentV1.to_prefix_bytes())
        {
            return Err(TryDecodeRainlangSourceError::MissingRainlangSourceV1);
        }

        // Decode meta to string
        let meta_bytes_slice = meta_bytes.as_slice();
        let rain_meta_document_item = RainMetaDocumentV1Item::cbor_decode(meta_bytes_slice)?;
        let rainlangsource_item = rain_meta_document_item
            .first()
            .ok_or(TryDecodeRainlangSourceError::MissingRainlangSourceV1)?;
        let rainlangsource = String::from_utf8(rainlangsource_item.payload.to_vec())?;

        Ok(rainlangsource)
    }
}

#[cfg(test)]
mod tests {
    use raindex_subgraph_client::types::common::SgBytes;
    use raindex_subgraph_client::types::common::SgRainMetaV1;

    use super::*;

    const RAINLANG_SOURCE: &str = r#"/* 0. calculate-io */ 
using-words-from 0xFe2411CDa193D9E4e83A5c234C7Fd320101883aC
max-output: max-value(),
io: if(
  equal-to(
    output-token()
    0x1d80c49bbbcd1c0911346656b529df9e5c2f783d
  )
  12
  inv(12)
);

/* 1. handle-io */ 
:;"#;
    const META: &str = "0xff0a89c674ee7874a30058ef2f2a20302e2063616c63756c6174652d696f202a2f200a7573696e672d776f7264732d66726f6d203078466532343131434461313933443945346538334135633233344337466433323031303138383361430a6d61782d6f75747075743a206d61782d76616c756528292c0a696f3a206966280a2020657175616c2d746f280a202020206f75747075742d746f6b656e28290a202020203078316438306334396262626364316330393131333436363536623532396466396535633266373833640a2020290a202031320a2020696e76283132290a293b0a0a2f2a20312e2068616e646c652d696f202a2f200a3a3b011bff13109e41336ff20278186170706c69636174696f6e2f6f637465742d73747265616d";

    #[test]
    fn test_try_decode_rainlangsource() {
        let meta: SgRainMetaV1 = SgBytes(META.to_string());
        let source = meta.0.try_decode_rainlangsource().unwrap();
        assert_eq!(source, RAINLANG_SOURCE);
    }

    #[test]
    fn test_try_decode_rainlangsource_missing() {
        let meta: SgRainMetaV1 = SgBytes("".to_string());
        let source = meta.0.try_decode_rainlangsource().unwrap_err();
        assert_eq!(
            source.to_string(),
            TryDecodeRainlangSourceError::MissingRainlangSourceV1.to_string()
        );
    }

    #[test]
    fn test_try_decode_rainlangsource_invalid() {
        let meta: SgRainMetaV1 = SgBytes("invalid".to_string());
        let source = meta.0.try_decode_rainlangsource().unwrap_err();
        assert_eq!(
            source.to_string(),
            TryDecodeRainlangSourceError::FromHexError(FromHexError::OddLength).to_string()
        );
    }

    #[test]
    fn test_try_decode_rainlangsource_corrupt() {
        let meta: SgRainMetaV1 = SgBytes("0xff0a89c674ee7874".to_string());
        let source = meta.0.try_decode_rainlangsource().unwrap_err();
        assert_eq!(
            source.to_string(),
            TryDecodeRainlangSourceError::RainMetadataError(RainMetadataError::CorruptMeta)
                .to_string()
        );
    }
}
