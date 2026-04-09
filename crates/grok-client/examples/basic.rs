use futures::StreamExt;
use grok_client::{
    CollectedResponse, GrokAuth, GrokClient, ModelName, NewConversationRequest, StreamChunk,
};

#[tokio::main]
async fn main() -> grok_client::Result<()> {
    let auth = GrokAuth::new(
        std::env::var("GROK_SSO_COOKIE").expect("GROK_SSO_COOKIE"),
        std::env::var("GROK_SSO_RW_COOKIE").expect("GROK_SSO_RW_COOKIE"),
    )?;
    let client = GrokClient::new(auth)?;

    let request = NewConversationRequest::builder("Explain Rust's ownership model in 2 sentences")
        .model(ModelName::Grok3)
        .temporary(true)
        .build();

    println!("--- collect_text (simplest) ---");
    let mut stream = client.create_conversation(&request).await?;
    let text = stream.collect_text().await?;
    println!("{text}");

    println!("\n--- collect_full (with metadata) ---");
    let mut stream = client.create_conversation(&request).await?;
    let CollectedResponse {
        conversation_id,
        text,
        thinking,
        ..
    } = stream.collect_full().await?;
    println!("conversation: {conversation_id:?}");
    println!("text: {text}");
    if !thinking.is_empty() {
        println!("thinking: {thinking}");
    }

    println!("\n--- streaming token by token ---");
    let mut stream = client.create_conversation(&request).await?;
    while let Some(chunk) = stream.next().await {
        match chunk? {
            StreamChunk::Token { text, .. } => print!("{text}"),
            StreamChunk::Done => break,
            _ => {}
        }
    }
    println!();

    Ok(())
}
