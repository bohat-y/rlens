mod tools;

use anyhow::bail;
use rig::operation::Completion;
use rig::prelude::*;
use rig::providers::{ollama, openai, zai};

use tools::ExtractDomain;

const PREAMBLE: &str = "You are a research assistant. Use the provided tools when they help \
answer the question, and say so when you cannot answer.";

/// Picks the model from `RLENS_PROVIDER` (`ollama`, `zai`, `openai`; default `ollama`)
/// and `RLENS_MODEL`. Keys come from `ZAI_API_KEY` / `OPENAI_API_KEY`.
fn model_from_env() -> anyhow::Result<DynModel<Completion>> {
    let provider = std::env::var("RLENS_PROVIDER").unwrap_or_else(|_| "ollama".to_string());
    let model = std::env::var("RLENS_MODEL").ok();
    let model = model.as_deref();

    Ok(match provider.as_str() {
        "ollama" => ollama::Ollama::from_env()?
            .completion(model.unwrap_or("qwen3:14b"))
            .into(),
        "zai" => zai::from_env()?
            .completion(model.unwrap_or(zai::GLM_4_6))
            .into(),
        "openai" => openai::OpenAI::from_env()?
            .completion(model.unwrap_or(openai::GPT_5_5))
            .into(),
        other => bail!("unknown RLENS_PROVIDER `{other}`, expected ollama, zai or openai"),
    })
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let agent = AgentBuilder::new(model_from_env()?)
        .preamble(PREAMBLE)
        .tool(ExtractDomain)
        .build();

    let question = std::env::args().skip(1).collect::<Vec<_>>().join(" ");
    let question = if question.is_empty() {
        "Which domain does https://Example.com:8443/a/b?c=d point to?".to_string()
    } else {
        question
    };

    // A tool round needs at least two turns: the tool call, then the answer.
    let answer = agent.prompt(&question).max_turns(4).await?;
    println!("{}", answer.output());

    Ok(())
}
