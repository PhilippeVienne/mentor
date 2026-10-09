---
title: "Examen de validation — AWS : concevoir des architectures (Solutions Architect Associate)"
draw: 20
pass_mark: 80
minutes: 40
shuffle: true
---

Cet examen valide le cours : architectures sûres, résilientes, performantes et économes. Vingt questions sont tirées au sort dans une réserve de soixante, réparties comme les quatre domaines du guide de l'examen SAA-C03. Ce sont des scénarios : plusieurs réponses peuvent fonctionner, une seule répond le mieux à la contrainte de l'énoncé.

Ce n'est **pas** l'examen d'AWS, et ces questions n'en proviennent pas : elles ont été écrites pour ce cours, et n'ont toutes qu'une bonne réponse alors que l'examen réel comporte aussi des questions à réponses multiples. Réussir ici montre que tu maîtrises le contenu du cours ; AWS décrit le public de la certification comme ayant au moins un an de pratique, que cet examen ne mesure pas.

:::quiz
Une entreprise possède un compte de développement et un compte de production. Les développeur·se·s doivent pouvoir consulter, sans les modifier, les journaux du compte de production, avec leurs identités du compte de développement. Quelle conception retenir ?

- [ ] Créer dans le compte de production un utilisateur IAM partagé par l'équipe
- [x] Créer dans le compte de production un rôle en lecture seule, dont la politique de confiance désigne le compte de développement, et autoriser les développeur·se·s à l'endosser
- [ ] Copier chaque nuit les journaux dans un bucket public
- [ ] Donner aux développeur·se·s la clé d'accès d'un administrateur de la production

> Un rôle inter-comptes donne des identifiants temporaires, limités à la lecture, sans identité durable à gérer dans la production.
:::

:::quiz
Une application qui tourne sur des instances EC2 doit lire des objets dans un bucket S3. Comment lui fournir des identifiants ?

- [x] Associer aux instances un profil d'instance portant un rôle IAM limité à ce bucket
- [ ] Passer la clé d'accès dans les données de démarrage de l'instance
- [ ] Écrire une clé d'accès dans un fichier de configuration de l'application
- [ ] Rendre le bucket public en lecture

> Avec un rôle, l'instance reçoit des identifiants temporaires renouvelés automatiquement : aucune clé à stocker ni à faire tourner.
:::

:::quiz
Le service de sécurité veut garantir qu'aucun compte de l'unité d'organisation « Projets » ne puisse désactiver AWS CloudTrail, quels que soient les droits accordés localement. Que mettre en place ?

- [ ] Une politique IAM `Deny` rattachée à chaque utilisateur de chaque compte
- [ ] Une alarme CloudWatch sur l'arrêt de CloudTrail
- [ ] Une limite d'autorisations sur l'utilisateur racine de chaque compte
- [x] Une politique de contrôle des services (SCP) rattachée à l'unité d'organisation, qui interdit ces actions

> Une SCP s'impose à toutes les identités des comptes membres concernés, administrateurs compris, et ne dépend pas de leur bonne volonté.
:::

:::quiz
Une équipe délègue à des chef·fe·s de projet la création de rôles IAM pour leurs applications, mais veut être certaine que ces rôles ne dépasseront jamais un ensemble de droits défini. Quel mécanisme répond à ce besoin ?

- [ ] Une politique de session
- [x] Une limite d'autorisations imposée aux rôles créés
- [ ] Une politique de bucket
- [ ] Un groupe IAM

> La limite d'autorisations fixe le maximum qu'une identité peut obtenir, quelles que soient les politiques qu'on lui rattache ensuite.
:::

:::quiz
Les salarié·e·s d'une entreprise s'authentifient déjà auprès de l'annuaire interne. L'entreprise veut qu'ils accèdent à ses douze comptes AWS sans mot de passe supplémentaire, avec des droits par métier. Quelle solution demande le moins de gestion ?

- [x] AWS IAM Identity Center, relié à l'annuaire, avec des jeux d'autorisations par compte
- [ ] Un utilisateur IAM unique par métier, partagé
- [ ] Un utilisateur IAM par personne dans chacun des douze comptes
- [ ] Amazon Cognito avec un groupe d'utilisateurs par compte

> Identity Center fournit une connexion unique à plusieurs comptes à partir d'une source d'identités existante. Cognito s'adresse aux utilisateur·rice·s d'applications.
:::

:::quiz
Un bucket contient des documents que seul un service d'un autre compte AWS doit pouvoir lire, sans rôle à endosser. Que configurer ?

- [x] Une politique de bucket qui autorise l'identité de l'autre compte, et une politique d'identité correspondante dans cet autre compte
- [ ] Une liste de contrôle d'accès réseau
- [ ] Le blocage de l'accès public désactivé
- [ ] Un point de terminaison de passerelle

> Entre deux comptes, la politique de ressource du bucket et la politique d'identité de l'appelant doivent toutes deux autoriser l'accès.
:::

:::quiz
Une architecture à trois niveaux place les serveurs applicatifs dans des sous-réseaux privés. Ils doivent télécharger des correctifs sur Internet, sans jamais accepter de connexion entrante. Quelle conception convient ?

- [ ] Leur attribuer des adresses IP publiques et fermer le groupe de sécurité
- [ ] Les déplacer dans les sous-réseaux publics
- [ ] Leur donner un point de terminaison de passerelle pour Internet
- [x] Une passerelle NAT dans un sous-réseau public, et une route par défaut des sous-réseaux privés vers elle

> La passerelle NAT permet les sorties et bloque toute connexion initiée depuis l'extérieur.
:::

:::quiz
Une adresse IP précise bombarde un site de requêtes. L'équipe veut bloquer cette adresse pour tout le sous-réseau, immédiatement. Quel outil du VPC le permet ?

- [ ] Une règle d'entrée du groupe de sécurité
- [x] Une règle d'interdiction dans la liste de contrôle d'accès réseau du sous-réseau
- [ ] Une route « trou noir » vers la passerelle NAT
- [ ] Un point de terminaison d'interface

> Les groupes de sécurité ne contiennent que des autorisations. Seule la liste de contrôle d'accès réseau accepte une règle d'interdiction.
:::

:::quiz
Un site public, servi par un Application Load Balancer, subit des tentatives d'injection SQL et de scripts intersites. Quel service ajouter devant l'application ?

- [x] AWS WAF, associé au répartiteur de charge
- [ ] Amazon GuardDuty
- [ ] AWS Shield Standard
- [ ] Une passerelle NAT

> AWS WAF filtre les requêtes HTTP selon des règles (injections, scripts, limitation de débit). Shield traite les attaques par déni de service.
:::

:::quiz
Une fonction Lambda placée dans un sous-réseau privé, sans passerelle NAT, doit lire un secret dans AWS Secrets Manager. Que faut-il ajouter au VPC ?

- [ ] Un point de terminaison de passerelle pour Secrets Manager
- [ ] Une passerelle Internet
- [ ] Un appairage de VPC
- [x] Un point de terminaison d'interface pour Secrets Manager

> Les points de terminaison de passerelle n'existent que pour S3 et DynamoDB ; les autres services s'atteignent en privé par un point de terminaison d'interface.
:::

:::quiz
Une entreprise relie son centre de données à AWS par Direct Connect et doit, pour des raisons réglementaires, chiffrer tout le trafic sur cette liaison. Que faire ?

- [ ] Rien : Direct Connect chiffre le trafic par défaut
- [x] Établir un VPN site à site par-dessus la liaison Direct Connect
- [ ] Remplacer Direct Connect par une passerelle NAT
- [ ] Activer le versionnage sur les buckets

> Direct Connect ne chiffre pas le trafic par défaut. Un VPN au-dessus de la liaison apporte le chiffrement en gardant la stabilité de la liaison dédiée.
:::

:::quiz
Une entreprise doit prouver à ses auditeurs qui a utilisé la clé de chiffrement de ses données S3, et pouvoir en retirer l'usage à une équipe à tout moment. Quel chiffrement choisir ?

- [x] SSE-KMS avec une clé gérée par le client
- [ ] SSE-KMS avec la clé gérée par AWS
- [ ] SSE-S3
- [ ] Aucun : le blocage de l'accès public suffit

> Une clé gérée par le client a une politique que l'on modifie, et ses usages sont tracés dans CloudTrail. La politique d'une clé gérée par AWS ne se modifie pas.
:::

:::quiz
Un instantané EBS chiffré avec une clé KMS gérée par le client doit être partagé avec un autre compte AWS, qui en fera un volume. Que faut-il, outre le partage de l'instantané ?

- [ ] Rendre l'instantané public
- [ ] Le rechiffrer avec la clé gérée par AWS `aws/ebs`
- [ ] Désactiver la rotation de la clé
- [x] Autoriser l'autre compte à utiliser la clé, dans la politique de clé

> Sans droit sur la clé, l'autre compte ne peut pas déchiffrer l'instantané. Une clé gérée par AWS ne peut pas être partagée avec un autre compte.
:::

:::quiz
Des enregistrements financiers doivent être conservés sept ans dans S3, sans qu'aucune personne, administrateur compris, puisse les supprimer ou les modifier avant l'échéance. Que configurer ?

- [ ] Le versionnage et une politique de bucket qui refuse la suppression
- [x] Le verrouillage d'objets en mode conformité, avec une durée de conservation de sept ans
- [ ] Le verrouillage d'objets en mode gouvernance
- [ ] Une règle de cycle de vie vers S3 Glacier Deep Archive

> En mode conformité, le verrou ne peut être levé par personne avant la date. Une politique de bucket reste modifiable ; le mode gouvernance se contourne avec un droit spécial.
:::

:::quiz
Un site web a besoin d'un certificat TLS public pour son Application Load Balancer, renouvelé sans intervention humaine. Quelle est la solution la plus simple ?

- [x] Demander un certificat à AWS Certificate Manager et l'associer au répartiteur de charge
- [ ] Générer un certificat autosigné
- [ ] Acheter un certificat et l'installer sur chaque instance
- [ ] Stocker le certificat dans AWS Secrets Manager avec rotation

> ACM émet les certificats publics pour les services intégrés et les renouvelle automatiquement.
:::

:::quiz
Une application lit un mot de passe de base de données dans un fichier de configuration. L'équipe veut que ce mot de passe change tous les trente jours sans redéployer l'application. Que proposer ?

- [ ] Le stocker dans une variable d'environnement de l'instance
- [ ] Le chiffrer avec KMS et le laisser dans le fichier
- [ ] Le ranger dans un objet S3 chiffré
- [x] Le ranger dans AWS Secrets Manager avec rotation automatique, et le lire à l'exécution

> Secrets Manager organise la rotation ; l'application demande la valeur courante au moment de se connecter.
:::

:::quiz
Une entreprise veut refuser toute requête vers un bucket qui n'utiliserait pas HTTPS. Comment l'imposer ?

- [x] Une politique de bucket avec un `Deny` conditionné par `aws:SecureTransport` à `false`
- [ ] Le chiffrement par défaut SSE-KMS
- [ ] Le versionnage du bucket
- [ ] Une règle de groupe de sécurité sur le port 443

> La condition `aws:SecureTransport` permet de refuser les requêtes non chiffrées. Le chiffrement côté serveur ne concerne que les données au repos.
:::

:::quiz
Une équipe veut découvrir automatiquement lesquels de ses buckets S3 contiennent des données personnelles. Quel service utiliser ?

- [x] Amazon Macie
- [ ] Amazon Inspector
- [ ] Amazon GuardDuty
- [ ] AWS Config

> Macie détecte les données sensibles dans S3. GuardDuty détecte des menaces, Inspector des vulnérabilités logicielles.
:::

:::quiz
Un site de billetterie reçoit des milliers de commandes en quelques secondes à l'ouverture des ventes. Le service de paiement en aval ne traite que cinquante commandes par seconde. Aucune commande ne doit être perdue. Que placer entre les deux ?

- [ ] Un répartiteur de charge supplémentaire
- [x] Une file Amazon SQS, consommée par le service de paiement à son rythme
- [ ] Un cache Amazon ElastiCache
- [ ] Un réplica en lecture

> La file absorbe la rafale et conserve les commandes jusqu'à leur traitement : les deux côtés sont découplés.
:::

:::quiz
Lorsqu'une photo est déposée, il faut créer une miniature, mettre à jour un index de recherche et prévenir un service d'analyse. Les trois traitements sont indépendants et pourront évoluer séparément. Quelle architecture retenir ?

- [x] Un sujet SNS notifié par le dépôt, auquel sont abonnées trois files SQS, une par traitement
- [ ] Trois files SQS alimentées chacune par l'application
- [ ] Une fonction unique qui fait les trois traitements à la suite
- [ ] Un flux Step Functions Express avec trois étapes séquentielles

> La diffusion en éventail remet l'événement à chaque traitement dans sa propre file : un traitement en panne ne bloque pas les autres, et on peut en ajouter un quatrième sans toucher au producteur.
:::

:::quiz
Les ordres d'achat d'un même portefeuille doivent être exécutés strictement dans leur ordre d'arrivée, une seule fois chacun. Les portefeuilles sont indépendants entre eux. Que choisir ?

- [ ] Une file SQS standard avec un délai de visibilité long
- [ ] Un sujet SNS standard
- [ ] Un flux Kinesis avec un fragment par ordre
- [x] Une file SQS FIFO, avec l'identifiant du portefeuille comme groupe de messages

> La file FIFO garantit l'ordre au sein d'un groupe et écarte les doublons, tout en traitant en parallèle des groupes différents.
:::

:::quiz
Une fonction Lambda consomme une file SQS. Un message mal formé la fait échouer à chaque tentative et revient sans cesse. Comment l'écarter sans le perdre ?

- [x] Configurer sur la file une file de rebut, avec un nombre maximal de réceptions
- [ ] Réduire la durée de rétention de la file à une minute
- [ ] Augmenter la mémoire de la fonction
- [ ] Passer la file en FIFO

> Après le nombre de réceptions fixé, SQS déplace le message vers la file de rebut, où il peut être examiné.
:::

:::quiz
Un traitement de commande comporte six étapes, dont une attente de validation humaine pouvant durer plusieurs jours, avec des reprises en cas d'erreur. Quel service orchestre ce traitement ?

- [ ] Amazon SQS avec six files
- [ ] AWS Step Functions, flux Express
- [ ] Amazon EventBridge avec six règles
- [x] AWS Step Functions, flux Standard

> Les flux Standard durent jusqu'à un an et gèrent attentes, branches et reprises. Les flux Express sont limités à cinq minutes.
:::

:::quiz
Une API interne, appelée de façon irrégulière, exécute un traitement de quelques centaines de millisecondes. L'équipe veut ne gérer aucun serveur et ne rien payer en l'absence d'appels. Quelle architecture convient ?

- [ ] Deux instances EC2 derrière un Application Load Balancer
- [x] Amazon API Gateway devant une fonction AWS Lambda
- [ ] Un cluster Amazon EKS de trois nœuds
- [ ] Une instance EC2 réservée sur trois ans

> API Gateway et Lambda sont facturés à l'usage et ne demandent aucune administration de serveur : idéal pour un trafic irrégulier et des traitements courts.
:::

:::quiz
Une application web garde les sessions des utilisateur·rice·s dans la mémoire de chaque serveur. Dès que le groupe Auto Scaling retire une instance, des personnes sont déconnectées. Que changer ?

- [x] Sortir les sessions des serveurs, dans Amazon ElastiCache ou DynamoDB
- [ ] Augmenter la taille des instances
- [ ] Désactiver la mise à l'échelle
- [ ] Passer à un Network Load Balancer

> Des serveurs sans état peuvent être ajoutés ou retirés librement. L'état partagé va dans un magasin externe.
:::

:::quiz
Un site tourne sur deux instances EC2 dans une seule zone de disponibilité, derrière un Application Load Balancer. Quelle modification améliore le plus sa disponibilité ?

- [ ] Doubler la taille des deux instances
- [ ] Ajouter une troisième instance dans la même zone
- [ ] Remplacer le répartiteur par une adresse IP élastique
- [x] Répartir les instances sur deux zones de disponibilité, avec un groupe Auto Scaling

> La zone est le point de défaillance unique. La répartition sur plusieurs zones, avec remplacement automatique, y remédie.
:::

:::quiz
Une base Amazon RDS pour MySQL de production doit rester disponible si sa zone de disponibilité tombe, sans intervention et sans changer la chaîne de connexion. Que configurer ?

- [x] Un déploiement Multi-AZ
- [ ] Un réplica en lecture dans la même zone
- [ ] Des instantanés automatiques quotidiens
- [ ] Amazon RDS Proxy seul

> Le Multi-AZ maintient une copie synchrone dans une autre zone et bascule automatiquement derrière le même point de terminaison.
:::

:::quiz
Des centaines de fonctions Lambda ouvrent chacune une connexion vers une base Amazon RDS, qui atteint son nombre maximal de connexions lors des pics. Quel service règle ce problème ?

- [ ] Un réplica en lecture
- [ ] Amazon ElastiCache
- [ ] AWS Global Accelerator
- [x] Amazon RDS Proxy

> RDS Proxy mutualise et réutilise les connexions à la base, ce qui absorbe les rafales de connexions courtes.
:::

:::quiz
Le métier exige qu'en cas de perte de la région principale, l'application reprenne en moins de dix minutes, avec au plus quelques minutes de données perdues. Le budget ne permet pas de faire tourner deux régions à pleine capacité. Quelle stratégie choisir ?

- [x] Secours tiède
- [ ] Veilleuse
- [ ] Sauvegarde et restauration
- [ ] Multi-site actif

> Le secours tiède garde une copie complète en service, à capacité réduite : elle prend le trafic tout de suite puis grossit. La veilleuse exigerait de démarrer des serveurs.
:::

:::quiz
Une petite application interne peut rester arrêtée une journée et perdre jusqu'à vingt-quatre heures de données. Quelle stratégie de reprise est la plus économique ?

- [x] Sauvegarde et restauration, avec des sauvegardes copiées dans une autre région
- [ ] Veilleuse
- [ ] Secours tiède
- [ ] Multi-site actif

> Avec un RPO et un RTO d'une journée, des sauvegardes copiées ailleurs et une infrastructure recréée à la demande suffisent.
:::

:::quiz
Les objets d'un bucket doivent survivre à la perte complète de leur région. Une suppression accidentelle dans la région d'origine ne doit pas effacer la copie. Que configurer ?

- [ ] Le versionnage seul
- [ ] Une règle de cycle de vie vers S3 One Zone-IA
- [x] La réplication entre régions vers un bucket versionné, sans réplication des marqueurs de suppression
- [ ] S3 Transfer Acceleration

> La réplication entre régions copie les objets ailleurs ; ne pas répliquer les marqueurs de suppression protège la copie d'une suppression faite à l'origine.
:::

:::quiz
Une application est déployée en actif-passif dans deux régions. Les utilisateur·rice·s doivent être redirigé·e·s automatiquement vers la région de secours quand la principale ne répond plus. Que configurer dans Route 53 ?

- [ ] Deux enregistrements simples portant le même nom
- [ ] Une politique par latence
- [x] Une politique de basculement, avec un contrôle de santé sur le point d'entrée principal
- [ ] Une politique de géolocalisation par continent

> La politique de basculement sert le secours dès que le contrôle de santé du principal échoue.
:::

:::quiz
Une table DynamoDB doit accepter des écritures dans deux régions à la fois, pour une application active dans les deux. Quelle fonctionnalité utiliser ?

- [ ] Un réplica en lecture entre régions
- [ ] DynamoDB Accelerator (DAX)
- [ ] La restauration à un instant donné
- [x] Les tables globales DynamoDB

> Les tables globales répliquent entre régions et acceptent lectures et écritures dans chacune.
:::

:::quiz
Une application ancienne, impossible à modifier, tourne sur une seule instance EC2 et ne supporte pas d'être dupliquée. Comment améliorer sa reprise après une panne de l'instance, avec le moins de changements ?

- [ ] La réécrire en fonctions Lambda
- [x] La placer dans un groupe Auto Scaling de taille un, sur plusieurs zones, à partir d'une image à jour
- [ ] La laisser telle quelle et augmenter la taille de l'instance
- [ ] Ajouter un réplica en lecture

> Un groupe de taille un relance automatiquement l'instance, dans une autre zone si besoin, sans toucher à l'application.
:::

:::quiz
Une base de données transactionnelle sur EC2 exige 40 000 opérations d'entrée-sortie par seconde avec une latence très régulière. Quel stockage choisir ?

- [ ] Un volume gp3 avec ses réglages par défaut
- [ ] Un volume st1
- [x] Un volume io2 Block Express avec les IOPS provisionnées
- [ ] Amazon EFS

> Les volumes à IOPS provisionnées sont conçus pour de fortes IOPS à latence constante. gp3 offre 3 000 IOPS de base ; st1 est fait pour le débit séquentiel.
:::

:::quiz
Un traitement analyse chaque nuit des centaines de gigaoctets de journaux lus séquentiellement, sur un volume attaché à une instance. Le coût du stockage doit rester bas. Quel type de volume convient ?

- [ ] io2 Block Express
- [x] st1
- [ ] gp3 avec 16 000 IOPS provisionnées
- [ ] Le stockage d'instance uniquement

> st1 est un disque dur optimisé pour le débit séquentiel, bien moins cher que les SSD pour ce profil.
:::

:::quiz
Vingt instances Linux d'un groupe Auto Scaling, réparties sur trois zones, doivent accéder au même répertoire d'images téléversées, dont la taille varie fortement. Quel stockage choisir ?

- [ ] Un volume EBS par instance, synchronisé par un script
- [ ] Un volume EBS io2 en attachement multiple
- [x] Amazon EFS
- [ ] Amazon S3 Glacier Flexible Retrieval

> EFS est un système de fichiers partagé, élastique, accessible depuis plusieurs zones. L'attachement multiple d'EBS est limité à une zone.
:::

:::quiz
Un cluster de calcul scientifique a besoin d'un système de fichiers parallèle à très haut débit, alimenté par des données rangées dans S3. Quel service convient ?

- [ ] Amazon EFS en mode usage général
- [ ] Amazon FSx for Windows File Server
- [x] Amazon FSx for Lustre
- [ ] AWS Storage Gateway

> FSx for Lustre fournit un système de fichiers parallèle pour le calcul intensif et peut être relié à un bucket S3.
:::

:::quiz
Une application de vote en direct reçoit des pics très brefs et très élevés, puis presque rien. Chaque vote est une écriture simple par identifiant. Quelle base suit cette charge sans dimensionnement préalable ?

- [ ] Amazon RDS pour PostgreSQL sur une grosse instance
- [x] Amazon DynamoDB en mode à la demande
- [ ] Amazon Redshift
- [ ] Amazon Neptune

> DynamoDB à la demande absorbe des pics soudains d'accès par clé, sans capacité à réserver.
:::

:::quiz
Une table DynamoDB a pour clé de partition l'identifiant de commande. Une nouvelle page doit afficher, pour un client donné, ses commandes triées par date, des milliers de fois par minute. Que faire ?

- [ ] Parcourir la table avec un filtre sur le client
- [ ] Augmenter la capacité de lecture
- [ ] Ajouter DAX devant la table
- [x] Créer un index secondaire global avec le client comme clé de partition et la date comme clé de tri

> L'index global permet une requête ciblée par client, triée par date. Un parcours lirait toute la table à chaque affichage.
:::

:::quiz
Les mêmes fiches produits sont lues des millions de fois par jour dans une base relationnelle, et changent rarement. La base est saturée en lecture. Quelle évolution soulage le plus la base ?

- [ ] Un déploiement Multi-AZ
- [x] Un cache Amazon ElastiCache devant la base, avec une durée de vie adaptée
- [ ] Un volume de stockage plus grand
- [ ] La restauration à un instant donné

> Mettre en cache des données très lues et peu modifiées retire la plupart des lectures de la base.
:::

:::quiz
Un site d'information sert des images et des vidéos à un public mondial depuis un bucket S3 situé en Europe. Les lecteurs d'Asie se plaignent de lenteur. Quelle solution adopter ?

- [ ] Déplacer le bucket en Asie
- [ ] Activer S3 Transfer Acceleration pour les téléchargements
- [x] Diffuser le contenu par Amazon CloudFront, avec le bucket comme origine
- [ ] Ajouter une passerelle NAT

> CloudFront garde des copies dans des points de présence proches des internautes. Transfer Acceleration vise les envois vers le bucket.
:::

:::quiz
Une application TCP propriétaire est déployée dans deux régions. Ses clients exigent deux adresses IP fixes et une bascule en quelques secondes si une région tombe. Que choisir ?

- [ ] Amazon CloudFront
- [ ] Une politique Route 53 par latence avec une durée de vie d'une heure
- [x] AWS Global Accelerator
- [ ] Un Application Load Balancer par région

> Global Accelerator offre des adresses fixes annoncées mondialement, route vers la région saine et ne dépend pas des caches DNS.
:::

:::quiz
Une entreprise compte quarante VPC et trois sites reliés par VPN. Le maillage par appairages devient ingérable. Quelle architecture réseau simplifie l'ensemble ?

- [ ] Un appairage de plus entre chaque paire de VPC
- [ ] Une passerelle Internet commune
- [ ] Un point de terminaison de passerelle par VPC
- [x] Un AWS Transit Gateway auquel sont rattachés les VPC et les VPN

> Le Transit Gateway est un concentrateur : chaque VPC et chaque site s'y rattache une fois, et le routage est central.
:::

:::quiz
Des nœuds de calcul intensif échangent en permanence et ont besoin de la latence réseau la plus faible possible entre eux. Comment les placer ?

- [ ] Dans un groupe de placement de type répartition
- [x] Dans un groupe de placement de type cluster
- [ ] Dans trois régions différentes
- [ ] Derrière un Network Load Balancer

> La stratégie cluster rapproche les instances dans une même zone pour un débit et une latence optimaux entre nœuds.
:::

:::quiz
Des capteurs envoient des mesures en continu. Deux applications doivent les lire en temps réel, et une troisième doit pouvoir rejouer les dernières vingt-quatre heures après une correction. Quel service d'ingestion choisir ?

- [ ] Amazon SQS standard
- [ ] Amazon Data Firehose seul
- [x] Amazon Kinesis Data Streams
- [ ] AWS DataSync

> Un flux Kinesis conserve les enregistrements, les sert à plusieurs consommateurs et autorise le rejeu.
:::

:::quiz
Des journaux applicatifs doivent arriver dans S3, compressés et convertis dans un format en colonnes, avec le moins de code et d'exploitation possible. Que choisir ?

- [x] Amazon Data Firehose, avec conversion de format
- [ ] Une instance EC2 exécutant un script de copie
- [ ] Amazon SQS et une fonction Lambda par message
- [ ] AWS Transfer Family

> Firehose livre vers S3, met en tampon, compresse et peut convertir le format, sans programme de consommation à écrire.
:::

:::quiz
Des analystes veulent interroger en SQL, de temps en temps, des fichiers Parquet rangés dans S3, sans cluster à maintenir. Quel service convient ?

- [ ] Amazon EMR, avec un cluster permanent
- [ ] Amazon RDS
- [x] Amazon Athena
- [ ] Amazon Kinesis Data Streams

> Athena exécute des requêtes SQL sur S3 sans infrastructure et se paie à la requête.
:::

:::quiz
Un traitement par lots tourne quatre heures chaque nuit et peut reprendre là où il s'est arrêté. Comment réduire le plus son coût de calcul ?

- [ ] Des instances à la demande plus petites
- [ ] Un Savings Plan sur trois ans dimensionné pour ces quatre heures
- [x] Des instances Spot
- [ ] Des hôtes dédiés

> Une charge qui tolère l'interruption est le cas d'usage des instances Spot, de loin les moins chères.
:::

:::quiz
Une flotte de serveurs applicatifs tourne en permanence depuis deux ans et continuera au moins trois ans. L'équipe prévoit de migrer une partie vers des conteneurs Fargate. Quel engagement réduit le coût tout en couvrant ce changement ?

- [ ] Des instances réservées Standard
- [x] Un Compute Savings Plan
- [ ] Des réservations de capacité
- [ ] Des instances Spot

> Le Compute Savings Plan s'applique aussi à Fargate et à Lambda, et suit les changements de famille ou de région.
:::

:::quiz
Les environnements de développement, sur EC2, ne servent que du lundi au vendredi, de 8 h à 19 h. Quelle mesure réduit leur coût sans changer d'architecture ?

- [ ] Acheter des instances réservées sur un an
- [ ] Les passer sur des hôtes dédiés
- [x] Planifier leur arrêt le soir et le week-end, et leur démarrage le matin
- [ ] Doubler leur taille pour finir le travail plus vite

> Une instance arrêtée n'est plus facturée pour le calcul. Une réservation ferait payer les heures où elles ne servent pas.
:::

:::quiz
Des documents sont consultés souvent pendant un mois, rarement pendant un an, puis doivent être conservés cinq ans sans presque jamais être lus. Quelle configuration minimise le coût ?

- [ ] Tout garder en S3 Standard
- [ ] Tout placer dès le dépôt en S3 Glacier Deep Archive
- [x] Une règle de cycle de vie : Standard, puis Standard-IA après 30 jours, puis Glacier Deep Archive après un an
- [ ] S3 One Zone-IA dès le dépôt

> Le cycle de vie fait suivre aux objets leur profil d'accès. Les placer tout de suite en archive rendrait le premier mois lent et coûteux en récupérations.
:::

:::quiz
Une application dépose dans S3 des fichiers dont on ne peut pas prévoir la fréquence de consultation, qui varie d'un fichier à l'autre et dans le temps. Quelle classe de stockage éviter de se tromper ?

- [ ] S3 Standard-IA
- [ ] S3 Glacier Instant Retrieval
- [ ] S3 One Zone-IA
- [x] S3 Intelligent-Tiering

> Intelligent-Tiering déplace chaque objet vers le niveau adapté à ses accès réels, sans frais de récupération.
:::

:::quiz
Un organisme de recherche met à disposition un très gros jeu de données dans S3. Il veut que les équipes extérieures qui le téléchargent en supportent le coût de transfert. Que configurer ?

- [ ] Une politique de bucket publique
- [ ] S3 Transfer Acceleration
- [x] Le paiement par le demandeur (Requester Pays) sur le bucket
- [ ] Une distribution CloudFront gratuite

> Avec le paiement par le demandeur, celui qui télécharge paie la requête et le transfert ; le propriétaire paie le stockage.
:::

:::quiz
La facture d'un compte de test montre une ligne « passerelle NAT » importante, dans trois zones, pour un trafic très faible. Aucune exigence de disponibilité ne pèse sur cet environnement. Que proposer ?

- [ ] Ajouter une quatrième passerelle NAT
- [x] N'en garder qu'une, partagée par les sous-réseaux privés
- [ ] Remplacer les passerelles NAT par des répartiteurs de charge
- [ ] Désactiver les tables de routage

> Chaque passerelle NAT est facturée à l'heure. En test, une seule suffit ; en production, on en garde une par zone pour la disponibilité.
:::

:::quiz
Une base Amazon Aurora de préproduction n'est utilisée que quelques heures par jour, à des moments variables. Comment réduire son coût sans l'arrêter et la démarrer à la main ?

- [ ] Des instances réservées
- [ ] Un second réplica en lecture
- [x] Aurora Serverless, dont la capacité s'ajuste à la charge
- [ ] Un déploiement dans deux régions

> Une capacité qui suit la charge évite de payer une instance fixe pendant les heures creuses.
:::

:::quiz
Une table DynamoDB de sessions grossit sans fin : les sessions expirées ne sont jamais supprimées. Quelle solution les retire au moindre coût ?

- [ ] Un script qui parcourt la table chaque nuit et supprime les éléments
- [x] Une durée de vie (TTL) sur un attribut d'expiration
- [ ] Un index secondaire global sur la date
- [ ] Le passage en mode provisionné

> Le TTL supprime les éléments expirés sans consommer de capacité d'écriture, contrairement à un script de purge.
:::

:::quiz
Une entreprise veut connaître le coût mensuel de chacune de ses cinq applications, qui partagent un même compte AWS. Que mettre en place ?

- [ ] Un compte racine par application
- [ ] AWS Config avec une règle par application
- [x] Une étiquette `application` sur les ressources, activée comme étiquette d'allocation des coûts, puis une analyse dans Cost Explorer
- [ ] Un Savings Plan par application

> Les étiquettes d'allocation des coûts activées permettent de filtrer et de regrouper les dépenses par application.
:::

:::quiz
Des volumes gp2 ont été créés très grands uniquement pour obtenir davantage d'IOPS ; ils sont presque vides. Quelle évolution réduit la facture en gardant la performance ?

- [ ] Les remplacer par des volumes sc1
- [ ] Les passer en attachement multiple
- [x] Migrer vers des volumes gp3 dimensionnés pour la capacité utile, avec les IOPS provisionnées à part
- [ ] Activer le chiffrement

> Avec gp3, la performance ne dépend plus de la taille, et le gigaoctet est moins cher qu'en gp2.
:::

:::quiz
Un service appelé par des milliers de clients doit être protégé d'un client qui enverrait trop de requêtes, pour maîtriser à la fois la charge et le coût. Où placer cette limitation ?

- [ ] Dans la liste de contrôle d'accès réseau
- [ ] Dans la politique de clé KMS
- [ ] Dans la table de routage
- [x] Dans Amazon API Gateway, avec des quotas et une limitation de débit par client

> API Gateway sait limiter le débit et fixer des quotas par clé d'API, ce qui protège les services en aval et borne la dépense.
:::
