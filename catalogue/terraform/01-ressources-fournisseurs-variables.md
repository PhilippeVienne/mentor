---
id: ressources-fournisseurs-variables
title: "Ressources, fournisseurs et variables"
summary: "Lire un fichier .tf du dépôt cluster-configuration : ressources, variables, valeurs locales, sorties et modules."
minutes: 40
objectives:
  - Distinguer ressource, fournisseur (provider), variable, valeur locale et sortie
  - Lire un module et ses entrées dans cluster-configuration
  - Repérer la syntaxe d'époque (Terraform 0.12) face à la syntaxe actuelle
---

## À quoi ça sert et pourquoi

Imagine une recette de cuisine : au lieu de refaire le plat de mémoire (et d'oublier un ingrédient), tu écris la recette, et n'importe qui peut la relire, la corriger et refaire le même plat. **Terraform** fait cela pour une infrastructure : tu écris dans des fichiers texte *ce que tu veux obtenir*, et l'outil crée, modifie ou supprime ce qu'il faut pour y arriver. On parle d'**infrastructure décrite en code**.

Les avantages concrets pour l'équipe :

- **Reproductible** : on peut remonter un cluster identique après une panne.
- **Relisable** : un changement passe en revue avant d'être appliqué, comme du code.
- **Traçable** : l'historique Git dit qui a changé quoi.

Quelques mots à connaître dès maintenant :

- Un **cluster** (Kubernetes) est un groupe de machines qui font tourner les applications de l'équipe. Un **espace de noms** (*namespace*) y est un « dossier » qui regroupe les objets d'un service.
- Un **secret** Kubernetes est un objet qui stocke un mot de passe ou une clé.
- Un **chart Helm** est un paquet qui installe une application dans le cluster (parcours *Kubernetes et Helm*).
- Le **terminal** est la fenêtre où tu tapes des commandes ; un **dépôt** Git est le dossier versionné qui contient les fichiers.
- Quelques services de l'équipe reviennent dans ce parcours : **MinIO** (un stockage de fichiers, compatible avec le service S3 d'Amazon), **PostgreSQL** (un système de bases de données), **KubeDB** (un outil installé dans le cluster qui crée et surveille des bases de données, détaillé à la leçon 5) et **Keycloak** (le service de connexion unique de l'équipe).

Le cluster de l'équipe n'a pas été monté à la main : il est décrit dans des fichiers `.tf` du dépôt `cluster-configuration`. Ces fichiers disent **ce qu'on veut obtenir** (un espace de noms, un secret, un chart Helm) et Terraform se charge de le créer. Pour ne pas casser la production, la première compétence est de savoir **lire** ces fichiers.

:::info Prérequis
Ce parcours suppose acquises les bases de Kubernetes et de Helm (parcours *Kubernetes et Helm*) : espace de noms, secret, chart, release.
:::

## Les briques du langage

Les fichiers Terraform sont écrits en **HCL** (*HashiCorp Configuration Language*), un langage de configuration lisible (pas de programmation compliquée), et leur nom se termine par `.tf`. Un bloc s'écrit `type "étiquette" { attributs }` : un mot-clé, un ou deux noms entre guillemets, puis des lignes `nom = valeur` entre accolades. Il n'a que quelques notions. Chaque fichier `.tf` d'un dossier est lu, l'ordre des fichiers n'a aucune importance.

:::cards
### Fournisseur (provider)

Une extension (*plugin*) qui sait parler à un service par son **API** (l'interface par laquelle un programme commande un autre programme) : `kubernetes`, `helm`, `random`, `tls`… Elle est téléchargée par `terraform init`.

### Ressource (resource)

Un objet géré par Terraform : un secret Kubernetes, une release Helm, un mot de passe aléatoire. Terraform le crée, le modifie, le détruit.

### Variable, locale, sortie

Une **variable** est une entrée (`variable`), une **locale** un nom de calcul interne (`locals`), une **sortie** (`output`) une valeur exposée à la fin.
:::

## Une ressource, pas à pas

Dans le dossier `minio/` (le dossier du dépôt qui installe MinIO), le mot de passe d'administration est généré par Terraform plutôt que choisi à la main. Voici la même idée avec la syntaxe actuelle (les valeurs sont factices) :

```hcl
terraform {
  required_providers {
    random = {
      source = "hashicorp/random"
    }
  }
}

variable "domain" {
  type    = string
  default = "s3.exemple.test"
}

locals {
  minio_namespace = "minio"
}

resource "random_string" "access_key" {
  length  = 18
  special = false
}

output "minio_access_key" {
  value     = random_string.access_key.result
  sensitive = true
}

output "minio_url" {
  value = "https://${var.domain}"
}
```

Lis ce fichier bloc par bloc :

- `terraform { required_providers { … } }` : déclare de quel plugin le dossier a besoin (`random`, qui sait fabriquer des valeurs aléatoires) et où le télécharger (`hashicorp/random`).
- `variable "domain" { … }` : crée une entrée nommée `domain`. `type = string` dit que c'est du texte ; `default` donne la valeur utilisée si personne n'en fournit une autre.
- `locals { … }` : définit un nom de confort, réutilisable dans le fichier (ici `minio_namespace`).
- `resource "random_string" "access_key" { … }` : demande un texte aléatoire de 18 caractères (`length`) sans caractères spéciaux (`special = false`).
- `output "minio_access_key"` : affiche le résultat à la fin, en le marquant sensible. L'autre `output` compose une adresse web avec `"https://${var.domain}"` : `https` désigne le web chiffré, et `${…}` insère la valeur d'une variable dans un texte.

Ce qu'il faut observer :

- `resource "random_string" "access_key"` : le **type** (`random_string`) et le **nom local** (`access_key`). Ensemble, ils forment l'adresse `random_string.access_key`.
- `var.domain` lit une variable, `local.minio_namespace` une locale, `random_string.access_key.result` un attribut d'une autre ressource.
- Terraform déduit de ces références **l'ordre de création** : le secret ne peut pas exister avant la valeur qu'il utilise.
- `sensitive = true` masque la valeur dans l'affichage du terminal. Elle reste écrite dans l'état (leçon 3).

## Les modules : réutiliser un dossier

Un **module** est un dossier de `.tf` que l'on appelle avec des paramètres. Dans `keycloak/main.tf`, le dossier `../postgres` (une base PostgreSQL gérée par KubeDB, l'outil de gestion de bases de données du cluster, présenté à la leçon 5) est appelé comme un module, avec un bucket de sauvegarde fourni par `../minio-bucket` :

```hcl
module "backup-bucket" {
  source     = "../minio-bucket"
  bucketName = "keycloak-backup-production"
}

module "database" {
  source   = "../postgres"
  name     = "db-keycloak"
  replicas = 2
  size     = "3Gi"
}
```

Lecture ligne à ligne : `module "backup-bucket"` appelle le dossier `../minio-bucket` et lui passe le nom du bucket voulu (`bucketName`) ; `module "database"` donne un nom à l'appel ; `source` est le dossier à réutiliser (`../postgres` signifie « le dossier voisin `postgres` ») ; les lignes suivantes (`name`, `replicas`, `size`) sont les paramètres : nom de la base, nombre de copies (2 pour résister à une panne), taille du disque (3 Gio). Un **bucket** est un « seau » de stockage d'objets, sorte de dossier géré par MinIO (service de stockage compatible S3, vu dans le parcours *Sauvegardes et stockage objet*).

Les sorties d'un module se lisent avec `module.<nom>.<sortie>`, par exemple `module.backup-bucket.access_token` dans le dépôt réel. C'est ainsi que l'identifiant du bucket arrive jusqu'à la base.

## Lire le dépôt de l'équipe : la syntaxe d'époque

Le dépôt date d'environ 2019 et utilise la syntaxe de Terraform 0.11/0.12. Tu verras donc des écritures que tu ne dois plus copier :

```hcl
# Ancienne écriture, vue dans cluster-configuration
resource "kubernetes_secret" "minio" {
  metadata {
    name      = "${local.minio_secret}"
    namespace = "${local.minio_namespace}"
  }
  depends_on = ["kubernetes_namespace.minio"]
}
```

Ce bloc crée un secret Kubernetes. Ligne par ligne : `resource "kubernetes_secret" "minio"` déclare la ressource ; `metadata { … }` contient son identité (`name` et `namespace`, ici lus dans des valeurs locales) ; `depends_on` impose de créer d'abord l'espace de noms. Les guillemets autour de `${…}` et autour de la liste sont les marques de l'ancienne syntaxe :

| Dans le dépôt | Écriture actuelle |
| --- | --- |
| `"${local.minio_secret}"` | `local.minio_secret` |
| `depends_on = ["kubernetes_namespace.minio"]` | `depends_on = [kubernetes_namespace.minio]` |
| `type = "list"` | `type = list(string)` |
| `data { … }` (attribut sans `=`) | `data = { … }` |

:::warning Ne mélange pas les générations
Une version récente de Terraform refuse une partie de ces écritures. Avant de « moderniser » un fichier, vérifie la version que l'équipe Infra utilise réellement : un fichier réécrit trop tôt ne s'exécutera plus avec l'ancienne version du programme.
:::

:::info À confirmer avec l'équipe Infra
Quelle version de Terraform (et quels fournisseurs) sont utilisés aujourd'hui pour `cluster-configuration` ? Le dépôt semble écrit pour Terraform 0.12 et le fournisseur `helm` pilotant Tiller (le composant de Helm 2 installé dans le cluster, voir la leçon 6), ce qui est probablement obsolète : y a-t-il eu une migration ?
:::

## Entraîne-toi

:::info Ce que ce labo fait (et ne fait pas)
Tu disposes d'un vrai terminal Linux avec le programme `terraform` installé. Il n'y a ni cluster Kubernetes, ni Helm, ni Internet : les fournisseurs `helm` et `kubernetes` de l'équipe **ne sont pas exécutés** ici. On s'exerce avec les fournisseurs `random` (valeurs aléatoires) et `local` (création de fichiers sur ton poste), qui utilisent exactement les mêmes notions : ressources, variables, sorties.
:::

Les commandes du labo, en bref :

- `terraform init` prépare le dossier : il installe les fournisseurs listés dans `required_providers` et écrit `.terraform.lock.hcl`, qui fige leurs versions.
- `terraform fmt` réindente les fichiers `.tf` selon le style officiel (`-check` se contente de dire s'ils sont déjà bien formatés).
- `terraform validate` vérifie que la configuration est cohérente (variables déclarées, syntaxe correcte) sans rien créer.
- `terraform apply -auto-approve` crée ce qui manque ; `-auto-approve` évite de taper `yes` (à réserver aux exercices). `-var nom=valeur` remplace une variable pour cette exécution.
- `terraform output -raw nom` affiche la valeur d'une sortie sans guillemets ; `>` envoie un résultat dans un fichier au lieu de l'écran.

:::lab
engine: real
intro: |
  Ton dossier de travail contient un fichier `main.tf` (le fichier de configuration Terraform, `tf` pour *terraform*). Il décrit une chaîne aléatoire (`random_string`), un fichier `bonjour.txt` à créer (`local_file`) et deux sorties. Il a deux défauts : il est mal mis en forme (pas d'indentation) et il utilise `var.domain` sans avoir déclaré la variable `domain`. Ouvre-le avec `cat main.tf` ou `nano main.tf`, lis les commentaires, puis corrige tout et applique.
commands:
  - cp -R /opt/exercices/01-ressources/. .
steps:
  - text: 'Initialise le dossier avec `terraform init` : il prépare les fournisseurs `random` et `local` (ici depuis un miroir local, sans réseau)'
    hint: 'Tape simplement `terraform init`. Il crée un dossier caché `.terraform` et un fichier `.terraform.lock.hcl` qui fige les versions des fournisseurs.'
    checks:
      - command-succeeds: 'find -L .terraform/providers -name "terraform-provider-random*" -type f | grep -q . && find -L .terraform/providers -name "terraform-provider-local*" -type f | grep -q . && grep -q "h1:" .terraform.lock.hcl'
    solution:
      - terraform init
  - text: 'Mets `main.tf` en forme avec `terraform fmt` (il réindente les fichiers du dossier)'
    hint: 'Contrôle d''abord avec `terraform fmt -check` : il répond par un code d''erreur tant que le fichier n''est pas bien formaté. Puis lance `terraform fmt`.'
    checks:
      - command-succeeds: terraform fmt -check
      - command-succeeds: 'grep -q "random_string" main.tf && grep -q "local_file" main.tf && ! cmp -s main.tf /opt/exercices/01-ressources/main.tf'
    solution:
      - terraform fmt
  - text: 'Corrige l''erreur : déclare la variable `domain` (texte, valeur par défaut `s3.exemple.test`) dans `main.tf`, jusqu''à ce que `terraform validate` réponde « Success »'
    after: [1]
    hint: |
      Ajoute dans `main.tf` un bloc `variable "domain"` qui contient deux lignes : `type = string` et `default = "s3.exemple.test"`. Puis lance `terraform validate` et relis le message d'erreur s'il y en a un.
    checks:
      - command-succeeds: terraform validate
      - output-contains: ['echo var.domain | terraform console', '^"s3\.exemple\.test"$']
      - command-succeeds: 'grep -q "random_string" main.tf && grep -q "local_file" main.tf'
    solution:
      - write:
          main.tf: |
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

            variable "domain" {
              type    = string
              default = "s3.exemple.test"
            }

            resource "random_string" "access_key" {
              length  = 18
              special = false
            }

            resource "local_file" "bonjour" {
              filename = "${path.module}/bonjour.txt"
              content  = "Domaine : ${var.domain}\n"
            }

            output "minio_access_key" {
              value     = random_string.access_key.result
              sensitive = true
            }

            output "minio_url" {
              value = "https://${var.domain}"
            }
  - text: 'Applique la configuration avec `terraform apply -auto-approve` : Terraform crée la chaîne aléatoire et le fichier `bonjour.txt`'
    after: [3]
    hint: '`-auto-approve` répond « yes » à ta place (réservé aux exercices !). Puis regarde `cat bonjour.txt` : il contient la valeur par défaut du domaine.'
    checks:
      - env-file-contains: [bonjour.txt, 'Domaine : s3\.exemple\.test']
      - command-succeeds: 'terraform state list | grep -q "^local_file.bonjour$" && terraform state list | grep -q "^random_string.access_key$"'
      - command-succeeds: terraform plan -input=false -detailed-exitcode
    solution:
      - terraform apply -auto-approve
  - text: 'Relance l''application en changeant la variable en ligne de commande : `terraform apply -auto-approve -var domain=s3.prod.test`. Le fichier `bonjour.txt` doit être mis à jour'
    after: [4]
    hint: '`-var nom=valeur` remplace la valeur par défaut, pour cette exécution seulement. Le plan montre `bonjour.txt` remplacé car son contenu change.'
    checks:
      - env-file-contains: [bonjour.txt, 'Domaine : s3\.prod\.test']
      - command-succeeds: terraform plan -input=false -detailed-exitcode -var domain=s3.prod.test
    solution:
      - terraform apply -auto-approve -var domain=s3.prod.test
  - text: 'Écris dans `url.txt` la valeur de la sortie `minio_url` (sans guillemets) avec `terraform output -raw minio_url > url.txt`'
    after: [5]
    hint: 'Compare avec `terraform output` seul : la sortie `minio_access_key` y apparaît comme `<sensitive>`, c''est l''effet de `sensitive = true`.'
    checks:
      - command-succeeds: 'test "$(cat url.txt)" = "https://s3.prod.test" && test "$(cat url.txt)" = "$(terraform output -raw minio_url)"'
    solution:
      - terraform output -raw minio_url > url.txt
:::

## Vérifie tes acquis

:::quiz
Que désigne `random_string.access_key.result` ?

- [ ] Une variable d'entrée du module
- [x] Un attribut de la ressource `random_string` nommée `access_key`
- [ ] Une sortie du fournisseur `random`
- [ ] Un fichier généré dans le dossier courant

> Une adresse de ressource se lit `type.nom`, puis l'attribut voulu.
:::

:::quiz
Dans quel ordre Terraform crée-t-il les ressources d'un dossier ?

- [ ] Dans l'ordre alphabétique des fichiers
- [ ] Dans l'ordre d'apparition dans le fichier
- [x] Selon les références entre ressources, déduites du code
- [ ] Au hasard, d'où les `depends_on` partout

> Terraform construit un graphe de dépendances. `depends_on` ne sert que pour une dépendance invisible dans le code.
:::

:::quiz
Que fait `sensitive = true` sur une sortie ?

- [ ] Il chiffre la valeur dans l'état
- [ ] Il empêche tout accès à la valeur
- [x] Il masque la valeur dans l'affichage du terminal
- [ ] Il supprime la ressource à la fin du `apply`

> La valeur reste présente en clair dans l'état : l'option évite seulement de l'afficher.
:::

:::quiz
Dans `cluster-configuration`, tu lis `depends_on = ["kubernetes_namespace.minio"]` avec des guillemets. Que conclus-tu ?

- [x] C'est la syntaxe d'époque, les guillemets ne sont plus nécessaires aujourd'hui
- [ ] C'est une erreur à corriger immédiatement en production
- [ ] Les guillemets désignent une ressource externe à Terraform
- [ ] Ils indiquent que la dépendance est facultative

> C'est l'écriture de Terraform 0.11/0.12. Ne la corrige pas sans connaître la version utilisée par l'équipe Infra.
:::
