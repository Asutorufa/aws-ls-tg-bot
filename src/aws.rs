use aws_config::BehaviorVersion;
use aws_sdk_lightsail::Client;
use chrono::{DateTime, Datelike, Local, NaiveDate, NaiveDateTime, NaiveTime};

#[derive(Clone)]
pub struct AwsClient {
    client: aws_sdk_lightsail::Client,
    instance: std::string::String,
}

impl AwsClient {
    pub async fn new(instance: String) -> Self {
        let shared_config = aws_config::load_defaults(BehaviorVersion::latest()).await;

        println!(
            "token: {:#?}, region: {:#?}",
            shared_config.endpoint_url(),
            shared_config.region()
        );

        // let cloudWatch = aws_sdk_cloudwatch::client::Client::new(&shared_config);

        // let metrics_data_query_downloaded = aws_sdk_cloudwatch::types::MetricDataQuery::builder()
        //     .set_id(Some("cloudfront_id".to_string()))
        //     .set_metric_stat(Some(
        //         aws_sdk_cloudwatch::types::MetricStat::builder()
        //             .set_stat(Some("Sum".to_string()))
        //             .set_period(Some(2700000))
        //             .set_unit(Some(aws_sdk_cloudwatch::types::StandardUnit::Bytes))
        //             .set_metric(Some(
        //                 aws_sdk_cloudwatch::types::Metric::builder()
        //                     .set_namespace(Some("AWS/CloudFront".to_string()))
        //                     .set_metric_name(Some("BytesDownloaded".to_string()))
        //                     .build(),
        //             ))
        //             .build(),
        //     ))
        //     .build();
        // let metrics_data_query_uploaded = aws_sdk_cloudwatch::types::MetricDataQuery::builder()
        //     .set_id(Some("cloudfront_id".to_string()))
        //     .set_metric_stat(Some(
        //         aws_sdk_cloudwatch::types::MetricStat::builder()
        //             .set_stat(Some("Sum".to_string()))
        //             .set_period(Some(2700000))
        //             .set_unit(Some(aws_sdk_cloudwatch::types::StandardUnit::Bytes))
        //             .set_metric(Some(
        //                 aws_sdk_cloudwatch::types::Metric::builder()
        //                     .set_namespace(Some("AWS/CloudFront".to_string()))
        //                     .set_metric_name(Some("BytesDownloaded".to_string()))
        //                     .build(),
        //             ))
        //             .build(),
        //     ))
        //     .build();

        // let mrtrics = cloudWatch
        //     .get_metric_data()
        //     .set_metric_data_queries(Some(vec![{ metrics_data_query_downloaded }, {
        //         metrics_data_query_uploaded
        //     }]))
        //     .send()
        //     .await;

        return AwsClient {
            client: Client::new(&shared_config),
            instance,
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

        let (month, year) = if local.month() < 12 {
            (local.month() + 1, local.year())
        } else {
            (1, local.year() + 1)
        };

        let end_date = match NaiveDate::from_ymd_opt(year, month, 1) {
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
        let metric_data = match resp.metric_data {
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
    use aws_config::BehaviorVersion;
    use chrono::{DateTime, Datelike, Local, NaiveDate, NaiveDateTime, NaiveTime};

    #[tokio::test]
    async fn get_cloudfront_data() {
        let mut shared_config = aws_config::load_defaults(BehaviorVersion::latest()).await;
        shared_config = aws_config::SdkConfig::builder()
            .http_client(shared_config.http_client().unwrap())
            .region(aws_types::region::Region::new("us-east-1"))
            .credentials_provider(shared_config.credentials_provider().unwrap())
            .build();
        let cloud_watch = aws_sdk_cloudwatch::client::Client::new(&shared_config);

        let metrics_data_query_downloaded = aws_sdk_cloudwatch::types::MetricDataQuery::builder()
            .id("q1".to_string())
            .return_data(true)
            .metric_stat(
                aws_sdk_cloudwatch::types::MetricStat::builder()
                    .stat("Sum".to_string())
                    .period(3600)
                    .unit(aws_sdk_cloudwatch::types::StandardUnit::Bytes)
                    .metric(
                        aws_sdk_cloudwatch::types::Metric::builder()
                            .dimensions(
                                aws_sdk_cloudwatch::types::Dimension::builder()
                                    .name("Region")
                                    .value("Global")
                                    .build(),
                            )
                            .dimensions(
                                aws_sdk_cloudwatch::types::Dimension::builder()
                                    .name("DistributionId")
                                    .value("E1EYY8GARCLTWQ")
                                    .build(),
                            )
                            .namespace("AWS/CloudFront".to_string())
                            .metric_name("BytesDownloaded".to_string())
                            .build(),
                    )
                    .build(),
            )
            .build();
        let metrics_data_query_uploaded = aws_sdk_cloudwatch::types::MetricDataQuery::builder()
            .id("cloudfront_id_upload".to_string())
            .return_data(true)
            .metric_stat(
                aws_sdk_cloudwatch::types::MetricStat::builder()
                    .stat("Sum".to_string())
                    .period(3600)
                    .unit(aws_sdk_cloudwatch::types::StandardUnit::Bytes)
                    .metric(
                        aws_sdk_cloudwatch::types::Metric::builder()
                            .namespace("AWS/CloudFront".to_string())
                            .metric_name("BytesDownloaded".to_string())
                            .dimensions(
                                aws_sdk_cloudwatch::types::Dimension::builder()
                                    .name("Region")
                                    .value("Global")
                                    .build(),
                            )
                            .build(),
                    )
                    .build(),
            )
            .build();

        let local: DateTime<Local> = Local::now();
        let nd = match NaiveDate::from_ymd_opt(local.year(), local.month(), 1) {
            None => panic!("none NaiveDate"),
            Some(v) => v,
        };

        // let month;
        // if local.month() < 12 {
        //     month = local.month() + 1;
        // } else {
        //     month = 1;
        // }

        // let end_date = match NaiveDate::from_ymd_opt(local.year(), month, 1) {
        //     None => panic!("None EndDate"),
        //     Some(v) => v,
        // };

        let mrtrics = cloud_watch
            .get_metric_data()
            .start_time(aws_smithy_types::DateTime::from_secs(
                NaiveDateTime::new(nd, NaiveTime::from_hms_micro_opt(0, 0, 0, 0).unwrap())
                    .timestamp(),
            ))
            .end_time(aws_smithy_types::DateTime::from_secs(local.timestamp()))
            .metric_data_queries(metrics_data_query_downloaded)
            .metric_data_queries(metrics_data_query_uploaded)
            .send()
            .await;

        println!("{:#?}", mrtrics.unwrap());
    }

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
