---
titre: "Terraform"
icone: "🏗️"
resume: "Infrastructure décrite en code : le cluster de l'équipe et ses workspaces."
prerequis: [kubernetes-helm]
publie: true
couleur: "#7B42BC"
banniere: images/banniere.svg
environnement: environnement
---

**Terraform** est un outil qui crée et modifie une infrastructure (serveurs, services, fichiers) à partir de fichiers texte, au lieu de clics et de commandes tapées à la main. Le dépôt [`cluster-configuration`](https://gitlab.example.org/equipe/dev/cluster-configuration) l'utilise pour appeler **Helm** (l'outil qui installe des applications dans Kubernetes) et monter le cluster de l'équipe : HAProxy (le point d'entrée qui répartit les visites vers les bons services), MinIO (un stockage de fichiers), KubeDB (un gestionnaire de bases de données) et Keycloak (le service de connexion unique). Les travaux de production se font dans le workspace `production`.

**Chaque leçon a un labo réel** : tu reçois un conteneur Linux jetable avec le vrai programme `terraform`, et le serveur vérifie ce que tu as produit. Il n'y a ni cluster ni Internet : les fournisseurs `helm` et `kubernetes` de l'équipe ne sont pas exécutés. Tu t'exerces sur des fichiers locaux, avec les mêmes notions et les mêmes gestes.

## À qui s'adresse-t-il ?

Aux membres de l'équipe Infra, et à toute personne qui doit comprendre ou modifier la configuration du cluster. Aucune connaissance de Terraform n'est nécessaire : chaque mot est défini quand il apparaît et chaque exemple de code est expliqué ligne par ligne. Le dépôt `cluster-configuration` date d'environ 2019 : le parcours signale ce qui est probablement dépassé et ce qu'il faut confirmer avec l'équipe Infra.

## Plan

| # | Leçon | Durée |
| --- | --- | --- |
| 1 | Ressources, fournisseurs et variables | 40 min |
| 2 | `plan` et `apply` : lire avant d'appliquer | 45 min |
| 3 | L'état et le backend distant | 40 min |
| 4 | Les workspaces : production et les autres | 40 min |
| 5 | Éviter les coupures et les pertes de données | 45 min |
| 6 | Déployer des charts Helm avec Terraform | 45 min |

## Ce que tu sauras faire

- Lire un `plan` Terraform et repérer les actions destructrices
- Lancer `init`, `plan` et `apply` sur des ressources locales, et lire l'état
- Modifier la configuration du cluster en sécurité

**Prérequis :** parcours *Kubernetes et Helm*. **Durée estimée :** environ 4 h 15.
