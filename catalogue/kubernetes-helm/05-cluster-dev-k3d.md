---
id: cluster-dev-k3d
title: "Un cluster de dev avec k3d, kubectl et Lens"
summary: "Monter sur ton poste un mini cluster Kubernetes avec les scripts d'infra-dev, y installer Keycloak et comprendre ses limites."
minutes: 45
objectives:
  - Expliquer ce qu'apportent k3d et k3s par rapport à un vrai cluster
  - Lire les scripts startme.sh et install-keycloak.sh ligne à ligne
  - Citer trois différences entre le cluster de dev et la production
---

## À quoi ça sert, et pourquoi ?

Pour apprendre Kubernetes ou tester un chart Helm, tu ne veux pas toucher au vrai cluster, où une erreur peut couper un site utilisé par les associations. Il te faut un **bac à sable** : un cluster jetable, sur ton ordinateur, que tu peux casser et reconstruire en une minute.

Le dépôt [`infra-dev`](https://gitlab.example.org/equipe/dev/infra-dev) de l'équipe fournit exactement cela. Il repose sur deux outils :

- **k3s** : une version allégée de Kubernetes, qui tient dans un seul programme ;
- **k3d** : un outil qui fait tourner k3s **dans des conteneurs Docker**. Pas de machine virtuelle : le README d'`infra-dev` annonce environ 1,1 Gio de mémoire.

Dans ce mini cluster, la même machine joue à la fois le rôle de chef (le *master*, qui décide) et d'ouvrier (le *worker*, qui exécute les pods) : il n'y a qu'un seul nœud.

## Ce qu'il te faut

Le README liste quatre commandes à installer : `docker`, `kubectl`, `helm` et `k3d`. Sur Linux, ton compte doit aussi appartenir au groupe `docker`, sinon chaque commande Docker demande les droits administrateur :

```bash
sudo usermod -aG docker "$(whoami)"
```

- `sudo` exécute la commande avec les droits administrateur ;
- `usermod -aG docker` **ajoute** (`-a`) ton compte au groupe (`-G`) nommé `docker` ;
- `"$(whoami)"` est remplacé par ton nom d'utilisateur.

Il faut ensuite te déconnecter puis te reconnecter pour que le groupe soit pris en compte. Le README recommande aussi **Lens**, une application graphique qui affiche les pods, les services et leurs journaux : pratique pour débuter.

:::warning VPN de l'organisation
Si tu es connecté·e au VPN de l'organisation (un réseau privé virtuel, qui fait passer ta connexion par le réseau de l'école), déconnecte-toi et relance Docker : le README signale des conflits de routes IP.
:::

## Créer le cluster : startme.sh

Le script `startme.sh` vérifie les outils puis crée le cluster. Voici son cœur :

```bash
k3d cluster create --api-port 6550 -p "80:80@loadbalancer"
export KUBECONFIG="$(k3d kubeconfig get k3s-default)"
```

- `k3d cluster create` : crée un cluster, nommé `k3s-default` faute d'autre nom ;
- `--api-port 6550` : l'**API** de Kubernetes (le point d'entrée auquel `kubectl` s'adresse) écoute sur le port 6550 de ta machine ;
- `-p "80:80@loadbalancer"` : relie le port 80 de ta machine au port 80 du **load balancer** du cluster (un répartiteur de trafic). Ton navigateur peut donc atteindre l'Ingress Controller sur `localhost:80` ;
- `KUBECONFIG` : variable d'environnement qui indique à `kubectl` où lire son **fichier de configuration** (adresse du cluster et identifiants). C'est ce fichier que `kubectl` utilise pour savoir à quel cluster il parle.

Lance-le ainsi, puis vérifie :

```bash
sh startme.sh
kubectl get nodes
```

```console
NAME                       STATUS   ROLES                  AGE   VERSION
k3d-k3s-default-server-0   Ready    control-plane,master   1m    v1.xx.x+k3s1
```

(L'âge et la version dépendent de ton installation.) Un seul nœud, `Ready` : le cluster fonctionne. La commande `kubectl get nodes` liste les machines du cluster.

:::info À vérifier selon ta version de k3d
Le script date de l'époque de k3d 3. Selon la version installée, `k3d kubeconfig get` n'affiche plus un chemin de fichier mais le contenu de la configuration ; dans ce cas, l'`export KUBECONFIG` ne fonctionne pas tel quel. Lis `k3d kubeconfig --help` pour ta version. Par ailleurs, le message du script annonce « le port 8081 » alors que la commande utilise le port 80.
:::

## Installer Keycloak : install-keycloak.sh

**Keycloak** est le serveur de connexion des applications de l'équipe : il fournit le **SSO** (*Single Sign-On*, « authentification unique » : une seule connexion donne accès à toutes les applications). Le script l'installe en quatre lignes :

```bash
kubectl create namespace keycloak
kubectl -n keycloak apply -f keycloak-h2pvc.yaml
kubectl -n keycloak create secret generic realm-secret --from-file=mon-realm.json
helm install -n keycloak --values values-keycloak.yaml keycloak-dev codecentric/keycloak
```

1. crée le namespace `keycloak` (le « dossier » du projet, voir la leçon 1) ;
2. réserve un volume (voir la leçon 2) avec le fichier ci-dessous ;
3. crée un Secret nommé `realm-secret` à partir du fichier `mon-realm.json`, qui contient la configuration de départ de Keycloak (le *realm*, c'est-à-dire l'espace des utilisateur·rice·s et des applications) ;
4. installe le chart Helm `keycloak` du dépôt `codecentric` sous le nom de release `keycloak-dev`, avec les valeurs du fichier `values-keycloak.yaml` (leçon 4).

La ligne 4 suppose que le dépôt de charts `codecentric` est déjà connu de ta machine (`helm repo add`), ce que le script ne fait pas.

Le volume demandé par `keycloak-h2pvc.yaml` :

```yaml
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: keycloak-h2db-pvc
  namespace: keycloak
spec:
  accessModes:
    - ReadWriteOnce
  storageClassName: local-path
  resources:
    requests:
      storage: 1Gi
```

On reconnaît le PVC de la leçon 2. `storageClassName: local-path` choisit la « classe de stockage » : ici, un dossier sur le disque du nœud.

Et un extrait de `values-keycloak.yaml` (voir la leçon 4) :

```yaml
ingress:
  enabled: true
  rules:
    - host: sso.172.17.0.1.nip.io
      paths:
        - /
  tls: []
```

Ligne par ligne : `ingress.enabled: true` allume l'Ingress de la leçon 3 ; `rules` liste les règles, avec le nom d'hôte (`host`) et les chemins (`paths`) ; `tls: []` est une liste **vide** : pas de certificat. L'Ingress est donc activé pour le nom `sso.172.17.0.1.nip.io`, sans TLS. Le README indique que Keycloak répond alors sur `http://sso.172.17.0.1.nip.io/auth/` avec un compte administrateur factice de développement. Ces identifiants triviaux sont acceptables **uniquement** sur ton poste.

## Les noms nip.io et l'adresse 172.17.0.1

`nip.io` est un service public qui résout `quelquechose.A.B.C.D.nip.io` vers l'adresse `A.B.C.D`. Ainsi, `sso.172.17.0.1.nip.io` pointe vers `172.17.0.1`, qui est, par défaut, l'adresse de ta machine vue depuis le réseau virtuel de Docker. Les applications dans le cluster et ton navigateur utilisent donc **le même nom**, comme en production.

Si le nom ne se résout pas, le README suspecte une protection « DNS rebind » de ta box Internet, à désactiver pour `nip.io`.

## Dev contre production

| Sujet | Cluster de dev (k3d) | Production |
| --- | --- | --- |
| Nœuds | un seul | plusieurs, d'après le README |
| HTTPS | non, l'Ingress Controller ne gère pas TLS (le chiffrement du HTTPS, leçon 3) | oui, via cert-manager (leçon 3) |
| Volumes `ReadWriteMany` | impossibles | demandés par le chart de Vitrine |
| Ingress Controller | Traefik (fourni par k3s) | HAProxy Ingress, d'après `cluster-configuration` |
| Hébergeur | ton poste | OVH, d'après le README d'`infra-dev` |

:::tip Repartir de zéro
Un cluster de dev se jette : `k3d cluster delete k3s-default` le supprime, puis `sh startme.sh` en recrée un neuf. N'hésite pas.
:::

:::info À confirmer avec l'équipe Infra
Le cluster de dev d'`infra-dev` est-il toujours maintenu (versions de k3d, de Keycloak et du chart `codecentric` actuelles) ? Et le cluster de production est-il toujours chez OVH avec HAProxy ? Les scripts datent de quelques années.
:::

## Entraîne-toi

:::lab
engine: real
intro: |
  **Il n'y a ni Docker, ni k3d, ni cluster dans cet atelier** : impossible de lancer `startme.sh` pour de vrai. Tu travailles sur les fichiers d'`infra-dev` (reconstitués et simplifiés d'après la leçon, avec des valeurs factices) : tu corriges ce qui peut l'être hors ligne, et tu fabriques avec `kubectl ... --dry-run=client` les objets que `install-keycloak.sh` créerait. Rien n'est envoyé à un cluster.
commands:
  - cp /opt/exercices/05-cluster-dev/* .
steps:
  - text: 'Le cluster de dev ne sait pas créer de volume `ReadWriteMany`. Dans `keycloak-h2pvc.yaml`, remplace le mode d''accès par `ReadWriteOnce` (garde `storageClassName: local-path`)'
    hint: 'Le mode est dans la liste `accessModes`. Valide ensuite avec `kubeconform keycloak-h2pvc.yaml`.'
    checks:
      - command-succeeds: 'kubeconform keycloak-h2pvc.yaml'
      - command-succeeds: "verifier-k8s champ --kind PersistentVolumeClaim --chemin 'spec.accessModes' --egal '[ReadWriteOnce]' keycloak-h2pvc.yaml"
      - command-succeeds: "verifier-k8s champ --kind PersistentVolumeClaim --chemin 'spec.storageClassName' --texte local-path keycloak-h2pvc.yaml"
    solution:
      - sed -i 's/ReadWriteMany/ReadWriteOnce/' keycloak-h2pvc.yaml
  - text: 'Le message de `startme.sh` annonce le port 8081, alors que la commande relie le port 80. Corrige le message (`nano startme.sh`) pour qu''il dise `http://localhost`, puis contrôle la syntaxe du script avec `bash -n startme.sh` (qui le lit sans l''exécuter)'
    hint: 'C''est la ligne `echo` à la fin du script. Ne touche pas à la ligne `k3d cluster create`.'
    checks:
      - command-succeeds: 'bash -n startme.sh'
      - output-contains: ['k3d() { echo "K3D $*"; }; . ./startme.sh', 'K3D cluster create .*80:80@loadbalancer']
      - output-contains: ['k3d() { echo "K3D $*"; }; . ./startme.sh', 'http://localhost($|[^:0-9])']
      - command-fails: 'k3d() { echo "K3D $*"; }; . ./startme.sh | grep -q 8081'
    solution:
      - sed -i 's/localhost:8081/localhost/' startme.sh
  - text: 'Première ligne de `install-keycloak.sh` : fabrique le namespace sans cluster avec `kubectl create namespace keycloak --dry-run=client -o yaml > namespace.yaml`'
    hint: 'Le résultat est un objet `Namespace` : vérifie-le avec `kubeconform namespace.yaml`.'
    checks:
      - command-succeeds: 'kubeconform namespace.yaml'
      - command-succeeds: "verifier-k8s champ --kind Namespace --nom keycloak --existe namespace.yaml"
    solution:
      - kubectl create namespace keycloak --dry-run=client -o yaml > namespace.yaml
  - text: 'Troisième ligne du script : fabrique le Secret `realm-secret` à partir du fichier `mon-realm.json`, dans le namespace `keycloak`, avec `kubectl -n keycloak create secret generic realm-secret --from-file=mon-realm.json --dry-run=client -o yaml > realm-secret.yaml`'
    hint: 'Dans `realm-secret.yaml`, une clé `mon-realm.json` contient le fichier encodé en base64 (leçon 2).'
    checks:
      - command-succeeds: 'kubeconform realm-secret.yaml'
      - command-succeeds: "verifier-k8s champ --kind Secret --nom realm-secret --chemin 'metadata.namespace' --texte keycloak realm-secret.yaml"
      - command-succeeds: "verifier-k8s champ --kind Secret --nom realm-secret --chemin 'data[mon-realm.json]' --decode-base64 --fichier mon-realm.json realm-secret.yaml"
    solution:
      - kubectl -n keycloak create secret generic realm-secret --from-file=mon-realm.json --dry-run=client -o yaml > realm-secret.yaml
  - text: 'Dans `values-keycloak.yaml`, mets l''hôte `sso.172.17.0.1.nip.io` (à la place de `sso.exemple.invalid`) et remplace le bloc `tls` par une liste vide : `tls: []` (pas de HTTPS en local)'
    hint: 'Le nom d''hôte apparaît deux fois dans le fichier d''origine ; après ta correction, il ne reste plus de `sso-tls`.'
    checks:
      - command-succeeds: "verifier-k8s champ --chemin 'ingress.enabled' --egal true values-keycloak.yaml"
      - command-succeeds: "verifier-k8s champ --chemin 'ingress.rules[0].host' --texte sso.172.17.0.1.nip.io values-keycloak.yaml"
      - command-succeeds: "verifier-k8s champ --chemin 'ingress.tls' --egal '[]' values-keycloak.yaml"
    solution:
      - write:
          values-keycloak.yaml: |-
            # Extrait de values-keycloak.yaml (leçon 4) : l'Ingress de Keycloak pour le cluster de dev.
            ingress:
              enabled: true
              rules:
                - host: sso.172.17.0.1.nip.io
                  paths:
                    - /
              tls: []
:::

## Vérifie tes acquis

:::quiz
Que fait k3d ?

- [ ] Il remplace Docker par un autre moteur de conteneurs
- [ ] Il installe Kubernetes sur un serveur de production
- [x] Il fait tourner un cluster k3s dans des conteneurs Docker
- [ ] Il génère des charts Helm

> k3d démarre un mini cluster Kubernetes dans Docker, ce qui évite une machine virtuelle.
:::

:::quiz
Dans `-p "80:80@loadbalancer"`, que représente le premier `80` ?

- [ ] Le port de l'API Kubernetes
- [x] Le port de ta machine, que ton navigateur utilise
- [ ] Le nombre de pods
- [ ] Le port de la base de données

> Le port 80 de ta machine est relié au port 80 du load balancer du cluster.
:::

:::quiz
Pourquoi `sso.172.17.0.1.nip.io` fonctionne-t-il sans configurer de DNS ?

- [ ] Parce que Docker crée l'enregistrement DNS
- [ ] Parce que Keycloak l'ajoute au fichier hosts
- [ ] Parce que cert-manager le résout
- [x] Parce que nip.io renvoie l'adresse IP écrite dans le nom

> Le service public nip.io répond `172.17.0.1` pour ce nom.
:::

:::quiz
Laquelle de ces fonctionnalités de production n'existe pas dans le cluster de dev d'`infra-dev` ?

- [ ] Les Ingress
- [ ] Les Secrets
- [x] Le HTTPS sur l'Ingress Controller
- [ ] Les PersistentVolumeClaims

> Le README précise que l'Ingress Controller du cluster de dev ne fait pas de SSL.
:::
