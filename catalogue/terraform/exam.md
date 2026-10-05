---
title: "Examen de validation — Terraform"
draw: 5
pass_mark: 80
minutes: 10
shuffle: true
---

Cet examen valide les bases de Terraform dans l'équipe : lire un `plan`, repérer les actions destructrices, comprendre l'état, les workspaces et l'appel de Helm. Tu as 10 minutes pour 5 questions tirées au hasard ; il faut au moins 80 % de bonnes réponses.

:::quiz
Quelle commande calcule les différences sans rien modifier ?

- [ ] `terraform apply`
- [x] `terraform plan`
- [ ] `terraform init`
- [ ] `terraform fmt`

> `plan` compare code, état et monde réel, et n'applique rien.
:::

:::quiz
Un plan indique `-/+` devant un secret Kubernetes. Que va-t-il se passer ?

- [ ] Il sera modifié en place, sans interruption du service
- [ ] Il sera simplement lu, comme une donnée
- [x] Il sera détruit puis recréé
- [ ] Il sera déplacé vers un autre workspace avec son état

> `-/+` est un remplacement : la ressource est détruite avant d'être recréée.
:::

:::quiz
Le plan se termine par `Plan: 0 to add, 1 to change, 1 to destroy.` alors que tu ne voulais rien supprimer. Quelle est la bonne réaction ?

- [ ] Appliquer : le total « change » est faible, donc le risque aussi
- [ ] Relancer `init` pour que le plan soit recalculé proprement
- [ ] Passer en workspace `default` pour comparer les deux plans
- [x] Chercher quelle ressource est détruite et pourquoi, avant d'appliquer

> Une destruction inattendue est le signal d'arrêt : on identifie sa cause avant tout `apply`.
:::

:::quiz
À quoi sert le fichier d'état de Terraform ?

- [ ] À stocker le code source de la configuration
- [x] À associer chaque ressource du code à l'objet réel
- [ ] À conserver les mots de passe de l'équipe de façon chiffrée, à l'abri des regards
- [ ] À télécharger les fournisseurs nécessaires

> L'état associe chaque ressource du code à l'objet réel et mémorise les valeurs générées.
:::

:::quiz
Pourquoi l'état doit-il être protégé ?

- [x] Il peut contenir des secrets en clair
- [ ] Parce qu'il pèse plusieurs gigaoctets et ralentit tout le monde
- [ ] Parce qu'il contient tout l'historique Git du dépôt
- [ ] Parce qu'il est exécuté directement par le cluster Kubernetes

> Mots de passe générés et clés d'API s'y retrouvent en clair, même avec `sensitive = true`.
:::

:::quiz
D'après le code de `cluster-configuration`, que signifie `workspaces { prefix = "ingress-" }` ?

- [ ] Les ressources créées par ce dossier ont un nom qui commence par `ingress-`
- [x] Les workspaces distants de ce dossier s'appellent `ingress-…`, chacun avec son état
- [ ] Le dossier `ingress/` est ignoré quand on travaille en production
- [ ] Le fournisseur `ingress` est chargé avant les autres

> Le préfixe sépare l'état de chaque dossier dans le backend distant.
:::

:::quiz
D'après le `README.md` du dépôt (à confirmer avec l'équipe Infra), dans quel workspace se fait le travail de production ?

- [ ] `default`
- [ ] `prod-cluster`
- [x] `production`
- [ ] `main`

> Le README du dépôt indique le workspace `production`. Le dépôt ne dit pas quels autres workspaces existent : à confirmer avec l'équipe Infra.
:::

:::quiz
Que vaut `terraform.workspace` dans le workspace `production` ?

- [x] `production`
- [ ] `default`
- [ ] Le préfixe du backend suivi du nom, soit `keycloak-production`
- [ ] Une valeur vide, tant qu'aucun `apply` n'a eu lieu

> Avec le backend distant préfixé, `terraform.workspace` renvoie le nom sans préfixe.
:::

:::quiz
Pourquoi un `null_resource` avec `local-exec` demande-t-il une vigilance particulière à la lecture d'un plan ?

- [ ] Il ne s'affiche jamais dans le plan
- [ ] Il est toujours détruit en premier, avant les autres ressources
- [x] Le plan n'affiche pas ce que sa commande fera
- [ ] Il ne s'exécute qu'en local et n'a donc aucun effet sur le cluster

> Le plan indique la création ou le remplacement de la ressource, pas l'effet des commandes qu'elle lance.
:::

:::quiz
Un provisioner `when = "destroy"` exécute `kubectl delete` sur une base. Quand s'exécute-t-il ?

- [ ] À chaque `plan`
- [ ] À la création de la ressource
- [ ] Uniquement quand on utilise l'option `-target`
- [x] À la destruction de la ressource, y compris quand on retire son bloc du code

> Retirer le bloc du code suffit à ordonner la destruction, donc à lancer la commande.
:::

:::quiz
Que protège `lifecycle { prevent_destroy = true }` ?

- [ ] Contre toute suppression, y compris faite avec `kubectl`
- [x] Contre la destruction demandée par Terraform, tant que le bloc est dans le code
- [ ] Contre les modifications d'attributs
- [ ] Contre la perte de l'état

> C'est un garde-fou de relecture : retirer le bloc, ou supprimer l'objet avec `kubectl`, le contourne.
:::

:::quiz
Quel est l'intérêt d'appliquer un plan enregistré (`terraform apply plan.tfplan`) ?

- [ ] Appliquer plus vite en sautant l'état
- [x] Appliquer exactement ce qui a été relu
- [ ] Appliquer sans fournisseur installé
- [ ] Appliquer sur tous les workspaces d'un coup

> Le plan figé garantit que l'action exécutée est celle qui a été relue.
:::

:::quiz
Dans quel cas `-target` est-il acceptable ?

- [ ] Pour gagner du temps à chaque `apply`
- [ ] Pour contourner un plan qui contient une destruction
- [x] Pour une procédure de dépannage documentée, comme celle de KubeDB
- [ ] Pour appliquer à la place du workspace `production`

> `-target` est une exception : les autres ressources restent en retard sur le code.
:::

:::quiz
Que fait un `helm_release` Terraform ?

- [ ] Il construit une image Docker à partir du chart
- [ ] Il génère un workspace pour chaque release
- [x] Il installe ou met à jour une release Helm dans le cluster
- [ ] Il sauvegarde la base de données du chart

> Il joue le rôle de `helm install` / `helm upgrade`, avec le chart, la version et les values du code.
:::

:::quiz
Pourquoi épingler la `version` d'un chart dans `helm_release` ?

- [ ] Pour chiffrer les values
- [ ] Pour accélérer le plan
- [ ] Pour passer en Helm 3
- [x] Pour éviter qu'un `apply` fasse évoluer le chart sans décision de l'équipe

> Sans version, le chart appliqué dépend du dépôt de charts au moment de l'apply.
:::

:::quiz
Tu retrouves `install_tiller = false` dans la configuration du fournisseur `helm` du dépôt. Qu'en déduis-tu ?

- [ ] Que le dépôt cible Helm 3
- [ ] Que Terraform installe Tiller lui-même
- [x] Que le dépôt date de l'époque de Helm 2 : à confirmer avec l'équipe Infra
- [ ] Que le fournisseur est en échec et doit être réinstallé

> Tiller n'existe plus dans Helm 3 ; sa présence signale un dépôt ancien.
:::

:::quiz
Tu renommes une ressource dans le code sans autre précaution. Que prévoit le plan ?

- [ ] Rien : Terraform suit le nom
- [x] La destruction de l'ancienne et la création de la nouvelle
- [ ] Un simple changement d'étiquette
- [ ] Une erreur systématique qui bloque le plan

> Le nom fait partie de l'adresse : sans `state mv` ou `moved`, c'est une destruction suivie d'une création.
:::

:::quiz
Quelle est la bonne routine avant un `apply` en production ?

- [ ] Appliquer, puis lire le résultat
- [x] Vérifier le workspace, lire tout le plan et les provisioners, demander une relecture si une base ou un volume est touché
- [ ] Lancer `terraform destroy` pour repartir de zéro
- [ ] Supprimer l'état local pour forcer un plan propre

> On s'arrête sur toute destruction ou tout remplacement de donnée, et on vérifie qu'une sauvegarde existe.
:::

:::quiz
Tu viens d'écrire un fichier `.tf` dont l'indentation est irrégulière. Quelle commande le remet au style officiel ?

- [x] `terraform fmt`
- [ ] `terraform validate`, qui corrige aussi la mise en forme
- [ ] `terraform refresh`, qui reformate les fichiers du dossier
- [ ] `terraform init -upgrade`, qui réécrit les fichiers selon la version

> `fmt` réindente les fichiers ; `validate` vérifie seulement la cohérence de la configuration, sans rien réécrire.
:::

:::quiz
Une configuration utilise `var.domain` mais ne déclare aucun bloc `variable "domain"`. Que fait `terraform validate` ?

- [ ] Il crée la variable avec une valeur vide
- [ ] Il demande la valeur au clavier au moment de la validation
- [ ] Il ignore l'erreur jusqu'au moment du `apply`
- [x] Il signale une erreur : la variable n'est pas déclarée

> Terraform refuse une référence à une variable non déclarée dès la validation, sans rien créer.
:::

:::quiz
Tu es dans le workspace `essai`. Que contient le fichier d'état de `production` ?

- [ ] Une copie à jour de l'état de `essai`, synchronisée à chaque `apply`
- [ ] Rien : il n'existe pas tant qu'on n'est pas dans `production`
- [x] Il reste inchangé : chaque workspace a son propre état
- [ ] L'état fusionné de tous les workspaces du dossier

> Un workspace est un état distinct : appliquer dans `essai` ne modifie pas l'état de `production`.
:::

:::quiz
Dans un exercice, tu fournis `cloudflare_apiKey` dans un fichier `terraform.tfvars`. Que fais-tu de ce fichier ?

- [ ] Tu le commites avec le reste du code pour que l'équipe l'ait
- [x] Tu l'inscris dans `.gitignore` pour que Git ne le suive pas
- [ ] Tu le chiffres avec `sensitive = true`
- [ ] Tu le copies dans le dossier du module appelé

> Un `.tfvars` qui contient un secret ne se commite jamais ; `sensitive` masque l'affichage mais ne chiffre rien.
:::
