## Le labo

| Commande | Rôle |
| --- | --- |
| `demarrer-aws` | Relance l'émulateur s'il ne répond plus (`demarrer-aws --etat` pour le vérifier) |
| `aws sts get-caller-identity` | Qui suis-je ? (compte fictif `000000000000`) |
| `aws <service> <opération> help` | Aide d'une commande |
| `--region eu-west-1` | Agir dans une autre région que `eu-west-3` |
| `--profile lea` | Agir avec les identifiants d'un autre profil |
| `--query '…' --output text` | Extraire une valeur de la réponse |

Le labo est un **émulateur local** (MiniStack) : rien n'est facturé, rien n'est réel, tout disparaît à l'arrêt.

## Qui fait quoi : responsabilité partagée

| AWS : sécurité **du** cloud | Toi : sécurité **dans** le cloud |
| --- | --- |
| Bâtiments, matériel, réseau mondial | Données, chiffrement, sauvegardes |
| Logiciel des services gérés | Comptes, droits IAM, MFA |
| Système et moteur de RDS, Lambda… | Système de tes instances EC2, règles réseau |

## IAM

| Brique | En une phrase |
| --- | --- |
| Utilisateur racine | Tous les droits : MFA, pas de clé d'accès, usage exceptionnel |
| Utilisateur | Une identité durable, pour une personne ou une application |
| Groupe | Des utilisateurs qui partagent les mêmes politiques |
| Rôle | Une identité à endosser, avec des identifiants temporaires |
| Politique | Un document JSON : `Effect`, `Action`, `Resource` |

Refus par défaut ; un `Allow` autorise ; un `Deny` explicite l'emporte toujours.

## Choisir un service

| Besoin | Service |
| --- | --- |
| Serveur virtuel | Amazon EC2 |
| Conteneurs sans serveur à gérer | Amazon ECS ou EKS avec AWS Fargate |
| Code déclenché par un événement (15 minutes au plus) | AWS Lambda |
| Fichiers, sauvegardes, site statique | Amazon S3 |
| Disque d'une instance | Amazon EBS |
| Dossier partagé entre serveurs Linux | Amazon EFS |
| Base relationnelle gérée | Amazon RDS, Amazon Aurora |
| Base clé-valeur à grande échelle | Amazon DynamoDB |
| Cache en mémoire | Amazon ElastiCache |
| Réseau privé | Amazon VPC |
| DNS | Amazon Route 53 |
| Diffusion de contenu près des internautes | Amazon CloudFront |
| File d'attente | Amazon SQS |
| Notification à plusieurs abonnés | Amazon SNS |
| Règles sur des événements, tâches planifiées | Amazon EventBridge |
| Infrastructure en code | AWS CloudFormation |

## Sécurité et gouvernance

| Question | Service |
| --- | --- |
| Qui a fait quoi ? | AWS CloudTrail |
| Mes ressources vont-elles bien ? | Amazon CloudWatch |
| Comment était configurée cette ressource ? | AWS Config |
| Y a-t-il un comportement suspect ? | Amazon GuardDuty |
| Y a-t-il des vulnérabilités logicielles ? | Amazon Inspector |
| Y a-t-il des données sensibles dans S3 ? | Amazon Macie |
| Où sont les rapports de conformité d'AWS ? | AWS Artifact |
| Où ranger un mot de passe ? | AWS Secrets Manager |
| Qui gère mes clés de chiffrement ? | AWS KMS |
| Attaque DDoS / requêtes web malveillantes | AWS Shield / AWS WAF |

## Classes de stockage S3

| Classe | Accès | Durée minimale |
| --- | --- | --- |
| Standard | Fréquent, immédiat | Aucune |
| Intelligent-Tiering | Inconnu ou changeant | Aucune |
| Standard-IA / One Zone-IA | Rare, immédiat | 30 jours |
| Glacier Instant Retrieval | Quelques fois par an, immédiat | 90 jours |
| Glacier Flexible Retrieval | Archives, minutes ou heures | 90 jours |
| Glacier Deep Archive | Archives, heures | 180 jours |

## Payer moins

| Profil d'usage | Option |
| --- | --- |
| Imprévisible, court | À la demande |
| Régulier, 1 ou 3 ans | Savings Plans, instances réservées |
| Interruptible | Instances Spot |
| Licences liées au matériel | Hôtes dédiés |

Estimer : Pricing Calculator. Alerter : Budgets. Analyser : Cost Explorer. Détail : Cost and Usage Report. Répartir : étiquettes d'allocation des coûts.

## Les six piliers du Well-Architected Framework

Excellence opérationnelle · Sécurité · Fiabilité · Efficacité des performances · Optimisation des coûts · Développement durable.

## Commandes vues dans les labos

```bash
aws s3 mb s3://BUCKET                       # créer un bucket
aws s3 cp FICHIER s3://BUCKET/CLE           # envoyer un fichier
aws s3api put-bucket-versioning --bucket BUCKET --versioning-configuration Status=Enabled
aws iam create-policy --policy-name NOM --policy-document file://politique.json
aws lambda invoke --function-name NOM --cli-binary-format raw-in-base64-out --payload '{}' reponse.json
aws dynamodb put-item --table-name TABLE --item file://element.json
aws cloudformation deploy --stack-name PILE --template-file modele.yml
aws cloudtrail lookup-events --lookup-attributes AttributeKey=EventName,AttributeValue=OPERATION
```
