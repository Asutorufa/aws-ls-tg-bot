package main

import (
	"fmt"

	"github.com/aws/aws-sdk-go/aws"
	"github.com/aws/aws-sdk-go/aws/endpoints"
	"github.com/aws/aws-sdk-go/aws/session"
	"github.com/aws/aws-sdk-go/service/lightsail"
	"github.com/jinzhu/now"
)

func AWS(metricName, instanceName string) (float64, error) {
	sess := session.Must(session.NewSession(&aws.Config{
		Region: aws.String(endpoints.ApNortheast1RegionID),
	}))

	svc := lightsail.New(sess, &aws.Config{
		Region: aws.String(endpoints.ApNortheast1RegionID),
	})

	/*

		"--period", "2700000",
		"--start-time", fmt.Sprint(now.BeginningOfMonth().Unix()),
		"--end-time", fmt.Sprint(now.EndOfMonth().Unix()),
		"--unit", "Bytes",
		"--statistics", "Sum",
	*/
	data, err := svc.GetInstanceMetricData(&lightsail.GetInstanceMetricDataInput{
		MetricName:   &metricName,
		InstanceName: &instanceName,
		StartTime:    aws.Time(now.BeginningOfMonth()),
		EndTime:      aws.Time(now.EndOfMonth()),
		Unit:         aws.String("Bytes"),
		Statistics:   aws.StringSlice([]string{"Sum"}),
		Period:       aws.Int64(2700000),
	})
	if err != nil {
		return 0, err
	}

	if len(data.MetricData) == 0 {
		return 0, fmt.Errorf("empty metrics")
	}

	return *data.MetricData[0].Sum, nil
}

// type Metrics struct {
// 	Name string       `json:"metricName"`
// 	Data []MetricData `json:"metricData"`
// }

// type MetricData struct {
// 	Sum       float64 `json:"sum"`
// 	Timestamp string  `json:"timestamp"`
// 	Unit      string  `json:"unit"`
// }

var others map[string]float64

// func init() {
// 	now := time.Now()

// 	if now.Year() == 2023 && now.Month() == time.February {
// 		others = map[string]float64{
// 			"Ubuntu-1": 76091156340 + 1024*1024*1024,
// 		}
// 	}
// }

// func network(metricName, instanceName string) (float64, error) {
// 	cmd := exec.Command(
// 		"aws",
// 		"lightsail",
// 		"get-instance-metric-data",
// 		"--instance-name", instanceName,
// 		"--metric-name", metricName,
// 		"--period", "2700000",
// 		"--start-time", fmt.Sprint(now.BeginningOfMonth().Unix()),
// 		"--end-time", fmt.Sprint(now.EndOfMonth().Unix()),
// 		"--unit", "Bytes",
// 		"--statistics", "Sum",
// 	)
// 	networkBytes, err := cmd.Output()
// 	if err != nil {
// 		return 0, err
// 	}

// 	var Network Metrics
// 	if err = json.Unmarshal(networkBytes, &Network); err != nil {
// 		return 0, err
// 	}

// 	if len(Network.Data) == 0 {
// 		return 0, errors.New("empty metics data")
// 	}

//		return Network.Data[0].Sum, nil
//	}
