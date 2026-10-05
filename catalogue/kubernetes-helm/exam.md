---
title: "Examen de validation — Kubernetes et Helm"
draw: 5
pass_mark: 80
minutes: 10
shuffle: true
---

Cet examen valide les bases de Kubernetes et de Helm dans l'équipe : objets de base, configuration, Ingress, charts, cluster de dev et diagnostic. Réponds sans te précipiter : une seule réponse est correcte par question.

:::quiz
Quel objet Kubernetes garantit qu'un nombre voulu de pods reste en marche et les remplace en cas de panne ?

- [x] Deployment : il décrit l'état voulu et remplace les pods perdus
- [ ] Service : une adresse
- [ ] Ingress : l'accueil
- [ ] ConfigMap : des réglages

> Le Deployment décrit l'état voulu (image, copies) et le maintient.
:::

:::quiz
Deux conteneurs d'un même pod communiquent entre eux. Pourquoi est-ce possible ?

- [ ] Ils utilisent tous deux la même image Docker
- [ ] Ils portent le même nom dans le fichier YAML
- [ ] Ils sont toujours placés sur des nœuds différents
- [x] Ils partagent le même réseau

> Un pod est une unité de réseau et de volumes partagés ; Vitrine y place Django et nginx.
:::

:::quiz
Un Service a le sélecteur `app: web` mais ses pods portent `app: vitrine-web`. Que constates-tu ?

- [ ] Le Service redémarre les pods pour les aligner
- [ ] Le Service choisit automatiquement le bon label
- [x] Le Service n'a aucun point de terminaison
- [ ] Le pod est supprimé par le cluster

> Sans label correspondant, la liste des points de terminaison est vide.
:::

:::quiz
Que fait la `livenessProbe` quand elle échoue plusieurs fois ?

- [ ] Elle retire le pod du service
- [x] Elle redémarre le conteneur qui ne répond plus
- [ ] Elle supprime le Deployment
- [ ] Elle change la classe de stockage

> La sonde de vie sert à relancer un conteneur bloqué ; celle de disponibilité retire du service.
:::

:::quiz
Quelle différence entre `requests` et `limits` ?

- [ ] `limits` ne concerne que les volumes persistants et non la mémoire ou le processeur
- [x] `requests` est la réserve pour le placement, `limits` le plafond
- [ ] Les deux mots désignent exactement le même réglage dans un conteneur
- [ ] `requests` est le plafond à ne pas dépasser, `limits` la réserve de départ

> La réserve guide le placement ; le plafond limite la consommation.
:::

:::quiz
Où ranges-tu l'URL publique d'un site et où ranges-tu une clé d'API ?

- [x] URL dans un ConfigMap, clé dans un Secret
- [ ] Les deux dans un ConfigMap
- [ ] Les deux dans un Secret
- [ ] URL dans un Secret et clé d'API dans un ConfigMap

> Les valeurs sensibles vont dans un Secret.
:::

:::quiz
Un Secret est affiché en base64 dans `kubectl get secret -o yaml`. Que dois-tu en conclure ?

- [ ] La valeur est chiffrée et peut aller dans Git
- [ ] Elle n'est lisible que par l'administrateur
- [ ] Elle expire au bout de 90 jours
- [x] Elle se décode en une commande : on ne la publie pas dans un dépôt

> Base64 est un encodage, pas un chiffrement.
:::

:::quiz
Tu as modifié un ConfigMap lu avec `envFrom`, mais l'application affiche toujours l'ancienne valeur. Que fais-tu ?

- [ ] Tu supprimes le Service puis tu le recrées
- [ ] Tu changes le type du Service en LoadBalancer
- [x] Tu relances les pods avec `kubectl rollout restart`
- [ ] Tu recrées le PersistentVolumeClaim de l'application

> Les variables sont lues au démarrage du conteneur.
:::

:::quiz
Que perd-on quand un pod est recréé, parmi ces éléments ?

- [ ] Le contenu d'un volume lié à un PVC
- [ ] Les données d'un Secret
- [x] Le contenu d'un `emptyDir`
- [ ] Le contenu d'un ConfigMap

> Un `emptyDir` disparaît avec le pod ; les autres objets existent indépendamment.
:::

:::quiz
Pourquoi le chart de Vitrine demande-t-il `ReadWriteMany` pour ses médias ?

- [ ] Pour chiffrer les fichiers déposés par les associations sur le disque
- [x] Pour que plusieurs nœuds lisent et écrivent le même volume
- [ ] Pour accélérer les écritures des fichiers sur le disque de chaque nœud
- [ ] Parce que le cluster de dev k3d impose ce mode d'accès aux volumes

> Le mode `ReadWriteMany` autorise l'accès depuis plusieurs nœuds ; le cluster de dev ne sait pas le fournir.
:::

:::quiz
Quelle est la différence entre un Ingress et un Ingress Controller ?

- [x] L'Ingress est la règle, le contrôleur l'applique
- [ ] L'Ingress est un programme, le contrôleur un fichier YAML
- [ ] L'Ingress gère les certificats, le contrôleur gère le DNS
- [ ] Il n'y en a aucune : les deux mots désignent le même objet

> Sans contrôleur (HAProxy, Traefik), un Ingress ne route rien.
:::

:::quiz
Quel outil crée automatiquement les enregistrements DNS à partir des Ingress ?

- [ ] cert-manager
- [ ] Terraform
- [ ] Lens
- [x] External-DNS, qui surveille les Ingress et met à jour la zone DNS

> External-DNS surveille le cluster et met à jour la zone DNS.
:::

:::quiz
Que prouve un enregistrement DNS-01 demandé par cert-manager ?

- [ ] Que le pod du site est prêt à recevoir du trafic
- [ ] Que l'image du conteneur est signée par son auteur
- [ ] Que le cluster possède au moins deux nœuds
- [x] Que tu contrôles bien le domaine du certificat

> Let's Encrypt vérifie la maîtrise du domaine via un enregistrement DNS temporaire.
:::

:::quiz
Quel fichier d'un chart Helm contient les valeurs par défaut ?

- [ ] Chart.yaml, la fiche d'identité du chart
- [ ] NOTES.txt, le message affiché après l'installation
- [x] values.yaml
- [ ] _helpers.tpl, les morceaux de modèle réutilisables

> On les surcharge avec `--set` ou `-f`.
:::

:::quiz
Comment voir le YAML qu'un chart produirait, sans rien déployer ?

- [ ] `helm rollback ma-release 1` (retour à la révision 1)
- [x] `helm template ma-release ./chart`
- [ ] `helm uninstall ma-release` (suppression)
- [ ] `kubectl logs mon-pod` (journaux du pod)

> `helm template` rend les modèles localement.
:::

:::quiz
Que fait `helm install ma-release helm/` dans le dossier d'un dépôt ?

- [x] Il installe le chart du dossier, nommé `ma-release`
- [ ] Il publie le chart sur un serveur de charts de l'équipe, accessible à tous
- [ ] Il supprime l'ancienne release du même nom, puis la recrée
- [ ] Il ouvre un terminal dans l'un des pods de la release

> L'équipe n'a pas de dépôt de charts : on installe depuis le dossier cloné.
:::

:::quiz
Dans le chart de Vitrine, que se passe-t-il à chaque `helm upgrade` pour `SECRET_KEY` ?

- [x] Une nouvelle valeur est tirée avec `randAlphaNum`
- [ ] Elle reste strictement identique d'une révision à l'autre
- [ ] Elle est relue depuis le serveur de connexion Keycloak
- [ ] Elle est supprimée puis recréée à la main

> La fonction de tirage aléatoire est évaluée à chaque rendu, ce qui peut invalider des sessions.
:::

:::quiz
Que signifie la valeur `nil` dans `secret.adhesionClientSecret` du `values.yaml` ?

- [ ] Le secret est chiffré par Helm au moment du rendu
- [ ] Le secret est relu depuis le cluster à chaque rendu
- [ ] La valeur est facultative et sans aucune conséquence
- [x] Un emplacement à remplir au déploiement

> Les vraies valeurs sont fournies au moment du déploiement.
:::

:::quiz
Que fait `k3d cluster create --api-port 6550 -p "80:80@loadbalancer"` ?

- [ ] Il supprime le cluster `k3s-default` ainsi que tous ses volumes de données
- [ ] Il installe Keycloak dans le namespace `keycloak` avec un volume local
- [x] Il crée un cluster : API sur le port 6550, port 80 relié au load balancer
- [ ] Il ouvre le port 6550 de ta machine sur Internet, pour tout le monde

> C'est la ligne centrale de `startme.sh`.
:::

:::quiz
Quel nom de domaine fonctionne sans configurer de DNS sur le cluster de dev ?

- [ ] sso.local
- [x] sso.172.17.0.1.nip.io, car nip.io renvoie l'adresse écrite dans le nom
- [ ] sso.k3d
- [ ] sso.exemple

> nip.io renvoie l'adresse écrite dans le nom.
:::

:::quiz
Quelle fonctionnalité de la production est absente du cluster de dev d'infra-dev ?

- [ ] Les Secrets Kubernetes, qui existent aussi en développement
- [x] Le HTTPS (TLS) sur l'Ingress Controller
- [ ] Les Services de type ClusterIP, créés comme en production
- [ ] Les ConfigMaps, utilisés pour la configuration non sensible

> Le README précise que le contrôleur du cluster de dev ne fait pas de TLS.
:::

:::quiz
Pourquoi lire `cluster-configuration` avec prudence avant de le réutiliser ?

- [x] Il date de 2019 environ (Tiller, charts `stable/`, cert-manager 0.6)
- [ ] Il est très récent et ses outils ne sont pas encore stables
- [ ] Il ne contient aucun fichier YAML, seulement des scripts
- [ ] Il n'est consultable qu'avec un compte de l'organisation

> Plusieurs éléments sont probablement obsolètes : à confirmer avec l'équipe Infra.
:::

:::quiz
Un pod est `Pending` et `describe` indique « unbound PersistentVolumeClaims ». Quelle est la cause la plus probable ?

- [ ] L'image Docker est trop volumineuse pour le nœud
- [ ] Le label du service est mal écrit dans le selector
- [ ] La sonde de disponibilité échoue au démarrage
- [x] Le volume demandé n'est pas fourni

> Le pod attend son PVC : vérifie la classe de stockage et le mode d'accès.
:::

:::quiz
Un conteneur est en `CrashLoopBackOff`. Quelle commande lit l'erreur de l'exécution qui a planté ?

- [ ] `kubectl get services --all-namespaces`
- [ ] `helm lint ./mini-portail --strict`
- [x] `kubectl logs <pod> --previous`
- [ ] `kubectl cordon k3d-k3s-default-server-0`

> `--previous` donne les journaux de l'exécution précédente.
:::

:::quiz
Comment tester qu'un service répond, sans passer par l'Ingress ?

- [ ] `helm template ma-release ./mini-portail --set replicas=2`
- [ ] `kubectl apply -f service.yaml --dry-run=client`
- [x] `kubectl port-forward service/NOM 8080:80`
- [ ] `kubectl rollout restart deployment/vitrine-web`

> Si la page s'affiche en local, cherche le problème côté Ingress ou DNS.
:::

:::quiz
Une nouvelle release Helm se comporte moins bien que la précédente. Que fais-tu ?

- [ ] Tu supprimes le namespace tout de suite, puis tu réinstalles
- [x] Tu lances `helm rollback ma-release <révision>`
- [ ] Tu modifies le Dockerfile de l'application et tu reconstruis tout
- [ ] Tu relances Docker sur ton poste pour repartir d'un état propre

> `helm history` liste les révisions, `rollback` rétablit l'une d'elles.
:::

:::quiz
Tu lances `helm template demo ./chart -f prod.yaml`, mais le résultat garde `replicas: 1` alors que `prod.yaml` contient `replica: 3`. Que se passe-t-il ?

- [x] La clé `replica` est ignorée sans erreur : aucun modèle ne la lit
- [ ] Helm refuse le fichier et affiche une erreur de syntaxe avant tout rendu
- [ ] Le cluster a plafonné le nombre de copies à une seule par application
- [ ] Le fichier `prod.yaml` n'est lu qu'au moment du `helm install`

> Une clé inconnue n'est pas une erreur : seul le rendu permet de le constater.
:::

:::quiz
Que contrôle `kubeconform fichier.yaml` ?

- [ ] Que l'application dans l'image démarre sans erreur au lancement
- [ ] Que le service trouve bien ses pods une fois le fichier appliqué
- [ ] Que le certificat du site n'est pas expiré depuis plus de 90 jours
- [x] Que le fichier respecte le schéma officiel de Kubernetes

> C'est un contrôle hors ligne : il dit que Kubernetes acceptera le fichier, pas que l'application marchera.
:::

:::quiz
Un pod est `Running` mais `0/1`. L'évènement indique « Readiness probe failed ... connection refused » sur le port 80, alors que l'application écoute sur 8000. Que corriges-tu ?

- [ ] L'étiquette de l'image Docker du conteneur web
- [ ] La classe de stockage du volume des médias
- [ ] Le mode d'accès du PVC des fichiers statiques
- [x] Le port interrogé par la `readinessProbe`

> La sonde interroge un port où rien n'écoute : le pod est donc retiré du service.
:::
