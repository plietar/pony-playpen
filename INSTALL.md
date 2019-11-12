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

### Start docker

```
sudo service docker start
```

### Install rust

```
curl https://sh.rustup.rs | sh
```

select `1` from prompt

```
source /root/.profile
rustup install nightly-2019-10-11
rustup default nightly
```

### Build playground image

```
git clone https://github.com/ponylang/pony-playground.git
cd pony-playground
docker build docker -t ponylang-playpen
```

### Set up gist access

Create a personal access token with gist access.
install in GITHUB_TOKEN environment variable

### Build it

```
cargo build --bin playpen
```

### Run it

```
export ROCKET_PORT=80
RUST_LOG=debug ./target/release/playpen 0.0.0.0 2>&1 | logger -t playpen
```
