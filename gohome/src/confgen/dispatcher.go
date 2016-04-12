package main

import (
	"fmt"
	"log"
	"time"
	"os"
	"os/signal"
	"syscall"
	
  docker "github.com/fsouza/go-dockerclient"
)

type DispatchError struct {
	Inner error
	Message string
}

func (e DispatchError) Error() string {
	return fmt.Sprintf("%s: %v", e.Message, e.Inner)
}

type Dispatcher struct {
  Client *docker.Client
	watch bool
	retry bool
}

func NewDispatcher() *Dispatcher {
	return &Dispatcher{
		Client: nil,
		watch: false,
		retry: false,
	}
}

func (d *Dispatcher) init() error {
	if d.Client != nil {
		return nil
	}
	var err error
	d.Client, err = docker.NewClientFromEnv()
	if err != nil {
		return err
	}
	d.watch = false
	d.retry = false
	d.update()
	return nil
}

func (d *Dispatcher) Start() error {
	eventChan := make(chan *docker.APIEvents, 100)
	sigChan := newSignalChannel()
	defer close(eventChan)
	for {
		if err := d.init(); err != nil {
			log.Fatalf("Fail initialization: %v", err)
			time.Sleep(10 * time.Second)
			continue
		}

		if err := d.listen(eventChan); err != nil {
			log.Fatalf("Fail listening: %v", err)
			time.Sleep(10 * time.Second)
			continue
		}

		select {
		case event, ok := <-eventChan:
			if !ok {
				log.Printf("Docker daemon connection interrupted")
				if err := d.flushListen(eventChan); err != nil {
					return err
				}
				eventChan = make(chan *docker.APIEvents, 100)
				time.Sleep(10 * time.Second)
				break
			}
			if err := d.dispatch(event); err != nil {
				log.Fatalf("Fail dispatch: %v", err)
			}
		case <-time.After(10 * time.Second):
			if err := d.Client.Ping(); err != nil {
				log.Printf("Unable to ping docker daemon: %s", err)
				d.flushListen(eventChan)
			}
		case sig := <-sigChan:
			log.Printf("Received signal: %s\n", sig)
			switch sig {
			case syscall.SIGQUIT, syscall.SIGKILL, syscall.SIGTERM, syscall.SIGINT:
				return nil
			}
		}
	}
}

func (d *Dispatcher) listen(listener chan *docker.APIEvents) error {
	if d.watch {
		return nil
	}
	if err := d.Client.AddEventListener(listener); err != nil && err != docker.ErrListenerAlreadyExists {
		return DispatchError{
			Inner: err,
			Message: "Error registring docker event listener",
		}
	}
	d.watch = true
	log.Println("Watching docker events")
	d.update()
	return nil
}

func (d *Dispatcher) flushListen(listener chan *docker.APIEvents) error {
	if !d.watch {
		return nil
	}
	d.Client.RemoveEventListener(listener)
	d.watch = false
	d.Client = nil
	return nil
}

func (d *Dispatcher) dispatch(event *docker.APIEvents) error {
	if event.Status != "start" && event.Status != "stop" && event.Status != "die" {
		return nil
	}
	log.Printf("Received event %s for container %s", event.Status, event.ID[:12])
	d.update()
	return nil
}

func (d *Dispatcher) update() error {
	log.Printf("Updating...")
	config, err := configFromContainers(d.Client)
	if err != nil {
		return err
	}
	err = config.Generate()
	return err
}

func newSignalChannel() <-chan os.Signal {
	sig := make(chan os.Signal, 1)
	signal.Notify(sig, syscall.SIGHUP, syscall.SIGINT, syscall.SIGTERM, syscall.SIGQUIT, syscall.SIGKILL)

	return sig
}
