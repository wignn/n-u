package nats

import (
	"fmt"
	"log"
	"time"

	"github.com/nats-io/nats.go"
)

func Connect(url string) (*nats.Conn, nats.JetStreamContext, error) {
	nc, err := nats.Connect(url,
		nats.RetryOnFailedConnect(true),
		nats.MaxReconnects(-1),
		nats.ReconnectWait(2*time.Second),
		nats.DisconnectErrHandler(func(_ *nats.Conn, err error) {
			log.Printf("NATS disconnected: %v", err)
		}),
		nats.ReconnectHandler(func(_ *nats.Conn) {
			log.Printf("NATS reconnected")
		}),
	)
	if err != nil {
		return nil, nil, fmt.Errorf("nats connect: %w", err)
	}

	js, err := nc.JetStream(nats.PublishAsyncMaxPending(256))
	if err != nil {
		return nil, nil, fmt.Errorf("jetstream: %w", err)
	}

	return nc, js, nil
}

func EnsureStream(js nats.JetStreamContext, streamName string, subjects []string) error {
	_, err := js.StreamInfo(streamName)
	if err == nats.ErrStreamNotFound {
		_, err = js.AddStream(&nats.StreamConfig{
			Name:      streamName,
			Subjects:  subjects,
			Retention: nats.WorkQueuePolicy,
			MaxAge:    7 * 24 * time.Hour,
			Storage:   nats.FileStorage,
		})
		if err != nil {
			return fmt.Errorf("create stream: %w", err)
		}
		log.Printf("Stream %s created", streamName)
		return nil
	}
	return err
}
