FROM nginx

MAINTAINER Kazuto Fukuchi <kazuto@mdl.cs.tsukuba.ac.jp>

RUN \
  apt-get update && \
  apt-get install -y wget && \
  apt-get clean && rm -rf /var/lib/apt/lists/*

RUN \
  echo "\ndaemon off;" >> /etc/nginx/nginx.conf && \
  sed -i 's/^http {/&\n    server_names_hash_bucket_size 128;/g' /etc/nginx/nginx.conf

RUN \
 wget -P /usr/local/bin https://godist.herokuapp.com/projects/ddollar/forego/releases/current/linux-amd64/forego && \
 chmod u+x /usr/local/bin/forego

COPY gohome/src/confgen/confgen /usr/local/bin/confgen

RUN chmod u+x /usr/local/bin/confgen

COPY app /app
WORKDIR /app/

ENV DOCKER_HOST unix:///tmp/docker.sock

EXPOSE 80 445

VOLUME ["/etc/nginx/certs"]

CMD ["forego", "start", "-r"]
