use aws_sdk_lightsail::Client;
use chrono::{DateTime, Datelike, Local, NaiveDate, NaiveDateTime, NaiveTime};

#[derive(Clone)]
pub struct AwsClient {
    client: aws_sdk_lightsail::Client,
    instance: std::string::String,
}

impl AwsClient {
    pub async fn new(instance: String) -> Self {
        let shared_config = aws_config::load_from_env().await;

        println!(
            "token: {:#?}, region: {:#?}",
            shared_config.endpoint_url(),
            shared_config.region()
        );

        return AwsClient {
            client: Client::new(&shared_config),
            instance: instance,
        };
    }

    pub async fn get_flow(
        &self,
        metrics_name: aws_sdk_lightsail::types::InstanceMetricName,
    ) -> Result<f64, Box<dyn std::error::Error>> {
        let local: DateTime<Local> = Local::now();
        let nd = match NaiveDate::from_ymd_opt(local.year(), local.month(), 1) {
            None => return Err("none NaiveDate".into()),
            Some(v) => v,
        };

        let month;
        if local.month() < 12 {
            month = local.month() + 1;
        } else {
            month = 1;
        }

        let end_date = match NaiveDate::from_ymd_opt(local.year(), month, 1) {
            None => return Err("None EndDate".into()),
            Some(v) => v,
        };

        let req = self
            .client
            .get_instance_metric_data()
            .metric_name(metrics_name)
            .instance_name(self.instance.clone())
            .period(2700000)
            .unit(aws_sdk_lightsail::types::MetricUnit::Bytes)
            .statistics(aws_sdk_lightsail::types::MetricStatistic::Sum)
            .start_time(aws_smithy_types::DateTime::from_secs(
                NaiveDateTime::new(nd, NaiveTime::from_hms_micro_opt(0, 0, 0, 0).unwrap())
                    .timestamp(),
            ))
            .end_time(aws_smithy_types::DateTime::from_secs(
                NaiveDateTime::new(end_date, NaiveTime::from_hms_micro_opt(0, 0, 0, 0).unwrap())
                    .timestamp(),
            ));

        let resp = req.send().await?;
        let metric_data = match resp.metric_data() {
            None => return Err("get metric data failed".into()),
            Some(v) => v,
        };

        return match metric_data[0].sum() {
            None => Err("sum is none".into()),
            Some(v) => Ok(v),
        };
    }
}

#[cfg(test)]
mod test {

    use super::AwsClient;
    use chrono::{DateTime, Datelike, Local, NaiveDate, NaiveDateTime, NaiveTime};

    #[tokio::test]
    async fn test_get_flow() {
        let client = AwsClient::new("Debian-2".to_string()).await;
        let network_in = client
            .get_flow(aws_sdk_lightsail::types::InstanceMetricName::NetworkIn)
            .await
            .unwrap();
        let network_out = client
            .get_flow(aws_sdk_lightsail::types::InstanceMetricName::NetworkOut)
            .await
            .unwrap();

        println!("networkIn: {}, networkOut: {}", network_in, network_out);
    }

    #[test]
    fn test_date() {
        let local: DateTime<Local> = Local::now();
        let nd = NaiveDate::from_ymd_opt(local.year(), local.month(), 1);

        let end_date;
        if local.month() < 12 {
            end_date = NaiveDate::from_ymd_opt(local.year(), local.month() + 1, 1);
        } else {
            end_date = NaiveDate::from_ymd_opt(local.year(), 1, 1);
        }

        println!(
            "{},{}",
            NaiveDateTime::new(
                nd.unwrap(),
                NaiveTime::from_hms_micro_opt(0, 0, 0, 0).unwrap(),
            )
            .timestamp(),
            NaiveDateTime::new(
                end_date.unwrap(),
                NaiveTime::from_hms_micro_opt(0, 0, 0, 0).unwrap(),
            )
            .timestamp(),
        );
    }
}
