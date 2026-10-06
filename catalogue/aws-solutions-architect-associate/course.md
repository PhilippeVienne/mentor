---
title: "AWS : concevoir des architectures (Solutions Architect Associate)"
icon: "🏛️"
summary: "Concevoir des architectures AWS sûres, résilientes, performantes et économes, en les construisant sur un émulateur local, pour préparer la certification SAA-C03."
requires: [aws-cloud-practitioner]
published: true
color: "#FF9900"
banner: images/banniere.svg
environment: environnement
---

Connaître les services ne suffit pas pour concevoir une architecture : il faut savoir **lequel choisir** face à une contrainte, et pourquoi les autres conviennent moins. Ce parcours t'entraîne à ce raisonnement, domaine par domaine, en suivant le programme officiel de la certification **AWS Certified Solutions Architect – Associate (SAA-C03)**.

**Chaque leçon a un labo réel**, dans le même environnement que le parcours *Cloud Practitioner* : la vraie commande `aws`, reliée à l'émulateur local **MiniStack**. Tu y construis des morceaux d'architecture (rôles inter-comptes, réseau à deux niveaux, files avec rebut, fonctions sans serveur, réplication entre régions…) et le serveur vérifie leur état.

:::warning Un émulateur, pas le vrai AWS
Rien n'est facturé et rien n'est réel : tout vit dans la mémoire de l'émulateur. Ce qui fonctionne **réellement** dans le labo : l'évaluation des politiques IAM (y compris entre deux comptes fictifs et pour les rôles des fonctions Lambda), S3, DynamoDB, SQS, SNS, EventBridge, Kinesis, Lambda en Python, CloudFormation. Ce qui n'est qu'une **fiche descriptive** : les instances EC2, les répartiteurs de charge, les bases RDS, le réseau (aucun paquet ne circule). Ce qui n'existe pas : la console, la facturation, CloudFront en conditions réelles, Direct Connect, Transit Gateway. Chaque leçon précise ces limites.
:::

## À qui s'adresse-t-il ?

Aux personnes qui ont terminé le parcours *AWS : les bases du cloud* ou qui en maîtrisent le contenu, et qui veulent passer de « je connais les services » à « je sais concevoir une solution ». Tu dois être à l'aise avec le terminal, le format JSON et les notions de réseau de base (adresse IP, port, DNS).

## La certification visée

Informations tirées du guide d'examen officiel et de la page de la certification, **consultés le 6 octobre 2026**. Vérifie-les avant de t'inscrire.

| | AWS Certified Solutions Architect – Associate (SAA-C03) |
| --- | --- |
| Niveau | Associate |
| Public décrit par AWS | Au moins un an d'expérience pratique de la conception de solutions sur AWS |
| Format | 65 questions : 50 notées et 15 non notées, que rien ne distingue |
| Types de questions | Choix unique (1 bonne réponse sur 4) ou réponses multiples (2 ou plus parmi 5 ou plus) |
| Durée | 130 minutes |
| Note | De 100 à 1 000 ; il faut **720** pour réussir. Pas de pénalité pour une mauvaise réponse |
| Prix affiché | 150 USD |
| Passage | Centre Pearson VUE ou examen surveillé en ligne ; proposé en français |
| Validité | 3 ans |

Les quatre domaines du guide d'examen, et les leçons qui les couvrent :

| Domaine officiel | Poids | Leçons |
| --- | :---: | --- |
| 1. *Design Secure Architectures* (architectures sûres) | 30 % | 1, 2, 3 |
| 2. *Design Resilient Architectures* (architectures résilientes) | 26 % | 4, 5, 6, 7 |
| 3. *Design High-Performing Architectures* (architectures performantes) | 24 % | 8, 9, 10, 11 |
| 4. *Design Cost-Optimized Architectures* (architectures économes) | 20 % | 12, et un paragraphe « côté coûts » dans les leçons 8 à 11 |

:::info Ce que ce parcours ne promet pas
AWS décrit la personne visée par cet examen comme ayant **au moins un an de pratique**. Ce parcours t'apporte la méthode et les connaissances, sur un émulateur ; il ne remplace pas cette pratique et ne garantit pas la réussite. Les questions de l'examen réel sont de longs scénarios où plusieurs réponses fonctionnent et où une seule répond **le mieux** à la contrainte (coût, délai, effort d'exploitation) : entraîne-toi à repérer cette contrainte. L'examen de validation du portail est écrit pour ce parcours, ne reprend aucune question officielle et ne contient que des questions à choix unique. Complète-le par le [guide d'examen](https://docs.aws.amazon.com/aws-certification/latest/solutions-architect-associate-03/solutions-architect-associate-03.html), la documentation et, si possible, de la pratique sur un vrai compte.
:::

## Plan

| # | Leçon | Durée | Pratique |
| --- | --- | --- | --- |
| 1 | Accès sûrs : rôles, comptes multiples et politiques | 35 min | Un rôle d'audit endossé depuis un autre compte |
| 2 | Un réseau à plusieurs niveaux | 35 min | Sous-réseau privé, NAT, groupes de sécurité chaînés, point de terminaison |
| 3 | Protéger les données | 35 min | Clé KMS, chiffrement par défaut, chiffrement d'enveloppe |
| 4 | Découpler avec des files et des événements | 35 min | File de rebut, file FIFO |
| 5 | Sans serveur : Lambda et ses déclencheurs | 35 min | Une fonction qui consomme une file avec le moindre privilège |
| 6 | Haute disponibilité : zones, répartition, mise à l'échelle | 35 min | Groupe Auto Scaling sur deux zones derrière un répartiteur |
| 7 | Reprise après sinistre | 35 min | Réplication entre régions et bascule DNS |
| 8 | Stockage performant | 30 min | Volumes EBS, instantané, envoi en plusieurs parties |
| 9 | Bases de données performantes | 35 min | Clé composée, index secondaire, requête ou parcours |
| 10 | Réseau performant et diffusion mondiale | 30 min | Appairage de VPC et routage DNS par latence |
| 11 | Ingestion et analyse de données | 30 min | Flux Kinesis et livraison Firehose vers S3 |
| 12 | Optimiser les coûts | 35 min | Cycle de vie complet, mode de capacité, budget par projet |

**Prérequis :** parcours *AWS : les bases du cloud (Cloud Practitioner)*. **Durée estimée :** environ 6 h 45, puis l'examen de validation (40 minutes).
