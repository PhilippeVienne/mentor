---
id: plan-et-apply
titre: "plan et apply : lire avant d'appliquer"
resume: "Lire un plan Terraform ligne par ligne et repérer les destructions avant qu'il soit trop tard."
duree: 45
objectifs:
  - Enchaîner init, plan et apply dans le bon ordre
  - Interpréter les symboles +, ~, - et -/+ d'un plan
  - Repérer « must be replaced » et « destroy » avant d'appliquer
---

## À quoi ça sert et pourquoi

Quand tu modifies un fichier `.tf`, rien ne se passe encore dans le cluster. Terraform fonctionne en deux temps, comme un devis avant des travaux : d'abord il **calcule et montre** ce qu'il compte faire (`plan`), puis, si tu es d'accord, il le **fait** (`apply`). Le plan est ton dernier moment pour détecter une erreur, surtout si elle supprime quelque chose.

Le `README.md` de `cluster-configuration` le dit en toutes lettres : lis attentivement ce que Terraform prévoit de faire, pour ne causer ni coupure, ni perte de données. Cette leçon apprend à le faire.

## Le cycle de base

```bash
terraform init      # télécharge les fournisseurs, se connecte au backend
terraform plan      # calcule les différences, ne modifie rien
terraform apply     # applique les différences, après confirmation
```

Ligne par ligne :

- `terraform init` : prépare le dossier. Il télécharge les extensions (fournisseurs) et se connecte à l'endroit où est conservé l'état, c'est-à-dire la mémoire de Terraform (leçon suivante).
- `terraform plan` : compare ce que décrit le code et ce qui existe, puis liste les différences.
- `terraform apply` : exécute ces différences.
- Le texte après `#` est un commentaire, ignoré par le shell.

- `init` se lance une fois par dossier (et après chaque changement de fournisseur ou de backend).
- `plan` compare trois choses : le **code**, l'**état** enregistré et le **monde réel**. Il ne touche à rien.
- `apply` refait le calcul, affiche le plan et attend que tu tapes `yes`.

:::tip Le réflexe
Lance toujours `terraform plan` avant `apply`, lis-le entièrement, et relis-le encore s'il est long. Un `yes` tapé trop vite est irréversible.
:::

## Lire les symboles

| Symbole | Sens | Gravité |
| --- | --- | --- |
| `+` | Ressource créée | Faible |
| `~` | Modifiée en place | À lire : que change-t-on ? |
| `-` | **Détruite** | Danger |
| `-/+` | **Détruite puis recréée** | Danger |
| `<=` | Donnée lue (*data source* : une information consultée sans rien créer) | Aucune |

La dernière ligne résume tout : `Plan: 1 to add, 1 to change, 1 to destroy.` Si le chiffre « destroy » n'est pas **0** alors que tu n'as rien voulu supprimer, **arrête-toi**.

## Un plan réel, annoté

Un **bucket** (« seau ») est un espace de stockage de fichiers dans MinIO, un service de stockage compatible S3. Un **provisioner** est une commande que Terraform lance sur ton poste quand il crée ou détruit une ressource (`local-exec` = « exécute une commande locale »). Une `null_resource` est une ressource « vide » qui ne sert qu'à porter de tels provisioners.

Imagine que quelqu'un change la valeur `region` d'un bucket MinIO (module `minio-bucket`). Le plan ressemble à ceci (extrait simplifié, valeurs factices) :

```console
Terraform will perform the following actions:

  # null_resource.bucket must be replaced
-/+ resource "null_resource" "bucket" {
      ~ id       = "4308502518" -> (known after apply)
      ~ triggers = { # forces replacement
          ~ "region" = "ovh-gra5" -> "ovh-gra9" # forces replacement
            "name"   = "keycloak-backup-production"
        }
    }

Plan: 1 to add, 0 to change, 1 to destroy.
```

Les lignes `~ id = … -> (known after apply)` signifient « cette valeur sera connue après l'application ». Le bloc `triggers` est une liste de valeurs qui, si l'une change, force la recréation de la ressource. Lis-le ainsi :

1. `must be replaced` et `-/+` : la ressource sera **supprimée puis recréée**, pas simplement modifiée.
2. `# forces replacement` désigne l'attribut responsable. Ici, changer `region` suffit.
3. Dans ce dépôt, `null_resource.bucket` a un provisioner `when = "destroy"` qui lance `mc rb` sur le bucket (`mc` est le *MinIO Client*, l'outil en ligne de commande de MinIO ; `rb` signifie *remove bucket*, « supprimer le seau »). **Détruire cette ressource exécute donc une commande de suppression**, que le plan n'affiche pas.

:::danger Le plan ne montre pas ce que font les provisioners
Un `local-exec` (très utilisé dans `minio-bucket`, `postgres` et `helm-setup`, trois dossiers du dépôt) exécute une commande arbitraire. Terraform n'affiche que « la ressource est remplacée », jamais ce que la commande fera. Avant d'appliquer, ouvre le `.tf` et lis les blocs `provisioner`, surtout ceux marqués `when = "destroy"`.
:::

## Sauvegarder un plan, appliquer exactement celui-là

Un plan peut être enregistré dans un fichier, puis appliqué tel quel :

```bash
terraform plan -out=plan.tfplan
terraform show plan.tfplan
terraform apply plan.tfplan
```

Ligne par ligne : `-out=plan.tfplan` écrit le plan dans un fichier ; `terraform show` l'affiche à nouveau ; `terraform apply plan.tfplan` l'exécute sans le recalculer. Appliquer un fichier de plan garantit que **ce que tu as relu est ce qui est exécuté**, même si l'état change entre-temps. Dans ce cas, `apply` ne redemande pas de confirmation : la relecture a déjà eu lieu.

## Cibler une ressource : à n'utiliser qu'en dépannage

Le `README.md` du dossier `kubedb/` contient une procédure d'époque en deux temps pour contourner un bug (CRD créée trop tard) :

```bash
terraform apply -target=helm_release.kubedb
kubectl get crds -l app=kubedb -w
terraform apply
```

Ligne par ligne : la première commande n'applique que la ressource `helm_release.kubedb` (l'installation de l'opérateur KubeDB par Helm) ; la deuxième attend que les CRD existent ; la troisième applique tout le reste.

Une **CRD** (*CustomResourceDefinition*) est une extension de Kubernetes qui lui apprend un nouveau type d'objet (ici, les bases de KubeDB) : elle doit exister avant les objets de ce type. La commande `kubectl get crds -l app=kubedb -w` surveille leur apparition : `kubectl` est l'outil en ligne de commande de Kubernetes, `get crds` liste les CRD, `-l app=kubedb` ne garde que celles étiquetées `kubedb`, et `-w` (*watch*) continue d'afficher les nouvelles au fil de l'eau. Le raisonnement : appliquer d'abord l'opérateur, attendre les CRD, puis tout le reste.

`-target` limite l'action à une ressource et ses dépendances. C'est une exception documentée, pas un mode normal : les autres ressources restent en retard sur le code, et le plan suivant risque de te surprendre.

:::info À confirmer avec l'équipe Infra
Le contournement `-target` de `kubedb/README.md` date de mai 2019 (version 0.12.0 de KubeDB). Est-il toujours nécessaire, ou KubeDB a-t-il été mis à jour ?
:::

## Entraîne-toi

:::info Un plan pour de faux
Ici, pas de cluster : les « ressources » sont des fichiers de configuration créés sur ton poste par le fournisseur `local`. La lecture du plan (symboles `+`, `-`, `-/+`, lignes `must be replaced` et `to destroy`) est **exactement** la même qu'en production. Les fournisseurs `helm` et `kubernetes` de l'équipe ne sont pas exécutés dans ce labo.
:::

:::labo
moteur: reel
intro: |
  Ton dossier contient un `main.tf` qui décrit deux fichiers : `app.conf` (avec le contenu `mode = test`) et `ancien.conf`. Tu vas suivre le cycle complet : `init`, plan enregistré, application de ce plan, modification, lecture d'un remplacement, puis repérage d'une destruction.
commandes:
  - cp -R /opt/exercices/02-plan-et-apply/. .
etapes:
  - texte: 'Initialise le dossier avec `terraform init`'
    indice: 'La commande est `terraform init` ; elle crée `.terraform.lock.hcl`.'
    verif:
      - commande-reussit: 'find -L .terraform/providers -name "terraform-provider-local*" -type f | grep -q . && grep -q "h1:" .terraform.lock.hcl'
    solution:
      - terraform init
  - texte: 'Calcule le plan et enregistre-le dans un fichier : `terraform plan -out=plan.tfplan`. Lis-le : deux lignes `+` (créations), `Plan: 2 to add`'
    apres: [1]
    indice: '`-out=plan.tfplan` écrit le plan dans un fichier. Rien n''est créé tant que tu n''appliques pas.'
    verif:
      - sortie-contient: ['terraform show -no-color plan.tfplan', 'local_file\.app will be created']
      - sortie-contient: ['terraform show -no-color plan.tfplan', 'local_file\.ancien will be created']
    solution:
      - terraform plan -out=plan.tfplan
  - texte: 'Applique exactement ce plan avec `terraform apply plan.tfplan` (pas de confirmation à taper : tu l''as déjà relu)'
    apres: [2]
    indice: 'Après l''application, `ls` montre `app.conf` et `ancien.conf`.'
    verif:
      - fichier-contient-dans-env: [app.conf, 'mode = test']
      - fichier-existe-dans-env: ancien.conf
      - commande-reussit: 'terraform state list | grep -q "^local_file.app$" && terraform state list | grep -q "^local_file.ancien$"'
    solution:
      - terraform apply plan.tfplan
  - texte: 'Dans `main.tf`, change le contenu de `app.conf` en `mode = production`, puis lis `terraform plan` : la ressource `local_file.app` doit apparaître avec `must be replaced` (symbole `-/+`). Ne l''applique pas encore'
    apres: [3]
    indice: 'Édite avec `nano main.tf` (ou `sed -i ''s/mode = test/mode = production/'' main.tf`), puis `terraform plan`. Repère la ligne `# forces replacement` : changer le contenu d''un `local_file` force son remplacement.'
    verif:
      - fichier-contient-dans-env: [main.tf, 'mode = production']
      - sortie-contient: ['terraform plan -no-color -input=false', 'local_file\.app must be replaced']
    solution:
      - sed -i 's/mode = test/mode = production/' main.tf
      - terraform plan
  - texte: 'Applique le changement avec `terraform apply -auto-approve` : `app.conf` doit contenir `mode = production`'
    apres: [4]
    indice: 'Le résumé final doit être `Apply complete! Resources: 1 added, 0 changed, 1 destroyed.`'
    verif:
      - fichier-contient-dans-env: [app.conf, 'mode = production']
      - commande-reussit: terraform plan -input=false -detailed-exitcode
    solution:
      - terraform apply -auto-approve
  - texte: 'Supprime de `main.tf` tout le bloc `resource "local_file" "ancien"`, puis lance `terraform plan` : le résumé doit être `Plan: 0 to add, 0 to change, 1 to destroy.`. N''applique pas : tu viens de voir qu''un simple retrait de code ordonne une destruction'
    apres: [5]
    indice: 'Supprime les lignes depuis `resource "local_file" "ancien" {` jusqu''à l''accolade fermante `}` incluse. Un plan avec un « destroy » que tu n''avais pas prévu est le signal d''arrêt.'
    verif:
      - sortie-contient: ['terraform plan -no-color -input=false', 'Plan: 0 to add, 0 to change, 1 to destroy\.']
    solution:
      - sed -i '/^resource "local_file" "ancien"/,/^}/d' main.tf
      - terraform plan
:::

## Vérifie tes acquis

:::quiz
Que signifie le symbole `-/+` devant une ressource ?

- [ ] Elle est modifiée en place
- [ ] Elle est lue sans changement
- [x] Elle sera détruite puis recréée
- [ ] Elle est ignorée par ce plan

> `-/+` est un remplacement : la ressource disparaît avant d'être recréée, avec les conséquences que cela implique.
:::

:::quiz
Le plan se termine par `Plan: 2 to add, 0 to change, 1 to destroy.` alors que tu voulais seulement ajouter un secret. Que fais-tu ?

- [ ] Tu appliques, Terraform sait ce qu'il fait
- [x] Tu cherches quelle ressource est détruite et pourquoi, avant tout `apply`
- [ ] Tu ajoutes `-target` pour appliquer plus vite
- [ ] Tu supprimes le fichier d'état

> Une destruction inattendue est le signal d'arrêt. Il faut comprendre sa cause (attribut qui force le remplacement, ressource retirée du code…).
:::

:::quiz
Quel est l'intérêt de `terraform plan -out=plan.tfplan` suivi de `terraform apply plan.tfplan` ?

- [ ] Appliquer sans télécharger les fournisseurs
- [ ] Contourner la protection contre la destruction
- [x] Appliquer exactement le plan qui a été relu
- [ ] Appliquer le plan à tous les workspaces d'un coup

> Le plan enregistré est figé : ce qui est exécuté est ce qui a été relu.
:::

:::quiz
Pourquoi un `plan` ne suffit-il pas toujours à mesurer un risque dans ce dépôt ?

- [ ] Parce qu'il ne détecte jamais les suppressions
- [x] Parce que les blocs `provisioner "local-exec"` exécutent des commandes que le plan n'affiche pas
- [ ] Parce qu'il modifie l'état réel
- [ ] Parce qu'il ne fonctionne qu'en workspace `default`

> Le plan indique qu'une ressource `null_resource` est remplacée, pas ce que ses commandes `mc` ou `kubectl` feront.
:::
