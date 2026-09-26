# wm-infra-netlab-app-rust

Phase 2 of [wm-infra-netlab](https://github.com/WilliamFly/wm-infra-netlab):
the Rust app, deployed via both a VM path and a Docker path.

## Current step: app VM on private-net (no app code yet)

`app-vm.tf` provisions the VM this app will eventually run on —
`private-net` only, reachable exclusively through the router. No Rust
code or toolchain on it yet; this step just proves the VM boots
correctly and can reach the shared database.

| | |
|---|---|
| Hostname | `netlab-app-rust` |
| Admin user | `netlab-admin` (SSH key only) |
| private-net IP | `10.0.2.20` |

The database lives in its own repo,
[wm-infra-netlab-db](https://github.com/WilliamFly/wm-infra-netlab-db) —
not here. This repo connects to it by IP (`db_host`, default
`10.0.3.20`) and never provisions or owns it. See
[ADR 0004](https://github.com/WilliamFly/wm-infra-netlab/blob/main/docs/decisions/0004-shared-db-own-repo.md).

Unlike `data-net`, `private-net` already has outbound internet access
(the `private → public` rule from Phase 1) — so no temp-egress
workaround (ADR 0005) is needed to provision this VM; cloud-init's
package steps work live here.

## Prerequisites

- `network-foundation`'s networks + router already applied and working
- `wm-infra-netlab-db` already applied and reachable at `10.0.3.20`
- Terraform >= 1.9, `dmacvicar/libvirt` provider `>= 0.8.1, < 0.9`

## Usage

```bash
cp terraform.tfvars.example terraform.tfvars
# edit terraform.tfvars: base_image_path, ssh_public_key, db_app_password
# (db_app_password must match what wm-infra-netlab-db was set up with)

terraform init
terraform plan
terraform apply
```

## Verifying

```bash
ssh -J netlab-admin@10.0.1.10 netlab-admin@10.0.2.20
```

From inside the app VM, confirm it can reach the DB through the router
(this is the `private-net → data-net` rule finally being exercised by a
real, persistent VM instead of the earlier throwaway test VM):
```bash
sudo apt install -y netcat-openbsd   # if nc isn't already present
nc -zv 10.0.3.20 5432
```
Should report the connection succeeded.

## Next step

Write the actual Rust app (axum + sqlx: `/`, `/health`, `/visits`) and
get it running on this VM.

## The app

Deliberately minimal — its job is to prove the infrastructure, not be
interesting:

| Route | Behavior |
|---|---|
| `GET /` | `{"version": "...", "served_by": "netlab-app-rust"}` |
| `GET /health` | `200 OK`, empty body — for load balancer health checks later |
| `GET /visits` | Inserts a row, returns `{"visits": <count>}` |

Migrations run automatically on startup (`sqlx::migrate!`), tracked in
their own `_sqlx_migrations` table — safe to restart the app repeatedly.

## Deploying — VM path (current, manual)

This step builds the app **on the VM itself** via a live-installed Rust
toolchain, since `private-net` has internet access (unlike `data-net` —
see ADR 0005). This is a deliberate, temporary approach: Phase 4
(CI/CD) will replace this with a pre-built artifact deployed via a
pull-based mechanism, consistent with the "nothing compiled on the
target" principle this whole project is built around. For now, getting
something running and provable matters more than the deploy mechanism
being final.

```bash
ssh -J netlab-admin@10.0.1.10 netlab-admin@10.0.2.20
```

On the app VM:
```bash
# Install Rust (private-net has internet access, unlike data-net)
curl https://sh.rustup.rs -sSf | sh -s -- -y
source "$HOME/.cargo/env"

# Get the code onto the VM
git clone https://github.com/WilliamFly/wm-infra-netlab-app-rust.git
cd wm-infra-netlab-app-rust

# Real env file — fill in the actual db_app_password
cp .env.example .env
nano .env   # set DATABASE_URL's password to match wm-infra-netlab-db

cargo build --release
```

Install as a systemd service so it survives reboots/crashes:
```bash
sudo cp deploy/netlab-app-rust.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable --now netlab-app-rust
sudo systemctl status netlab-app-rust
```

## Verifying

From the app VM itself:
```bash
curl localhost:8080/
curl localhost:8080/health
curl localhost:8080/visits
curl localhost:8080/visits   # run again — count should increment
```

From the **router** (it has a direct interface on `private-net`,
`10.0.2.254` — no forwarding rule needed to reach `10.0.2.20` from there):
```bash
ssh netlab-admin@10.0.1.10
curl 10.0.2.20:8080/
```

## Security / Hardening

This VM has `harden-baseline` applied via `ansible/playbook-app.yml`:

- SSH hardened (no root login, no password auth)
- `ufw` enabled, default-deny incoming
- Port 8080 (the app) is scoped to `10.0.2.0/24` (private-net) only —
  reachable from the router and anything else on private-net, not the
  open internet
- `fail2ban` active on sshd
- `unattended-upgrades` enabled

Re-run after any change to `ansible/group_vars/app.yml`:

```bash
cd ansible
ansible-playbook playbook-app.yml
```

`private-net` has a NAT route via the router, so unlike the DB repo, no
egress toggle is needed here for package installs.
