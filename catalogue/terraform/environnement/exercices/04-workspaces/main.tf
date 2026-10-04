# Exercice de la leçon 4 : le même code, un fichier par workspace (le nom change avec terraform.workspace).

terraform {
  required_providers {
    local = {
      source = "hashicorp/local"
    }
  }
}

locals {
  bucket = "keycloak-backup-${terraform.workspace}"

  replicas_par_workspace = {
    production = 2
    default    = 1
  }

  replicas = lookup(local.replicas_par_workspace, terraform.workspace, 1)
}

resource "local_file" "bucket" {
  filename = "${path.module}/${local.bucket}.txt"
  content  = "replicas = ${local.replicas}\n"
}

output "bucket" {
  value = local.bucket
}
