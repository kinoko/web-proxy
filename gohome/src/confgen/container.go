package main

import (
	"log"
	"strings"

	docker "github.com/fsouza/go-dockerclient"
)

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
		env := splitKeyValueSlice(inspect.Config.Env)
		hostname, okHost := env["WEB_VIRTUAL_HOST"]
		location, okLoc := env["WEB_LOCATION"]
		port, okPort := env["WEB_PORT"]
		if !okHost || !okLoc {
			continue
		}
		if !okPort {
			port = "80"
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
