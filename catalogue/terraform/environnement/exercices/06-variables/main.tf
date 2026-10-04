# Exercice de la leçon 6 : la configuration de l'ingress, version d'entraînement.
# Au lieu d'appeler Helm (impossible ici), on écrit les « values » dans un fichier : le raisonnement sur les
# variables et les secrets est exactement le même.

terraform {
  required_providers {
    local = {
      source = "hashicorp/local"
    }
  }
}

variable "domain" {
  type    = string
  default = "s3.exemple.test"
}

variable "cloudflare_email" {
  type = string
}

variable "cloudflare_apiKey" {
  type      = string
  sensitive = true
}

resource "local_file" "values" {
  filename = "${path.module}/values.yaml"
  content  = <<-YAML
    ingress:
      enabled: true
      hosts:
        - ${var.domain}
  YAML
}

resource "local_sensitive_file" "dns" {
  filename = "${path.module}/dns.env"
  content  = "CF_API_EMAIL=${var.cloudflare_email}\nCF_API_KEY=${var.cloudflare_apiKey}\n"
}
