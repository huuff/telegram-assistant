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
    let credentials =
        openai::Credentials::new(&env.openrouter_token, "https://openrouter.ai/api/v1");
    let bot = Bot::new(env.teloxide_token);

    let schema = Update::filter_message().endpoint(answer);

    Dispatcher::builder(bot, schema)
        .dependencies(dptree::deps![credentials])
        .build()
        .dispatch()
        .await;
}

async fn answer(
    bot: Bot,
    msg: Message,
    credentials: openai::Credentials,
) -> Result<(), anyhow::Error> {
    let mut messages = vec![openai::chat::ChatCompletionMessage {
            role: openai::chat::ChatCompletionMessageRole::System,
            content: Some("You're a helpful telegram bot. You don't reply with huge walls of text, but try to be concise and to the point".into()),
            ..Default::default()
        }];

    if let Some(text) = msg.text() {
        messages.push(openai::chat::ChatCompletionMessage {
            role: openai::chat::ChatCompletionMessageRole::User,
            content: Some(text.into()),
            ..Default::default()
        });

        let chat_completion =
            openai::chat::ChatCompletion::builder("google/gemini-2.5-pro", messages)
                .credentials(credentials.clone())
                .create()
                .await?;

        let returned_message = chat_completion
            .choices
            .first()
            .unwrap()
            .message
            .clone()
            .content
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
