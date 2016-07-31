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
{{range $container := .Containers}}
upstream {{$container.Name}} {
  server {{$container.Address}}:{{$container.Port}};
}
{{end}}
{{range $key,$vhost := .Hosts}}
{{if $vhost.ExistsCrtAndKey}}
server {
  server_name {{$vhost.Name}};
  listen 80;
  access_log /var/log/nginx/access.log vhost;
  return 301 https://$host$request_uri;
}
server {
  server_name {{$vhost.Name}};
  listen 443 ssl http2;
  access_log /var/log/nginx/access.log vhost;
  ssl_protocols TLSv1 TLSv1.1 TLSv1.2;
  ssl_ciphers ECDHE-RSA-AES128-GCM-SHA256:ECDHE-ECDSA-AES128-GCM-SHA256:ECDHE-RSA-AES256-GCM-SHA384:ECDHE-ECDSA-AES256-GCM-SHA384:DHE-RSA-AES128-GCM-SHA256:DHE-DSS-AES128-GCM-SHA256:kEDH+AESGCM:ECDHE-RSA-AES128-SHA256:ECDHE-ECDSA-AES128-SHA256:ECDHE-RSA-AES128-SHA:ECDHE-ECDSA-AES128-SHA:ECDHE-RSA-AES256-SHA384:ECDHE-ECDSA-AES256-SHA384:ECDHE-RSA-AES256-SHA:ECDHE-ECDSA-AES256-SHA:DHE-RSA-AES128-SHA256:DHE-RSA-AES128-SHA:DHE-DSS-AES128-SHA256:DHE-RSA-AES256-SHA256:DHE-DSS-AES256-SHA:DHE-RSA-AES256-SHA:AES128-GCM-SHA256:AES256-GCM-SHA384:AES128-SHA256:AES256-SHA256:AES128-SHA:AES256-SHA:AES:CAMELLIA:DES-CBC3-SHA:!aNULL:!eNULL:!EXPORT:!DES:!RC4:!MD5:!PSK:!aECDH:!EDH-DSS-DES-CBC3-SHA:!EDH-RSA-DES-CBC3-SHA:!KRB5-DES-CBC3-SHA;
  ssl_prefer_server_ciphers on;
  ssl_session_timeout 5m;
  ssl_session_cache shared:SSL:50m;
  ssl_certificate {{$vhost.CrtPath}};
  ssl_certificate_key {{$vhost.KeyPath}};
  add_header Strict-Transport-Security "max-age=31536000";
{{else}}
server {
  server_name {{$vhost.Name}};
  listen 80;
  access_log /var/log/nginx/access.log vhost;
{{end}}

{{if $vhost.ExistsHostConf}}
  include {{$vhost.HostConfPath}};
{{end}}

{{range $index,$location := $vhost.Locations}}
  location {{$location.Prefix}} {
    proxy_pass http://{{$location.Container.Name}};
{{if $location.ExistsHtpasswd}}
    auth_basic "Restricted {{$vhost.Name}}";
    auth_basic_user_file {{$location.HtpasswdPath}};
{{end}}
  }
{{end}}
}
{{end}}
