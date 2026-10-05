---
id: terraform-et-helm
title: "Déployer des charts Helm avec Terraform"
summary: "Lire une ressource helm_release du dépôt, comprendre ses valeurs, ses dépendances et ce qui est probablement dépassé."
minutes: 45
objectives:
  - Lire une ressource helm_release et ses values
  - Expliquer l'ordre entre dossiers (ingress, kubedb, minio, keycloak)
  - Repérer les éléments d'époque à faire confirmer (Tiller, dépôt stable, secrets)
---

## À quoi ça sert et pourquoi

Un **chart** Helm est un paquet qui installe une application dans Kubernetes ; une **release** est une installation précise de ce chart. Lancer `helm install` à la main fonctionne, mais personne ne sait ensuite quelles options ont été utilisées. En demandant à Terraform d'appeler Helm, ces options sont écrites dans un fichier relu et versionné.

Dans le parcours *Kubernetes et Helm*, tu lançais `helm install` à la main. Dans `cluster-configuration`, **Terraform appelle Helm** : chaque service du cluster est une ressource `helm_release`. Les services concernés sont **HAProxy** (le point d'entrée qui répartit les visites entre les services), cert-manager, MinIO, KubeDB et Keycloak (présentés dans les leçons précédentes). Le résultat est le même, mais reproductible et relisable.

## Une release, ligne par ligne

Un **ingress** est la porte d'entrée du cluster : il aiguille les adresses web vers les bons services. **cert-manager** obtient les certificats HTTPS (les preuves d'identité d'un site, qui permettent de chiffrer les échanges) ; **External-DNS** crée les noms de domaine. Un **domaine** est l'adresse lisible d'un site (`s3.exemple.test`). Voici une ressource inspirée de `ingress/helm.tf` (valeurs simplifiées) :

```hcl
terraform {
  required_providers {
    helm = {
      source  = "hashicorp/helm"
      version = "~> 2.0"
    }
  }
}

variable "domain" {
  type    = string
  default = "s3.exemple.test"
}

resource "helm_release" "minio" {
  name       = "minio"
  repository = "https://charts.example.test/stable"
  chart      = "minio"
  version    = "1.2.3"
  namespace  = "minio"
  timeout    = 900

  values = [
    <<-YAML
      ingress:
        enabled: true
        hosts:
          - ${var.domain}
      existingSecret: minio-admin
    YAML
  ]

  set {
    name  = "persistence.existingClaim"
    value = "minio"
  }
}
```

Ligne par ligne : `terraform { required_providers { helm = { … } } }` demande le fournisseur `helm`, en version `~> 2.0` (toute 2.x à partir de 2.0) ; `variable "domain"` déclare une entrée texte avec une valeur par défaut ; `resource "helm_release" "minio"` décrit une installation : `name` est le nom de la release, `repository` l'adresse du dépôt de charts, `chart` le chart voulu, `version` sa version, `namespace` l'espace de noms visé, `timeout` le délai maximal en secondes.

Dans `values`, `<<-YAML … YAML` est un texte sur plusieurs lignes (le mot `YAML` ouvre et ferme le bloc). Il contient les réglages du chart au format YAML. `${var.domain}` y insère la variable. Le bloc `set` modifie une seule valeur, repérée par son chemin (`persistence.existingClaim`). Un **YAML** est un fichier de configuration fait de lignes `clé: valeur`, l'indentation montrant l'imbrication.

| Champ | Équivalent Helm | Remarque |
| --- | --- | --- |
| `name` | nom de la release | Il change = nouvelle release |
| `chart`, `version` | chart et `--version` | Épingler la version évite les surprises |
| `namespace` | `--namespace` | |
| `values` | `-f values.yaml` | Du YAML dans une chaîne, où `${…}` est remplacé par Terraform |
| `set` | `--set clé=valeur` | Une valeur à la fois |
| `timeout` | `--timeout` | HAProxy utilise 900 secondes dans le dépôt |

Dans le dépôt, `kubedb/main.tf` épingle `version = "0.12.0"`, alors que `ingress/` laisse certains charts sans version : leur mise à jour dépend de ce que propose le dépôt de charts à ce moment-là.

:::warning Lis les values comme du code
Un plan sur `helm_release` indique « `values` modifié », sans toujours détailler ce que le chart fera de la différence. Une modification d'une valeur peut redémarrer des pods (les petits groupes de conteneurs qui font tourner l'application), recréer un volume ou changer un ingress. Lis la différence de `values` ligne par ligne.
:::

## L'ordre entre les dossiers

Chaque dossier (`ingress/`, `kubedb/`, `minio/`, `keycloak/`) a son état et se déploie séparément. Il existe donc un ordre **de fait** :

1. `ingress/` : HAProxy, cert-manager, External-DNS. Rien ne sera joignable sans eux.
2. `kubedb/` : l'opérateur de bases. Il doit exister avant de créer une base.
3. `minio/` : le stockage objet. Les bases y sauvegardent leurs données.
4. `keycloak/` : il utilise les trois précédents (certificat, base, bucket de sauvegarde).

Dans un dossier, `depends_on` fixe l'ordre quand le code ne le montre pas : `helm_release.keycloak` attend `module.database` et `module.backup-bucket`.

:::tip Une dépendance, c'est un ordre de destruction aussi
À la destruction, Terraform inverse l'ordre. Un `depends_on` oublié peut supprimer le namespace avant les ressources qu'il contient.
:::

## Les secrets dans les values

Dans le dépôt, les mots de passe sont générés (`random_string`) et injectés dans les `values` ; les identifiants pour **Cloudflare** (le service qui gère les noms de domaine de l'association : le DNS, c'est-à-dire l'annuaire qui associe un nom à une adresse) sont fournis par les variables `cloudflare_apiKey` (la clé d'accès à l'API) et `cloudflare_email`. Leurs valeurs viennent d'un fichier `terraform.tfvars` : un fichier `.tfvars` contient des lignes `nom = "valeur"` qui donnent leur valeur aux variables, et celui-ci est ignoré par Git (`ingress/.gitignore`, un fichier qui liste ce que Git ne doit pas suivre). Retiens :

- **Jamais de secret réel dans un `.tf` ni dans un `.tfvars` commité.**
- Une valeur passée à `values` est aussi écrite dans l'**état** (leçon 3).
- Avec `sensitive = true`, un plan masque l'affichage mais pas le stockage.

## Ce qui est probablement obsolète

Le dépôt date d'environ 2019. Plusieurs éléments sont à confirmer avant de s'en inspirer :

- **Tiller (Helm 2)** : composant installé dans le cluster pour exécuter les commandes Helm 2. `helm-setup/` installe Tiller avec TLS (le chiffrement des échanges, le « S » de HTTPS, qui protège le dialogue entre Helm et Tiller), et les fournisseurs `helm` sont configurés avec `install_tiller = false` (ne pas l'installer, il existe déjà) et `enable_tls = true` (chiffrer les échanges). Helm 3 n'a plus de Tiller.
- **Dépôts de charts `stable` et `incubator`** : la source `stable/…` est l'ancien dépôt central de Helm, qui n'est plus maintenu.
- **cert-manager 0.6** : les CRD sont appliquées depuis une URL de la branche `release-0.6`, avec des API `certmanager.k8s.io` abandonnées depuis.
- **KubeDB 0.12.0** : très ancienne version, dont le `README.md` décrit un contournement de bug.
- **Commandes `local-exec`** : `mc` et `kubectl` doivent être installés et configurés sur le poste de la personne qui applique.

:::info À confirmer avec l'équipe Infra
Le cluster tourne-t-il toujours avec ces charts et ces versions, ou le dépôt est-il en retard sur la réalité ? Où trouver la configuration actuelle (un autre dépôt, une autre branche) ? Les identifiants (`~/.mentor-admin/`, `setup.sh`) se récupèrent-ils toujours de la même façon ? Ne lance aucun `apply` à partir de ce dépôt sans leur accord.
:::

## Entraîne-toi

:::info Pas de Helm dans ce labo
Le fournisseur `helm` et un cluster ne sont **pas** disponibles ici. À la place de `helm_release`, tu écris les « values » dans un fichier avec le fournisseur `local`. Ce qui s'exerce est ce qui compte vraiment : déclarer des variables sans valeur par défaut, les fournir par un fichier `.tfvars`, ne pas commiter les secrets, surcharger une valeur.
:::

Commandes du labo : `printf 'texte\n' > fichier` écrit un texte dans un fichier (`\n` = retour à la ligne) ; `-var-file=fichier` demande à Terraform de lire les variables d'un fichier de ton choix ; `grep` cherche un texte dans un fichier.

:::lab
engine: real
intro: |
  Ton dossier contient un `main.tf` inspiré de `ingress/` : trois variables (`domain`, et deux sans valeur par défaut, `cloudflare_email` et `cloudflare_apiKey`, la seconde étant sensible) ; un fichier `values.yaml` ; un fichier `dns.env` qui reçoit les identifiants. Les valeurs que tu saisis sont **factices** : ne tape jamais un vrai secret dans un exercice.
commands:
  - cp -R /opt/exercices/06-variables/. .
steps:
  - text: 'Initialise le dossier avec `terraform init`'
    hint: 'La commande est `terraform init`.'
    checks:
      - command-succeeds: 'find -L .terraform/providers -name "terraform-provider-local*" -type f | grep -q . && grep -q "h1:" .terraform.lock.hcl'
    solution:
      - terraform init
  - text: 'Lance `terraform plan -input=false` : il échoue car deux variables n''ont pas de valeur. Crée un fichier `terraform.tfvars` avec `cloudflare_email = "admin@exemple.test"` et `cloudflare_apiKey = "cle-factice-pour-le-labo"` (une ligne chacune) jusqu''à ce que le plan réussisse'
    after: [1]
    hint: 'Un fichier `terraform.tfvars` est lu automatiquement. Chaque ligne a la forme `nom = "valeur"`.'
    checks:
      - env-file-contains: [terraform.tfvars, 'cloudflare_email']
      - env-file-contains: [terraform.tfvars, 'cloudflare_apiKey']
      - command-succeeds: 'terraform plan -input=false'
    solution:
      - write:
          terraform.tfvars: |
            cloudflare_email  = "admin@exemple.test"
            cloudflare_apiKey = "cle-factice-pour-le-labo"
  - text: 'Applique avec `terraform apply -auto-approve` : les fichiers `values.yaml` et `dns.env` sont créés'
    after: [2]
    hint: 'Regarde `cat values.yaml` : le domaine par défaut y figure.'
    checks:
      - env-file-contains: [values.yaml, 's3\.exemple\.test']
      - env-file-contains: [dns.env, 'CF_API_EMAIL=admin@exemple\.test']
      - command-succeeds: 'terraform state list | grep -q "^local_file.values$" && terraform state list | grep -q "^local_sensitive_file.dns$"'
    solution:
      - terraform apply -auto-approve
  - text: 'Protège les secrets : crée un fichier `.gitignore` qui liste `terraform.tfvars` et `*.tfstate` (une ligne chacun), comme `ingress/.gitignore` dans le dépôt réel'
    after: [2]
    hint: 'Git ne suit pas les fichiers listés dans `.gitignore`. Ici : `printf ''terraform.tfvars\n*.tfstate\n'' > .gitignore`.'
    checks:
      - env-file-contains: [.gitignore, '(?m)^terraform\.tfvars\s*$']
      - env-file-contains: [.gitignore, '(?m)^\*\.tfstate\s*$']
    solution:
      - printf 'terraform.tfvars\n*.tfstate\n' > .gitignore
  - text: 'Change le domaine avec un second fichier de variables : crée `production.tfvars` contenant `domain = "s3.prod.test"` et applique avec `terraform apply -auto-approve -var-file=production.tfvars`. `values.yaml` doit citer ce domaine'
    after: [3]
    hint: 'Un fichier nommé autrement que `terraform.tfvars` n''est pas lu seul : on le désigne avec `-var-file=…`. C''est ainsi qu''on garde une valeur par environnement.'
    checks:
      - env-file-contains: [production.tfvars, 'domain']
      - env-file-contains: [values.yaml, 's3\.prod\.test']
      - command-succeeds: terraform plan -input=false -detailed-exitcode -var-file=production.tfvars
    solution:
      - write:
          production.tfvars: |
            domain = "s3.prod.test"
      - terraform apply -auto-approve -var-file=production.tfvars
  - text: 'Change l''e-mail dans `terraform.tfvars` (par exemple `autre@exemple.test`), puis enregistre le plan dans `plan.txt` avec `terraform plan -no-color -var-file=production.tfvars > plan.txt`. Vérifie avec `grep` : la clé `cle-factice-pour-le-labo` n''apparaît pas dans le plan, car la variable est sensible'
    after: [5]
    hint: 'Cherche `sensitive` dans `plan.txt` : Terraform y écrit `(sensitive value)` à la place du contenu. Mais la valeur reste dans l''état (`grep cle-factice terraform.tfstate`).'
    checks:
      - env-file-contains: [terraform.tfvars, 'autre@exemple\.test']
      - env-file-contains: [plan.txt, 'sensitive']
      - command-succeeds: 'terraform plan -no-color -var-file=production.tfvars | cmp -s - plan.txt'
      - command-succeeds: '! grep -q cle-factice-pour-le-labo plan.txt'
    solution:
      - sed -i 's/admin@exemple.test/autre@exemple.test/' terraform.tfvars
      - terraform plan -no-color -var-file=production.tfvars > plan.txt
:::

## Vérifie tes acquis

:::quiz
Quel est l'équivalent d'un bloc `set { name = "a.b" value = "c" }` dans Helm ?

- [ ] `helm repo add a.b c`
- [x] `--set a.b=c`
- [ ] `helm uninstall a.b`
- [ ] `-f a.b.yaml`

> `set` passe une valeur du chart, comme l'option `--set` de `helm install`.
:::

:::quiz
Pourquoi épingler `version` dans un `helm_release` ?

- [ ] Pour accélérer le téléchargement
- [ ] Pour que Terraform ignore le chart
- [x] Pour qu'un `apply` ne fasse pas évoluer le chart à l'insu de l'équipe
- [ ] Pour chiffrer les values

> Sans version, le chart appliqué dépend de ce que le dépôt propose au moment de l'apply.
:::

:::quiz
Un mot de passe est injecté dans `values` avec une ressource `random_string`. Où est-il aussi enregistré ?

- [ ] Nulle part, il n'existe que dans le pod
- [ ] Dans les journaux Git du dépôt
- [x] Dans l'état Terraform
- [ ] Dans le fichier `Chart.yaml`

> L'état conserve les valeurs générées et utilisées : il doit donc être protégé.
:::

:::quiz
`helm-setup/` installe Tiller. Que dois-tu en conclure aujourd'hui ?

- [ ] Que Helm 3 en a besoin
- [x] Que cette partie date de Helm 2 et doit être confirmée avec l'équipe Infra
- [ ] Que Tiller est obligatoire pour tout `helm_release`
- [ ] Qu'il faut le réinstaller avant chaque plan

> Helm 3 a supprimé Tiller. C'est un signe de l'âge du dépôt : il faut demander ce qui est réellement en service.
:::
