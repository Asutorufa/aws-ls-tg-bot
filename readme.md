#

[![build rust musl ci](https://github.com/Asutorufa/aws-ls-tg-bot/actions/workflows/rust.yml/badge.svg)](https://github.com/Asutorufa/aws-ls-tg-bot/actions/workflows/rust.yml)

## lambda

### build

```shell
cargo lambda build --release --bin lambda
```

### deploy

- add blow env to lambda

  - TELOXIDE_TOKEN - telegram bot token
  - MAINTAINER_ID - telegram user id

- add lightsail `GetInstanceMetricData`, `GetInstances` policy to iam role

- deploy

    ```shell
    cargo lambda deploy --binary-name lambda my_function
    ```

## server

### build

```shell
cargo build --release --bin awstgbot
```

### run

```shell
# MAINTAINER_ID is telegram user id
TELOXIDE_TOKEN=dasd:dszscz MAINTAINER_ID=12321321 awstgbot
```
