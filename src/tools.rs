use rig::prelude::*;
use rig::tool::{ToolContext, ToolExecutionError};
use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Deserialize, JsonSchema)]
pub struct ExtractDomainArgs {
    /// The URL to extract the host name from
    url: String,
}

pub struct ExtractDomain;

impl Tool for ExtractDomain {
    const NAME: &'static str = "extract_domain";
    type Args = ExtractDomainArgs;
    type Output = String;
    type Error = ToolExecutionError;

    fn description(&self) -> String {
        "Extract the host name from a URL".to_string()
    }

    fn parameters(&self) -> serde_json::Value {
        schemars::schema_for!(ExtractDomainArgs).to_value()
    }

    async fn call(
        &self,
        _ctx: &mut ToolContext,
        args: ExtractDomainArgs,
    ) -> Result<String, ToolExecutionError> {
        host_of(&args.url).ok_or_else(|| ToolExecutionError::invalid_args("url has no host"))
    }
}

fn host_of(url: &str) -> Option<String> {
    let rest = url.split_once("://").map_or(url, |(_, r)| r);
    let authority = rest.split(['/', '?', '#']).next()?;
    let host = authority.rsplit_once('@').map_or(authority, |(_, h)| h);
    let host = host.split(':').next()?;

    (!host.is_empty()).then(|| host.to_lowercase())
}

#[cfg(test)]
mod tests {
    use super::host_of;

    #[test]
    fn extracts_host() {
        assert_eq!(
            host_of("https://Example.com:8443/a/b?c=d").as_deref(),
            Some("example.com")
        );
        assert_eq!(host_of("user:pw@Host.org/x").as_deref(), Some("host.org"));
        assert_eq!(host_of("example.com").as_deref(), Some("example.com"));
    }

    #[test]
    fn rejects_missing_host() {
        assert_eq!(host_of("https:///path"), None);
        assert_eq!(host_of(""), None);
    }
}
