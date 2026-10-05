---
id: eviter-coupures-et-pertes
title: "Éviter les coupures et les pertes de données"
summary: "Reconnaître les actions destructrices dans un plan et utiliser les garde-fous : prevent_destroy, sauvegardes et revue."
minutes: 45
objectives:
  - Identifier dans un plan les remplacements qui coupent un service ou détruisent des données
  - Utiliser prevent_destroy et comprendre ses limites
  - Appliquer une routine de relecture avant tout apply en production
---

## À quoi ça sert et pourquoi

Supprimer une ligne de code semble anodin. Pour Terraform, c'est un ordre : « détruis ce qui correspondait ». Si cela concerne un disque ou une base, les données disparaissent. Cette leçon rassemble les réflexes qui évitent une coupure de service (les utilisateur·rice·s ne peuvent plus se connecter) ou une perte de données.

Quelques termes : un **volume persistant** (*PersistentVolumeClaim*, PVC) est une demande de disque dans Kubernetes ; **KubeDB** est l'outil installé dans le cluster qui crée et surveille les bases de données ; **Keycloak** est le service de connexion unique de l'équipe ; un **YAML** est un fichier de configuration fait de lignes `clé: valeur`, très utilisé par Kubernetes ; **Gio** veut dire gibioctet (une unité de taille de disque).

Dans `cluster-configuration`, plusieurs ressources portent des données : le stockage de MinIO (75 Gio), les bases PostgreSQL de Keycloak, les secrets, les certificats. Une erreur ne se « rattrape » pas toujours. Cette leçon rassemble les réflexes qui protègent.

## Les trois façons de détruire sans le vouloir

:::cards
### Un remplacement

Un attribut change, la ressource est détruite puis recréée (`-/+`, `forces replacement`). Pour un volume, une base ou un secret, cela peut effacer ou régénérer des données.

### Une suppression du code

Retirer un bloc `resource` ou `module` du code ordonne à Terraform de **détruire** l'objet correspondant. Même chose en renommant sans `moved` (un bloc de code qui dit à Terraform « cette ressource a changé de nom ») ni `state mv`.

### Un effet de bord

Un `provisioner` `when = "destroy"` (une commande lancée par Terraform à la destruction) exécute `mc rb` pour supprimer un bucket MinIO, ou `kubectl delete` pour supprimer une base.
:::

## Cas du dépôt : les garde-fous existants

Dans `minio/minio.tf`, le volume de MinIO est protégé :

```hcl
resource "kubernetes_persistent_volume_claim" "minio_storage" {
  metadata {
    name      = "minio"
    namespace = "minio"
  }

  spec {
    access_modes = ["ReadWriteOnce"]
    resources {
      requests = {
        storage = "75Gi"
      }
    }
  }

  lifecycle {
    prevent_destroy = true
  }
}
```

Lis le bloc `resource` ligne par ligne : `metadata` donne le nom et l'espace de noms de la demande de disque ; `spec` la décrit (`access_modes` : un seul nœud à la fois peut écrire ; `requests.storage` : 75 Gio demandés). Enfin, `lifecycle` règle le comportement de la ressource ; `prevent_destroy = true` fait **échouer** tout plan qui voudrait détruire cette ressource. Voici un exemple minimal que tu peux valider chez toi. `terraform_data` est une ressource neutre fournie par Terraform, qui ne crée rien de réel : idéale pour s'exercer. `input` est la valeur qu'elle conserve.

```hcl
resource "terraform_data" "donnees" {
  input = "important"

  lifecycle {
    prevent_destroy = true
  }
}
```

Si le code change de façon à détruire cette ressource, `terraform plan` s'arrête avec une erreur du type `Instance cannot be destroyed`. Mais attention aux limites :

- La protection disparaît si tu **retires le bloc** `lifecycle` (ou la ressource entière) du code : le plan n'a plus rien à protéger.
- Elle ne couvre que la ressource qui la porte. Les ressources voisines n'en bénéficient pas.
- Un `terraform destroy` complet est lui aussi bloqué, mais seulement tant que le bloc existe.

:::danger Les garde-fous sont dans le code, pas dans le cluster
`prevent_destroy` n'empêche pas quelqu'un de supprimer l'objet avec `kubectl`, ni un `state rm` de le « détacher ». C'est un filet contre les erreurs de relecture, pas contre les gestes volontaires.
:::

## Cas du dépôt : la base de données Keycloak

Le module `postgres/` ne crée pas la base avec une ressource native : il rend un fichier YAML KubeDB (`database.yml`) puis le passe à `kubectl apply` (la commande qui envoie un fichier de configuration au cluster) via un `local-exec` (`null_resource.apply`). Une ressource `null_resource.cleaner` exécute `kubectl delete` quand elle est détruite. Conséquences pour la lecture d'un plan :

- Modifier `replicas` ou `size` ne montre qu'un `null_resource.apply` remplacé : **le plan ne dit pas ce que fera KubeDB** sur la base.
- Retirer le module `database` du code détruit `null_resource.cleaner`, donc exécute `kubectl delete` sur l'objet `Postgres`.
- Le fichier `database.yml` choisit `terminationPolicy: Delete` (la règle de KubeDB sur ce qu'il supprime avec la base) : supprimer l'objet Kubernetes supprime la base, mais **conserve** les sauvegardes et les instantanés dans le bucket (c'est ce que dit son commentaire). Sans sauvegarde récente et testée, cela ne suffit pas.

La procédure de restauration (rejouer depuis MinIO) est donnée en lien dans `keycloak/main.tf`, vers la documentation de KubeDB 0.12.

## Ta routine avant un apply en production

1. `terraform workspace show` : je suis dans `production`.
2. `terraform plan -out=plan.tfplan` : je produis un plan.
3. Je cherche `destroy`, `replace`, `forces replacement` et le total « to destroy ».
4. Pour chaque ressource concernée, j'ouvre le `.tf` et lis les `provisioner`.
5. Si un volume, une base ou un secret est touché : j'**arrête** et je demande une relecture à une autre personne.
6. Je vérifie qu'une sauvegarde récente existe (parcours *Sauvegardes et stockage objet*).
7. `terraform apply plan.tfplan`, puis je contrôle l'état du service.

:::tip Deux paires d'yeux
Un plan de production relu par une seconde personne détecte la plupart des erreurs. N'hésite pas à coller le plan dans un échange avec l'équipe Infra avant d'appliquer.
:::

:::info À confirmer avec l'équipe Infra
Y a-t-il une règle de relecture pour les `apply` en production (fenêtre de maintenance, validation obligatoire) ? Les sauvegardes des états et des bases (Keycloak, MinIO) sont-elles toujours en place, et qui prévenir en cas de problème ?
:::

## Entraîne-toi

:::info Des garde-fous pour de faux
La ressource protégée est un `terraform_data` (une ressource qui ne crée rien de réel), pas un volume MinIO : tu peux casser sans risque. Les fournisseurs `helm` et `kubernetes` de l'équipe ne sont pas exécutés dans ce labo ; le raisonnement sur `prevent_destroy` et sur la lecture du plan est le même.
:::

Commandes du labo : `terraform plan -destroy` simule la destruction de tout, sans rien détruire ; `2>&1` envoie aussi les messages d'erreur vers la même destination que le texte normal ; `sed -i 's/v1/v2/' fichier` remplace `v1` par `v2` dans un fichier ; `nano fichier` ouvre un petit éditeur de texte.

:::lab
engine: real
intro: |
  Ton dossier contient un `main.tf` avec une ressource « précieuse » `terraform_data.donnees` protégée par `prevent_destroy = true`, et un fichier `note.txt` sans importance. Son attribut `triggers_replace = ["v1"]` dit : « si cette liste change, détruis et recrée la ressource ». Tu vas provoquer ce remplacement, voir le garde-fou t'arrêter, constater ses limites, puis tout remettre en ordre.
commands:
  - cp -R /opt/exercices/05-garde-fous/. .
steps:
  - text: 'Initialise et applique : `terraform init` puis `terraform apply -auto-approve`. `terraform state list` doit citer `terraform_data.donnees`'
    hint: 'Deux commandes à la suite.'
    checks:
      - output-contains: ['terraform state list', 'terraform_data\.donnees']
    solution:
      - terraform init
      - terraform apply -auto-approve
  - text: 'Dans `main.tf`, remplace `"v1"` par `"v2"` dans `triggers_replace`, puis lance `terraform plan` : il doit échouer (code d''erreur) avec `Instance cannot be destroyed`'
    after: [1]
    hint: 'Édite avec `nano main.tf` ou `sed -i ''s/v1/v2/'' main.tf`. Le plan affiche d''abord `must be replaced`, puis l''erreur : `prevent_destroy` a bloqué la destruction.'
    checks:
      - env-file-contains: [main.tf, '"v2"']
      - output-contains: ['terraform plan -no-color -input=false 2>&1', 'cannot be destroyed']
      - command-fails: 'terraform plan -input=false'
    solution:
      - sed -i 's/v1/v2/' main.tf
      - terraform plan || echo "plan refusé, comme prévu"
  - text: 'Garde une trace de l''erreur : `terraform plan -no-color > erreur.txt 2>&1` (`2>&1` envoie aussi les messages d''erreur dans le fichier)'
    after: [2]
    hint: 'Le fichier `erreur.txt` doit contenir le message `Instance cannot be destroyed`.'
    checks:
      - env-file-contains: [erreur.txt, 'cannot be destroyed']
      - command-succeeds: 'terraform plan -no-color 2>&1 | cmp -s - erreur.txt'
    solution:
      - terraform plan -no-color > erreur.txt 2>&1 || true
  - text: 'Constate la limite du garde-fou : supprime le bloc `lifecycle { … }` de `main.tf`. `terraform plan` doit alors réussir et annoncer `must be replaced` : la donnée serait détruite'
    after: [2]
    hint: 'Supprime les lignes du bloc `lifecycle` (accolade fermante comprise). Sans le bloc, plus de protection : c''est pourquoi un plan se relit et une relecture à deux est utile.'
    checks:
      - command-succeeds: 'terraform plan -input=false'
      - output-contains: ['terraform plan -no-color -input=false', 'terraform_data\.donnees must be replaced']
    solution:
      - sed -i '/lifecycle {/,/^  }/d' main.tf
      - terraform plan
  - text: 'Remets tout en ordre : retrouve `"v1"` et remets le bloc `lifecycle` avec `prevent_destroy = true`. `terraform plan -detailed-exitcode` doit répondre « aucun changement »'
    after: [4]
    hint: 'Le bloc est `lifecycle {` puis `prevent_destroy = true` puis `}`, à l''intérieur de la ressource `terraform_data.donnees`.'
    checks:
      - env-file-contains: [main.tf, 'prevent_destroy\s*=\s*true']
      - env-file-contains: [main.tf, '"v1"']
      - command-succeeds: 'terraform plan -input=false -detailed-exitcode'
    solution:
      - write:
          main.tf: |
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
  - text: 'Vérifie que la protection bloque aussi une destruction complète : `terraform plan -destroy -no-color > destroy.txt 2>&1` (le plan de destruction simule `terraform destroy` sans rien détruire)'
    after: [5]
    hint: 'Le fichier doit contenir `cannot be destroyed`. N''utilise jamais `terraform destroy` sur un vrai environnement sans accord de l''équipe Infra.'
    checks:
      - env-file-contains: [destroy.txt, 'cannot be destroyed']
      - command-succeeds: 'terraform plan -destroy -no-color 2>&1 | cmp -s - destroy.txt'
    solution:
      - terraform plan -destroy -no-color > destroy.txt 2>&1 || true
:::

## Vérifie tes acquis

:::quiz
Que fait `prevent_destroy = true` dans un bloc `lifecycle` ?

- [ ] Il empêche toute modification de la ressource
- [x] Il fait échouer un plan qui détruirait la ressource
- [ ] Il crée une sauvegarde avant la destruction
- [ ] Il protège aussi contre `kubectl delete`

> Il ne protège que contre une destruction demandée par Terraform, tant que le bloc est présent dans le code.
:::

:::quiz
Tu retires du code le bloc `module "database"` de Keycloak. Que prévoit le plan ?

- [ ] Rien, le module est simplement ignoré
- [ ] Une sauvegarde automatique de la base
- [x] La destruction des ressources du module, dont une commande `kubectl delete`
- [ ] Un déplacement de la base dans un autre workspace

> Retirer un bloc du code demande à Terraform de détruire l'objet. Ici le `provisioner` de destruction supprime l'objet KubeDB.
:::

:::quiz
Le plan montre `null_resource.apply must be replaced` après un changement de `replicas`. Qu'est-ce que cela t'apprend ?

- [ ] La base sera détruite et recréée vide
- [ ] Rien n'a changé dans le cluster
- [x] Une commande `kubectl apply` sera exécutée, mais l'effet réel sur la base n'est pas visible dans le plan
- [ ] KubeDB refuse la modification

> Les ressources `null_resource` cachent l'action réelle dans le provisioner : lis le `.tf` et prépare une sauvegarde.
:::

:::quiz
Quel est le bon moment pour demander une relecture à une autre personne ?

- [ ] Après l'apply, pour confirmer que tout va bien
- [x] Avant l'apply, dès que le plan touche un volume, une base ou un secret
- [ ] Seulement quand le plan contient plus de dix ressources
- [ ] Jamais, le plan suffit

> Avant d'appliquer, on peut encore tout arrêter. Après, la destruction est faite.
:::
