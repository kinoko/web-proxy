package main

import (
	"os"
	"sync"
)

// Env : configuration from environment variables
type Env struct {
	Dest string
}

var env *Env
var once sync.Once

// GetEnv : get env instance. create if not initialized
func GetEnv() *Env {
	once.Do(func() {
		env = newEnv()
	})
	return env
}

func newEnv() *Env {
	return &Env{
		Dest: getDefault("WEB_PROXY_DEST", "/etc/nginx/conf.d/default.conf"),
	}
}

func getDefault(key string, def string) string {
	v := os.Getenv(key)
	if v != "" {
		return v
	}
	return def
}
