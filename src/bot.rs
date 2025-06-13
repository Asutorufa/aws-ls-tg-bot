use crate::aws::AwsClient;
use crate::command::run;
use teloxide::{
    dispatching::DefaultKey,
    prelude::*,
    types::{ReplyParameters, Update, UserId},
    utils::{command::BotCommands, markdown},
    RequestError,
};

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:"
)]
pub enum Command {
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

pub async fn run_bot(run_opt: RunOpt) -> Dispatcher<Bot, RequestError, DefaultKey> {
    let bot = Bot::from_env();

    bot.set_my_commands(Command::bot_commands())
        .send()
        .await
        .unwrap();

    let handler = dptree::entry()
        .branch(
            Update::filter_message()
                .branch(dptree::entry().filter_command::<Command>().endpoint(answer)),
        )
        .branch(
            Update::filter_edited_message()
                .branch(dptree::entry().filter_command::<Command>().endpoint(answer)),
        );

    let deps = dptree::deps![run_opt];

    let dispatcher = Dispatcher::builder(bot, handler)
        .dependencies(deps)
        .enable_ctrlc_handler()
        .build();

    return dispatcher;
}

struct Flow {
    data: f64,
    err_string: Option<String>,
}

impl ToString for Flow {
    fn to_string(&self) -> String {
        match &self.err_string {
            None => reduce_unit(self.data),
            Some(v) => v.clone(),
        }
    }
}

async fn get_flow(client: &AwsClient, instance: String, out: bool) -> Flow {
    let metrics_name = if out {
        aws_sdk_lightsail::types::InstanceMetricName::NetworkOut
    } else {
        aws_sdk_lightsail::types::InstanceMetricName::NetworkIn
    };

    return match client.get_flow(instance, metrics_name).await {
        Err(e) => Flow {
            data: 0.0,
            err_string: Some(e.to_string()),
        },
        Ok(v) => Flow {
            data: v,
            err_string: None,
        },
    };
}

fn flow_message(network_out: Flow, network_in: Flow) -> String {
    return format!(
        "NetworkIn: {}\nNetworkOut: {}\nAll: {}\n\n",
        network_in.to_string(),
        network_out.to_string(),
        reduce_unit(network_in.data + network_out.data)
    );
}

pub async fn answer(
    opt: RunOpt,
    bot: teloxide::prelude::Bot,
    msg: Message,
    cmd: Command,
) -> ResponseResult<()> {
    let from_user = match &msg.from {
        None => return Ok(()),
        Some(v) => v.id,
    };

    println!("new request from: {}", from_user);

    match cmd {
        Command::Network => {
            if from_user != opt.maintainer {
                return Ok(());
            }

            let mut text: String = String::new();

            match opt.aws_client.get_instances().await {
                Err(e) => text.push_str(markdown::escape(e.to_string().as_str()).as_str()),
                Ok(instances) => {
                    for instance in instances {
                        let network_out = get_flow(&opt.aws_client, instance.clone(), true).await;
                        let network_in = get_flow(&opt.aws_client, instance.clone(), false).await;

                        text.push_str(
                            format!(
                                "{}\n```\n{}\n```",
                                markdown::escape(instance.as_str()),
                                markdown::escape(flow_message(network_out, network_in).as_str())
                            )
                            .as_str(),
                        );
                    }
                }
            };

            bot.send_message(msg.chat.id, text)
                .reply_parameters(ReplyParameters::new(msg.id))
                .parse_mode(teloxide::types::ParseMode::MarkdownV2)
                .await?;
            return Ok(());
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
                Ok(v) => {
                    if v.is_empty() {
                        "result is empty".to_string()
                    } else {
                        v
                    }
                }
            };

            println!("command: {}, args: {:?}", process, args,);

            bot.send_message(
                msg.chat.id,
                format!("```bash\n{}\n```", markdown::escape(&text)),
            )
            .reply_parameters(ReplyParameters::new(msg.id))
            .parse_mode(teloxide::types::ParseMode::MarkdownV2)
            .await?
        }

        Command::UserID => {
            let m = match msg.from {
                None => "None".into(),
                Some(v) => v.id.to_string(),
            };
            bot.send_message(msg.chat.id, m)
                .reply_parameters(ReplyParameters::new(msg.id))
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
