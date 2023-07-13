use crate::aws::AwsClient;
use crate::command::run;
use teloxide::{
    prelude::*,
    types::{Update, UserId},
    utils::command::BotCommands,
};

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:"
)]
enum Command {
    #[command(description = "get network flow")]
    Network,
    #[command(description = "run a shell command in 5 seconds")]
    Shell(String),
    #[command(description = "get current user id")]
    UserID,
}

#[derive(Clone)]
pub struct RunOpt {
    pub aws_client: AwsClient,
    pub maintainer: UserId,
}

pub async fn run_bot(run_opt: RunOpt) {
    let bot = teloxide::prelude::Bot::from_env();

    bot.set_my_commands(Command::bot_commands())
        .send()
        .await
        .unwrap();

    let handler: Handler<
        '_,
        DependencyMap,
        Result<(), teloxide::RequestError>,
        teloxide::dispatching::DpHandlerDescription,
    > = Update::filter_message()
        .branch(dptree::entry().filter_command::<Command>().endpoint(answer));

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![run_opt])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}

async fn answer(
    opt: RunOpt,
    bot: teloxide::prelude::Bot,
    msg: Message,
    cmd: Command,
) -> ResponseResult<()> {
    let from_user = match msg.from() {
        None => return Ok(()),
        Some(v) => v.id,
    };

    println!("new request from: {}", from_user);

    match cmd {
        Command::Network => {
            if from_user != opt.maintainer {
                return Ok(());
            }
            let mut all = 0.0;

            let network_out = match opt
                .aws_client
                .get_flow(aws_sdk_lightsail::types::InstanceMetricName::NetworkOut)
                .await
            {
                Err(e) => e.to_string(),
                Ok(v) => {
                    all += v;
                    reduce_unit(v)
                }
            };
            let network_in = match opt
                .aws_client
                .get_flow(aws_sdk_lightsail::types::InstanceMetricName::NetworkIn)
                .await
            {
                Err(e) => e.to_string(),
                Ok(v) => {
                    all += v;
                    reduce_unit(v)
                }
            };

            let text = format!(
                "NetworkIn: {}\nNetworkOut: {}\nAll: {}",
                network_in,
                network_out,
                reduce_unit(all)
            );
            let text_len = text.len();

            bot.send_message(msg.chat.id, text)
                .reply_to_message_id(msg.id)
                .entities(vec![teloxide::types::MessageEntity {
                    kind: teloxide::types::MessageEntityKind::Pre {
                        language: Some("text".to_string()),
                    },
                    offset: 0,
                    length: text_len,
                }])
                .await?
        }
        Command::Shell(command) => {
            if from_user != opt.maintainer {
                return Ok(());
            }

            let mut cs = command.split_whitespace();
            let process = match cs.next() {
                None => return Ok(()),
                Some(v) => v,
            };
            let args = cs.collect::<Vec<&str>>();

            let text = match run(process, &args).await {
                Err(e) => e.to_string(),
                Ok(v) => v,
            };
            let text_len = text.len();

            bot.send_message(msg.chat.id, text)
                .reply_to_message_id(msg.id)
                .entities(vec![teloxide::types::MessageEntity {
                    kind: teloxide::types::MessageEntityKind::Pre {
                        language: Some("shell".to_string()),
                    },
                    offset: 0,
                    length: text_len,
                }])
                .await?
        }
        Command::UserID => {
            let m = match msg.from() {
                None => "None".into(),
                Some(v) => v.id.to_string(),
            };
            bot.send_message(msg.chat.id, m)
                .reply_to_message_id(msg.id)
                .await?
        }
    };
    Ok(())
}

fn reduce_unit(byte: f64) -> String {
    if byte >= 1125899906842624.0 {
        return format!("{} PB", byte / 1125899906842624.0);
    }
    if byte >= 1099511627776.0 {
        return format!("{} TB", byte / 1099511627776.0);
    }
    if byte >= 1073741824.0 {
        return format!("{} GB", byte / 1073741824.0);
    }
    if byte >= 1048576.0 {
        return format!("{} MB", byte / 1048576.0);
    }
    if byte >= 1024.0 {
        return format!("{} KB", byte / 1024.0);
    }
    return format!("{} B", byte);
}
