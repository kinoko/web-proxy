
nginx-proxy sets up a container running nginx and docker-gen. docker-gen generate reverse proxy configs for nginx and reloads nginx when containers are started and stopped. requests accessed to this container is distributed by the url location. this is inpired by jwilder/nginx-proxy.

# Usage

To run it:
```
$ docker run -d -p 80:80 -v /var/run/docker.sock:/tmp/docker.sock kinoko/web-proxy
```
Then start any containers you want proxied with an env var `WEB_LOCATION=/subloc`
```
$ docker run -e WEB_LOCATION="/subloc" ...
```
