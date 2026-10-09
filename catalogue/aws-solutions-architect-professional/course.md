---
title: "AWS : architectures d'entreprise (Solutions Architect Professional)"
icon: "🧭"
summary: "Concevoir pour une organisation entière : comptes multiples, réseau hybride, déploiements sûrs, continuité, amélioration continue et migrations, vers la certification SAP-C02."
requires: [aws-solutions-architect-associate]
published: true
color: "#FF9900"
banner: images/banniere.svg
environment: environnement
---

Au niveau Associate, on conçoit **une** application. Au niveau Professional, on conçoit pour **une organisation** : des dizaines de comptes, des réseaux à relier, des équipes qui déploient en parallèle, un existant à migrer, et des compromis à arbitrer entre sécurité, coût, délai et effort d'exploitation. Ce cours suit le programme officiel de la certification **AWS Certified Solutions Architect – Professional (SAP-C02)**.

Neuf leçons sur dix ont un **labo réel**, dans le même environnement que les deux cours précédents (la commande `aws` reliée à l'émulateur local **MiniStack**). La dixième est une étude d'architecture, sans labo : son sujet ne s'émule pas.

:::warning Ce que l'émulateur permet, et ce qu'il ne permet pas
À ce niveau, une grande part du programme porte sur des services **impossibles à émuler** honnêtement : AWS Direct Connect, AWS Transit Gateway, AWS Control Tower, les services de migration, la facturation à l'échelle d'une organisation. Le cours les enseigne par la lecture, des schémas, des études de cas et des questions ; il ne fait pas semblant de les exécuter.

Les labos portent sur ce que l'émulateur exécute **réellement** : l'évaluation d'IAM entre plusieurs comptes fictifs, les politiques de ressource entre comptes (S3, SQS), les ensembles de modifications CloudFormation, les alias Lambda, les tables DynamoDB répliquées entre régions, les alarmes CloudWatch qui déclenchent une notification, les règles EventBridge qui lancent une correction automatique, les flux Step Functions. D'autres étapes ne font qu'**enregistrer une configuration** (unités d'organisation, politiques de contrôle des services, VPN, plans de sauvegarde) : le labo vérifie alors ta configuration, pas son effet. Chaque leçon dit dans quel cas on se trouve.
:::

## À qui s'adresse-t-il ?

Aux personnes qui ont terminé le cours *Solutions Architect Associate* ou en maîtrisent le contenu, **et** qui pratiquent AWS. AWS décrit le public de cet examen comme ayant **au moins deux ans d'expérience** de la conception et de la mise en œuvre de solutions sur AWS : ce cours structure et complète une telle expérience, il ne la remplace pas.

## La certification visée

Informations tirées du guide d'examen officiel et de la page de la certification, **consultés le 6 octobre 2026**. Vérifie-les avant de t'inscrire.

| | AWS Certified Solutions Architect – Professional (SAP-C02) |
| --- | --- |
| Niveau | Professional |
| Public décrit par AWS | Deux ans d'expérience ou plus dans la conception et la mise en œuvre de solutions sur AWS |
| Format | 75 questions : 65 notées et 10 non notées, que rien ne distingue |
| Types de questions | Choix unique (1 bonne réponse sur 4) ou réponses multiples (2 ou plus parmi 5 ou plus ; il faut toutes les trouver) |
| Durée | 180 minutes |
| Note | De 100 à 1 000 ; il faut **750** pour réussir. Pas de pénalité pour une mauvaise réponse |
| Prix affiché | 300 USD |
| Passage | Centre Pearson VUE ou examen surveillé en ligne. À la date de consultation, le français ne figure **pas** parmi les langues proposées (anglais, japonais, coréen, portugais du Brésil, chinois simplifié, espagnol d'Amérique latine) |
| Validité | 3 ans |

Les quatre domaines du guide d'examen, et les leçons qui les couvrent :

| Domaine officiel | Poids | Leçons |
| --- | :---: | --- |
| 1. *Design Solutions for Organizational Complexity* (complexité organisationnelle) | 26 % | 1, 2, 3 |
| 2. *Design for New Solutions* (nouvelles solutions) | 29 % | 4, 5, 6 |
| 3. *Continuous Improvement for Existing Solutions* (amélioration continue) | 25 % | 7, 8 |
| 4. *Accelerate Workload Migration and Modernization* (migration et modernisation) | 20 % | 9, 10 |

Le guide signale aussi des **sujets émergents** (contrôles de sécurité pour l'IA générative, par exemple), qui peuvent apparaître dans des questions non notées : ce cours ne les traite pas.

:::info Ce que ce cours ne promet pas
L'examen réel est long et exigeant : des scénarios d'une demi-page, quatre réponses toutes plausibles, à lire en anglais ou dans l'une des langues proposées. Ce cours t'entraîne au raisonnement et te fait manipuler ce qui peut l'être ; il **ne garantit pas** la réussite et ne remplace ni la pratique sur de vrais comptes, ni la lecture de la documentation et des livres blancs d'AWS. L'examen de validation du portail est écrit pour ce cours, ne reprend aucune question officielle et ne contient que des questions à choix unique. Référence : le [guide d'examen officiel](https://docs.aws.amazon.com/aws-certification/latest/solutions-architect-professional-02/solutions-architect-professional-02.html).
:::

## Plan

| # | Leçon | Durée | Pratique |
| --- | --- | --- | --- |
| 1 | Structurer une organisation multi-comptes | 40 min | Unités d'organisation, comptes et politique de contrôle des services |
| 2 | Accès et partage entre comptes | 40 min | Rôle pour un tiers avec identifiant externe, file et bucket partagés |
| 3 | Réseau hybride et multi-VPC | 40 min | VPN site à site, propagation de routes, journaux de flux |
| 4 | Déployer sans interrompre | 40 min | Ensemble de modifications, protection de pile, bascule d'alias |
| 5 | Continuité d'activité sur plusieurs régions | 40 min | Table répliquée en écriture dans deux régions, plan de sauvegarde |
| 6 | Sécurité à grande échelle | 40 min | Journalisation centralisée et règle de conformité |
| 7 | Excellence opérationnelle : observer et corriger | 40 min | Alarme, notification et correction automatique |
| 8 | Revoir une architecture existante | 35 min | Étude de cas (sans labo) |
| 9 | Migrer : évaluer, planifier, transférer | 40 min | Synchronisation de données et bascule progressive |
| 10 | Moderniser : découpler et orchestrer | 40 min | Un flux Step Functions avec décision et file de revue |

**Prérequis :** cours *AWS : concevoir des architectures (Solutions Architect Associate)*. **Durée estimée :** environ 6 h 30, puis l'examen de validation (45 minutes).
