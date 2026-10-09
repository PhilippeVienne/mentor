---
title: "Examen de validation — AWS : architectures d'entreprise (Solutions Architect Professional)"
draw: 15
pass_mark: 80
minutes: 45
shuffle: true
---

Cet examen valide le cours : organisation multi-comptes, nouvelles solutions, amélioration de l'existant, migration et modernisation. Quinze questions sont tirées au sort dans une réserve de quarante-cinq, réparties comme les quatre domaines du guide de l'examen SAP-C02. Ce sont des scénarios : les quatre réponses sont plausibles, une seule respecte **toutes** les contraintes de l'énoncé.

Ce n'est **pas** l'examen d'AWS, et ces questions n'en proviennent pas : elles ont été écrites pour ce cours, et n'ont toutes qu'une bonne réponse, alors que l'examen réel comporte aussi des questions à réponses multiples, plus longues, dans une autre langue que le français. Réussir ici montre que tu maîtrises le contenu du cours ; AWS décrit le public de la certification comme ayant au moins deux ans de pratique, que cet examen ne mesure pas.

:::quiz
Un groupe de soixante comptes AWS veut empêcher toute création de ressources hors de deux régions européennes, y compris par les administrateurs locaux, et sans maintenance à chaque nouveau service lancé par AWS. Les services globaux comme IAM doivent continuer de fonctionner. Quelle solution retenir ?

- [ ] Une politique IAM d'interdiction, déployée par script sur chaque rôle de chaque compte
- [x] Une SCP rattachée aux unités d'organisation, qui interdit les actions hors de ces régions par une condition sur la région demandée, en exemptant les services globaux
- [ ] Une SCP en liste d'autorisations énumérant chaque service permis dans chaque région
- [ ] Une règle AWS Config qui détecte les ressources créées ailleurs

> La SCP s'impose à toutes les identités des comptes membres. L'interdiction conditionnée à la région, avec exemption des services globaux, ne demande aucun entretien quand AWS ajoute un service. Une règle Config détecte après coup ; elle n'empêche pas.
:::

:::quiz
Une entreprise crée chaque mois de nouveaux comptes AWS. Chaque compte doit recevoir automatiquement un rôle d'audit, un jeu de règles de conformité et une journalisation, sans action manuelle. Quelle combinaison répond le mieux ?

- [ ] Une procédure écrite que chaque équipe applique à la création de son compte
- [ ] Une fonction Lambda planifiée qui parcourt les comptes chaque nuit
- [ ] AWS RAM, en partageant les rôles depuis le compte de gestion
- [x] AWS Control Tower pour créer les comptes, et des StackSets ciblant les unités d'organisation avec déploiement automatique

> L'Account Factory de Control Tower crée des comptes conformes, et un StackSet à autorisations gérées par le service se déploie de lui-même dans tout compte qui rejoint l'unité d'organisation ciblée.
:::

:::quiz
Une organisation veut qu'aucune équipe ne puisse supprimer ou modifier les journaux CloudTrail, y compris les administrateurs des comptes où l'activité a lieu. Quelle architecture est la plus solide ?

- [x] Un trail d'organisation vers un bucket d'un compte d'archive dédié, avec validation d'intégrité et verrouillage d'objets, et une SCP interdisant d'arrêter CloudTrail
- [ ] Un trail par compte, vers un bucket du compte de gestion ouvert en écriture à tous
- [ ] Un bucket de journaux dans chaque compte, avec une politique de bucket restrictive
- [ ] L'export hebdomadaire de l'historique d'événements vers un poste d'administration

> Les journaux quittent le compte surveillé, le trail échappe aux comptes membres, l'intégrité est vérifiable et le verrouillage empêche la suppression. Une politique de bucket locale reste modifiable par l'administrateur du compte.
:::

:::quiz
L'équipe de sécurité doit gérer Amazon GuardDuty et AWS Security Hub pour l'ensemble des comptes, y compris les futurs, sans jamais se connecter au compte de gestion. Que mettre en place ?

- [x] Désigner le compte de sécurité comme administrateur délégué de ces services, avec activation automatique pour les nouveaux comptes
- [ ] Créer un utilisateur IAM de sécurité dans le compte de gestion
- [ ] Activer les services à la main dans chaque compte et envoyer les constats par courriel
- [ ] Appairer les VPC de tous les comptes avec celui de la sécurité

> L'administration déléguée confie la gestion du service, pour toute l'organisation, à un compte membre, et couvre automatiquement les comptes qui arrivent.
:::

:::quiz
Un éditeur de logiciels en SaaS doit lire des métriques dans les comptes de ses clients. Un client veut lui ouvrir un accès sans créer d'identifiants durables et sans risquer qu'un autre client de l'éditeur s'en serve. Quelle conception ?

- [x] Un rôle dont la politique de confiance désigne le compte de l'éditeur et exige un identifiant externe propre au client
- [ ] Un rôle dont la politique de confiance désigne le compte de l'éditeur, sans condition
- [ ] Un utilisateur IAM dont la clé d'accès est transmise à l'éditeur
- [ ] Un bucket public où le client dépose ses métriques

> Le rôle fournit des identifiants temporaires ; l'identifiant externe évite qu'un autre client de l'éditeur lui fasse endosser ce rôle (problème de l'adjoint désorienté).
:::

:::quiz
Tous les comptes d'une organisation doivent déposer leurs journaux applicatifs dans un bucket central. Aucun compte ne doit pouvoir lire ou écraser les journaux des autres, et la politique ne doit pas être modifiée à l'arrivée d'un compte. Quelle politique de bucket convient ?

- [ ] Autoriser `s3:*` à tous les comptes de l'organisation
- [ ] Énumérer chaque compte dans `Principal` avec `s3:PutObject` sur tout le bucket
- [ ] Autoriser le dépôt à tout le monde et compter sur des noms d'objets difficiles à deviner
- [x] Autoriser `s3:PutObject` aux identités de l'organisation (`aws:PrincipalOrgID`), sur un préfixe contenant l'identifiant du compte appelant (`${aws:PrincipalAccount}`)

> La condition sur l'organisation couvre les comptes futurs ; le préfixe construit à partir du compte appelant cantonne chacun à son espace ; n'accorder que le dépôt empêche la lecture.
:::

:::quiz
Une entreprise relie quarante VPC et deux centres de données. Les VPC de production et de test ne doivent jamais communiquer, mais tous doivent atteindre les services partagés et les centres de données, par une seule liaison Direct Connect redondée. Quelle architecture ?

- [ ] Un maillage d'appairages de VPC et une interface virtuelle privée par VPC
- [x] Un Transit Gateway avec des tables de routage distinctes par environnement, relié aux sites par une interface virtuelle de transit et une Direct Connect gateway
- [ ] Un VPC unique partagé entre tous les comptes, sans segmentation
- [ ] Un VPN par VPC vers chaque centre de données

> Le Transit Gateway centralise le routage et segmente par tables de routage ; l'interface de transit et la Direct Connect gateway amènent les sites une seule fois pour tous les VPC.
:::

:::quiz
Une société acquiert une filiale dont les VPC utilisent les mêmes plages d'adresses que les siens. Une application de la filiale doit consommer une API interne de la maison mère, rapidement, sans renuméroter aucun réseau. Que proposer ?

- [x] Exposer l'API par AWS PrivateLink et la consommer par un point de terminaison d'interface dans le VPC de la filiale
- [ ] Un rattachement des deux VPC au même Transit Gateway
- [ ] Un appairage de VPC entre les deux réseaux
- [ ] Une passerelle NAT dans chaque VPC

> L'appairage et le Transit Gateway exigent des plages distinctes. PrivateLink expose un service par une interface locale au VPC consommateur, indépendamment des plages.
:::

:::quiz
Des serveurs d'un centre de données doivent résoudre les noms de zones hébergées privées d'AWS, et les instances AWS doivent résoudre les noms du domaine interne de l'entreprise. Les règles doivent valoir pour tous les comptes. Quelle conception ?

- [x] Des points de terminaison Route 53 Resolver entrant et sortant dans le compte réseau, et des règles de transfert partagées avec les autres comptes par AWS RAM
- [ ] Remplacer les zones privées par des zones publiques
- [ ] Recopier chaque nuit les enregistrements d'un système DNS à l'autre
- [ ] Un serveur DNS sur une instance dans chaque VPC

> Le point entrant sert les requêtes du site, le sortant transmet au site celles du domaine interne, et le partage des règles par RAM évite de les redéfinir dans chaque compte.
:::

:::quiz
Une liaison Direct Connect de 10 Gbit/s relie un siège à AWS par un seul emplacement. Le métier exige que la connectivité survive à la perte complète de cet emplacement, au même débit. Que faut-il ?

- [ ] Un second port dans le même emplacement, agrégé au premier
- [ ] Une interface virtuelle publique en plus de l'interface privée
- [ ] Un VPN site à site de secours
- [x] Une seconde connexion dans un autre emplacement Direct Connect

> L'agrégation de liens dans un même emplacement tombe avec lui. Un VPN survivrait, mais sans garantir 10 Gbit/s. Seule une connexion dans un second emplacement répond aux deux exigences.
:::

:::quiz
Une organisation veut que chaque euro dépensé sur AWS soit rattaché à une unité métier, et que les étiquettes utilisées soient cohérentes entre des centaines de comptes. Quelle combinaison mettre en place ?

- [x] Un compte par application, des étiquettes d'allocation des coûts activées, et des politiques d'étiquettes d'organisation pour normaliser clés et valeurs
- [ ] Un compte unique pour toute l'entreprise, avec des noms de ressources parlants
- [ ] Un tableur tenu à jour par chaque équipe
- [ ] Des budgets identiques dans tous les comptes

> Les comptes donnent une première répartition sans effort ; les étiquettes activées affinent ; les politiques d'étiquettes garantissent la cohérence sans laquelle les rapports sont inexploitables.
:::

:::quiz
Dans une organisation, un compte achète un Savings Plan de calcul qu'il n'utilise qu'à moitié. Que devient la partie inutilisée, par défaut ?

- [ ] Elle est perdue chaque heure
- [x] Elle s'applique à l'usage éligible des autres comptes de l'organisation
- [ ] Elle est remboursée en fin de mois
- [ ] Elle est reportée sur le mois suivant

> Avec la facturation consolidée, les réductions des Savings Plans et des instances réservées se partagent par défaut entre les comptes de l'organisation. Ce partage peut être désactivé.
:::

:::quiz
Une mise à jour d'un modèle CloudFormation de production modifie le nom d'une base de données. Avant de l'appliquer, l'équipe veut savoir exactement quelles ressources seront touchées et de quelle façon. Que faire ?

- [x] Créer un ensemble de modifications et lire, pour chaque ressource, l'action et l'indicateur de remplacement
- [ ] Lancer une détection de dérive
- [ ] Appliquer la mise à jour en dehors des heures ouvrées
- [ ] Activer la protection contre la suppression de la pile

> L'ensemble de modifications montre ce qui serait ajouté, modifié ou remplacé, sans rien exécuter. La détection de dérive compare l'existant au modèle actuel, pas au futur.
:::

:::quiz
Une fonction Lambda sert une API très sollicitée. Chaque nouvelle version doit recevoir 10 % du trafic pendant dix minutes, puis la totalité si aucune alarme ne s'est déclenchée, avec retour arrière automatique dans le cas contraire. Quelle solution demande le moins de développement ?

- [ ] Deux fonctions distinctes derrière un répartiteur de charge réglé à la main
- [x] Un alias pointant sur les versions publiées, dont le trafic est déplacé par CodeDeploy avec une configuration canari et des alarmes CloudWatch
- [ ] Une mise à jour directe du code de la fonction, surveillée par un opérateur
- [ ] Des enregistrements Route 53 pondérés vers deux API

> CodeDeploy gère nativement le déplacement canari du trafic d'un alias Lambda et le retour arrière sur alarme.
:::

:::quiz
Une plateforme de paiement doit rester disponible si une région entière disparaît. Les transactions sont relationnelles, écrites dans une seule région à la fois, avec une perte tolérée de quelques secondes et une reprise en moins de cinq minutes. Quelle base de données ?

- [ ] Amazon RDS Multi-AZ, avec des instantanés copiés chaque heure dans une autre région
- [x] Aurora Global Database, avec promotion de la région secondaire
- [ ] Une table globale DynamoDB
- [ ] Une base autogérée sur EC2, répliquée par un script

> Aurora Global Database réplique de façon asynchrone avec un retard faible et permet une promotion rapide. Des instantanés horaires donneraient un RPO d'une heure ; une table globale n'est pas relationnelle.
:::

:::quiz
Un plan de reprise prévoit de modifier des enregistrements DNS et de lancer des instances dans la région de secours au moment du sinistre. Lors d'un exercice, ces opérations échouent parce que les API de gestion sont perturbées. Quel principe aurait évité cet échec ?

- [x] La stabilité statique : capacité déjà en service dans la région de secours, et bascule par des contrôles de santé ou de routage relevant du plan de données
- [ ] Réduire la durée de vie des enregistrements DNS
- [ ] Augmenter le nombre de personnes d'astreinte
- [ ] Sauvegarder plus souvent

> Créer et modifier relève du plan de contrôle, moins disponible. Une bascule robuste ne repose que sur des mécanismes de plan de données et sur une capacité déjà présente.
:::

:::quiz
Une application de réseau social accepte des mises à jour de profil dans trois régions. Une incohérence de quelques secondes entre régions est tolérée, mais chaque région doit continuer d'accepter les écritures si une autre tombe. Quelle solution ?

- [x] Une table globale DynamoDB
- [ ] Aurora Global Database avec écritures dans la seule région principale
- [ ] Amazon RDS avec un réplica en lecture dans chaque région
- [ ] Un bucket S3 répliqué dans un seul sens

> Les tables globales acceptent lectures et écritures dans chaque région, avec réplication entre elles et cohérence à terme, ce que l'énoncé accepte.
:::

:::quiz
Des sauvegardes doivent être conservées sept ans sans qu'aucun compte de l'organisation, même compromis, puisse les supprimer. Elles concernent des bases RDS, des volumes EBS et des tables DynamoDB dans vingt comptes. Quelle conception ?

- [ ] Des instantanés pris par un script dans chaque compte
- [ ] La réplication de chaque ressource vers une autre région
- [ ] Des exports manuels trimestriels vers S3 Standard
- [x] Des politiques de sauvegarde d'organisation avec AWS Backup, une copie vers un coffre d'un compte dédié, verrouillé par Vault Lock en mode conformité

> AWS Backup centralise les plans pour plusieurs services ; la politique d'organisation les impose aux comptes ; le compte dédié et le verrou en mode conformité mettent les copies hors d'atteinte.
:::

:::quiz
Un site d'information à forte audience subit des attaques par saturation et des robots qui aspirent son contenu. L'entreprise veut absorber les attaques au plus loin de ses serveurs et appliquer les mêmes protections à tous ses comptes. Quelle combinaison ?

- [ ] Des groupes de sécurité restrictifs et des instances plus grosses
- [x] CloudFront devant l'application, AWS WAF avec des règles fondées sur le débit, Shield Advanced, le tout imposé par AWS Firewall Manager
- [ ] Un Network Load Balancer et des listes de contrôle d'accès réseau
- [ ] Un VPN obligatoire pour les lecteurs

> CloudFront et Shield absorbent à la périphérie, WAF filtre et limite le débit par adresse, Firewall Manager déploie ces règles sur tous les comptes de l'organisation.
:::

:::quiz
Des données chiffrées avec une clé KMS gérée par le client sont répliquées de Paris vers Francfort. Lors d'un exercice de reprise, l'application de Francfort ne parvient pas à les déchiffrer quand Paris est isolée. Quelle conception corrige ce défaut ?

- [ ] Utiliser la clé gérée par AWS dans les deux régions
- [x] Utiliser une clé KMS multi-régions, avec une réplique à Francfort
- [ ] Désactiver le chiffrement des données répliquées
- [ ] Ouvrir un appairage de VPC entre les deux régions

> Une clé KMS ordinaire n'existe que dans sa région. Les clés multi-régions partagent leur matériel : le déchiffrement se fait localement, sans dépendre de l'autre région.
:::

:::quiz
Une application reçoit dix fois son trafic habituel pendant deux heures, une fois par mois, à une date connue. Le reste du temps, la charge est faible et stable. La base relationnelle est le composant limitant pendant le pic, à cause d'un afflux d'écritures. Quelle évolution est la plus adaptée ?

- [x] Placer une file SQS devant les écritures et les traiter à un rythme que la base supporte, en prévenant les utilisateurs du délai
- [ ] Ajouter des réplicas en lecture
- [ ] Dimensionner la base pour le pic toute l'année
- [ ] Passer la base dans une autre région pendant le pic

> Les réplicas en lecture n'aident pas les écritures. La mise en tampon lisse le pic sans payer toute l'année une capacité utile deux heures par mois.
:::

:::quiz
Une nouvelle application traite des commandes en six étapes, dont un paiement et une réservation de stock. Si une étape échoue, les étapes déjà faites doivent être annulées, et l'état de chaque commande doit être consultable pendant trente jours. Quel service d'intégration est le plus adapté ?

- [ ] Amazon SNS avec six sujets
- [ ] Amazon Kinesis Data Streams
- [ ] Six fonctions Lambda qui s'appellent en chaîne
- [x] AWS Step Functions, flux Standard, avec des étapes de compensation

> Un flux Standard conserve l'état de chaque exécution, décrit branches et compensations, et dure le temps nécessaire. Des fonctions chaînées dispersent cette logique et n'offrent aucune vue d'ensemble.
:::

:::quiz
Une entreprise lance un service dont elle ne connaît pas la charge future. Elle veut le moins d'infrastructure à provisionner et à corriger, et une facture proportionnelle à l'usage. Quelle orientation d'architecture ?

- [ ] Des instances EC2 réservées trois ans, dimensionnées largement
- [x] Des services gérés et sans serveur : API Gateway, Lambda, DynamoDB à la demande, SQS
- [ ] Un cluster Kubernetes autogéré sur EC2
- [ ] Des hôtes dédiés

> Les services sans serveur suppriment la gestion des serveurs et leur mise à jour, s'ajustent seuls à la charge et se paient à l'usage : le choix raisonnable tant que la charge est inconnue.
:::

:::quiz
Une plateforme d'analyse lance chaque nuit des centaines d'instances pendant quatre heures. Les traitements reprennent sans dommage après une interruption. L'entreprise fait aussi tourner en permanence une flotte stable de serveurs web. Quelle stratégie d'achat minimise le coût ?

- [ ] Des instances réservées pour les deux usages
- [ ] Tout à la demande
- [ ] Des instances Spot pour les deux usages
- [x] Des instances Spot pour les traitements nocturnes, et un Savings Plan pour la flotte permanente

> L'interruptible va en Spot, la base stable sous engagement. Mettre les serveurs web en Spot exposerait le site aux interruptions.
:::

:::quiz
Une application est répartie sur trois zones de disponibilité. Ses instances privées envoient chaque jour des téraoctets vers Amazon S3 à travers des passerelles NAT, et la facture de transfert explose. Quelle modification réduit ce coût sans toucher à l'application ?

- [ ] Une seule passerelle NAT pour les trois zones
- [x] Un point de terminaison de passerelle pour S3, ajouté aux tables de routage des sous-réseaux privés
- [ ] Des instances plus grosses
- [ ] Le passage des objets en S3 Glacier

> Le point de terminaison de passerelle est sans frais et fait sortir ce trafic des passerelles NAT, facturées au volume. Une NAT unique ajouterait du trafic entre zones et un point de défaillance.
:::

:::quiz
Quarante alarmes se déclenchent à chaque incident, et l'astreinte ne distingue plus la cause des conséquences. Parallèlement, une alarme sur une métrique d'erreurs reste « sans données » en temps normal. Quelles corrections apporter ?

- [ ] Supprimer la moitié des alarmes et allonger toutes les périodes
- [ ] Envoyer toutes les alarmes dans un canal de discussion commun
- [ ] Remplacer les alarmes par un tableau de bord consulté chaque matin
- [x] Regrouper les alarmes dans des alarmes composites, et traiter les données manquantes comme non violantes pour la métrique d'erreurs

> Les alarmes composites réduisent le bruit en ne notifiant que les combinaisons significatives ; le traitement des données manquantes évite l'état « sans données » d'une métrique émise seulement en cas d'erreur.
:::

:::quiz
Des instances de production sont régulièrement modifiées à la main par connexion SSH, ce qui crée des écarts de configuration, et le port 22 est ouvert sur Internet. Quelle évolution traite les deux problèmes ?

- [ ] Changer la clé SSH chaque semaine
- [ ] Restreindre le port 22 à l'adresse du bureau
- [x] Fermer le port 22, utiliser Session Manager pour les accès tracés, et State Manager ou l'infrastructure immuable pour imposer la configuration
- [ ] Ajouter un second bastion

> Session Manager supprime le port ouvert et journalise les sessions ; une configuration imposée, ou des instances remplacées plutôt que modifiées, fait disparaître les écarts.
:::

:::quiz
Une règle AWS Config signale chaque semaine des buckets S3 sans chiffrement par défaut, corrigés à la main par un administrateur. L'entreprise veut supprimer cette tâche répétitive tout en gardant une trace. Que mettre en place ?

- [ ] Un courriel de rappel aux équipes
- [ ] Une SCP interdisant la création de buckets
- [ ] La suppression de la règle Config
- [x] Une correction automatique associée à la règle, exécutant une procédure Systems Manager Automation

> AWS Config peut déclencher une procédure de correction sur chaque ressource non conforme, avec un historique d'exécution.
:::

:::quiz
Une application de billetterie tombe à chaque ouverture de vente. Les mesures montrent : serveurs applicatifs à 30 % de processeur, base relationnelle à 100 %, 90 % de lectures portant sur le même catalogue. Quelle amélioration traiter en premier, avec le moins de changements ?

- [ ] Doubler le nombre de serveurs applicatifs
- [ ] Migrer la base vers une autre région
- [x] Mettre le catalogue en cache avec ElastiCache
- [ ] Réécrire l'application en microservices

> La base est le goulot, saturée par des lectures identiques : un cache les lui retire. Ajouter des serveurs applicatifs augmenterait encore la pression sur la base.
:::

:::quiz
Une équipe pense que son service supporte la perte d'une zone de disponibilité. Le contrat promet 99,9 % de disponibilité. Comment vérifier l'hypothèse sans attendre une vraie panne ?

- [ ] Relire le schéma d'architecture avec un pair
- [x] Mener une expérience avec AWS Fault Injection Service qui rend une zone indisponible, avec des conditions d'arrêt, et mesurer les indicateurs de service
- [ ] Ajouter une quatrième zone
- [ ] Doubler la capacité dans chaque zone

> Une expérience contrôlée, bornée par des conditions d'arrêt, confronte l'hypothèse aux indicateurs réels du service.
:::

:::quiz
Un compte contient de nombreuses instances, volumes et buckets dont personne ne connaît l'usage. La direction demande d'identifier rapidement ce qui est surdimensionné ou inutilisé. Quels outils fournissent ces constats ?

- [ ] AWS Artifact et AWS Shield
- [ ] Amazon Inspector et Amazon Macie
- [x] AWS Compute Optimizer, AWS Trusted Advisor et S3 Storage Lens
- [ ] AWS Config seul

> Compute Optimizer recommande des tailles d'après l'usage mesuré, Trusted Advisor signale les ressources inactives, Storage Lens montre l'usage du stockage S3.
:::

:::quiz
Une base de données de production tourne sur une instance unique. Les lectures de rapports la ralentissent, et sa panne arrêterait l'activité. L'application ne peut pas être modifiée ce trimestre, hormis des chaînes de connexion. Quelles améliorations apporter ?

- [ ] Une instance deux fois plus grosse
- [ ] Un passage à DynamoDB
- [ ] Une sauvegarde quotidienne supplémentaire
- [x] Le déploiement Multi-AZ pour la disponibilité, et un réplica en lecture vers lequel pointer les rapports

> Le Multi-AZ traite le point de défaillance unique sans changement applicatif ; le réplica sort les rapports de l'instance principale par un simple changement de chaîne de connexion.
:::

:::quiz
Une entreprise analyse sa facture et veut comprendre, service par service et heure par heure, quelles ressources ont généré une hausse soudaine. Quel moyen donne ce niveau de détail ?

- [ ] Le tableau de bord de facturation mensuel
- [ ] AWS Budgets
- [x] Le AWS Cost and Usage Report, interrogé avec Amazon Athena
- [ ] AWS Pricing Calculator

> Le Cost and Usage Report contient le détail le plus fin, par ressource et par heure, et s'interroge en SQL une fois livré dans S3.
:::

:::quiz
Une entreprise prépare la migration de quatre cents serveurs. Avant de planifier, elle doit établir un dossier économique et connaître les dépendances réseau entre serveurs. Quels outils utiliser ?

- [ ] AWS Config pour l'inventaire et AWS Budgets pour le dossier économique
- [x] Migration Evaluator pour le dossier économique, et les agents d'Application Discovery Service pour les dépendances
- [ ] AWS DataSync pour l'inventaire et AWS DMS pour les dépendances
- [ ] Amazon Inspector pour l'ensemble

> Migration Evaluator produit l'analyse de coût ; les agents d'Application Discovery Service relèvent l'utilisation et les connexions entre serveurs, nécessaires au découpage en vagues.
:::

:::quiz
Un centre de données ferme dans cinq mois. Trois cents serveurs applicatifs standards doivent être migrés, sans budget de transformation. Les équipes veulent tester chaque serveur sur AWS avant la bascule, avec une interruption de quelques minutes. Quelle approche ?

- [ ] Recréer chaque serveur à la main et y copier les fichiers
- [ ] Mettre chaque application en conteneur avant de migrer
- [x] Les réhéberger avec AWS Application Migration Service : réplication continue, instances de test, puis bascule
- [ ] Attendre que des équivalents SaaS soient disponibles

> La réplication continue au niveau bloc permet des tests sans toucher à la source et une bascule courte : le réhébergement est le plus sûr quand le délai est serré.
:::

:::quiz
Une base Oracle de 5 To doit migrer vers Aurora PostgreSQL. L'application doit rester en service pendant la migration, avec une interruption finale de moins de trente minutes. Quelle démarche ?

- [ ] Un export complet, puis un import, pendant un week-end d'arrêt
- [ ] AWS DataSync sur les fichiers de la base
- [ ] AWS Application Migration Service sur le serveur Oracle
- [x] AWS SCT pour convertir le schéma, puis AWS DMS en chargement complet suivi d'une réplication continue jusqu'à la bascule

> Le changement de moteur impose une conversion de schéma ; la réplication continue garde la cible à jour, ce qui limite l'interruption à la bascule.
:::

:::quiz
Un site éloigné doit transférer 200 To d'archives vers S3 en six semaines. Sa liaison Internet est de 100 Mbit/s et sert aussi à l'activité quotidienne. Que conclure ?

- [ ] Le transfert en ligne tient sans difficulté
- [ ] S3 Transfer Acceleration suffira à tenir le délai
- [x] Le transfert en ligne ne tient pas dans le délai : il faut un débit supplémentaire ou un transfert hors ligne
- [ ] Il faut d'abord convertir les archives en Parquet

> 200 To font 1 600 000 Gbit ; même à 100 Mbit/s pleins, il faudrait environ 185 jours. L'accélération n'augmente pas le débit de la liaison du site.
:::

:::quiz
Un serveur de fichiers de 30 To continue d'être modifié pendant sa migration vers Amazon EFS. L'équipe veut une copie initiale, puis des passes régulières ne transférant que les changements, avec vérification de l'intégrité. Quel service ?

- [x] AWS DataSync
- [ ] AWS Transfer Family
- [ ] Amazon Data Firehose
- [ ] AWS Storage Gateway, passerelle de bandes

> DataSync copie depuis NFS ou SMB vers EFS ou S3, de façon incrémentale et planifiée, et vérifie l'intégrité des données transférées.
:::

:::quiz
Une migration de cent applications est planifiée en vagues. Deux applications échangent en permanence de gros volumes de données. Comment les traiter dans le plan ?

- [ ] Les placer dans deux vagues éloignées, pour répartir le risque
- [ ] Migrer d'abord la plus grosse, puis l'autre six mois plus tard
- [x] Les migrer dans la même vague, pour éviter que leurs échanges traversent longtemps la liaison entre le site et AWS
- [ ] Les exclure de la migration

> Séparer deux applications très liées fait passer leurs échanges par la liaison hybride, avec latence et coût de transfert. On regroupe par dépendances.
:::

:::quiz
Un monolithe critique, récemment réhébergé sur EC2, doit évoluer plus vite. Une réécriture complète est jugée trop risquée. Quelle trajectoire proposer ?

- [ ] Geler les évolutions et réécrire l'ensemble en dix-huit mois
- [ ] Le laisser tel quel et augmenter la taille des instances
- [x] Placer une façade devant lui et en extraire progressivement les fonctionnalités vers de nouveaux services, en commençant par les plus détachables
- [ ] Le dupliquer dans une seconde région

> L'extraction progressive, derrière une façade, livre de la valeur à chaque étape et reste réversible : c'est le motif de l'étrangleur.
:::

:::quiz
Une application en conteneurs doit être hébergée sur AWS. L'équipe est petite, n'utilise pas Kubernetes et ne veut administrer ni serveurs ni plan de contrôle. Quel choix ?

- [ ] Amazon EKS sur des instances EC2 gérées par l'équipe
- [ ] Docker installé sur des instances EC2
- [x] Amazon ECS avec AWS Fargate
- [ ] Un cluster Kubernetes autogéré

> ECS avec Fargate supprime à la fois la gestion des serveurs et celle d'un orchestrateur Kubernetes, que rien ne justifie ici.
:::

:::quiz
Un système existant échange des messages par un serveur compatible JMS, que l'entreprise ne veut plus administrer. Les applications ne peuvent pas être modifiées avant un an. Que proposer pour la migration ?

- [ ] Amazon SQS, en réécrivant les producteurs et les consommateurs
- [ ] Amazon EventBridge
- [x] Amazon MQ
- [ ] Amazon Kinesis Data Streams

> Amazon MQ est un service géré compatible avec les protocoles des serveurs de messages classiques : la migration ne demande pas de réécrire les applications.
:::

:::quiz
Une base relationnelle d'un outil interne n'est utilisée que quelques heures par semaine, à des moments imprévisibles. Elle tourne aujourd'hui en permanence sur une grosse instance. Quelle cible de modernisation réduit le coût sans gestion manuelle ?

- [ ] Une instance réservée sur trois ans
- [ ] Une base autogérée sur une instance Spot
- [x] Aurora Serverless, dont la capacité suit la charge
- [ ] Un déploiement Multi-AZ avec deux réplicas

> Une capacité qui s'ajuste automatiquement évite de payer une grosse instance en continu pour quelques heures d'usage.
:::

:::quiz
Plusieurs équipes d'une entreprise veulent réagir aux mêmes événements métier (« commande créée », « paiement reçu ») sans se connaître, chacune dans son compte AWS, avec un filtrage selon le contenu des événements. Quel service d'intégration ?

- [ ] Une file SQS partagée
- [ ] Un flux Step Functions par équipe
- [ ] Amazon MQ
- [x] Amazon EventBridge, avec un bus central et des règles par équipe

> EventBridge route les événements selon leur contenu vers des cibles multiples, y compris dans d'autres comptes, sans que producteurs et consommateurs se connaissent.
:::

:::quiz
Après une migration, une application de traitement de dossiers stocke ses fichiers sur un volume EBS attaché à une seule instance, ce qui empêche de la répartir. Les fichiers sont lus et écrits par chemin, et l'application ne peut pas être modifiée. Quel stockage permet de la répartir sur plusieurs zones ?

- [ ] Un volume EBS plus grand
- [ ] Amazon S3, en remplaçant les accès par des appels à l'API
- [x] Amazon EFS, monté par toutes les instances
- [ ] Le stockage d'instance

> EFS offre un système de fichiers partagé, accessible par chemin depuis plusieurs zones, sans modifier l'application. S3 demanderait de réécrire les accès.
:::
