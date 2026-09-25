variable "base_image_path" {
  description = "Path to the Ubuntu 24.04 cloud image (qcow2) on the host"
  type        = string
}

variable "storage_pool" {
  description = "libvirt storage pool to create VM disks in"
  type        = string
  default     = "default"
}

variable "ssh_public_key" {
  description = "SSH public key installed on this repo's VMs' admin user"
  type        = string
}

variable "app_vcpu" {
  description = "vCPUs for the Rust app VM"
  type        = number
  default     = 1
}

variable "app_memory_mb" {
  description = "Memory (MB) for the Rust app VM"
  type        = number
  default     = 1024
}

# DB connection details — the database itself is NOT provisioned here.
# See wm-infra-netlab-db (separate repo, single owner of that resource).
variable "db_host" {
  description = "IP of the shared Postgres VM (owned by wm-infra-netlab-db)"
  type        = string
  default     = "10.0.3.20"
}

variable "db_name" {
  description = "Database name (must match what wm-infra-netlab-db created)"
  type        = string
  default     = "netlab_app"
}

variable "db_app_user" {
  description = "Postgres role this app connects as"
  type        = string
  default     = "app_user"
}

variable "db_app_password" {
  description = "Password for db_app_user — get this out-of-band, never commit the real value"
  type        = string
  sensitive   = true
}
