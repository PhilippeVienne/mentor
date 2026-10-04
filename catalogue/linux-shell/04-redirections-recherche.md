---
id: redirections-recherche
titre: "Redirections, tubes et recherche"
resume: "Enchaîner des commandes avec les tubes, rediriger les sorties et retrouver fichiers et lignes avec find et grep."
duree: 30
objectifs:
  - Rediriger une sortie vers un fichier avec `>` et `>>`
  - Enchaîner des commandes avec le tube `|`
  - Rechercher dans le contenu (`grep`) et dans les noms de fichiers (`find`)
---

Un fichier de **journal** (*log* : le fichier texte où une application note ce qu'elle fait, une ligne par évènement) de 40 000 lignes, et une seule erreur à retrouver. Lire à l'œil est impossible. Le shell te donne deux idées simples pour s'en sortir : brancher les commandes les unes sur les autres, et chercher avec des filtres.

## Entrées et sorties

Chaque commande lit sur son **entrée standard** et écrit sur deux sorties : la **sortie standard** (le résultat) et la **sortie d'erreur** (les messages d'erreur). Par défaut, tout s'affiche à l'écran, mais on peut détourner ces flux.

```bash
echo "bonjour" > notes.txt
echo "suite" >> notes.txt
ls dossier-inconnu 2> erreurs.txt
ls /etc /inconnu > tout.txt 2>&1
```

| Symbole | Effet |
| --- | --- |
| `>` | Écrit la sortie dans un fichier, en **écrasant** son contenu |
| `>>` | **Ajoute** à la fin du fichier |
| `2>` | Redirige la sortie d'erreur |
| `2>&1` | Envoie les erreurs au même endroit que la sortie standard |

:::warning > écrase sans prévenir
`commande > fichier` remplace le contenu existant, sans confirmation. Si tu veux conserver ce qui s'y trouve, utilise `>>`.
:::

## Le tube

Le tube `|` envoie la sortie d'une commande à l'entrée de la suivante.

```bash
ls /etc | wc -l
cat journal.log | grep ERROR | sort | uniq -c
```

- `wc -l` compte les lignes : la première commande liste `/etc`, la seconde compte les lignes obtenues.
- La seconde chaîne lit un journal, garde les lignes contenant `ERROR`, les trie, puis `uniq -c` compte les doublons consécutifs.

```mermaid
flowchart LR
    A[cat journal.log] -->|tube| B[grep ERROR]
    B -->|tube| C[sort]
    C -->|tube| D[uniq -c]
```

Chaque petite commande fait une seule chose, et le tube les assemble.

## Chercher dans le contenu avec grep

```bash
grep ERROR journal.log
grep -i "timeout" journal.log
grep -rn "TODO" projet/
grep -v DEBUG journal.log
grep -c ERROR journal.log
```

1. `grep ERROR fichier` affiche les lignes qui contiennent `ERROR`.
2. `-i` ignore la casse (majuscules et minuscules).
3. `-r` descend dans les dossiers et `-n` affiche les numéros de ligne.
4. `-v` **inverse** : garde les lignes qui ne contiennent pas le motif.
5. `-c` compte les lignes au lieu de les afficher.

## Chercher des fichiers avec find

```bash
find . -name "*.log"
find /var/log -type f -mtime -1
find . -name "*.tmp" -size +10M
```

`find dossier critères` parcourt l'arborescence. Ici : les fichiers dont le nom finit par `.log`, les fichiers (`-type f`) modifiés depuis moins d'un jour (`-mtime -1`), et les `.tmp` de plus de 10 Mo.

:::tip grep cherche dans les fichiers, find cherche les fichiers
Pour retrouver **où** se trouve un fichier selon son nom, sa taille ou sa date, utilise `find`. Pour retrouver **quelle ligne** contient un mot, utilise `grep`. Les deux se combinent : `find . -name "*.log"` donne la liste, `grep -l ERROR` montre lesquels contiennent l'erreur.
:::

## Entraîne-toi

:::labo
moteur: reel
intro: |
  Dans ton dossier de travail, le dossier `journaux` contient trois journaux d'application (`app.log`, `adhesion.log`, `portail.log`) et un fichier de notes (`notes.txt`). Chaque ligne d'un journal commence par une date et un niveau (`INFO`, `WARNING`, `ERROR`, `DEBUG`). Utilise les redirections, les tubes, `grep` et `find` pour répondre aux questions. Les fichiers de résultat se créent dans le dossier de travail, à côté de `journaux`.
commandes:
  - cp -R /opt/exercices/04-recherche/. .
etapes:
  - texte: 'Écris dans `erreurs.txt` toutes les lignes de `journaux/app.log` qui contiennent `ERROR` (et seulement celles-là)'
    indice: '`grep ERROR journaux/app.log > erreurs.txt` : `grep` filtre et `>` envoie le résultat dans le fichier au lieu de l''écran. Relis le résultat avec `cat erreurs.txt`.'
    verif:
      - commande-reussit: 'grep ERROR /opt/exercices/04-recherche/journaux/app.log | cmp -s - erreurs.txt'
    solution:
      - grep ERROR journaux/app.log > erreurs.txt
  - texte: 'Écris dans `nombre-erreurs.txt` le nombre de lignes `ERROR` de `journaux/app.log`, rien d''autre que ce nombre'
    indice: 'Deux façons : `grep -c ERROR journaux/app.log > nombre-erreurs.txt`, ou `grep ERROR journaux/app.log | wc -l > nombre-erreurs.txt`.'
    verif:
      - commande-reussit: 'test "$(tr -d "[:space:]" < nombre-erreurs.txt)" = "$(grep -c ERROR /opt/exercices/04-recherche/journaux/app.log)"'
    solution:
      - grep -c ERROR journaux/app.log > nombre-erreurs.txt
  - texte: 'Avec `find`, écris dans `liste-logs.txt` les chemins des fichiers dont le nom finit par `.log` dans `journaux` (pas `notes.txt`)'
    indice: '`find journaux -name "*.log" > liste-logs.txt`. Les guillemets autour de `*.log` sont importants.'
    verif:
      - commande-reussit: 'test "$(cd /opt/exercices/04-recherche && find journaux -name "*.log" | sort)" = "$(sort liste-logs.txt)"'
    solution:
      - find journaux -name "*.log" > liste-logs.txt
  - texte: 'Combien de lignes `ERROR` en tout dans les trois journaux ? Écris ce nombre seul dans `total.txt`, avec un tube'
    indice: '`grep -h ERROR journaux/*.log | wc -l > total.txt`. `-h` évite d''afficher le nom du fichier devant chaque ligne ; le joker `*` désigne tous les fichiers `.log` du dossier.'
    verif:
      - commande-reussit: 'test "$(tr -d "[:space:]" < total.txt)" = "$(grep -h ERROR /opt/exercices/04-recherche/journaux/*.log | wc -l | tr -d " ")"'
    solution:
      - grep -h ERROR journaux/*.log | wc -l > total.txt
  - texte: 'Cherche le mot `TODO` dans tout le dossier `journaux` (sous-dossiers compris) et écris le résultat dans `todo.txt`'
    indice: '`grep -rn TODO journaux > todo.txt` : `-r` descend dans les dossiers, `-n` ajoute le numéro de ligne.'
    verif:
      - commande-reussit: 'cd /opt/exercices/04-recherche && test "$(grep -rn TODO journaux | sort)" = "$(cd /workspace && sort todo.txt)" || test "$(grep -r TODO journaux | sort)" = "$(cd /workspace && sort todo.txt)"'
    solution:
      - grep -rn TODO journaux > todo.txt
:::

## Vérifie tes acquis

:::quiz
Quelle est la différence entre `>` et `>>` ?

- [ ] `>` ajoute à la fin et `>>` écrase
- [ ] `>>` redirige les erreurs, pas `>`
- [x] `>` écrase le fichier et `>>` ajoute à la fin

> Utilise `>>` pour conserver le contenu existant, par exemple pour alimenter un fichier de notes.
:::

:::quiz
Que fait `cat journal.log | grep ERROR` ?

- [ ] Elle supprime les lignes `ERROR` du fichier
- [x] Elle n'affiche que les lignes de `journal.log` qui contiennent `ERROR`
- [ ] Elle écrit `ERROR` à la fin du fichier

> Le tube envoie la sortie de `cat` à `grep`, qui filtre. Le fichier lui-même n'est pas modifié.
:::

:::quiz
Quelle commande trouve tous les fichiers `.log` sous `/var/log` ?

- [ ] `grep -r .log /var/log`
- [ ] `ls -log /var/log`
- [x] `find /var/log -name "*.log"`

> `find` cherche par nom, taille ou date. `grep -r .log` chercherait le texte « .log » dans le contenu des fichiers.
:::

:::quiz
Que fait `grep -v DEBUG app.log` ?

- [x] Affiche les lignes qui ne contiennent pas `DEBUG`
- [ ] Affiche seulement les lignes qui contiennent `DEBUG`
- [ ] Affiche le nombre de lignes `DEBUG`

> `-v` inverse la sélection, pratique pour masquer le bruit d'un journal.
:::
