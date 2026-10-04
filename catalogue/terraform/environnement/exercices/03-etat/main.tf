# Exercice de la leçon 3 : un nom et un mot de passe générés au hasard, et un fichier qui les mentionne.

terraform {
  required_providers {
    random = {
      source = "hashicorp/random"
    }
    local = {
      source = "hashicorp/local"
    }
  }
}

resource "random_pet" "nom" {
  length = 2
}

resource "random_password" "admin" {
  length  = 16
  special = false
}

resource "local_file" "rapport" {
  filename = "${path.module}/rapport.txt"
  content  = "Service : ${random_pet.nom.id}\n"
}

output "nom" {
  value = random_pet.nom.id
}

output "mot_de_passe" {
  value     = random_password.admin.result
  sensitive = true
}
