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
