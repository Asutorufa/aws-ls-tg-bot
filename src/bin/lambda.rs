use aws_lambda_events::lambda_function_urls::LambdaFunctionUrlRequest;
use awstgbot::{
    aws::AwsClient,
    bot::{handler, Command, RunOpt},
};
use teloxide::{
    dptree::{self},
    prelude::*,
    types::{Me, Update, UserId},
    utils::command::BotCommands,
};

#[tokio::main]
async fn main() -> Result<(), lambda_runtime::Error> {
    let bot = Bot::from_env();

    let me = bot.get_me().await?;
    println!("Bot: {}, {}", me.username.as_ref().unwrap(), me.id.0);

    let maintainer_id = UserId(std::env::var("MAINTAINER_ID")?.parse::<u64>()?);

    let aws_client = AwsClient::new().await;

    let run_opt = RunOpt {
        aws_client,
        maintainer: maintainer_id,
    };

    let handler = LambdaHandler { bot, me, run_opt };
    lambda_runtime::run(lambda_runtime::service_fn(|event| {
        handler.bot_handler(event)
    }))
    .await
}

struct LambdaHandler {
    bot: Bot,
    me: Me,
    run_opt: RunOpt,
}

impl LambdaHandler {
    async fn bot_handler(
        &self,
        event: lambda_runtime::LambdaEvent<LambdaFunctionUrlRequest>,
    ) -> Result<(), lambda_runtime::Error> {
        match event.payload.raw_path {
            Some(path) => {
                if path == "/tgbot/register" {
                    println!(
                        "Registering webhook: {}",
                        format!(
                            "https://{}/tgbot",
                            event.payload.request_context.domain_name.clone().unwrap()
                        )
                    );
                    let _ = self.bot.set_my_commands(Command::bot_commands()).await;
                    let _ = self
                        .bot
                        .set_webhook(url::Url::parse(
                            format!(
                                "https://{}/tgbot",
                                event.payload.request_context.domain_name.clone().unwrap()
                            )
                            .as_str(),
                        )?)
                        .send()
                        .await?;

                    return Ok(());
                }
            }
            None => {}
        }

        let body = if event.payload.is_base64_encoded {
            bas64::decode(event.payload.body.unwrap())?
        } else {
            event.payload.body.unwrap().as_bytes().to_vec()
        };

        println!("body: {}", String::from_utf8_lossy(&body));

        let update: Update = serde_json::from_slice(&body)?;

        let handler = handler();

        let dependencies = dptree::deps![
            self.me.clone(),
            self.bot.clone(),
            update,
            self.run_opt.clone()
        ];

        let result = handler.dispatch(dependencies).await;

        match result {
            ControlFlow::Break(Ok(())) => {
                println!("Update was handled by bot.");
                Ok(())
            }
            ControlFlow::Break(Err(e)) => {
                println!("Error: {}", e);
                Err(lambda_runtime::Error::from(e))
            }
            ControlFlow::Continue(_) => {
                println!("Update was not handled by bot.");
                Ok(())
            }
        }
    }
}
