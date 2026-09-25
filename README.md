# wm-infra-netlab-app-rust

Phase 2 of [wm-infra-netlab](https://github.com/WilliamFly/wm-infra-netlab):
the Rust app, deployed via both a VM path and a Docker path.

## Current step: not started yet

The database this app uses lives in its own repo,
[wm-infra-netlab-db](https://github.com/WilliamFly/wm-infra-netlab-db) —
not here. This repo connects to it by IP (`db_host`, default `10.0.3.20`)
and never provisions or owns it. See
[ADR 0004](https://github.com/WilliamFly/wm-infra-netlab/blob/main/docs/decisions/0004-shared-db-own-repo.md)
for why.

## Prerequisites

- `network-foundation`'s networks + router already applied and working
- `wm-infra-netlab-db` already applied and reachable at `10.0.3.20`
- Terraform >= 1.9, `dmacvicar/libvirt` provider `>= 0.8.1, < 0.9`

## Next step

Rust app VM on `private-net` — VM only, no app code yet, same pattern as
the DB VM and the earlier router test VM. Once that boots and can reach
`10.0.3.20:5432` through the router, the actual Rust app (axum + sqlx)
gets written and deployed onto it.
