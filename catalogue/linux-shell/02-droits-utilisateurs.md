---
id: droits-utilisateurs
title: "Droits, utilisateurs et groupes"
summary: "Lire les droits d'un fichier, les modifier avec chmod et comprendre le rôle de root."
minutes: 25
objectives:
  - Lire la ligne de droits renvoyée par `ls -l`
  - Modifier des droits avec `chmod` (notation symbolique et octale)
  - Expliquer ce que change `sudo` et pourquoi l'éviter par défaut
---

« Permission denied ». Tu l'as sûrement déjà lu en essayant de lancer un script ou d'écrire dans un dossier. Ce message n'est pas un bug : c'est Linux qui protège le système en vérifiant, pour chaque action, qui a le droit de la faire.

## Qui peut faire quoi

Rappel de vocabulaire : un **fichier** contient des données, un **dossier** range des fichiers, et `ls -l` (l'option `-l`, *long*, vue dans la leçon précédente) affiche une ligne de détails par fichier. Un **utilisateur** (ou compte) est une identité sur la machine ; un **groupe** rassemble plusieurs utilisateurs, par exemple toute l'équipe `infra`.

Chaque fichier appartient à **une personne** (propriétaire) et **un groupe**. Trois droits existent, pour trois catégories de personnes.

```console
$ ls -l deploy.sh
-rwxr-x--- 1 alice infra 412 oct.  3 10:12 deploy.sh
```

| Partie | Signification |
| --- | --- |
| `-` | Type : `-` fichier, `d` dossier |
| `rwx` | Droits du **propriétaire** (`alice`) |
| `r-x` | Droits du **groupe** (`infra`) |
| `---` | Droits des **autres** |

Les lettres : `r` lire, `w` écrire, `x` exécuter (pour un dossier : le traverser). Ici, `alice` peut tout faire, le groupe `infra` peut lire et exécuter, les autres rien du tout.

## Changer les droits avec chmod

```bash
chmod u+x deploy.sh
chmod g-w rapport.txt
chmod 640 secret.env
```

1. `u+x` ajoute (`+`) l'exécution (`x`) au propriétaire (`u`). On trouve aussi `g` (groupe), `o` (autres) et `a` (tous).
2. `g-w` retire l'écriture au groupe.
3. `640` est la notation **octale** : chaque chiffre additionne `r`=4, `w`=2, `x`=1, dans l'ordre propriétaire, groupe, autres. Donc `6` = lecture + écriture, `4` = lecture seule, `0` = rien.

Pour changer le propriétaire ou le groupe, on utilise `chown alice:infra fichier` (en général réservé à `root`).

## Utilisateurs, groupes et root

```bash
whoami
id
sudo systemctl restart nginx
```

- `whoami` (« qui suis-je ») affiche ton nom d'utilisateur·rice, `id` ajoute ton identifiant et tes groupes.
- `root` est le compte administrateur, qui ignore presque toutes les restrictions.
- `sudo commande` exécute **une seule** commande avec les droits de `root`, si ton compte y est autorisé.

:::warning Ne travaille pas en root
Se connecter directement en `root` ou préfixer chaque commande par `sudo` supprime les garde-fous : une faute de frappe devient une catastrophe. Travaille avec ton compte, et n'élève tes droits que pour la commande qui l'exige.
:::

:::tip Pas de chmod 777
Quand un accès est refusé, la tentation est de tout ouvrir avec `chmod 777`. C'est presque toujours une mauvaise idée : n'importe qui sur la machine peut alors modifier le fichier. Cherche plutôt à qui il manque quel droit, avec `ls -l`.
:::

:::info Et dans le labo ?
Ton terminal du labo n'a volontairement **pas** `sudo` : tu travailles avec un compte ordinaire, comme tu devrais le faire presque tout le temps. Tout ce qui suit se fait sur tes propres fichiers, sans élévation de droits.
:::

## Entraîne-toi

:::lab
engine: real
intro: |
  Dans ton dossier de travail, tu trouves trois fichiers : `deploy.sh` (un script de déploiement simulé), `secret.env` (un fichier qui contient un faux mot de passe d'exemple) et `rapport.txt`. Observe leurs droits avec `ls -l`, puis corrige-les. Les droits de départ sont `rw-r--r--` (644) pour chacun.
commands:
  - cp -R /opt/exercices/02-droits/. .
steps:
  - text: 'Rends `deploy.sh` exécutable pour toi (son propriétaire)'
    hint: 'Essaie d''abord `./deploy.sh` : tu obtiens « Permission denied ». Puis `chmod u+x deploy.sh` ajoute le droit d''exécution au propriétaire.'
    checks:
      - output-contains: ['stat -c %A deploy.sh', '^-..x']
    solution:
      - chmod u+x deploy.sh
  - text: 'Lance `./deploy.sh` : il doit créer le fichier `deploiement.txt`'
    hint: 'Le `./` devant le nom dit au shell « le script qui est dans le dossier courant ».'
    after: [1]
    checks:
      - command-succeeds: 'T=$(sed -n "s/^Déploiement simulé terminé le \\([0-9]*\\) (empreinte .*/\\1/p" deploiement.txt) && test -n "$T" && grep -q "(empreinte $(printf "%s demo-deploy" "$T" | sha256sum | cut -c1-12))" deploiement.txt'
    solution:
      - ./deploy.sh
  - text: '`secret.env` est lisible par tout le monde, ce qui n''est pas acceptable : ne laisse des droits qu''à son propriétaire (lecture et écriture, `rw-------`)'
    hint: 'En notation octale : 6 (rw-) pour le propriétaire, 0 pour le groupe et 0 pour les autres. Contrôle avec `ls -l secret.env`.'
    checks:
      - output-contains: ['stat -c %a secret.env', '^600$']
      - command-succeeds: 'cmp -s secret.env /opt/exercices/02-droits/secret.env'
    solution:
      - chmod 600 secret.env
  - text: 'Mets `rapport.txt` en lecture seule pour tout le monde, toi compris (droits `r--r--r--`, soit 444)'
    hint: 'Soit `chmod 444 rapport.txt`, soit `chmod a-w rapport.txt` (retire l''écriture, `w`, pour tous : `a`).'
    checks:
      - output-contains: ['stat -c %a rapport.txt', '^444$']
      - command-succeeds: 'cmp -s rapport.txt /opt/exercices/02-droits/rapport.txt'
    solution:
      - chmod a-w rapport.txt
  - text: 'Crée un dossier `partage` dont toi seul·e peux tout faire, que le groupe peut lire et traverser, et dont les autres ne peuvent rien faire (`rwxr-x---`, soit 750)'
    hint: '`mkdir partage` puis `chmod 750 partage` : 7 = 4+2+1, 5 = 4+1, 0 = rien.'
    checks:
      - output-contains: ['stat -c %a partage', '^750$']
      - command-succeeds: 'test -d partage'
    solution:
      - mkdir partage
      - chmod 750 partage
:::

## Vérifie tes acquis

:::quiz
Que signifie `-rw-r-----` pour un fichier ?

- [ ] Tout le monde peut lire et écrire
- [ ] Seul le propriétaire peut le lire
- [x] Le propriétaire lit et écrit, le groupe lit, les autres n'ont aucun droit

> Les neuf caractères se lisent par trois : `rw-` (propriétaire), `r--` (groupe), `---` (autres).
:::

:::quiz
Quelle commande rend `deploy.sh` exécutable pour son propriétaire ?

- [x] `chmod u+x deploy.sh`
- [ ] `chown +x deploy.sh`
- [ ] `chmod 000 deploy.sh`

> `chmod` modifie les droits, `chown` le propriétaire. `u+x` ajoute l'exécution au propriétaire.
:::

:::quiz
Que représente le `7` de `chmod 750 fichier` ?

- [ ] Lecture seule pour le propriétaire
- [ ] Les droits du groupe
- [x] Lecture, écriture et exécution (4 + 2 + 1) pour le propriétaire

> Le premier chiffre vise le propriétaire, le deuxième le groupe, le troisième les autres. `5` = lecture + exécution, `0` = rien.
:::

:::quiz
À quoi sert `sudo` devant une commande ?

- [ ] À la lancer en arrière-plan
- [x] À l'exécuter avec les droits de `root`, pour cette commande seulement
- [ ] À changer définitivement ton mot de passe

> `sudo` élève tes droits le temps d'une commande, si ton compte y est autorisé.
:::
