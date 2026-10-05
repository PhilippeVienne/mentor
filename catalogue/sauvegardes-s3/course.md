---
title: "Sauvegardes et stockage objet"
icon: "💾"
summary: "S3, MinIO, `pg_dump`, sauvegarde de volumes Kubernetes : sauvegarder, conserver et restaurer."
requires: [docker-advanced]
published: true
color: "#16A34A"
banner: images/banniere.svg
environment: environnement
---

Les données des associations doivent survivre à une panne. L'équipe utilise des images dédiées : [`backups3`](https://gitlab.example.org/equipe/dev/backups3), [`docker-postgres-backup`](https://gitlab.example.org/equipe/dev/docker-postgres-backup) ou [`backup-files-swift`](https://gitlab.example.org/equipe/dev/backup-files-swift). Ce parcours t'apprend à comprendre ces outils **en pratiquant dans un vrai environnement** : chaque labo te prête un conteneur Linux jetable avec un serveur de stockage objet MinIO, une base PostgreSQL et les outils de sauvegarde (`mc`, `aws`, `pg_dump`, `pg_restore`, des versions simplifiées de `backup.py` et `restore.py`), tous démarrés pour toi. C'est le serveur du portail qui vérifie le résultat de chaque étape.

## À qui s'adresse-t-il ?

Aux membres de l'équipe Infra, et à toute personne qui doit sauvegarder ou restaurer un service de l'équipe. Aucune connaissance préalable des bases de données ni de Kubernetes n'est nécessaire : le parcours *Docker avancé* (conteneurs, images, volumes Docker) suffit, et les notions manquantes sont expliquées au fil des leçons.

:::info Deux notions que *Docker avancé* ne couvre pas
- **SQL et les bases de données** : une base de données range des informations dans des tables (des tableaux de lignes et de colonnes) ; SQL est le langage pour lui parler. Seules quelques commandes `pg_dump` et `psql` t'en serviront ici (leçons 3 et 6).
- **Kubernetes** : un logiciel qui fait tourner des conteneurs Docker sur plusieurs machines et qui décrit tout dans des fichiers YAML. Il intervient dans la leçon 4, où ses mots (pod, Job, CronJob, secret, volume) sont définis avant d'être utilisés.
:::

## Plan

1. Règle 3-2-1 et notions de RPO / RTO (tu archives un dossier et le restaures depuis un bucket)
2. Stockage objet : S3, MinIO, Ceph, Swift (tu crées un bucket, une politique et un compte limité)
3. Sauvegarder PostgreSQL et MySQL (tu sauvegardes une vraie base vers S3)
4. Sauvegarder des volumes Kubernetes (tu archives un dossier avec `backup.py`)
5. Politiques de conservation (tu actives le versionnement et le cycle de vie)
6. Tester une restauration (tu restaures une version datée et une base)

## Ce que tu sauras faire

- Mettre en place une sauvegarde automatique d'un service
- Restaurer une sauvegarde

## Ce que les labos ne font pas

Le conteneur du labo n'a ni Docker ni Kubernetes, et aucun accès à Internet. Les commandes `docker run`, `docker exec` et les fichiers CronJob ou Job sont donc **expliqués dans le cours mais pas exécutés** : tu pratiques ce qu'ils lancent (`pg_dump`, `backup.py`, `restore.py`, `aws`, `mc`) directement dans le terminal. Le MinIO du labo tourne dans le même conteneur que ton terminal : c'est une simulation de la copie hors site, avec des identifiants fictifs. Les scripts `backup.py` et `restore.py` du labo sont des versions simplifiées écrites pour la formation, dans l'esprit de ceux du dépôt `backups3`.

## Ressources utilisées par l'équipe

- [backups3](https://gitlab.example.org/equipe/dev/backups3) : image de sauvegarde vers S3
- [docker-postgres-backup](https://gitlab.example.org/equipe/dev/docker-postgres-backup) : sauvegarde PostgreSQL
- [mysql-backup-container](https://gitlab.example.org/equipe/dev/mysql-backup-container) : sauvegarde MySQL vers Swift
- [backup-files-swift](https://gitlab.example.org/equipe/dev/backup-files-swift) : sauvegarde de fichiers vers Swift
- [cluster-configuration](https://gitlab.example.org/equipe/dev/cluster-configuration) : déploiement de MinIO et des buckets

**Prérequis :** parcours *Docker avancé*. **Durée estimée :** environ 3 h 15 (labos et quiz compris).

Si ton portail ne propose pas encore les environnements réels, tu peux quand même suivre les leçons, et **valider tout le parcours avec l'examen** si tu maîtrises déjà le sujet.
