use teloxide::prelude::*;

#[tokio::main]
async fn main() {
    println!("Starting echo bot...");

    if cfg!(debug_assertions) {
        dotenvy::dotenv().expect("no .env file");
    } else {
        let _ = dotenvy::dotenv();
    }

    let env = envy::from_env::<Env>().expect("failed to deserialize env vars");

    let config = async_openai::config::OpenAIConfig::new()
        .with_api_key(env.openrouter_token)
        .with_api_base("https://openrouter.ai/api/v1");

    let client = async_openai::Client::with_config(config);

    let bot = Bot::new(env.teloxide_token);

    let schema = Update::filter_message().endpoint(answer);

    Dispatcher::builder(bot, schema)
        .dependencies(dptree::deps![client])
        .build()
        .dispatch()
        .await;
}

async fn answer(
    bot: Bot,
    msg: Message,
    client: async_openai::Client<async_openai::config::OpenAIConfig>,
) -> Result<(), anyhow::Error> {
    use async_openai::types::{
        ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestUserMessageArgs,
        CreateChatCompletionRequestArgs,
    };

    let mut messages = vec![ChatCompletionRequestSystemMessageArgs::default()
        .content(
            "You're a helpful telegram bot. You don't reply with huge walls of text, but try to be concise and to the point",
        )
        .build()?
        .into()];

    if let Some(text) = msg.text() {
        messages.push(
            ChatCompletionRequestUserMessageArgs::default()
                .content(text)
                .build()?
                .into(),
        );

        let request = CreateChatCompletionRequestArgs::default()
            .model("google/gemini-2.5-pro")
            .messages(messages)
            .build()?;

        let chat_completion = client.chat().create(request).await?;

        let returned_message = chat_completion
            .choices
            .first()
            .unwrap()
            .message
            .content
            .clone()
            .unwrap();
        bot.send_message(msg.chat.id, returned_message).await?;
    }

    Ok(())
}

#[derive(serde::Deserialize)]
struct Env {
    teloxide_token: String,
    openrouter_token: String,
}
