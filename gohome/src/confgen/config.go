package main

import (
	"fmt"
	"html/template"
	"io/ioutil"
	"log"
	"os"
	"path/filepath"
	"sort"
)

// Location : location
type Location struct {
	Name    string
	Prefix  string
	Port    string
	Address string
}

// Locations : list of locations
type Locations []*Location

// VirtualHost : virtual host
type VirtualHost struct {
	Name      string
	Locations Locations
}

// VirtualHosts : map of virtual hosts
type VirtualHosts map[string]*VirtualHost

// Config : config
type Config struct {
	Hosts VirtualHosts
}

// NewConfig : create a new config
func NewConfig() *Config {
	return &Config{
		Hosts: make(VirtualHosts),
	}
}

// Sort : sort
func (c *Config) Sort() {
	for _, host := range c.Hosts {
		host.Sort()
	}
}

// Sort : sort
func (v *VirtualHost) Sort() {
	sort.Sort(v.Locations)
}

// AddLocation : add a location
func (v *VirtualHost) AddLocation(loc *Location) {
	v.Locations = append(v.Locations, loc)
}

// CrtPath : returns crt file path
func (v *VirtualHost) CrtPath() string {
	return fmt.Sprintf("/etc/nginx/certs/%s.crt", v.Name)
}

// KeyPath : returns key file path
func (v *VirtualHost) KeyPath() string {
	return fmt.Sprintf("/etc/nginx/certs/%s.key", v.Name)
}

// ExistsCrtAndKey : existance of crt and key files
func (v *VirtualHost) ExistsCrtAndKey() bool {
	return Exists(v.CrtPath()) && Exists(v.KeyPath())
}

// GetOrInit : get a virtual host with hostname.
func (vs VirtualHosts) GetOrInit(hostname string) *VirtualHost {
	if vhost, ok := vs[hostname]; ok {
		return vhost
	}
	vhost := &VirtualHost{
		Name:      hostname,
		Locations: make(Locations, 5),
	}
	vs[hostname] = vhost
	return vhost
}

func (ls Locations) Len() int {
	return len(ls)
}
func (ls Locations) Less(i, j int) bool {
	return ls[i].Prefix < ls[j].Prefix
}
func (ls Locations) Swap(i, j int) {
	t := ls[i]
	ls[i] = ls[j]
	ls[j] = t
	return
}

// HtpasswdPath : returns htpasswd path
func (l *Location) HtpasswdPath() string {
	return fmt.Sprintf("/etc/nginx/htpasswd/%s", l.Name)
}

// ExistsHtpasswd : existance of htpasswd file
func (l *Location) ExistsHtpasswd() bool {
	return Exists(l.HtpasswdPath())
}

// Generate : to generate config file
func (c *Config) Generate() error {
	env = GetEnv()
	dest, err := ioutil.TempFile(filepath.Dir(env.Dest), "web-proxy")
	defer func() {
		dest.Close()
		os.Remove(dest.Name())
	}()
	if err != nil {
		log.Fatalf("unable to create temp file: %s\n", err)
		return err
	}

	c.write(dest)

	err = os.Rename(dest.Name(), env.Dest)
	if err != nil {
		log.Fatalf("unable to create dest file %s: %s\n", env.Dest, err)
		return err
	}
	log.Printf("Generated '%s'", env.Dest)
	return nil
}

func (c *Config) write(dest *os.File) {
	log.Println("Writing...")
	tpl := template.Must(template.ParseFiles("main.tpl"))
	if err := tpl.ExecuteTemplate(dest, "main.tpl", c); err != nil {
		log.Fatalf("Failed to execute template main.tpl %s\n", err)
	}
}

// Exists : check the existance of a file
func Exists(filename string) bool {
	_, err := os.Stat(filename)
	return err == nil
}
