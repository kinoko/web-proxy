FROM nginx

MAINTAINER Kazuto Fukuchi <kazuto@mdl.cs.tsukuba.ac.jp>

RUN \
  apt-get update && \
  apt-get install -y -q --no-install-recommends ca-certificates && \
  apt-get clean && rm -rf /var/lib/apt/lists/*

RUN \
  echo "\ndaemon off;" >> /etc/nginx/nginx.conf && \
  sed -i 's/^http {/&\n    server_names_hash_bucket_size 128;/g' /etc/nginx/nginx.conf

ADD https://github.com/jwilder/forego/releases/download/v0.16.1/forego /usr/local/bin/forego
RUN chmod u+x /usr/local/bin/forego

COPY gohome/src/confgen/confgen /usr/local/bin/confgen

RUN chmod u+x /usr/local/bin/confgen

COPY app /app
WORKDIR /app/

ENV DOCKER_HOST unix:///tmp/docker.sock

EXPOSE 80 443

VOLUME ["/etc/nginx/certs", "/etc/nginx/htpasswd"]

CMD ["forego", "start", "-r"]
