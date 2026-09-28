# wm-infra-netlab-docker-host

Standalone, reusable Ansible role that installs Docker Engine (official
Docker apt repo — `docker-ce`, `docker-ce-cli`, `containerd.io`,
`docker-buildx-plugin`, `docker-compose-plugin`) on Ubuntu 24.04.

Used as a dependency by app repos that run a Docker-path deployment,
consumed via `ansible-galaxy install -r requirements.yml` — not copied.

## Usage

Add to the consuming repo's `ansible/requirements.yml`:

```yaml
roles:
  - src: https://github.com/WilliamFly/wm-infra-netlab-docker-host
    name: docker-host
```

Set which users get added to the `docker` group in that repo's
`group_vars`:

```yaml
docker_host_users:
  - netlab-admin
```

## Requirements

- Ansible >= 2.15
- Ubuntu 24.04 (noble) — other versions untested
