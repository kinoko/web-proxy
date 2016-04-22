package main

import (
	"fmt"
	"io/ioutil"
	"log"
	"os"
	"path/filepath"
	"sort"
)

type Location struct {
	Name    string
	Prefix  string
	Port    string
	Address string
}

type Locations []*Location

type VirtualHost struct {
	Name      string
	Locations Locations
}
type Config struct {
	Hosts map[string]*VirtualHost
}

func (c *Config) Sort() {
	for _, host := range c.Hosts {
		host.Sort()
	}
}

func (v *VirtualHost) Sort() {
	sort.Sort(v.Locations)
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
	fmt.Fprintf(dest, `
map $http_x_forwarded_proto $proxy_x_forwarded_proto {
  default $http_x_forwarded_proto;
  ''      $scheme;
}
map $http_upgrade $proxy_connection {
  default upgrade;
  '' close;
}
gzip_types text/plain text/css application/javascript application/json application/x-javascript text/xml application/xml application/xml+rss text/javascript;
log_format vhost '$host $remote_addr - $remote_user [$time_local] '
                 '"$request" $status $body_bytes_sent '
                 '"$http_referer" "$http_user_agent"';
access_log off;
proxy_http_version 1.1;
proxy_buffering off;
proxy_set_header Host $http_host;
proxy_set_header Upgrade $http_upgrade;
proxy_set_header Connection $proxy_connection;
proxy_set_header X-Real-IP $remote_addr;
proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
proxy_set_header X-Forwarded-Proto $proxy_x_forwarded_proto;
server {
  server_name _; # This is just an invalid value which will never trigger on a real hostname.
  listen 80;
  access_log /var/log/nginx/access.log vhost;
  return 503;
}
`)
	for _, vhost := range c.Hosts {
		fmt.Fprintf(dest, "upstream %s {\n", vhost.Name)
		for _, loc := range vhost.Locations {
			fmt.Fprintf(dest, "  # %s\n  server %s:%s;\n", loc.Name, loc.Address, loc.Port)
		}
		fmt.Fprintln(dest, "}")
	}
	for _, vhost := range c.Hosts {
		crt := fmt.Sprintf("/etc/nginx/certs/%s.crt", vhost.Name)
		key := fmt.Sprintf("/etc/nginx/certs/%s.key", vhost.Name)
		if Exists(crt) && Exists(key) {
			fmt.Fprintf(dest, `server {
  server_name %s;
  listen 80;
  access_log /var/log/nginx/access.log vhost;
  return 301 https://$host$request_uri;
}
`, vhost.Name)
			fmt.Fprintf(dest, `server {
  server_name %s;
  listen 443 ssl http2;
  access_log /var/log/nginx/access.log vhost;
  ssl_protocols TLSv1 TLSv1.1 TLSv1.2;
  ssl_ciphers ECDHE-RSA-AES128-GCM-SHA256:ECDHE-ECDSA-AES128-GCM-SHA256:ECDHE-RSA-AES256-GCM-SHA384:ECDHE-ECDSA-AES256-GCM-SHA384:DHE-RSA-AES128-GCM-SHA256:DHE-DSS-AES128-GCM-SHA256:kEDH+AESGCM:ECDHE-RSA-AES128-SHA256:ECDHE-ECDSA-AES128-SHA256:ECDHE-RSA-AES128-SHA:ECDHE-ECDSA-AES128-SHA:ECDHE-RSA-AES256-SHA384:ECDHE-ECDSA-AES256-SHA384:ECDHE-RSA-AES256-SHA:ECDHE-ECDSA-AES256-SHA:DHE-RSA-AES128-SHA256:DHE-RSA-AES128-SHA:DHE-DSS-AES128-SHA256:DHE-RSA-AES256-SHA256:DHE-DSS-AES256-SHA:DHE-RSA-AES256-SHA:AES128-GCM-SHA256:AES256-GCM-SHA384:AES128-SHA256:AES256-SHA256:AES128-SHA:AES256-SHA:AES:CAMELLIA:DES-CBC3-SHA:!aNULL:!eNULL:!EXPORT:!DES:!RC4:!MD5:!PSK:!aECDH:!EDH-DSS-DES-CBC3-SHA:!EDH-RSA-DES-CBC3-SHA:!KRB5-DES-CBC3-SHA;
  ssl_prefer_server_ciphers on;
  ssl_session_timeout 5m;
  ssl_session_cache shared:SSL:50m;
  ssl_certificate %s;
  ssl_certificate_key %s;
  add_header Strict-Transport-Security "max-age=31536000";
`, vhost.Name, crt, key)
		} else {
			fmt.Fprintf(dest, `server {
  server_name %s;
  listen 80;
  access_log /var/log/nginx/access.log vhost;
`, vhost.Name)
		}
		for _, loc := range vhost.Locations {
			basic := fmt.Sprintf("/etc/nginx/htpasswd/%s", loc.Name)
			fmt.Fprintf(dest, `  location %s {
    proxy_pass http://%s;
`, loc.Prefix, loc.Name)
			if Exists(basic) {
				fmt.Fprintf(dest, `    auth_basic "Restricted %s";\n`, vhost.Name)
				fmt.Fprintf(dest, `    auth_basic_user_file %s;\n`, basic)
			}
			fmt.Fprintln(dest, "  }")
		}
		fmt.Fprintln(dest, "}")
	}
}

func Exists(filename string) bool {
	_, err := os.Stat(filename)
	return err == nil
}
