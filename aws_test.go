package main

import (
	"context"
	"testing"

	"github.com/aws/aws-sdk-go-v2/service/lightsail/types"
)

func TestAws(t *testing.T) {
	al, err := NewAwsLs(context.Background())
	if err != nil {
		t.Fatal(err)
	}

	t.Log(al.Network(context.Background(), types.InstanceMetricNameNetworkIn, ""))
}
