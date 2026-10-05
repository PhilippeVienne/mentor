---
id: naviguer-fichiers
title: "Naviguer et manipuler des fichiers"
summary: "Se repérer dans l'arborescence, créer, copier, déplacer et supprimer sans se perdre."
minutes: 30
objectives:
  - Te repérer avec `pwd`, `ls` et `cd`
  - Créer, copier, déplacer et supprimer des fichiers et des dossiers
  - Distinguer un chemin absolu d'un chemin relatif
---

Tu te connectes à un serveur et tu n'as devant toi qu'un curseur clignotant : pas de fenêtre, pas de souris. Pas de panique ! Le terminal fait toujours la même chose : il t'attend dans un dossier, et tu lui demandes d'agir sur des fichiers.

## Une seule arborescence

Une **arborescence** est l'organisation des dossiers en arbre : des dossiers qui contiennent des dossiers. Sous Linux, tout part d'une racine unique, `/`. Il n'y a pas de lecteurs `C:` ou `D:` : tout est rangé dans des dossiers, y compris les disques et les périphériques.

```text
/
├── etc/      ← configuration du système et des services
├── home/     ← dossiers personnels (/home/alice)
├── var/      ← données qui changent (journaux dans /var/log)
├── tmp/      ← fichiers temporaires
└── usr/      ← programmes installés
```

Un **chemin absolu** commence par `/` (`/etc/hosts`) et désigne toujours le même endroit. Un **chemin relatif** part du dossier courant (`notes/todo.txt`). Deux raccourcis : `.` est le dossier courant, `..` son parent, et `~` ton dossier personnel.

## Se repérer et se déplacer

```bash
pwd
ls -la
cd /var/log
cd ..
cd ~
```

- `pwd` affiche le dossier courant.
- `ls -la` liste le contenu, y compris les fichiers cachés (dont le nom commence par un point), avec taille et date.
- `cd /var/log` te déplace vers un chemin absolu : l'invite change.
- `cd ..` remonte d'un niveau, `cd ~` te ramène chez toi (`cd` seul fait pareil).

## Créer, copier, déplacer, supprimer

```bash
mkdir -p projet/notes
touch projet/notes/todo.txt
cp projet/notes/todo.txt projet/notes/todo.bak
mv projet/notes/todo.bak /tmp/
rm projet/notes/todo.txt
rm -r projet
```

1. `mkdir -p` crée le dossier et ses parents manquants.
2. `touch` crée un fichier vide (ou met à jour sa date).
3. `cp source destination` copie : ajoute `-r` pour un dossier.
4. `mv` déplace, et sert aussi à **renommer** : `mv ancien nouveau`.
5. `rm` supprime un fichier, `rm -r` un dossier et tout ce qu'il contient.

:::danger Pas de corbeille
`rm` supprime **définitivement** : rien ne va dans une corbeille. Relis ta commande avant d'appuyer sur Entrée, et ne lance jamais `rm -rf` avec un chemin que tu n'as pas vérifié avec `ls` juste avant.
:::

## Lire un fichier

```bash
cat /etc/hostname
less /var/log/syslog
head -n 5 fichier.txt
tail -n 20 fichier.txt
```

`cat` affiche tout le fichier. `less` le fait défiler (quitte avec `q`). `head` et `tail` montrent le début et la fin : `tail -f` suit un fichier qui grossit, très utile pour un journal.

:::tip La touche Tab
Appuie sur Tab pour compléter un nom de fichier ou de commande, et deux fois pour voir les choix possibles. Tu tapes moins et tu fais moins de fautes de frappe. La flèche haut rappelle les commandes précédentes.
:::

## Entraîne-toi

:::lab
engine: real
intro: |
  Ton terminal est ouvert dans le dossier de travail `/workspace`. Il contient un dossier d'entraînement, `projet-demo`, qui ressemble à un petit projet de l'équipe : des fichiers de configuration, des journaux et des brouillons. Tape tes commandes dans le terminal ; chaque étape se valide toute seule quand le résultat est bon. Astuce : `ls` te permet de voir où tu en es.
commands:
  - cp -R /opt/exercices/01-navigation/. .
steps:
  - text: 'Explore `projet-demo` avec `ls -la projet-demo/config`, puis lis le fichier caché qui s''y trouve avec `cat`. Il contient une ligne `ENVIRONNEMENT=...` : crée dans `projet-demo` un dossier `archives`, puis dans `archives` un dossier qui porte le nom de cet environnement'
    hint: 'Le fichier caché s''appelle `.env.exemple` (son nom commence par un point : `ls -la` le montre, pas `ls`). Crée ensuite `projet-demo/archives/NOM` avec `mkdir -p`.'
    checks:
      - command-succeeds: 'test -d "projet-demo/archives/$(sed -n "s/^ENVIRONNEMENT=//p" /opt/exercices/01-navigation/projet-demo/config/.env.exemple)"'
    solution:
      - ls -la projet-demo/config
      - cat projet-demo/config/.env.exemple
      - mkdir -p projet-demo/archives/recette
  - text: 'Copie `projet-demo/journaux/ancien.log` dans `projet-demo/archives/` : l''original doit rester à sa place'
    hint: '`cp source destination`. Si la destination est un dossier, le fichier garde son nom.'
    checks:
      - command-succeeds: 'cmp -s projet-demo/archives/ancien.log /opt/exercices/01-navigation/projet-demo/journaux/ancien.log'
      - command-succeeds: 'cmp -s projet-demo/journaux/ancien.log /opt/exercices/01-navigation/projet-demo/journaux/ancien.log'
    solution:
      - cp projet-demo/journaux/ancien.log projet-demo/archives/
  - text: 'Déplace `projet-demo/brouillons/idee.txt` dans `projet-demo/archives/` en le renommant `idee-2025.txt`'
    hint: '`mv` déplace et renomme en une seule commande : `mv ancien-chemin nouveau-chemin`.'
    checks:
      - command-succeeds: 'cmp -s projet-demo/archives/idee-2025.txt /opt/exercices/01-navigation/projet-demo/brouillons/idee.txt'
      - env-file-absent: projet-demo/brouillons/idee.txt
    solution:
      - mv projet-demo/brouillons/idee.txt projet-demo/archives/idee-2025.txt
  - text: 'Crée un fichier vide `projet-demo/notes.txt`, puis supprime le dossier `projet-demo/brouillons` avec tout ce qu''il contient'
    hint: '`touch` crée le fichier ; pour un dossier, il faut `rm -r`. Fais un `ls projet-demo/brouillons` avant de supprimer : il n''y a pas de corbeille.'
    checks:
      - command-succeeds: 'test -f projet-demo/notes.txt'
      - env-file-absent: projet-demo/brouillons
    solution:
      - touch projet-demo/notes.txt
      - rm -r projet-demo/brouillons
  - text: 'Copie tout le dossier `projet-demo/config` (avec son contenu, fichier caché compris) dans `projet-demo/archives/`'
    hint: 'Pour un dossier, `cp` demande l''option `-r` (récursif) : `cp -r source destination`.'
    checks:
      - command-succeeds: 'diff -r projet-demo/archives/config /opt/exercices/01-navigation/projet-demo/config'
    solution:
      - cp -r projet-demo/config projet-demo/archives/
:::

## Vérifie tes acquis

:::quiz
Quelle commande affiche le dossier dans lequel tu te trouves ?

- [ ] `ls`
- [ ] `cd ~`
- [x] `pwd`

> `pwd` (*print working directory*) donne le chemin du dossier courant. `ls` en liste le contenu, `cd ~` te déplace.
:::

:::quiz
Tu es dans `/home/alice`. Que fait `cd ../bob` ?

- [x] Elle te mène dans `/home/bob`
- [ ] Elle crée un dossier `bob` dans `/home/alice`
- [ ] Elle te mène dans `/bob`
- [ ] Elle échoue toujours, car `..` n'existe pas

> `..` désigne le dossier parent (`/home`), puis on descend dans `bob`. C'est un chemin relatif.
:::

:::quiz
Comment renommer `rapport.txt` en `rapport-v2.txt` ?

- [ ] `cp rapport.txt rapport-v2.txt` (le renommage est automatique)
- [ ] `rename rapport.txt` seul
- [x] `mv rapport.txt rapport-v2.txt`

> `mv` sert à déplacer et à renommer. `cp` laisserait l'ancien fichier en place.
:::

:::quiz
Que se passe-t-il après `rm -r projet` ?

- [ ] Le dossier est envoyé dans la corbeille
- [ ] Seuls les fichiers sont supprimés, le dossier reste
- [x] Le dossier et tout son contenu sont supprimés définitivement

> Il n'y a pas de corbeille en ligne de commande : vérifie toujours avant.
:::
