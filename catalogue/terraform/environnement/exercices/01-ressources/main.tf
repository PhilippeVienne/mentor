# Exercice de la leçon 1 : ce fichier est mal mis en forme ET contient une erreur. Tu vas corriger les deux.

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

# La variable « domain » devrait être déclarée ici, mais elle a été oubliée.

resource "random_string" "access_key" {
length = 18
special = false
}

resource "local_file" "bonjour" {
  filename = "${path.module}/bonjour.txt"
  content  = "Domaine : ${var.domain}\n"
}

output "minio_access_key" {
value = random_string.access_key.result
sensitive = true
}

output "minio_url" {
  value = "https://${var.domain}"
}
