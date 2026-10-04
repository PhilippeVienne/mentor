# Exercice de la leçon 5 : une ressource « précieuse » protégée par prevent_destroy.
# terraform_data ne crée rien de réel : elle sert à s'exercer sans risque.

terraform {
  required_providers {
    local = {
      source = "hashicorp/local"
    }
  }
}

resource "terraform_data" "donnees" {
  input            = "important"
  triggers_replace = ["v1"]

  lifecycle {
    prevent_destroy = true
  }
}

resource "local_file" "note" {
  filename = "${path.module}/note.txt"
  content  = "Ce fichier peut être recréé sans dégât.\n"
}
