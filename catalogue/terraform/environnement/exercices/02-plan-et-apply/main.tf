# Exercice de la leçon 2 : deux fichiers de configuration fabriqués par Terraform.
# Le fournisseur « local » sait créer des fichiers sur ce poste : c'est notre « cluster » d'entraînement.

terraform {
  required_providers {
    local = {
      source = "hashicorp/local"
    }
  }
}

resource "local_file" "app" {
  filename = "${path.module}/app.conf"
  content  = "mode = test\n"
}

resource "local_file" "ancien" {
  filename = "${path.module}/ancien.conf"
  content  = "reste d'une ancienne version\n"
}
