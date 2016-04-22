package main

import (
	"log"
	"strings"

	docker "github.com/fsouza/go-dockerclient"
)

type envVariables struct {
	Env map[string]string
	Ok  bool
}

func extractEnv(in []string) *envVariables {
	return &EnvVariables{
		Env: splitKeyValueSlice(in),
		Ok:  true,
	}
}

func splitKeyValueSlice(in []string) map[string]string {
	env := make(map[string]string)
	for _, entry := range in {
		parts := strings.SplitN(entry, "=", 2)
		if len(parts) != 2 {
			parts = append(parts, "")
		}
		env[parts[0]] = parts[1]
	}
	return env
}

func (env *envVariables) Require(name string) string {
	if val, ok = env.Env[name]; ok {
		return val
	}
	env.Ok = false
	return nil
}

func (env *envVariables) Optional(name string, string def) string {
	if val, ok = env.Env[name]; ok {
		return val
	}
	return def
}

func configFromContainers(client *docker.Client) (*Config, error) {
	containers, err := client.ListContainers(docker.ListContainersOptions{
		All:  false,
		Size: false,
	})
	if err != nil {
		return nil, err
	}
	config := NewConfig()
	for _, container := range containers {
		inspect, err := client.InspectContainer(container.ID)
		if err != nil {
			log.Printf("error inspecting container: %s: %s\n", container.ID, err)
			continue
		}
		env := extractEnv(inspect.Config.Env)
		hostname := env.Require("WEB_VIRTUAL_HOST")
		location := env.Require("WEB_LOCATION")
		port := env.Optional("WEB_PORT", "80")
		if !env.Ok {
			continue
		}
		vhost := config.Hosts.GetOrInit(hostname)
		vhost.AddLocation(&Location{
			Name:    strings.TrimLeft(inspect.Name, "/"),
			Prefix:  location,
			Port:    port,
			Address: inspect.NetworkSettings.IPAddress,
		})
	}
	config.Sort()
	return config, nil
}
