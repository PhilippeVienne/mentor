---
id: diagnostiquer-un-pod
titre: "Diagnostiquer un pod qui ne démarre pas"
resume: "Lire l'état d'un pod, puis enquêter avec describe, logs et les évènements pour trouver la cause."
duree: 45
objectifs:
  - Interpréter les états Pending, ImagePullBackOff, CrashLoopBackOff et Running 0/1
  - Enchaîner get, describe et logs dans le bon ordre
  - Relier un symptôme à une cause vue dans les leçons précédentes
---

## À quoi ça sert, et pourquoi ?

Tu as déployé ton application et… rien ne s'affiche. C'est le quotidien de toute personne qui utilise Kubernetes. Comme un médecin qui prend d'abord la température avant de prescrire, il faut une **méthode** : regarder l'état, lire les évènements, puis lire les journaux. Cette leçon la déroule, avec les pannes que tu rencontreras réellement. Dans l'atelier, **il n'y a pas de cluster, donc pas de vrai pod en panne** : tu travailles sur des sorties de `kubectl get`, `describe` et `logs` **enregistrées dans des fichiers**, comme celles qu'un collègue te collerait dans un message. Elles sont inventées pour la formation (inspirées des applications de l'équipe), pas copiées d'un incident réel.

Deux mots à connaître : un **évènement** est un message que Kubernetes écrit quand il fait ou ne parvient pas à faire quelque chose (« image introuvable », « volume non disponible ») ; un **journal** (*log*) est ce que le programme dans le conteneur écrit lui-même.

## Étape 1 : regarder l'état

```bash
kubectl get pods
```

```console
NAME                          READY   STATUS             RESTARTS   AGE
vitrine-web-6d8f7c9b5-abcde 1/2     Running            0          3m
adhesion-backend-7f9c-xyz12   0/2     CrashLoopBackOff   5          4m
keycloak-dev-0                0/1     Pending            0          6m
mediafiles-test-5c4d-qwert    0/1     ImagePullBackOff   0          2m
```

Les colonnes :

- `READY` : « conteneurs prêts / conteneurs du pod ». `1/2` signifie qu'un seul des deux conteneurs répond à sa sonde de disponibilité ;
- `STATUS` : l'état résumé du pod ;
- `RESTARTS` : combien de fois Kubernetes a relancé un conteneur.

## Étape 2 : comprendre l'état

| État | Ce que ça veut dire | Pistes (leçons) |
| --- | --- | --- |
| `Pending` | Le pod n'est placé sur aucun nœud, ou son volume n'est pas prêt | PVC en attente, par exemple un `ReadWriteMany` demandé sur le cluster de dev (2 et 5) ; ressources `requests` trop grosses (1) |
| `ImagePullBackOff` | L'image Docker ne peut pas être téléchargée | nom ou étiquette (*tag*) de l'image faux ; registry privé (1 et 4) |
| `CrashLoopBackOff` | Le conteneur démarre, plante, est relancé, replante… avec des pauses de plus en plus longues | erreur dans l'application, variable ou secret manquant (2) |
| `Running` mais `0/1` | Le conteneur tourne, mais sa sonde de disponibilité échoue | application encore en démarrage, mauvais port ou chemin de sonde (1) |

Le `BackOff` signifie « on attend de plus en plus longtemps avant de réessayer ».

## Étape 3 : lire les évènements avec describe

```bash
kubectl describe pod keycloak-dev-0 -n keycloak
```

`describe pod` suivi du nom du pod affiche sa fiche détaillée, et `-n keycloak` précise son namespace (leçon 1). Cette commande affiche beaucoup de texte. **Va directement tout en bas**, à la section `Events` :

```console
Events:
  Type     Reason            Message
  ----     ------            -------
  Warning  FailedScheduling  0/1 nodes are available: pod has unbound immediate PersistentVolumeClaims.
```

« Unbound PersistentVolumeClaim » : le pod attend un volume que le cluster n'a pas pu fournir. La cause est donc dans la leçon 2 : vérifie la classe de stockage et le mode d'accès du PVC avec `kubectl get pvc -n keycloak`.

## Étape 4 : lire les journaux avec logs

Si le conteneur a démarré, son programme a probablement écrit la raison de son échec :

```bash
kubectl logs adhesion-backend-7f9c-xyz12 -c backend
kubectl logs adhesion-backend-7f9c-xyz12 -c backend --previous
```

- `-c backend` : le pod d'Adhésion contient plusieurs conteneurs ; `-c` choisit le bon (leçon 1). Sans lui, `kubectl` te demande de préciser ;
- `--previous` : affiche les journaux de l'exécution **précédente**, celle qui a planté. Indispensable en `CrashLoopBackOff`, car le conteneur actuel vient de redémarrer et n'a encore rien écrit.

```console
django.db.utils.OperationalError: could not translate host name "adhesion-db" to address
```

Le programme ne trouve pas sa base de données : le nom d'hôte est faux dans la configuration, ou le service de la base n'existe pas.

## Étape 5 : entrer ou tester de l'extérieur

```bash
kubectl exec -it vitrine-web-6d8f7c9b5-abcde -c web -- sh
kubectl port-forward service/vitrine-nginx-svc 8080:80
```

- `exec -it … -- sh` ouvre un terminal **dans** le conteneur (`-it` : interactif, avec un terminal). Pratique pour vérifier une variable ou un fichier ;
- `port-forward` relie le port 8080 de **ta machine** au port 80 du service, sans passer par l'Ingress. Ouvre `http://localhost:8080` : si la page s'affiche, le problème est à chercher du côté de l'Ingress ou du DNS (leçon 3), pas du pod.

## Corriger, puis revenir en arrière

Après avoir changé une valeur dans un chart, mets à jour la release (leçon 4). Si la nouvelle version est pire que l'ancienne :

```bash
helm history ma-release
helm rollback ma-release 1
```

:::warning Patience au démarrage
Les sondes de Vitrine attendent 60 secondes (disponibilité) et 120 secondes (vie) avant leur premier test. Le README d'`infra-dev` prévient qu'il y a des erreurs 503 tant que les applications ne sont pas lancées. Un `0/2` pendant une minute n'est pas forcément une panne : relance `kubectl get pods` avant de t'inquiéter.
:::

:::tip L'ordre qui évite de perdre du temps
`get pods` (quel état ?), puis `describe` (que dit Kubernetes ?), puis `logs` (que dit l'application ?). Ne supprime jamais un pod « pour voir » avant d'avoir lu ses évènements et ses journaux : tu perdrais les indices.
:::

:::info À confirmer avec l'équipe Infra
Quels outils de supervision et d'alerte existent sur le cluster de production (au-delà de `kubectl`), et qui prévenir en cas d'incident ? Les dépôts consultés ne le décrivent pas.
:::

## Entraîne-toi

:::labo
moteur: reel
intro: |
  Cinq fichiers d'`etat/` contiennent des sorties **enregistrées** (`get pods`, `describe`, `logs`), et `deployment-web.yaml` le déploiement du pod `vitrine-web`. **Il n'y a pas de cluster** : tu ne peux ni relancer une commande ni supprimer un pod, tu ne fais que lire et raisonner, puis corriger un fichier. Pour chaque pod en difficulté, tu écriras dans `diagnostic.txt` une ligne `nom-du-pod: cause`, où la cause est l'un de ces mots : `volume`, `image`, `application` ou `sonde`.
commandes:
  - cp -R /opt/exercices/06-diagnostic/. .
etapes:
  - texte: 'Lis `etat/get-pods.txt` (`cat etat/get-pods.txt`) et écris dans `a-voir.txt`, un par ligne, les noms des pods qui ne sont **pas** pleinement prêts (colonne `READY` incomplète). Le pod en bonne santé ne doit pas y figurer'
    indice: 'Un pod est prêt quand les deux nombres de `READY` sont égaux (`2/2`). Tu peux filtrer avec `grep -v` puis `awk ''{print $1}''` pour ne garder que la première colonne.'
    verif:
      - commande-reussit: 'verifier-k8s lignes a-voir.txt --exactement vitrine-web-6d8f7c9b5-abcde adhesion-backend-7f9c-xyz12 keycloak-dev-0 mediafiles-test-5c4d-qwert'
    solution:
      - "grep -v -e NAME -e ' 2/2 ' etat/get-pods.txt | awk '{print $1}' > a-voir.txt"
  - texte: 'Le pod `keycloak-dev-0` est `Pending`. Lis `etat/describe-keycloak-dev-0.txt` jusqu''à la section `Events`, puis ajoute à `diagnostic.txt` la ligne `keycloak-dev-0: volume` ou la cause qui te semble juste (par exemple avec `echo ''keycloak-dev-0: …'' >> diagnostic.txt`)'
    indice: 'Le message parle d''un PersistentVolumeClaim non lié (« unbound »). Reprends la leçon 2.'
    verif:
      - commande-reussit: "verifier-k8s paires diagnostic.txt --cle keycloak-dev-0 --valeur volume"
    solution:
      - "echo 'keycloak-dev-0: volume' >> diagnostic.txt"
  - texte: 'Le pod `mediafiles-test-5c4d-qwert` est en `ImagePullBackOff`. Lis `etat/describe-mediafiles-test.txt`, repère l''image demandée et ajoute à `diagnostic.txt` la ligne `mediafiles-test-5c4d-qwert: …` avec la bonne cause'
    indice: 'Regarde l''étiquette de l''image, tout à la fin de son nom : y a-t-il une faute de frappe (un `O` majuscule à la place d''un zéro) ?'
    verif:
      - commande-reussit: "verifier-k8s paires diagnostic.txt --cle mediafiles-test-5c4d-qwert --valeur image"
    solution:
      - "echo 'mediafiles-test-5c4d-qwert: image' >> diagnostic.txt"
  - texte: 'Le pod `adhesion-backend-7f9c-xyz12` est en `CrashLoopBackOff`. Le `describe` ne dit rien d''utile : lis `etat/logs-adhesion-backend-previous.txt` (les journaux de l''exécution précédente) et ajoute à `diagnostic.txt` la ligne `adhesion-backend-7f9c-xyz12: …`'
    indice: 'Le programme lui-même explique pourquoi il s''arrête : il ne trouve pas le nom d''hôte de sa base de données. C''est une erreur de l''application ou de sa configuration.'
    verif:
      - commande-reussit: "verifier-k8s paires diagnostic.txt --cle adhesion-backend-7f9c-xyz12 --valeur application"
    solution:
      - "echo 'adhesion-backend-7f9c-xyz12: application' >> diagnostic.txt"
  - texte: 'Le pod `vitrine-web-6d8f7c9b5-abcde` est `Running` mais `1/2`. Lis `etat/describe-vitrine-web.txt` et ajoute à `diagnostic.txt` la ligne `vitrine-web-6d8f7c9b5-abcde: …`'
    indice: 'Compare le port de la ligne `Readiness:` avec le port sur lequel écoute le conteneur `web` (ligne `Port:`).'
    verif:
      - commande-reussit: "verifier-k8s paires diagnostic.txt --cle vitrine-web-6d8f7c9b5-abcde --valeur sonde"
    solution:
      - "echo 'vitrine-web-6d8f7c9b5-abcde: sonde' >> diagnostic.txt"
  - texte: 'Corrige la cause du dernier pod dans `deployment-web.yaml` : la `readinessProbe` doit interroger le port `8000`, celui de Django. Valide avec `kubeconform deployment-web.yaml`. (Dans un vrai cluster, tu ferais ensuite `kubectl apply -f`, ce que cet atelier ne permet pas.)'
    indice: 'Dans la section `readinessProbe`, `httpGet.port` vaut `80`. Ne change pas `containerPort`, qui est déjà juste.'
    apres: [5]
    verif:
      - commande-reussit: 'kubeconform deployment-web.yaml'
      - commande-reussit: "verifier-k8s champ --kind Deployment --nom vitrine-web --chemin 'spec.template.spec.containers[0].readinessProbe.httpGet.port' --egal 8000 deployment-web.yaml"
      - commande-reussit: "verifier-k8s champ --kind Deployment --nom vitrine-web --chemin 'spec.template.spec.containers[0].ports[0].containerPort' --egal 8000 deployment-web.yaml"
    solution:
      - sed -i 's/^\( *port:\) 80$/\1 8000/' deployment-web.yaml
:::

## Vérifie tes acquis

:::quiz
Un pod est en `ImagePullBackOff`. Que cherches-tu en premier ?

- [ ] Une erreur dans les journaux de l'application
- [ ] Un volume non monté
- [x] Le nom, l'étiquette ou l'accès au registry de l'image
- [ ] Une sonde de vie trop stricte

> Le pod n'a même pas démarré : l'image n'a pas pu être téléchargée.
:::

:::quiz
Un pod est en `CrashLoopBackOff`. Quelle commande lit la cause de la dernière panne ?

- [ ] `kubectl get pods --previous`
- [ ] `kubectl port-forward`
- [x] `kubectl logs <pod> --previous`
- [ ] `helm template`

> `--previous` montre les journaux du conteneur qui vient de planter.
:::

:::quiz
`kubectl get pods` affiche `0/1 Pending`. Où trouves-tu la raison ?

- [ ] Dans `kubectl logs`, car le programme est lancé
- [x] Dans la section `Events` de `kubectl describe pod`
- [ ] Dans le fichier `values.yaml`
- [ ] Dans `helm history`

> Un pod en attente n'a pas de journal : les raisons sont dans les évènements.
:::

:::quiz
Un pod est `Running` mais `0/1`. Que sais-tu ?

- [ ] Que l'image est introuvable
- [ ] Que le cluster est saturé
- [ ] Que le volume n'est pas créé
- [x] Que le conteneur tourne mais que sa sonde de disponibilité échoue

> Tant que la sonde échoue, le pod est retiré du service.
:::
