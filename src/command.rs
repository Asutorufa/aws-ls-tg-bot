use futures::prelude::*;
use std::process::Stdio;
use tokio::process::Command;
use tokio_util::codec::{FramedRead, LinesCodec};

pub async fn run(command: &str, args: &[&str]) -> Result<String, Box<dyn std::error::Error>> {
    let mut child = Command::new(command)
        .kill_on_drop(true)
        .args(args)
        .stdout(Stdio::piped())
        .spawn()?;

    let output = match child.stdout.take() {
        None => {
            child.kill().await.unwrap();
            return Err("can't take child stdout".into());
        }
        Some(v) => v,
    };

    let mut reader = FramedRead::new(output, LinesCodec::new());

    let (tx, rx) = tokio::sync::oneshot::channel::<bool>();

    tokio::spawn(async move {
        // 5 秒後にコマンドの実行を中止します
        match tokio::time::timeout(std::time::Duration::from_secs(5), rx).await {
            Err(e) => {
                println!("drop child: {}", e);
                drop(child);
            }
            Ok(_) => return,
        }
    });

    let mut os = String::from("");
    while let Some(line) = reader.next().await {
        os.push_str(line?.as_str());
        os.push_str("\n");
    }
    tx.send(true).unwrap_or_default();
    Ok(os)
}

#[cfg(test)]
mod test {
    use crate::command::run;

    #[tokio::test]
    async fn process() {
        println!("{}", run("ls", &[]).await.unwrap());
    }

    #[tokio::test]
    async fn timeout() {
        let (tx, rx) = tokio::sync::oneshot::channel::<bool>();
        let (tx2, rx2) = tokio::sync::oneshot::channel::<bool>();

        tokio::spawn(async move {
            // Wrap the future with a `Timeout` set to expire in 10 milliseconds.
            match tokio::time::timeout(std::time::Duration::from_millis(2), rx).await {
                Err(e) => {
                    println!("timeout: {}", e)
                }
                Ok(v) => println!("not timeout: {}", v.unwrap_or_default()),
            }
            tx2.send(true).unwrap();
        });

        tokio::time::sleep(std::time::Duration::from_secs(3)).await;
        tx.send(true).unwrap_or_default();
        println!("{}", rx2.await.unwrap_or_default());
    }
}
