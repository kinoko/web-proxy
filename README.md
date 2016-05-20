web-proxy sets up a container running nginx, and generate reverse proxy configs for nginx and reloads nginx when containers are started and stopped. requests accessed to this container is distributed by the url location. this is inpired by jwilder/nginx-proxy.

[![Build Status](https://travis-ci.org/kinoko/web-proxy.svg?branch=master)](https://travis-ci.org/kinoko/web-proxy)

# Usage

To run it:
```
$ docker run -d -p 80:80 -p 443:443 -v /var/run/docker.sock:/tmp/docker.sock kinoko/web-proxy
```
Then start any containers you want proxied with env vars `WEB_VIRTUAL_HOST=example.com`, `WEB_LOCATION=/subloc`, and `WEB_PORT=3000`
```
$ docker run -e WEB_HOST="example.com" -e WEB_LOCATION="/subloc" -e WEB_PORT="3000" --name test-web-page ...
```
This case the proxied container can be accessed with `http://example.com/subloc`. The variable `WEB_PORT` specifies the external port of the container. `WEB_PORT=80` by the default.

## HTTPS Certification
If you want proxied with https connection, you need to put `*.crt` and `*.key` files on the directry `/etc/nginx/certs`. The certification files are named by domain as `example.com.crt` or `example.com.key`.

## Basic Authentication
If you want to authenticate some location on some domain, you need to put a htpasswd file on the directry `/etc/nginx/htpasswd`. The file is named by the container name as `test-web-page`.
