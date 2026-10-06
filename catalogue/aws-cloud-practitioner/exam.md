---
title: "Examen de validation — AWS : les bases du cloud (Cloud Practitioner)"
draw: 20
pass_mark: 80
minutes: 30
shuffle: true
---

Cet examen valide le parcours : concepts du cloud, sécurité et conformité, services essentiels, facturation et support. Vingt questions sont tirées au sort dans une réserve de soixante-dix, réparties comme les quatre domaines du guide de l'examen CLF-C02.

Ce n'est **pas** l'examen d'AWS, et ces questions n'en proviennent pas : elles ont été écrites pour ce parcours. Le vrai examen comporte 65 questions, dont certaines à réponses multiples, alors que celles-ci n'ont toutes qu'une bonne réponse. Réussir ici montre que tu maîtrises le contenu du parcours ; cela ne garantit pas le résultat le jour de la certification.

:::quiz
Une jeune entreprise ne sait pas si son application aura cent ou cent mille utilisateur·rice·s dans six mois. Quel avantage du cloud lui évite de trancher aujourd'hui ?

- [ ] Les économies d'échelle
- [x] Ne plus avoir à deviner la capacité nécessaire
- [ ] Le déploiement mondial en quelques minutes
- [ ] La fin des dépenses d'exploitation de centres de données

> Dans le cloud, la capacité s'ajuste à la demande réelle : inutile de la prévoir des mois à l'avance.
:::

:::quiz
Pourquoi AWS peut-il proposer un coût unitaire inférieur à celui qu'une petite structure obtiendrait seule ?

- [x] Parce que l'usage cumulé de très nombreux clients lui donne des économies d'échelle
- [ ] Parce que ses clients s'engagent tous sur trois ans
- [ ] Parce que ses services ne sont pas chiffrés par défaut
- [ ] Parce qu'il n'exploite qu'une seule région

> AWS achète et exploite à très grande échelle, et répercute une partie de ce gain sur ses prix.
:::

:::quiz
Une équipe obtient un environnement de test en dix minutes au lieu de six semaines, et peut le jeter après usage. Quel bénéfice du cloud cela illustre-t-il ?

- [x] L'agilité
- [ ] La souveraineté des données
- [ ] La tolérance aux pannes
- [ ] La facturation consolidée

> Obtenir et rendre des ressources en quelques minutes permet d'expérimenter vite : c'est l'agilité.
:::

:::quiz
Un site marchand voit son trafic décupler pendant les soldes, puis revenir à la normale. Quelle propriété lui permet de ne payer la capacité supplémentaire que pendant les soldes ?

- [x] L'élasticité
- [ ] La durabilité
- [ ] La redondance géographique
- [ ] La haute disponibilité

> L'élasticité ajuste la capacité à la demande, à la hausse puis à la baisse.
:::

:::quiz
Une revue d'architecture relève que les déploiements se font à la main, sans procédure écrite, et que personne ne regarde les tableaux de bord. Quel pilier du Well-Architected Framework est en cause ?

- [ ] Sécurité
- [x] Excellence opérationnelle
- [ ] Optimisation des coûts
- [ ] Efficacité des performances

> L'excellence opérationnelle porte sur la façon d'exploiter, de surveiller et d'améliorer un système, notamment par l'automatisation.
:::

:::quiz
Une application utilise des instances très puissantes alors qu'un type d'instance plus récent ferait le même travail avec moins de ressources. Quel pilier invite d'abord à choisir le type de ressource adapté au besoin ?

- [x] Efficacité des performances
- [ ] Sécurité
- [ ] Fiabilité
- [ ] Excellence opérationnelle

> L'efficacité des performances consiste à utiliser les ressources adaptées au besoin et à suivre l'évolution des offres.
:::

:::quiz
Quel pilier du Well-Architected Framework traite de la réduction de l'impact environnemental d'une charge de travail ?

- [x] Développement durable
- [ ] Fiabilité
- [ ] Excellence opérationnelle
- [ ] Optimisation des coûts

> Le pilier Développement durable (sustainability) vise à réduire la consommation d'énergie et de ressources.
:::

:::quiz
Un vieux logiciel de paie n'a plus que deux utilisatrices et sera remplacé par une offre SaaS du marché. De quelle stratégie de migration parle-t-on ?

- [ ] Rehost
- [x] Repurchase
- [ ] Replatform
- [ ] Refactor

> Remplacer une application par un produit du marché, souvent en SaaS, est un repurchase.
:::

:::quiz
Une entreprise copie telles quelles ses machines virtuelles vers des instances EC2, sans toucher aux applications, pour quitter vite son centre de données. Quelle stratégie applique-t-elle ?

- [x] Rehost
- [ ] Refactor
- [ ] Retain
- [ ] Repurchase

> Le rehost (« lift and shift ») déplace l'application sans la modifier.
:::

:::quiz
Dans le AWS Cloud Adoption Framework, quelle perspective traite de la formation des équipes et de l'évolution de la culture de l'organisation ?

- [ ] Plateforme
- [ ] Opérations
- [ ] Gouvernance
- [x] Personnes

> La perspective Personnes (People) fait le lien entre la technique et le métier : culture, compétences, conduite du changement.
:::

:::quiz
Parmi ces coûts, lequel disparaît quand on quitte son propre centre de données pour le cloud ?

- [ ] Le coût du transfert de données sortant
- [x] L'achat et le renouvellement du matériel serveur
- [ ] Le coût du support technique
- [ ] Le coût des licences des applications métier

> Sur site, le matériel est une dépense fixe à ta charge. Dans le cloud, tu paies un usage ; les autres coûts cités peuvent subsister.
:::

:::quiz
Une entreprise possède déjà des licences d'un système de base de données et veut les réutiliser sur AWS. Comment s'appelle ce modèle ?

- [x] BYOL (Bring Your Own License)
- [ ] Savings Plan
- [ ] Licence incluse
- [ ] Capacité réservée

> Avec BYOL, tu apportes tes propres licences ; avec « licence incluse », leur prix est compris dans celui du service.
:::

:::quiz
Quel est le principal intérêt d'automatiser la création d'une infrastructure plutôt que de la monter à la main ?

- [ ] Elle devient gratuite
- [ ] Elle n'a plus besoin de droits IAM
- [ ] Elle est automatiquement répartie sur plusieurs régions
- [x] Elle se reproduit à l'identique, sans erreur de manipulation

> L'automatisation rend les opérations répétables et fiables, et réduit le temps humain nécessaire.
:::

:::quiz
Une équipe monte une preuve de concept unique, qui sera supprimée demain. Quel moyen d'accès à AWS est raisonnable pour cette opération ponctuelle ?

- [x] La console web
- [ ] Un modèle CloudFormation relu par trois personnes
- [ ] Un pipeline de déploiement continu
- [ ] Un SDK intégré à une application

> Une opération ponctuelle et exploratoire peut se faire à la console. Ce qui doit être répété mérite d'être écrit.
:::

:::quiz
Dans le modèle de responsabilité partagée, qui applique les correctifs du système d'exploitation d'une instance Amazon EC2 ?

- [ ] AWS, toujours
- [x] Le client
- [ ] Le partenaire qui a vendu l'AMI
- [ ] Personne : les instances sont immuables

> Sur EC2, le système d'exploitation de l'instance relève du client. AWS s'occupe de l'infrastructure sous-jacente.
:::

:::quiz
Pour une fonction AWS Lambda, laquelle de ces tâches reste à la charge du client ?

- [ ] Mettre à jour le système d'exploitation qui exécute la fonction
- [x] Sécuriser le code de la fonction et décider qui peut l'appeler
- [ ] Remplacer les serveurs défaillants
- [ ] Assurer la sécurité physique des centres de données

> Avec un service sans serveur, AWS gère l'infrastructure et l'environnement d'exécution. Le code, les données et les droits restent au client.
:::

:::quiz
Quelle responsabilité incombe **toujours** au client, quel que soit le service utilisé ?

- [ ] La maintenance du réseau entre les zones de disponibilité
- [ ] La mise à jour des moteurs de base de données gérés
- [ ] Le remplacement des disques en panne
- [x] La gestion des droits d'accès à ses données

> Les données et les droits d'accès sont de la responsabilité du client dans tous les cas.
:::

:::quiz
Une entreprise soumise à une norme de paiement par carte veut le rapport d'audit prouvant qu'AWS respecte cette norme pour son infrastructure. Où le télécharge-t-elle ?

- [ ] Dans AWS Trusted Advisor
- [ ] Dans AWS CloudTrail
- [x] Dans AWS Artifact
- [ ] Dans le AWS Health Dashboard

> AWS Artifact met à disposition les rapports de conformité et d'audit d'AWS.
:::

:::quiz
Les données d'un site voyagent entre le navigateur et le serveur par HTTPS. De quelle protection s'agit-il ?

- [ ] Un chiffrement au repos
- [x] Un chiffrement en transit
- [ ] Une authentification multifacteur
- [ ] Un contrôle d'accès réseau

> HTTPS s'appuie sur TLS pour chiffrer les données pendant leur transport : c'est le chiffrement en transit.
:::

:::quiz
Un auditeur veut savoir comment le groupe de sécurité d'un serveur était configuré il y a trois mois et qui en respectait les règles internes. Quel service garde l'historique des configurations ?

- [ ] Amazon CloudWatch
- [ ] Amazon Inspector
- [ ] AWS Shield
- [x] AWS Config

> AWS Config enregistre les configurations des ressources dans le temps et les évalue par rapport à des règles.
:::

:::quiz
Le processeur d'une instance dépasse 90 % depuis dix minutes et l'équipe veut être alertée. Quel service fournit la mesure et l'alarme ?

- [x] Amazon CloudWatch
- [ ] AWS CloudTrail
- [ ] AWS Config
- [ ] Amazon Macie

> CloudWatch collecte les mesures et déclenche des alarmes sur des seuils. CloudTrail trace les appels d'API.
:::

:::quiz
Sans rien avoir configuré, pendant combien de temps l'historique des événements de gestion de CloudTrail reste-t-il consultable ?

- [ ] 7 jours
- [ ] 30 jours
- [x] 90 jours
- [ ] Indéfiniment

> L'historique d'événements couvre 90 jours. Pour conserver plus longtemps, on crée un trail qui écrit dans un bucket S3.
:::

:::quiz
Une clé d'accès de l'utilisateur racine traîne dans un script. Quelle est la bonne correction ?

- [ ] La déplacer dans un fichier caché du même dépôt
- [ ] La partager seulement avec les administrateurs
- [ ] La renouveler tous les mois
- [x] La supprimer et donner au script un rôle ou un utilisateur IAM aux droits limités

> L'utilisateur racine ne doit pas avoir de clé d'accès : ses droits ne peuvent pas être restreints. Un script reçoit une identité limitée.
:::

:::quiz
Quelle mesure renforce le plus la protection de la connexion d'un compte à la console AWS ?

- [ ] Changer la région par défaut
- [x] Activer l'authentification multifacteur (MFA)
- [ ] Créer un second utilisateur racine
- [ ] Désactiver CloudTrail

> La MFA ajoute un second facteur au mot de passe : un mot de passe volé ne suffit plus.
:::

:::quiz
Laquelle de ces tâches exige de se connecter comme utilisateur racine ?

- [ ] Créer un bucket S3
- [ ] Lancer une instance EC2
- [x] Fermer un compte AWS autonome
- [ ] Créer un groupe IAM

> Quelques opérations sont réservées à l'utilisateur racine, comme fermer un compte autonome ou modifier ses propres identifiants.
:::

:::quiz
Une salariée a besoin de lire un seul bucket. On lui rattache la politique `AmazonS3FullAccess` « pour être tranquille ». Quel principe est violé ?

- [ ] La haute disponibilité
- [ ] La séparation des régions
- [x] Le moindre privilège
- [ ] La défense en profondeur physique

> Le moindre privilège consiste à n'accorder que les droits nécessaires à la tâche : ici, la lecture d'un bucket.
:::

:::quiz
Une entreprise veut que ses salarié·e·s se connectent à tous ses comptes AWS avec les identifiants de l'annuaire de l'entreprise, sans créer d'utilisateur IAM par personne. Quel service est fait pour cela ?

- [ ] AWS KMS
- [ ] Amazon GuardDuty
- [ ] AWS Artifact
- [x] AWS IAM Identity Center

> IAM Identity Center fournit une connexion unique à plusieurs comptes et peut s'appuyer sur un annuaire existant (identité fédérée).
:::

:::quiz
Une personne d'un compte AWS partenaire doit accéder ponctuellement à des ressources de ton compte. Quel mécanisme évite de lui créer des identifiants permanents chez toi ?

- [x] Un rôle IAM qu'elle endosse depuis son compte
- [ ] Le partage du mot de passe d'un utilisateur IAM
- [ ] Une clé d'accès de l'utilisateur racine
- [ ] Un groupe IAM commun aux deux comptes

> Un rôle inter-comptes donne des identifiants temporaires à une identité d'un autre compte, sans identifiants durables à gérer.
:::

:::quiz
Quelle différence y a-t-il entre une politique gérée par AWS et une politique gérée par le client ?

- [ ] La première ne peut contenir que des interdictions
- [ ] La seconde ne s'applique qu'à l'utilisateur racine
- [x] La première est écrite et maintenue par AWS ; la seconde est écrite par toi
- [ ] Il n'y en a aucune

> AWS fournit des politiques prêtes à l'emploi, comme `ReadOnlyAccess`. Les politiques que tu écris sont « gérées par le client ».
:::

:::quiz
Un site subit une attaque qui l'inonde de trafic pour le rendre indisponible. Quel service d'AWS protège contre ce type d'attaque ?

- [ ] Amazon Inspector
- [ ] AWS Config
- [ ] Amazon Macie
- [x] AWS Shield

> AWS Shield protège contre les attaques par déni de service distribué (DDoS).
:::

:::quiz
Des requêtes HTTP tentent d'injecter du SQL dans le formulaire d'un site. Quel service filtre ces requêtes avant qu'elles n'atteignent l'application ?

- [ ] AWS Shield Standard
- [x] AWS WAF
- [ ] Amazon GuardDuty
- [ ] AWS Secrets Manager

> AWS WAF est un pare-feu applicatif web : il inspecte les requêtes HTTP et bloque celles qui correspondent à des règles.
:::

:::quiz
Une équipe veut savoir si ses instances EC2 contiennent des logiciels aux vulnérabilités connues. Quel service les recherche automatiquement ?

- [ ] AWS CloudTrail
- [ ] Amazon Macie
- [x] Amazon Inspector
- [ ] AWS Artifact

> Amazon Inspector analyse en continu les instances, les images de conteneurs et les fonctions Lambda à la recherche de vulnérabilités.
:::

:::quiz
Une entreprise veut vérifier qu'aucun de ses buckets S3 ne contient de numéros de carte bancaire en clair. Quel service est conçu pour cela ?

- [x] Amazon Macie
- [ ] AWS WAF
- [ ] Amazon Inspector
- [ ] AWS Firewall Manager

> Macie découvre les données sensibles stockées dans S3.
:::

:::quiz
Quel service regroupe au même endroit les alertes de sécurité de plusieurs services et de plusieurs comptes, et vérifie les bonnes pratiques ?

- [ ] AWS KMS
- [ ] Amazon CloudFront
- [ ] AWS Artifact
- [x] AWS Security Hub

> Security Hub centralise les constats de sécurité et évalue la conformité à des référentiels de bonnes pratiques.
:::

:::quiz
Où une entreprise peut-elle acheter un logiciel de sécurité d'un éditeur tiers, prêt à être déployé sur AWS ?

- [ ] Dans AWS Artifact
- [x] Sur AWS Marketplace
- [ ] Dans AWS Trusted Advisor
- [ ] Dans le Knowledge Center

> AWS Marketplace est le catalogue des logiciels et services d'éditeurs tiers.
:::

:::quiz
Qu'est-ce qui distingue une région AWS d'une zone de disponibilité ?

- [ ] Une région est un point de présence ; une zone est un pays
- [ ] Une région contient une seule zone, dédiée à un client
- [x] Une région est un lieu géographique qui contient plusieurs zones isolées les unes des autres
- [ ] Il n'y a aucune différence

> Une région regroupe plusieurs zones de disponibilité, chacune faite d'un ou plusieurs centres de données indépendants.
:::

:::quiz
Les utilisateur·rice·s d'une application sont réparti·e·s entre l'Europe et l'Asie, et la latence en Asie est mauvaise. Quelle raison justifie ici de déployer dans une seconde région ?

- [ ] La facturation consolidée
- [ ] La conformité à une norme de paiement
- [ ] Le moindre privilège
- [x] La faible latence pour les utilisateurs éloignés

> Rapprocher l'application de ses utilisateur·rice·s réduit la latence : c'est l'une des raisons d'utiliser plusieurs régions.
:::

:::quiz
Quel énoncé sur les zones de disponibilité d'une même région est exact ?

- [x] Elles sont conçues pour ne pas partager de point de défaillance unique
- [ ] Elles partagent la même alimentation électrique
- [ ] Elles sont situées dans le même bâtiment
- [ ] Elles ne peuvent pas communiquer entre elles

> Chaque zone a sa propre alimentation, son propre réseau et sa propre connectivité, pour que la panne de l'une n'entraîne pas les autres.
:::

:::quiz
Un traitement de paie tourne une heure chaque nuit sur une instance, avec un logiciel ancien qui exige un système d'exploitation précis. Quel service de calcul convient ?

- [ ] AWS Lambda
- [x] Amazon EC2
- [ ] Amazon S3
- [ ] Amazon Route 53

> Un logiciel qui impose son système et dure une heure dépasse ce que Lambda permet (15 minutes au plus) : il faut un serveur dont on contrôle le système, donc EC2.
:::

:::quiz
Quel service exécute des conteneurs en utilisant Kubernetes, géré par AWS ?

- [ ] Amazon ECS
- [ ] AWS Elastic Beanstalk
- [x] Amazon EKS
- [ ] AWS Batch

> Amazon EKS est le service Kubernetes géré. Amazon ECS est l'orchestrateur de conteneurs propre à AWS.
:::

:::quiz
Un développeur veut déposer le code d'une application web et laisser AWS créer pour lui les instances, le répartiteur de charge et la mise à l'échelle. Quel service le fait ?

- [ ] Amazon Lightsail
- [ ] AWS Fargate
- [ ] AWS Outposts
- [x] AWS Elastic Beanstalk

> Elastic Beanstalk déploie une application en provisionnant et en gérant l'infrastructure nécessaire.
:::

:::quiz
À quoi sert un répartiteur de charge (Elastic Load Balancing) placé devant plusieurs instances ?

- [ ] À chiffrer les disques des instances
- [ ] À réduire le prix horaire des instances
- [x] À distribuer les requêtes entre les instances saines
- [ ] À sauvegarder les instances chaque nuit

> Le répartiteur envoie le trafic vers les instances en bonne santé, sur plusieurs zones, et offre un point d'entrée unique.
:::

:::quiz
Une base de données en mémoire de plusieurs centaines de gigaoctets doit tourner sur une seule instance. Quelle catégorie d'instances EC2 est faite pour cela ?

- [ ] Optimisé pour le calcul
- [x] Optimisé pour la mémoire
- [ ] Optimisé pour le stockage
- [ ] Usage général

> Les instances optimisées pour la mémoire offrent beaucoup de mémoire vive par processeur, pour les bases en mémoire et l'analyse.
:::

:::quiz
Une application de réseau social stocke des milliards de profils, lus par leur identifiant, avec un temps de réponse de quelques millisecondes à toute échelle. Quelle base convient ?

- [ ] Amazon RDS pour MySQL
- [ ] Amazon Redshift
- [x] Amazon DynamoDB
- [ ] Amazon Neptune

> DynamoDB est une base clé-valeur sans serveur, conçue pour des accès par clé rapides à très grande échelle.
:::

:::quiz
Quelle base de données d'AWS est un moteur relationnel compatible avec MySQL et PostgreSQL, conçu par AWS pour le cloud ?

- [x] Amazon Aurora
- [ ] Amazon DynamoDB
- [ ] Amazon ElastiCache
- [ ] Amazon DocumentDB

> Aurora est le moteur relationnel d'AWS, compatible avec MySQL et PostgreSQL.
:::

:::quiz
Une entreprise veut déplacer sa base de données vers AWS en gardant la base d'origine en service pendant la copie. Quel service utilise-t-elle ?

- [ ] AWS Storage Gateway
- [ ] Amazon Athena
- [ ] AWS Backup
- [x] AWS Database Migration Service (DMS)

> DMS migre les données pendant que la base source reste opérationnelle.
:::

:::quiz
Deux applications réclament une analyse de plusieurs téraoctets de ventes avec des requêtes SQL complexes. Quel service est un entrepôt de données ?

- [ ] Amazon DynamoDB
- [x] Amazon Redshift
- [ ] Amazon ElastiCache
- [ ] Amazon SQS

> Redshift est l'entrepôt de données d'AWS, conçu pour l'analyse de gros volumes.
:::

:::quiz
Dans un VPC, quel composant permet à une instance d'un sous-réseau public de recevoir des connexions venant d'Internet ?

- [ ] Une passerelle NAT
- [ ] Un point de présence
- [x] Une passerelle Internet
- [ ] Un réplica en lecture

> La passerelle Internet relie le VPC à Internet dans les deux sens. La passerelle NAT ne permet que les sorties depuis un sous-réseau privé.
:::

:::quiz
Quelle affirmation décrit correctement un groupe de sécurité ?

- [ ] Il s'applique à un sous-réseau entier et il est sans état
- [ ] Il contient des règles d'interdiction numérotées
- [ ] Il chiffre le trafic entre deux instances
- [x] Il est avec état : la réponse à un trafic autorisé passe automatiquement

> Un groupe de sécurité s'applique à une ressource, ne contient que des autorisations et garde la mémoire des connexions.
:::

:::quiz
Quel service traduit le nom `www.asso.example` en adresse IP et peut orienter les internautes vers un point d'entrée en bonne santé ?

- [x] Amazon Route 53
- [ ] Amazon CloudFront
- [ ] AWS Direct Connect
- [ ] Amazon VPC

> Route 53 est le service DNS d'AWS : résolution de noms, enregistrement de domaines, contrôles de santé et routage.
:::

:::quiz
Une entreprise a besoin, dès cette semaine, d'une liaison chiffrée entre son bureau et son VPC, en utilisant sa connexion Internet existante. Que choisir ?

- [ ] AWS Direct Connect
- [x] AWS Site-to-Site VPN
- [ ] Amazon CloudFront
- [ ] AWS Global Accelerator

> Un VPN site à site s'établit rapidement sur Internet et chiffre le trafic. Direct Connect demande l'installation d'une liaison dédiée.
:::

:::quiz
Un disque doit rester attaché à une instance EC2 et conserver ses données quand l'instance est arrêtée puis redémarrée. Quel stockage utiliser ?

- [ ] Le stockage d'instance
- [ ] Amazon S3 Glacier Deep Archive
- [x] Un volume Amazon EBS
- [ ] Amazon SQS

> Un volume EBS persiste indépendamment de l'état de l'instance. Le stockage d'instance est éphémère.
:::

:::quiz
Des images recréables à partir des originaux sont rarement consultées, mais doivent s'afficher immédiatement quand on les demande. Quelle classe S3 est la plus économique pour elles ?

- [ ] S3 Standard
- [ ] S3 Glacier Deep Archive
- [ ] S3 Glacier Flexible Retrieval
- [x] S3 One Zone-IA

> One Zone-IA convient aux données rarement lues, à accès immédiat, que l'on sait recréer : elles ne sont gardées que dans une zone, donc moins cher.
:::

:::quiz
Une entreprise veut gérer depuis un seul endroit les sauvegardes de ses volumes EBS, de ses bases RDS et de ses tables DynamoDB. Quel service répond à ce besoin ?

- [ ] Amazon S3 Intelligent-Tiering
- [ ] AWS Storage Gateway
- [x] AWS Backup
- [ ] AWS DataSync

> AWS Backup centralise les plans de sauvegarde de plusieurs services.
:::

:::quiz
Les applications d'un site historique doivent continuer à écrire sur un partage de fichiers local, alors que les données sont en réalité conservées dans S3. Quel service fait ce pont ?

- [x] AWS Storage Gateway
- [ ] Amazon EFS
- [ ] Amazon EBS
- [ ] AWS Config

> Storage Gateway présente aux applications sur site des partages, volumes ou bandes dont les données sont stockées dans AWS.
:::

:::quiz
Une application doit lancer chaque jour à 6 h un traitement, et réagir quand une instance EC2 change d'état. Quel service d'intégration exprime ces deux besoins par des règles ?

- [ ] Amazon SQS
- [ ] Amazon SES
- [ ] AWS X-Ray
- [x] Amazon EventBridge

> EventBridge déclenche des cibles selon des règles : sur un horaire, ou sur le contenu d'un événement, y compris ceux des services AWS.
:::

:::quiz
Une application met dix secondes à répondre et traverse cinq services. Quel outil aide à voir où le temps est perdu ?

- [ ] AWS CodeBuild
- [x] AWS X-Ray
- [ ] Amazon Connect
- [ ] AWS Amplify

> X-Ray suit une requête de bout en bout à travers les composants d'une application.
:::

:::quiz
Une entreprise veut fournir à ses salarié·e·s en télétravail un bureau Windows complet, accessible à distance. Quel service choisir ?

- [ ] Amazon AppStream 2.0
- [ ] Amazon Connect
- [x] Amazon WorkSpaces
- [ ] AWS IoT Core

> WorkSpaces fournit des bureaux virtuels complets. AppStream 2.0 diffuse plutôt une application précise.
:::

:::quiz
Une équipe veut extraire automatiquement le texte et les champs de milliers de factures numérisées. Quel service d'IA est prévu pour cela ?

- [ ] Amazon Polly
- [ ] Amazon Lex
- [ ] Amazon Comprehend
- [x] Amazon Textract

> Textract extrait le texte, les formulaires et les tableaux de documents numérisés.
:::

:::quiz
Une application web régulière tournera sans interruption pendant les trois prochaines années, mais l'équipe veut pouvoir changer de famille d'instances, voire passer à Fargate. Quelle option d'achat réduit la facture tout en gardant cette liberté ?

- [ ] Des instances réservées Standard
- [x] Un Compute Savings Plan
- [ ] Des instances Spot
- [ ] Des hôtes dédiés

> Un Compute Savings Plan engage sur un montant horaire, pas sur une configuration : il s'applique quels que soient la famille, la taille, la région, et couvre aussi Fargate et Lambda.
:::

:::quiz
Avant de lancer un projet, une cheffe de projet veut estimer ce que coûterait par mois l'architecture envisagée. Quel outil utilise-t-elle ?

- [ ] AWS Cost Explorer
- [ ] AWS Budgets
- [x] AWS Pricing Calculator
- [ ] AWS Cost and Usage Report

> Le Pricing Calculator estime le coût d'une architecture qui n'existe pas encore. Cost Explorer analyse des dépenses déjà faites.
:::

:::quiz
Une équipe veut un graphique de ses dépenses des six derniers mois, ventilées par service, et une prévision pour les mois à venir. Quel outil le fournit ?

- [x] AWS Cost Explorer
- [ ] AWS Pricing Calculator
- [ ] AWS Trusted Advisor
- [ ] AWS Artifact

> Cost Explorer visualise les coûts passés, permet de les filtrer et propose une prévision.
:::

:::quiz
Trois projets partagent le même compte AWS. Que faut-il mettre en place pour savoir ce que chacun coûte ?

- [ ] Un utilisateur racine par projet
- [ ] Une région par projet
- [ ] Un répartiteur de charge par projet
- [x] Des étiquettes d'allocation des coûts posées sur les ressources, puis activées

> Les étiquettes d'allocation des coûts, une fois activées dans la console de facturation, permettent de ventiler la facture par projet.
:::

:::quiz
Parmi ces transferts de données, lequel n'est pas facturé ?

- [ ] Des données envoyées d'une région AWS vers une autre
- [x] Des données envoyées depuis Internet vers AWS
- [ ] Des données téléchargées depuis S3 vers Internet, au-delà de la franchise mensuelle
- [ ] Des données répliquées d'un bucket parisien vers un bucket irlandais

> Les données qui entrent dans AWS depuis Internet ne sont pas facturées. Les sorties vers Internet et les transferts entre régions le sont.
:::

:::quiz
Un éditeur de logiciels facture ses licences par processeur physique et exige que le client sache sur quel serveur tourne son produit. Quelle option d'achat EC2 répond à cette contrainte ?

- [ ] Instances Spot
- [ ] Savings Plans
- [x] Hôtes dédiés
- [ ] À la demande, dans une seule zone

> Un hôte dédié est un serveur physique réservé à un client, ce qui permet d'utiliser des licences liées au matériel.
:::

:::quiz
Une organisation AWS compte cinq comptes. L'un d'eux a acheté des instances réservées qu'il n'utilise pas entièrement. Que se passe-t-il avec la facturation consolidée ?

- [ ] La réduction inutilisée est perdue
- [ ] Les instances réservées sont revendues automatiquement
- [ ] Les autres comptes doivent racheter leurs propres réservations
- [x] La réduction peut profiter à l'usage correspondant des autres comptes de l'organisation

> Pour la facturation, les comptes d'une organisation sont traités comme un seul : les réductions des instances réservées peuvent être partagées.
:::

:::quiz
Une entreprise fait tourner une application critique et veut un·e interlocuteur·rice technique attitré·e chez AWS, qui connaît son contexte. Quelle offre de support est la première à l'inclure ?

- [ ] Basic
- [ ] AWS Business Support+
- [x] AWS Enterprise Support
- [ ] Le forum AWS re:Post

> Le responsable technique de compte (TAM) attitré fait partie d'Enterprise Support.
:::

:::quiz
Quel outil signale, entre autres, des instances inutilisées, des limites de service presque atteintes et des groupes de sécurité trop ouverts ?

- [ ] AWS Health Dashboard
- [ ] AWS Budgets
- [x] AWS Trusted Advisor
- [ ] AWS Pricing Calculator

> Trusted Advisor compare le compte aux bonnes pratiques : coûts, performance, sécurité, tolérance aux pannes, limites de service, excellence opérationnelle.
:::

:::quiz
AWS prévoit une maintenance sur le matériel qui héberge l'une de tes instances. Où en es-tu informé·e, avec la liste de tes ressources concernées ?

- [x] Dans le AWS Health Dashboard
- [ ] Dans AWS Artifact
- [ ] Dans AWS Cost Explorer
- [ ] Dans AWS Marketplace

> Le Health Dashboard présente les incidents et les maintenances qui touchent tes propres ressources.
:::

:::quiz
Ton site reçoit du trafic malveillant provenant d'adresses qui appartiennent à AWS. À qui le signaler ?

- [ ] À AWS Professional Services
- [ ] Au réseau de partenaires AWS
- [ ] À ton responsable technique de compte uniquement
- [x] À l'équipe AWS Trust & Safety

> L'équipe Trust & Safety traite les signalements d'abus de ressources AWS.
:::
