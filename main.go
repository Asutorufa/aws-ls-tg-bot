package main

import (
	"bytes"
	"context"
	"encoding/json"
	"errors"
	"flag"
	"fmt"
	"log"
	"os/exec"
	"strings"
	"time"

	tgbotapi "github.com/go-telegram-bot-api/telegram-bot-api/v5"
	"github.com/jinzhu/now"
)

type Metrics struct {
	Name string       `json:"metricName"`
	Data []MetricData `json:"metricData"`
}

type MetricData struct {
	Sum       float64 `json:"sum"`
	Timestamp string  `json:"timestamp"`
	Unit      string  `json:"unit"`
}

var others map[string]float64

func init() {
	now := time.Now()

	if now.Year() == 2023 && now.Month() == time.February {
		others = map[string]float64{
			"Ubuntu-1": 76091156340 + 1024*1024*1024,
		}
	}
}

func Network(instanceName string) string {
	var all float64
	buf := strings.Builder{}

	buf.WriteString("NetworkIn: ")
	in, err := network("NetworkIn", instanceName)
	if err != nil {
		buf.WriteString(err.Error())
	} else {
		buf.WriteString(fmt.Sprint(ReducedUnit(in)))
		all += in
	}
	buf.WriteByte('\n')

	buf.WriteString("NetworkOut: ")
	out, err := network("NetworkOut", instanceName)
	if err != nil {
		buf.WriteString(err.Error())
	} else {
		buf.WriteString(fmt.Sprint(ReducedUnit(out)))
		all += out
	}
	buf.WriteByte('\n')

	for k, v := range others {
		buf.WriteString(k)
		buf.WriteString(": ")
		buf.WriteString(fmt.Sprint(ReducedUnit(v)))
		all += v
		buf.WriteByte('\n')
	}

	buf.WriteString("All: ")
	buf.WriteString(fmt.Sprint(ReducedUnit(all)))

	return buf.String()
}

func network(metricName, instanceName string) (float64, error) {
	cmd := exec.Command(
		"aws",
		"lightsail",
		"get-instance-metric-data",
		"--instance-name", instanceName,
		"--metric-name", metricName,
		"--period", "2700000",
		"--start-time", fmt.Sprint(now.BeginningOfMonth().Unix()),
		"--end-time", fmt.Sprint(now.EndOfMonth().Unix()),
		"--unit", "Bytes",
		"--statistics", "Sum",
	)
	networkBytes, err := cmd.Output()
	if err != nil {
		return 0, err
	}

	var Network Metrics
	if err = json.Unmarshal(networkBytes, &Network); err != nil {
		return 0, err
	}

	if len(Network.Data) == 0 {
		return 0, errors.New("empty metics data")
	}

	return Network.Data[0].Sum, nil
}
func main() {
	token := flag.String("t", "", "-t xxx, telegram bot token")
	instanceName := flag.String("i", "", "-i xxx, aws lightsail instance name")
	id := flag.Int64("id", 0, "-id xx, user id")
	flag.Parse()

	if *token == "" || *instanceName == "" {
		log.Panic("telegram bot token or instance name is empty")
	}

	bot, err := tgbotapi.NewBotAPI(*token)
	if err != nil {
		log.Panic(err)
	}

	// bot.Debug = true

	log.Printf("Authorized on account %s", bot.Self.UserName)

	u := tgbotapi.NewUpdate(0)
	u.Timeout = 60

	bot.Request(tgbotapi.NewSetMyCommands(
		tgbotapi.BotCommand{
			Command:     "network",
			Description: "network in/out all",
		},
		tgbotapi.BotCommand{
			Command:     "user_id",
			Description: "get current user id",
		},
		tgbotapi.BotCommand{
			Command:     "shell",
			Description: "exec a shell command",
		},
	))

	updates := bot.GetUpdatesChan(u)

	for update := range updates {
		if update.Message == nil || (update.Message.Command() != "user_id" && update.Message.From.ID != *id) {
			continue
		}

		// If we got a message
		log.Printf("[%v] %s", update.Message.From.ID, update.Message.Text)

		switch update.Message.Command() {
		case "network":
			str := Network(*instanceName)
			msg := tgbotapi.NewMessage(update.Message.Chat.ID, str)
			msg.ReplyToMessageID = update.Message.MessageID
			msg.Entities = append(msg.Entities, tgbotapi.MessageEntity{
				Type:   "pre",
				Length: len(str),
			})

			bot.Send(msg)

		case "shell":
			ctx, cancel := context.WithTimeout(context.TODO(), 5*time.Second)
			filds := strings.Fields(update.Message.CommandArguments())
			if len(filds) == 0 {
				continue
			}

			c := exec.CommandContext(ctx, filds[0], filds[1:]...)
			var b bytes.Buffer
			c.Stdout = &b
			c.Stderr = &b
			err = c.Start()

			go func() {
				if er := c.Wait(); er != nil {
					errors.Join(err, er)
				}
				cancel()
			}()
			<-ctx.Done()

			if c.Process != nil {
				if er := c.Process.Kill(); er != nil {
					errors.Join(err, er)
				}
			}

			if err != nil {
				b.WriteByte('\n')
				b.WriteString(err.Error())
			}

			msg := tgbotapi.NewMessage(update.Message.Chat.ID, b.String())
			msg.ReplyToMessageID = update.Message.MessageID
			msg.Entities = append(msg.Entities, tgbotapi.MessageEntity{
				Type:   "pre",
				Length: b.Len(),
			})
			bot.Send(msg)

		case "user_id":
			msg := tgbotapi.NewMessage(update.Message.Chat.ID, fmt.Sprint(update.Message.From.ID))
			msg.ReplyToMessageID = update.Message.MessageID
			bot.Send(msg)

		default:
			continue
		}
	}
}
