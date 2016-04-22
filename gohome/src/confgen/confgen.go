package main

import (
	"log"
)

func main() {
	dispatcher := NewDispatcher()
	if err := dispatcher.Start(); err != nil {
		log.Fatalf("error while running: %v", err)
	}
}
