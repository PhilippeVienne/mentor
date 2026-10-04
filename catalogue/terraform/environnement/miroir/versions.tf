# Liste des fournisseurs à copier dans le miroir local (versions épinglées).
terraform {
  required_providers {
    local = {
      source  = "hashicorp/local"
      version = "2.9.1"
    }
    random = {
      source  = "hashicorp/random"
      version = "3.9.1"
    }
    null = {
      source  = "hashicorp/null"
      version = "3.3.2"
    }
    time = {
      source  = "hashicorp/time"
      version = "0.14.2"
    }
  }
}
