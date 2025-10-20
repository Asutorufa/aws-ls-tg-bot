use std::fmt;

use aws_config::BehaviorVersion;
use aws_sdk_lightsail::{types::Instance, Client};
use aws_smithy_types::body::SdkBody;
use chrono::{DateTime, Datelike, Local, NaiveDate, NaiveDateTime, NaiveTime};

#[derive(Clone)]
pub struct AwsClient {
    client: aws_sdk_lightsail::Client,
}

#[derive(Debug)]
pub struct StringError(String);

impl fmt::Display for StringError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for StringError {}

impl From<&str> for StringError {
    fn from(s: &str) -> Self {
        StringError(s.to_string())
    }
}

impl From<String> for StringError {
    fn from(s: String) -> Self {
        StringError(s)
    }
}

impl AwsClient {
    pub async fn new() -> Self {
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
        };
    }

    fn raw_response_to_string(
        &self,
        e: Option<&aws_smithy_runtime_api::http::Response<SdkBody>>,
    ) -> Result<String, StringError> {
        match String::from_utf8(
            e.ok_or("raw_response is none")?
                .body()
                .bytes()
                .ok_or("bytes is none")?
                .to_vec(),
        ) {
            Err(e) => Err(StringError::from(e.to_string())),
            Ok(v) => Ok(v),
        }
    }

    pub async fn get_instances(&self) -> Result<Vec<String>, StringError> {
        let instances = match self.client.get_instances().send().await {
            Err(e) => {
                return Err(StringError::from(
                    self.raw_response_to_string(e.raw_response())?,
                ));
            }
            Ok(v) => v,
        };

        let instances = instances
            .instances()
            .iter()
            .map(|instance| instance.name().unwrap_or_else(|| "").to_string())
            .collect::<Vec<String>>();

        Ok(instances)
    }

    pub async fn get_flow(
        &self,
        instance_name: String,
        metrics_name: aws_sdk_lightsail::types::InstanceMetricName,
    ) -> Result<f64, StringError> {
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
            .instance_name(instance_name)
            .period(2700000)
            .unit(aws_sdk_lightsail::types::MetricUnit::Bytes)
            .statistics(aws_sdk_lightsail::types::MetricStatistic::Sum)
            .start_time(aws_smithy_types::DateTime::from_secs(
                NaiveDateTime::new(nd, NaiveTime::from_hms_micro_opt(0, 0, 0, 0).unwrap())
                    .and_utc()
                    .timestamp(),
            ))
            .end_time(aws_smithy_types::DateTime::from_secs(
                NaiveDateTime::new(end_date, NaiveTime::from_hms_micro_opt(0, 0, 0, 0).unwrap())
                    .and_utc()
                    .timestamp(),
            ));

        let resp = req.send().await;

        let metric_data = match resp {
            Err(e) => {
                return Err(StringError::from(
                    self.raw_response_to_string(e.raw_response())?,
                ));
            }
            Ok(v) => v,
        };

        let metric_data = metric_data.metric_data();

        return match metric_data[0].sum() {
            None => Err("sum is none".into()),
            Some(v) => Ok(v),
        };
    }

    pub async fn get_instance_infos(self) -> Result<Vec<Instance>, StringError> {
        let instances = match self.client.get_instances().send().await {
            Err(e) => {
                return Err(StringError::from(
                    self.raw_response_to_string(e.raw_response())?,
                ));
            }
            Ok(v) => v,
        };

        Ok(instances.instances().to_vec())
    }

    pub async fn reboot_instance(self, instance_name: String) -> Result<(), StringError> {
        let resp = self
            .client
            .reboot_instance()
            .instance_name(instance_name)
            .send()
            .await;

        match resp {
            Err(e) => {
                return Err(StringError::from(
                    self.raw_response_to_string(e.raw_response())?,
                ));
            }
            Ok(_) => Ok(()),
        }
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
                    .and_utc()
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
        let client = AwsClient::new().await;
        let network_in = client
            .get_flow(
                "Debian-2".to_string(),
                aws_sdk_lightsail::types::InstanceMetricName::NetworkIn,
            )
            .await
            .unwrap();
        let network_out = client
            .get_flow(
                "Debian-2".to_string(),
                aws_sdk_lightsail::types::InstanceMetricName::NetworkOut,
            )
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
            .and_utc()
            .timestamp(),
            NaiveDateTime::new(
                end_date.unwrap(),
                NaiveTime::from_hms_micro_opt(0, 0, 0, 0).unwrap(),
            )
            .and_utc()
            .timestamp(),
        );
    }
}
