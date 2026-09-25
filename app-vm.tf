# App VM on private-net — no app code yet, this step only proves the VM
# exists, boots correctly, and can reach the shared DB through the
# router. Same decoupling pattern as wm-infra-netlab-db: own base image
# import, attaches to the network by NAME (not network_id), no
# cross-repo Terraform state references.

locals {
  mac_app_private = "52:54:00:ab:03:01"
}

resource "libvirt_volume" "ubuntu_base" {
  name   = "wm-netlab-app-rust-ubuntu-24.04-base"
  pool   = var.storage_pool
  source = var.base_image_path
  format = "qcow2"
}

resource "libvirt_volume" "app_disk" {
  name           = "wm-netlab-app-rust.qcow2"
  pool           = var.storage_pool
  base_volume_id = libvirt_volume.ubuntu_base.id
  format         = "qcow2"
}

resource "libvirt_cloudinit_disk" "app" {
  name = "wm-netlab-app-rust-cloudinit.iso"
  pool = var.storage_pool

  user_data = templatefile("${path.module}/cloud-init/app-user-data.yaml.tftpl", {
    ssh_public_key = var.ssh_public_key
  })

  network_config = templatefile("${path.module}/cloud-init/app-network-config.yaml.tftpl", {
    mac_private = local.mac_app_private
  })
}

resource "libvirt_domain" "app" {
  name   = "wm-netlab-app-rust"
  vcpu   = var.app_vcpu
  memory = var.app_memory_mb

  cloudinit = libvirt_cloudinit_disk.app.id

  disk {
    volume_id = libvirt_volume.app_disk.id
  }

  network_interface {
    network_name   = "wm-netlab-private" # by name — not managed by this state
    mac            = local.mac_app_private
    wait_for_lease = false
  }

  console {
    type        = "pty"
    target_type = "serial"
    target_port = "0"
  }

  autostart = true
}

output "app_vm_ip" {
  value = "10.0.2.20 (only reachable via the router, e.g. `ssh -J netlab-admin@10.0.1.10 netlab-admin@10.0.2.20`)"
}
