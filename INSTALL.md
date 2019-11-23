# Setup playground server

## From Ubuntu 18.04

```
# let's update first
sudo apt-get update

# allow packages to be installed over https
sudo apt-get install \
    apt-transport-https \
    ca-certificates \
    curl \
    gnupg-agent \
    software-properties-common \
    libssl-dev \
    pkg-config \
    build-essential \
    python-pip

sudo pip install pygments
```

### Install Docker
```
# add Docker GPG key
curl -fsSL https://download.docker.com/linux/ubuntu/gpg | sudo apt-key add -

# add Docker "stable" repository.
sudo add-apt-repository \
  "deb [arch=amd64] https://download.docker.com/linux/ubuntu \
  $(lsb_release -cs) \
  stable"

# update to get latest package listings after adding Docker repository
sudo apt-get update

# install latest Docker
sudo apt-get install -y docker-ce docker-ce-cli containerd.io
```

### Webserver setup

```
add-apt-repository ppa:certbot/certbot
apt-get update
apt-get install -y nginx python-certbot-nginx
```

Create /etc/nginx/sites-enabled/playground.ponylang.io.conf

```
server {
    listen 80 default_server;
    listen [::]:80 default_server;
    root /var/www/html;
    server_name playground.ponylang.io;

    location / {
      proxy_pass      http://127.0.0.1:8080;
    }
}
```

```
rm /etc/nginx/sites-enabled/default
ln -sf /etc/nginx/sites-available/playground.ponylang.io.conf /etc/nginx/sites-enabled/playground.ponylang.io.conf

nginx -t && nginx -s reload
```

### SSL setup

```
certbot --nginx -d playground.ponylang.io -m ponylang.main@gmail.com
```

crontab -e

```
0 12 * * * /usr/bin/certbot renew --quiet
```

### Start docker

```
systemctl enable docker
systemctl start docker
```

### Install rust

```
curl https://sh.rustup.rs | sh
```

select `1` from prompt

```
source /root/.profile
rustup install nightly-2019-10-11 --force # rustfmt is missing from this nightly
rustup default nightly-2019-10-11
```

### Build playground image

```
git clone https://github.com/ponylang/pony-playground.git
cd pony-playground
docker build docker --pull -t ponylang-playpen
```

### Set up gist access

Create a personal access token with gist access.
install in GITHUB_TOKEN environment variable e.g. to `$HOME/.profile`.

Should ONLY be the token, not "user:token"

### Build it

```
cargo build --release --bin playpen
```

### Run it

```
export ROCKET_PORT=8080
RUST_LOG=debug ./target/release/playpen 127.0.0.1 2>&1 | logger -t playpen &
```
