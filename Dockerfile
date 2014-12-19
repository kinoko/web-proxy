FROM kinoko/base

MAINTAINER Kazuto Fukuchi <kazuto@mdl.cs.tsukuba.ac.jp>

RUN \
  echo "deb http://ppa.launchpad.net/nginx/stable/ubuntu lucid main" > /etc/apt/sources.list.d/nginx-stable-lucid.list && \
  apt-key adv --keyserver keyserver.ubuntu.com --recv-keys C300EE8C && \
  apt-get update && \
  apt-get install -y -q --no-install-recommends nginx ca-certificates wget && \
  apt-get clean && \
  rm -rf /var/lib/apt/lists/* && \
  echo "\ndaemon off;" >> /etc/nginx/nginx.conf && \
  sed -i 's/^http {/&\n    server_names_hash_bucket_size 64;/g' /etc/nginx/nginx.conf && \
  rm -rf /etc/nginx/sites-enabled/* && \
  chown -R www-data:www-data /var/lib/nginx

RUN wget -P /usr/local/bin https://godist.herokuapp.com/projects/ddollar/forego/releases/current/linux-amd64/forego \
 && chmod u+x /usr/local/bin/forego

ENV DOCKER_GEN_VERSION 0.3.6

RUN \
  wget https://github.com/jwilder/docker-gen/releases/download/$DOCKER_GEN_VERSION/docker-gen-linux-amd64-$DOCKER_GEN_VERSION.tar.gz && \
  tar -C /usr/local/bin -xvzf docker-gen-linux-amd64-$DOCKER_GEN_VERSION.tar.gz && \
  rm /docker-gen-linux-amd64-$DOCKER_GEN_VERSION.tar.gz

RUN echo "forego start -r" > /var/app/start

COPY . /app/
WORKDIR /app/

ENV DOCKER_HOST unix:///tmp/docker.sock

VOLUME ["/etc/nginx/certs"]
