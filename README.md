![alt text](https://github.com/akropolisio/cloudflare-manager/blob/master/img/web3%20foundation_grants_badge_black.png "Project supported by web3 foundation grants program")

# Cloudflare Manager

This is Cloudflare Manager.

# Status

POC. Active development.

# Building

Install Rust:

```bash
curl https://sh.rustup.rs -sSf | sh
```

Build:

```bash
cargo build
```

# Run

```bash
cargo run
```

# Environment variables description
SERVER_IP - IP address for binding, e.g. 127.0.0.1
SERVER_PORT - port for binding, e.g. 8080
SECRET_PATH - path to directory with secrets, e.g. "secret"

# Secret files
$SECRET_PATH/auth_key - Cloudflare API auth_key, for details visite https://api.cloudflare.com/
$SECRET_PATH/token - Cloudflare API token, for details visite https://api.cloudflare.com/
$SECRET_PATH/content - Cloudflare API content, ip address v4 for DNS record type A, e.g. 127.0.0.1
$SECRET_PATH/zone_name - Cloudflare API zone_name, for details visite https://api.cloudflare.com/
$SECRET_PATH/zone_id - Cloudflare API zone_id, for details visite https://api.cloudflare.com/
