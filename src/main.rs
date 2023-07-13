use awstgbot::aws::AwsClient;
use awstgbot::bot::{run_bot, RunOpt};
use teloxide::prelude::UserId;

/*
 telegram bot token env: TELOXIDE_TOKEN=
 aws instance env: AWS_INSTANCE=
 maintainer od env: MAINTAINER_ID=
*/
#[tokio::main]
async fn main() {
    let instance = std::env::var("AWS_INSTANCE").unwrap();
    let maintainer_id = std::env::var("MAINTAINER_ID")
        .unwrap()
        .parse::<u64>()
        .unwrap();

    let aws_client = AwsClient::new(instance).await;
    run_bot(RunOpt {
        aws_client,
        maintainer: UserId(maintainer_id),
    })
    .await;
}
