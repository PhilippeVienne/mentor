## Lire un scénario

1. Repère la **contrainte dominante** : le moins cher, le moins d'exploitation, le plus disponible, le plus rapide à mettre en place.
2. Élimine les réponses qui fonctionnent mais ignorent cette contrainte.
3. À égalité, préfère le service **géré** et la solution la plus simple.

## Accès et identités

| Besoin | Réponse |
| --- | --- |
| Accès depuis un autre compte | Rôle + politique de confiance + `sts:AssumeRole` côté appelant |
| Tiers extérieur | Rôle avec condition `sts:ExternalId` |
| Code qui tourne sur AWS | Rôle (profil d'instance, rôle d'exécution) : jamais de clé stockée |
| Personnel, plusieurs comptes | IAM Identity Center |
| Utilisateur·rice·s d'une application | Amazon Cognito |
| Plafond pour tout un compte | SCP (n'accorde rien ; ne touche pas le compte de gestion) |
| Plafond pour une identité | Limite d'autorisations |

Ordre de décision : `Deny` explicite, puis plafonds (SCP, limite), puis `Allow` (identité ou ressource), sinon refus par défaut.

## Réseau

| Besoin | Réponse |
| --- | --- |
| Joignable depuis Internet | Sous-réseau public (route vers la passerelle Internet) |
| Sortir sans être joignable | Passerelle NAT (une par zone en production) |
| S3 ou DynamoDB sans NAT | Point de terminaison de passerelle (gratuit) |
| Autre service AWS en privé | Point de terminaison d'interface (PrivateLink) |
| Autoriser « les serveurs web » | Groupe de sécurité source, pas une plage d'adresses |
| Interdire une adresse | Liste de contrôle d'accès réseau |
| Deux VPC | Appairage (non transitif) |
| Beaucoup de VPC et des sites | Transit Gateway |
| Site vers AWS, vite | VPN site à site |
| Site vers AWS, débit stable | Direct Connect (non chiffré par défaut) |

## Données

| Besoin | Réponse |
| --- | --- |
| Chiffrer sans effort | SSE-S3 |
| Contrôler et auditer la clé | SSE-KMS, clé gérée par le client |
| AWS ne doit rien voir en clair | Chiffrement côté client |
| Inaltérable pendant N années | Verrouillage d'objets, mode conformité |
| Erreur de suppression | Versionnage (+ suppression MFA) |
| Certificats TLS publics | ACM (renouvellement automatique) |
| Secret avec rotation | Secrets Manager |

## Résilience

| Besoin | Réponse |
| --- | --- |
| Absorber un pic, ne rien perdre | File SQS |
| Plusieurs destinataires | SNS vers plusieurs files SQS |
| Ordre strict, pas de doublon | File FIFO |
| Messages en échec répété | File de rebut |
| Étapes avec reprises | Step Functions |
| Panne d'une zone | Multi-AZ, répartiteur de charge, Auto Scaling |
| Base disponible | RDS Multi-AZ (synchrone, bascule automatique) |
| Base plus rapide en lecture | Réplicas en lecture (asynchrones), cache |

| Stratégie de reprise | Dans la région de secours | RPO / RTO |
| --- | --- | --- |
| Sauvegarde et restauration | Des sauvegardes | Heures |
| Veilleuse | Données répliquées, serveurs éteints | Dizaines de minutes |
| Secours tiède | Copie complète à capacité réduite | Minutes |
| Multi-site actif | Tout, en service | Proche de zéro |

## Performance

| Besoin | Réponse |
| --- | --- |
| Fortes IOPS, latence constante | EBS io2 Block Express |
| Usage général | EBS gp3 (3 000 IOPS et 125 Mio/s de base) |
| Débit séquentiel économique | EBS st1 ; données froides : sc1 |
| Fichiers partagés Linux, plusieurs zones | EFS |
| Fichiers Windows | FSx for Windows File Server |
| Calcul intensif | FSx for Lustre, groupe de placement cluster |
| Accès par clé à grande échelle | DynamoDB (requête, pas parcours ; index global) |
| Microsecondes sur DynamoDB | DAX |
| Contenu près des internautes | CloudFront |
| Adresse IP fixe, UDP, bascule rapide | Global Accelerator |
| Région la plus rapide | Route 53, routage par latence |
| Flux relu par plusieurs applications | Kinesis Data Streams |
| Flux déposé dans S3 sans code | Amazon Data Firehose |
| SQL sur S3 sans serveur | Athena (Parquet + partitions) |

## Coûts

| Profil | Option |
| --- | --- |
| Régulier, 1 à 3 ans | Savings Plans, instances réservées |
| Interruptible | Spot |
| Imprévisible | À la demande, Lambda, modes « à la demande » et Serverless |
| Données qui vieillissent | Cycle de vie S3 (+ anciennes versions, envois incomplets) |
| Accès imprévisible | S3 Intelligent-Tiering |
| Suivre un projet | Étiquettes d'allocation des coûts + budget filtré |

## Commandes vues dans les labos

```bash
aws iam create-role --role-name R --assume-role-policy-document file://confiance.json
aws configure set role_arn arn:aws:iam::COMPTE:role/R --profile P   # avec source_profile
aws ec2 create-nat-gateway --subnet-id SUBNET --allocation-id EIP
aws ec2 authorize-security-group-ingress --group-id SG --protocol tcp --port 5432 --source-group SG_SOURCE
aws ec2 create-vpc-endpoint --vpc-id VPC --service-name com.amazonaws.eu-west-3.s3 --route-table-ids RTB
aws kms encrypt --key-id alias/CLE --plaintext fileb://f --query CiphertextBlob --output text | base64 -d > f.chiffre
aws sqs create-queue --queue-name Q --attributes file://attributs.json          # RedrivePolicy
aws lambda create-event-source-mapping --function-name F --event-source-arn ARN_FILE
aws autoscaling create-auto-scaling-group --auto-scaling-group-name G --launch-template LaunchTemplateName=M --min-size 2 --max-size 6 --vpc-zone-identifier "S1,S2"
aws s3api put-bucket-replication --bucket B --replication-configuration file://replication.json
aws dynamodb query --table-name T --key-condition-expression 'pk = :v' --expression-attribute-values file://valeurs.json
aws kinesis put-record --stream-name S --partition-key K --cli-binary-format raw-in-base64-out --data '…'
```
