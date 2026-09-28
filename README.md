# wm-infra-netlab-app-rust

Phase 2 of wm-infra-netlab — Rust (axum + sqlx) app, deployed to its own
VM on `private-net`. Same routes, same env var names, and (as of the
Docker-path work) same single-port model as `wm-infra-netlab-app-node`,
so the two apps are directly comparable.

Two deployment methods are supported: **VM-native** (systemd) and
**Docker**. Only one runs at a time, both on port `8080` — closer to how
you'd actually clone and deploy just this repo on its own.

## Routes

| Route | Behavior |
|---|---|
| `GET /` | `{version, served_by}` |
| `GET /health` | bare `200` |
| `GET /visits` | inserts a row, returns `{visits: count}` |

## Infrastructure

```bash
terraform init
cp terraform.tfvars.example terraform.tfvars   # fill in real values
terraform apply
```

Provisions a VM on `private-net` at `10.0.2.20`, attached by network
**name** (`wm-netlab-private`), not by another repo's Terraform state —
same decoupling pattern as every other repo here.

## Hardening (do this before deploying the app)

```bash
cd ansible
ansible-galaxy install -r requirements.yml
ansible-playbook playbook-app.yml
```

Applies `harden-baseline` (SSH lockdown, ufw, fail2ban,
unattended-upgrades) and `docker-host` (Docker Engine + the
`DOCKER-USER` firewall fix — see ADR 0009 in `wm-infra-netlab`).

Wait for cloud-init to finish before running Ansible against a fresh VM:
```bash
ssh -J netlab-admin@10.0.1.10 netlab-admin@10.0.2.20 'cloud-init status --wait'
```

## Deploy — VM-native

```bash
ssh -J netlab-admin@10.0.1.10 netlab-admin@10.0.2.20
sudo apt install -y build-essential pkg-config libssl-dev
curl https://sh.rustup.rs -sSf | sh
source "$HOME/.cargo/env"
git clone https://github.com/WilliamFly/wm-infra-netlab-app-rust.git
cd wm-infra-netlab-app-rust
cargo build --release
cp .env.example .env   # fill in the real DATABASE_URL
sudo cp deploy/netlab-app-rust.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable --now netlab-app-rust
```

This build-on-VM approach is deliberately temporary, pending Phase 4
CI/CD (which will build the binary/image elsewhere and ship an artifact
rather than compiling on the target VM).

## Deploy — Docker

```bash
ssh -J netlab-admin@10.0.1.10 netlab-admin@10.0.2.20
git clone https://github.com/WilliamFly/wm-infra-netlab-app-rust.git
cd wm-infra-netlab-app-rust
cp .env.example .env   # fill in the real DATABASE_URL
docker compose up -d --build
```

Only run one of the two methods at a time — both listen on `8080`.

## Verification

```bash
curl 10.0.2.20:8080/
curl 10.0.2.20:8080/health
curl 10.0.2.20:8080/visits
```
`served_by` in the response tells you which method is actually running
(`netlab-app-rust` for VM-native, `netlab-app-rust-docker` for Docker).

## Security / Hardening

- SSH hardened, `ufw` enabled, default-deny incoming
- Port `8080` scoped to `10.0.2.0/24` — both via ufw (VM-native) and via
  the `DOCKER-USER` chain (Docker path); ufw alone does **not** cover
  Docker-published ports, see ADR 0009 in `wm-infra-netlab`
- `fail2ban` active on sshd, `unattended-upgrades` enabled
