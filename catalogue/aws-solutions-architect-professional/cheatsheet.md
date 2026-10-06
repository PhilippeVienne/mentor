## Lire un scénario de niveau Professional

1. Souligne les **contraintes** : délai, budget, « sans changement de code », « le moins d'effort d'exploitation », RPO et RTO chiffrés, « pour tous les comptes, présents et futurs ».
2. Les quatre réponses fonctionnent presque toujours : élimine celles qui **ignorent une contrainte**.
3. Entre deux réponses valables, préfère le mécanisme **d'organisation** et le service **géré**.

## Organisation et gouvernance

| Besoin | Réponse |
| --- | --- |
| Isoler, répartir les coûts, déléguer | Un compte par application et par environnement, rangés en unités d'organisation |
| Interdire partout, quels que soient les droits locaux | SCP (plafonne ; n'accorde rien ; pas d'effet sur le compte de gestion) |
| Peu d'entretien | Liste d'interdictions, en gardant `FullAWSAccess` |
| Structure prête à l'emploi | AWS Control Tower (zone d'accueil, contrôles préventifs, détectifs, proactifs) |
| Gérer un service pour toute l'organisation hors du compte de gestion | Administration déléguée |
| Même configuration dans tous les comptes d'une unité | StackSets à autorisations gérées par le service |
| Partager sous-réseaux, Transit Gateway, règles DNS | AWS RAM |
| Journaux hors de portée | Trail d'organisation vers un compte d'archive, validation d'intégrité |

## Accès entre comptes

| Besoin | Réponse |
| --- | --- |
| Accès large ou interactif | Rôle inter-comptes |
| Flux précis entre services, ou copie d'un compte à l'autre | Politique de ressource |
| Tiers extérieur | Rôle + `sts:ExternalId` |
| Toute l'organisation, sans énumérer les comptes | Condition `aws:PrincipalOrgID` |
| Personnel | IAM Identity Center, relié à l'annuaire |
| Trouver ce qui est partagé à l'extérieur | IAM Access Analyzer |

## Réseau

| Besoin | Réponse |
| --- | --- |
| Beaucoup de VPC, segments isolés | Transit Gateway, plusieurs tables de routage |
| Exposer un service, plages qui se chevauchent | PrivateLink |
| Débit stable vers un site | Direct Connect (semaines ; non chiffré par défaut) |
| Vite, ou en secours | VPN site à site (deux tunnels ; BGP) |
| Résilience Direct Connect | Plusieurs emplacements ; l'agrégation de liens n'en est pas une |
| Plusieurs régions par une liaison | Direct Connect gateway |
| Noms résolus des deux côtés | Route 53 Resolver : points de terminaison entrant et sortant, règles de transfert |
| La configuration autorise-t-elle ce chemin ? | Reachability Analyzer |
| Qu'est-ce qui a circulé ? | Journaux de flux VPC |

## Déploiement

| Stratégie | Retour arrière | Coût |
| --- | --- | --- |
| Tout d'un coup | Redéployer | Nul |
| Progressif | Redéployer par lots | Nul |
| Immuable | Supprimer les nouvelles instances | Double capacité brève |
| Bleu-vert | Rebasculer, immédiat | Double environnement |
| Canari, linéaire | Rebasculer, peu d'utilisateurs touchés | Faible |

CloudFormation : ensemble de modifications (lire `Replacement`), politique de pile, protection contre la suppression, `DeletionPolicy` et `UpdateReplacePolicy`, détection de dérive.

## Continuité

| Besoin | Réponse |
| --- | --- |
| Écritures dans plusieurs régions, cohérence à terme | Tables globales DynamoDB (dernière écriture gagnante) |
| Relationnel multi-régions, RPO de quelques secondes | Aurora Global Database |
| Déchiffrer dans une autre région | Clés KMS multi-régions |
| Bascule sans dépendre du plan de contrôle | Contrôles de santé Route 53, Application Recovery Controller, Global Accelerator |
| Sauvegardes inaltérables | AWS Backup, copie vers un autre compte, Vault Lock en mode conformité |
| Serveurs à reprendre | AWS Elastic Disaster Recovery |
| Prouver que cela marche | Exercices, AWS Fault Injection Service |

Stabilité statique : pendant une panne, ne rien avoir à créer ni à modifier.

## Exploitation et amélioration

| Besoin | Réponse |
| --- | --- |
| Alarme utile | Indicateur de résultat pour l'utilisateur, données manquantes traitées |
| Trop d'alarmes | Alarme composite |
| Correction automatique | EventBridge vers Lambda ou Systems Manager Automation ; correction associée à une règle Config |
| Session sans port SSH | Session Manager |
| Correctifs d'un parc | Patch Manager |
| Trouver le goulot | Mesurer d'abord : la métrique saturée désigne le composant |
| Lectures répétées | Cache ; lectures lourdes : réplica ; pics d'écriture : file |
| Coût | Supprimer, dimensionner, **puis** s'engager |

## Migration et modernisation

| Besoin | Réponse |
| --- | --- |
| Dossier économique | Migration Evaluator |
| Inventaire et dépendances | Application Discovery Service (agents pour les connexions réseau) |
| Suivi | Migration Hub |
| Réhéberger des serveurs | AWS Application Migration Service |
| Base, même moteur | Outils natifs ou DMS |
| Base, autre moteur | SCT puis DMS |
| Interruption minimale | DMS, chargement complet puis réplication continue |
| Fichiers en ligne, incrémental | DataSync |
| Volume trop grand pour le débit | Plus de débit, ou transfert hors ligne |
| Sortir d'un monolithe | Motif de l'étrangleur |
| Étapes, décisions, compensations | Step Functions |
| Kubernetes existant | EKS ; sinon ECS, avec Fargate |
| Protocole JMS ou AMQP existant | Amazon MQ |

## Commandes vues dans les labos

```bash
aws organizations create-organizational-unit --parent-id PARENT --name NOM
aws organizations create-policy --name NOM --type SERVICE_CONTROL_POLICY --description D --content file://scp.json
aws organizations attach-policy --policy-id p-… --target-id ou-…
aws sts assume-role --role-arn ARN --role-session-name S --external-id ID
aws sqs set-queue-attributes --queue-url URL --attributes file://attributs.json
aws ec2 create-vpn-connection --type ipsec.1 --customer-gateway-id cgw-… --vpn-gateway-id vgw-…
aws cloudformation create-change-set --stack-name P --change-set-name C --template-body file://pile.yml --parameters …
aws lambda update-alias --function-name F --name prod --function-version N
aws dynamodb update-table --table-name T --replica-updates 'Create={RegionName=eu-west-1}'
aws backup create-backup-plan --backup-plan file://plan.json
aws events test-event-pattern --event-pattern file://motif.json --event file://evenement.json
aws cloudwatch put-metric-alarm --alarm-name A --namespace N --metric-name M … --alarm-actions ARN_SNS
aws s3 sync dossier s3://BUCKET/ --dryrun
aws stepfunctions start-execution --state-machine-arn ARN --name NOM --input '{…}'
```
