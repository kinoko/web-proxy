
nginx-proxy sets up a container running nginx and docker-gen. docker-gen generate reverse proxy configs for nginx and reloads nginx when containers are started and stopped. requests accessed to this container is distributed by the url location. this is inpired by jwilder/nginx-proxy.

# Usage

To run it:
```
$ docker run -d -p 80:80 -p 443:443 -v /var/run/docker.sock:/tmp/docker.sock kinoko/web-proxy
```
Then start any containers you want proxied with env vars `WEB_VIRTUAL_HOST=example.com` and `WEB_LOCATION=/subloc`
```
$ docker run -e WEB_HOST="example.com" -e WEB_LOCATION="/subloc" --name test-web-page ...
```

## HTTPS Certification
If you want proxied with https connection, you need to put `*.crt` and `*.key` files on the directry `/etc/nginx/certs`. The certification files are named by domain as `example.com.crt` or `example.com.key`. 

## Basic Authentication
If you want to authenticate some location on some domain, you need to put a htpasswd file on the directry `/etc/nginx/htpasswd`. The file is named by the container name as `test-web-page`.
