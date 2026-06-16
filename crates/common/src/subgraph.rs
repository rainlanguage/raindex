use raindex_subgraph_client::RaindexSubgraphClient;
use serde::{Deserialize, Serialize};
use url::{ParseError, Url};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct SubgraphArgs {
    pub url: String,
}

impl SubgraphArgs {
    pub fn to_subgraph_client(&self) -> Result<RaindexSubgraphClient, ParseError> {
        Ok(RaindexSubgraphClient::new(Url::parse(self.url.as_str())?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_subgraph_client_ok() {
        let url = "https://api.thegraph.com/subgraphs/name/org1/sg1";
        let subgraph_args = SubgraphArgs {
            url: url.to_string(),
        };
        let subgraph_client = subgraph_args.to_subgraph_client().unwrap();
        assert_eq!(subgraph_client.url().as_str(), url);

        let url = "https://api.thegraph.com/subgraphs/name/org1/sg1?version=latest&format=json";
        let subgraph_args = SubgraphArgs {
            url: url.to_string(),
        };
        let subgraph_client = subgraph_args.to_subgraph_client().unwrap();
        assert_eq!(subgraph_client.url().as_str(), url);
    }

    #[test]
    fn test_to_subgraph_client_err() {
        let url = "api.thegraph.com/subgraphs/name/org1/sg1".to_string();
        let subgraph_args = SubgraphArgs { url };
        let err = subgraph_args.to_subgraph_client().unwrap_err();
        assert_eq!(err, ParseError::RelativeUrlWithoutBase);

        let url = "https:///".to_string();
        let subgraph_args = SubgraphArgs { url };
        let err = subgraph_args.to_subgraph_client().unwrap_err();
        assert_eq!(err, ParseError::EmptyHost);

        let url = "".to_string();
        let subgraph_args = SubgraphArgs { url };
        let err = subgraph_args.to_subgraph_client().unwrap_err();
        // Empty URL results in RelativeUrlWithoutBase error
        assert_eq!(err, ParseError::RelativeUrlWithoutBase);

        let url = ":".to_string();
        let subgraph_args = SubgraphArgs { url };
        let err = subgraph_args.to_subgraph_client().unwrap_err();
        assert_eq!(err, ParseError::RelativeUrlWithoutBase);
    }
}
