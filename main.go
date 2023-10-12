package main

import (
	"bytes"
	"context"
	"errors"
	"flag"
	"fmt"
	"log"
	"os/exec"
	"strings"
	"time"

	"github.com/aws/aws-sdk-go-v2/service/lightsail/types"
	tgbotapi "github.com/go-telegram-bot-api/telegram-bot-api/v5"
)

func (a *AwsLs) NetworkFormat(ctx context.Context, instanceName string) string {
	var all float64
	buf := strings.Builder{}

	buf.WriteString("NetworkIn: ")
	in, err := a.Network(ctx, types.InstanceMetricNameNetworkIn, instanceName)
	if err != nil {
		buf.WriteString(err.Error())
	} else {
		buf.WriteString(fmt.Sprint(ReducedUnit(in)))
		all += in
	}
	buf.WriteByte('\n')

	buf.WriteString("NetworkOut: ")
	out, err := a.Network(ctx, types.InstanceMetricNameNetworkOut, instanceName)
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

	awsLs, err := NewAwsLs(context.Background())
	if err != nil {
		log.Panic(err)
	}

	u := tgbotapi.NewUpdate(0)
	u.Timeout = 60

	bot.Request(tgbotapi.NewSetMyCommands( // nolint:errcheck
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
			str := awsLs.NetworkFormat(context.Background(), *instanceName)
			msg := tgbotapi.NewMessage(update.Message.Chat.ID, str)
			msg.ReplyToMessageID = update.Message.MessageID
			msg.Entities = append(msg.Entities, tgbotapi.MessageEntity{
				Type:   "pre",
				Length: len(str),
			})

			if _, err := bot.Send(msg); err != nil {
				log.Println(err)
			}

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
			err = c.Run()
			cancel()

			if c.Process != nil {
				if er := c.Process.Kill(); er != nil {
					err = errors.Join(err, er)
				}
			}

			if err != nil {
				b.WriteByte('\n')
				b.WriteString(err.Error())
			}

			msg := tgbotapi.NewMessage(update.Message.Chat.ID, b.String())
			msg.ReplyToMessageID = update.Message.MessageID
			msg.Entities = append(msg.Entities, tgbotapi.MessageEntity{Type: "pre", Length: b.Len()})
			if _, err := bot.Send(msg); err != nil {
				log.Println(err)
			}

		case "user_id":
			msg := tgbotapi.NewMessage(update.Message.Chat.ID, fmt.Sprint(update.Message.From.ID))
			msg.ReplyToMessageID = update.Message.MessageID
			if _, err := bot.Send(msg); err != nil {
				log.Println(err)
			}

		default:
			continue
		}
	}
}
