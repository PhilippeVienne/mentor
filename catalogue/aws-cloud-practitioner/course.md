---
title: "AWS : les bases du cloud (Cloud Practitioner)"
icon: "☁️"
summary: "Comprendre AWS et ses services essentiels en les manipulant sur un émulateur local, pour préparer la certification Cloud Practitioner (CLF-C02)."
requires: []
published: true
color: "#FF9900"
banner: images/banniere.svg
environment: environnement
---

**Amazon Web Services (AWS)** loue à la demande des serveurs, du stockage, des bases de données et des dizaines d'autres services. Ce cours t'en donne une vue d'ensemble solide : à quoi sert chaque grande famille de services, qui est responsable de quoi en matière de sécurité, et comment la facture se construit. Il suit le programme officiel de la certification **AWS Certified Cloud Practitioner (CLF-C02)**.

**Chaque leçon a un labo réel.** Tu reçois un environnement Linux jetable avec la vraie commande `aws` (l'AWS CLI officielle) et le serveur vérifie l'état de ce que tu as créé.

:::warning Un émulateur, pas le vrai AWS
Les labos ne parlent **pas** à Amazon. La commande `aws` est reliée à **MiniStack**, un logiciel libre qui imite les API d'AWS à l'intérieur de ton environnement. Conséquences :

- **rien n'est facturé** et aucun compte AWS n'est nécessaire ;
- tes ressources sont **fictives** : une « instance EC2 » du labo est une ligne dans la mémoire de l'émulateur, pas un serveur ; tout est effacé à l'arrêt de l'environnement ;
- l'émulateur est **plus permissif** que le vrai AWS sur certains contrôles, et il ne connaît ni la console web, ni la facturation, ni le support. Chaque leçon signale ce qui diffère.

Les **commandes**, les **noms de services** et les **documents** (politiques IAM, modèles CloudFormation) sont en revanche ceux du vrai AWS.
:::

## À qui s'adresse-t-il ?

À toute personne qui découvre AWS : développeur·se, administrateur·rice, chef·fe de projet, ou curieux·se. Il faut savoir taper une commande dans un terminal (cours *Linux et shell* conseillé si ce n'est pas le cas). Aucune connaissance du cloud n'est requise : chaque terme est défini quand il apparaît.

## La certification visée

Ce qui suit vient du guide d'examen officiel et de la page de la certification, **consultés le 6 octobre 2026**. AWS met ces informations à jour : vérifie-les avant de t'inscrire.

| | AWS Certified Cloud Practitioner (CLF-C02) |
| --- | --- |
| Niveau | Fondamental (*Foundational*) |
| Format | 65 questions : 50 notées et 15 non notées, que rien ne distingue |
| Types de questions | Choix unique (1 bonne réponse sur 4) ou réponses multiples (2 ou plus parmi 5 ou plus) |
| Durée | 90 minutes |
| Note | De 100 à 1 000 ; il faut **700** pour réussir. Pas de pénalité pour une mauvaise réponse |
| Prix affiché | 100 USD |
| Passage | Centre Pearson VUE ou examen surveillé en ligne ; proposé en français |
| Validité | 3 ans |

Le guide d'examen répartit les questions notées en quatre domaines. Le cours les couvre tous :

| Domaine officiel | Poids | Leçons |
| --- | :---: | --- |
| 1. *Cloud Concepts* (concepts du cloud) | 24 % | 1, 2, 11 |
| 2. *Security and Compliance* (sécurité et conformité) | 30 % | 3, 4, 5 |
| 3. *Cloud Technology and Services* (technologies et services) | 34 % | 1, 2, 6, 7, 8, 9, 10 |
| 4. *Billing, Pricing, and Support* (facturation, tarifs et support) | 12 % | 12 |

:::info Ce que ce cours ne promet pas
Il te **prépare** à l'examen ; il ne garantit pas de le réussir. L'examen porte sur le vrai AWS : complète le cours par la lecture du [guide d'examen officiel](https://docs.aws.amazon.com/aws-certification/latest/cloud-practitioner-02/cloud-practitioner-02.html), de la documentation des services cités et, si tu le peux, par un peu de pratique sur un vrai compte. L'examen de validation du portail est écrit pour ce cours : ce ne sont pas des questions de l'examen officiel, et il ne contient que des questions à choix unique.
:::

## Plan

| # | Leçon | Durée | Pratique |
| --- | --- | --- | --- |
| 1 | Le cloud AWS et ton labo | 25 min | Premier bucket, CLI et SDK |
| 2 | Régions, zones de disponibilité et points de présence | 20 min | Une ressource, deux régions |
| 3 | Le modèle de responsabilité partagée | 25 min | Protéger un bucket : ta part du contrat |
| 4 | IAM : utilisateurs, groupes et politiques | 30 min | Un compte limité, réellement refusé |
| 5 | Sécurité, journaux et conformité | 25 min | Secret, clé de chiffrement et journal d'audit |
| 6 | Calculer : EC2, conteneurs et Lambda | 30 min | Une instance et une fonction Lambda |
| 7 | Stocker : S3, EBS, EFS | 30 min | Classes de stockage, versions et cycle de vie |
| 8 | Bases de données | 25 min | Une table DynamoDB |
| 9 | Réseau : VPC, Route 53 et connectivité | 30 min | Un VPC avec un sous-réseau public |
| 10 | Intégration, analyse, IA et autres services | 25 min | Une notification diffusée dans deux files |
| 11 | Bien architecturer et migrer | 30 min | Une infrastructure décrite en code |
| 12 | Coûts, facturation et support | 30 min | Étiquettes de coûts et budget |

**Prérequis :** aucun cours. **Durée estimée :** environ 5 h 30, puis l'examen de validation (30 minutes).
