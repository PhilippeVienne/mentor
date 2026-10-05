---
id: script-shell
title: "Écrire un petit script shell"
summary: "Variables, conditions, boucles et codes de retour : lire et écrire un script comme ceux des dépôts de l'équipe."
minutes: 35
objectives:
  - Écrire un script avec shebang, variables et arguments
  - Utiliser un code de retour, `if` et `for`
  - Lire un script réel de l'équipe et expliquer ce que fait `set -e`
---

Tu tapes les mêmes cinq commandes à chaque déploiement. Au bout de la troisième fois, tu en fais un **script** : un fichier texte qui enchaîne les commandes à ta place, toujours dans le même ordre.

## Un premier script

```bash
#!/bin/bash
set -e

NOM="${1:-monde}"
echo "Bonjour $NOM"
date
```

1. `#!/bin/bash` (le *shebang*) indique quel **interpréteur** (le programme qui lit et exécute le fichier) doit le faire.
2. `set -e` arrête le script à la première commande qui échoue.
3. `NOM="${1:-monde}"` crée une variable (sans espace autour du `=`) : `$1` est le premier argument, `monde` sert de valeur par défaut.
4. `echo "Bonjour $NOM"` affiche la variable, grâce au `$`.

Pour l'exécuter, rends-le exécutable puis lance-le avec un chemin.

```bash
chmod +x salut.sh
./salut.sh Alice
```

## Codes de retour, if et for

Toute commande se termine avec un **code de retour** : `0` signifie « succès », tout autre nombre signale une erreur. La variable `$?` donne celui de la dernière commande.

```bash
#!/bin/bash
if ping -c 1 -W 2 example.org > /dev/null 2>&1; then
    echo "réseau OK"
else
    echo "réseau KO" >&2
    exit 1
fi

for FICHIER in *.log; do
    echo "Journal : $FICHIER"
done
```

- `if commande; then … else … fi` teste le code de retour de la commande (`0` = vrai).
- `> /dev/null 2>&1` jette toute la sortie : seul le code nous intéresse.
- `>&2` écrit le message sur la sortie d'erreur et `exit 1` termine le script en échec, ce qui permet à un autre outil (une CI par exemple) de le détecter.
- `for FICHIER in *.log; do … done` répète le bloc pour chaque fichier dont le nom finit par `.log` (le joker `*` remplace n'importe quelle suite de caractères) : à chaque tour, la variable `FICHIER` prend le nom d'un fichier.

Pour tester l'existence d'un fichier dans un `if`, on utilise la commande `test` : `test -f liste.txt` réussit (code `0`) si `liste.txt` existe et est un fichier ordinaire.

:::info Dans le labo
Le terminal du labo n'a pas de réseau : `ping` n'y répondrait pas. Tu vas t'exercer avec `test -f` et `for` sur des fichiers locaux, ce qui suffit à maîtriser la structure.
:::

## Un script réel de l'équipe

Le dépôt `infra-dev` de l'équipe contient `startme.sh`, qui prépare un cluster local de développement (un petit ensemble de conteneurs sur ton poste, pour tester sans toucher aux vrais serveurs). En voici le début ; chaque bloc vérifie qu'un outil est installé : `docker ps` et `kubectl version --client` interrogent Docker et Kubernetes, `helm version` et `k3d version` affichent la version de ces outils.

```bash
#!/bin/bash
set -e
echo "check: Docker"
docker ps
echo "check: kubectl"
kubectl version --client
echo "check: helm"
helm version
echo "check: k3d"
k3d version
set +e
```

Avec `set -e`, si Docker, `kubectl`, `helm` ou `k3d` manque ou ne répond pas, le script s'arrête tout de suite avec une erreur : on ne continue pas sur un poste mal préparé. `set +e` désactive ensuite ce comportement pour la suite. Un simple script de vérification vaut mieux qu'une page d'instructions que personne ne relit.

:::warning Mets toujours tes variables entre guillemets
`rm -r $DOSSIER/*` avec une variable vide devient `rm -r /*`. Écris `"$DOSSIER"` entre guillemets et, dans un script sensible, ajoute `set -u` pour qu'une variable non définie provoque une erreur.
:::

:::tip Teste avant d'exécuter
Lis ton script ligne par ligne avant de le lancer sur un vrai serveur. Pour voir ce qu'il fait réellement, lance-le avec `bash -x script.sh` : chaque commande s'affiche avant son exécution.
:::

## Entraîne-toi

:::lab
engine: real
intro: |
  Tu écris quatre petits scripts dans ton dossier de travail. Un fichier `liste.txt` s'y trouve déjà. Crée chaque script avec `nano` (ou l'éditeur de ton choix), puis teste-le. Pour lancer un script avec `./`, il faut d'abord le rendre exécutable (`chmod +x`).
commands:
  - cp -R /opt/exercices/05-script/. .
steps:
  - text: 'Crée `salut.sh` : avec un argument, il affiche `Bonjour <argument>` ; sans argument, il affiche `Bonjour monde`. Commence par le shebang `#!/bin/bash`'
    hint: 'Reprends le premier exemple de la leçon : `NOM="${1:-monde}"` puis `echo "Bonjour $NOM"`. Teste avec `bash salut.sh Alice` puis `bash salut.sh`.'
    checks:
      - output-contains: ['bash salut.sh Alice', '^Bonjour Alice$']
      - output-contains: ['bash salut.sh Zoé', '^Bonjour Zoé$']
      - output-contains: ['bash salut.sh', '^Bonjour monde$']
      - env-file-contains: [salut.sh, '^#!/bin/bash']
    solution:
      - |
        cat > salut.sh <<'EOF'
        #!/bin/bash
        NOM="${1:-monde}"
        echo "Bonjour $NOM"
        EOF
  - text: 'Rends `salut.sh` exécutable et lance-le avec `./salut.sh Alice`'
    hint: '`chmod +x salut.sh`, puis `./salut.sh Alice`. Le `./` désigne le script du dossier courant.'
    after: [1]
    checks:
      - output-contains: ['./salut.sh Alice', '^Bonjour Alice$']
    solution:
      - chmod +x salut.sh
      - ./salut.sh Alice
  - text: 'Crée `verifier.sh` : s''il trouve `liste.txt` dans le dossier courant, il affiche `liste trouvée` ; sinon il écrit `liste absente` sur la sortie d''erreur et termine avec le code `1`'
    hint: 'Structure : `if test -f liste.txt; then echo "liste trouvée"; else echo "liste absente" >&2; exit 1; fi`. Teste-le ici, puis depuis un autre dossier avec `cd /tmp` : `bash /workspace/verifier.sh; echo $?`.'
    checks:
      - output-contains: ['bash verifier.sh', '^liste trouvée$']
      - command-fails: 'cd /tmp && bash /workspace/verifier.sh'
    solution:
      - |
        cat > verifier.sh <<'EOF'
        #!/bin/bash
        if test -f liste.txt; then
            echo "liste trouvée"
        else
            echo "liste absente" >&2
            exit 1
        fi
        EOF
  - text: 'Crée `boucle.sh` : pour chaque fichier `*.txt` du dossier courant, il affiche `Fichier : <nom>`'
    hint: '`for FICHIER in *.txt; do echo "Fichier : $FICHIER"; done`, sur plusieurs lignes comme dans la leçon.'
    checks:
      - output-contains: ['bash boucle.sh', '^Fichier : liste\.txt$']
      - command-succeeds: 'D=$(mktemp -d) && cd "$D" && touch a.txt b.txt c.md && test "$(bash /workspace/boucle.sh)" = "$(printf "Fichier : a.txt\nFichier : b.txt")"'
    solution:
      - |
        cat > boucle.sh <<'EOF'
        #!/bin/bash
        for FICHIER in *.txt; do
            echo "Fichier : $FICHIER"
        done
        EOF
  - text: 'Crée `strict.sh` qui commence par `set -e`, puis lance `ls dossier-inconnu` (qui échoue), puis `echo "fin"`. Grâce à `set -e`, `fin` ne doit jamais s''afficher'
    hint: 'Lance `bash strict.sh; echo $?` : le code affiché ne doit pas être `0`, et le mot `fin` doit être absent.'
    checks:
      - env-file-contains: [strict.sh, '^set -e']
      - command-fails: 'bash strict.sh > /dev/null 2>&1'
      - command-fails: 'bash strict.sh 2>/dev/null | grep -q "^fin$"'
    solution:
      - |-
        cat > strict.sh <<'EOF'
        #!/bin/bash
        set -e
        ls dossier-inconnu
        echo "fin"
        EOF
:::

## Vérifie tes acquis

:::quiz
À quoi sert la première ligne `#!/bin/bash` ?

- [ ] C'est un simple commentaire sans effet
- [ ] Elle rend le fichier exécutable
- [x] Elle indique quel interpréteur doit exécuter le script

> Le shebang est lu au lancement de `./script.sh`. L'exécutabilité, elle, vient de `chmod +x`.
:::

:::quiz
Que fait `set -e` en tête de script ?

- [x] Le script s'arrête dès qu'une commande échoue
- [ ] Il affiche chaque commande avant de l'exécuter
- [ ] Il rend les variables en lecture seule

> C'est ce qui permet à `startme.sh` de s'arrêter si un outil requis est absent. Pour afficher les commandes, c'est `set -x`.
:::

:::quiz
Quel est le code de retour d'une commande qui réussit ?

- [ ] `1`
- [ ] `-1`
- [x] `0`

> `0` signifie succès. Tout autre code indique une erreur, et `$?` donne celui de la dernière commande.
:::

:::quiz
Comment affecter correctement une valeur à une variable ?

- [ ] `NOM = "Alice"`
- [x] `NOM="Alice"`
- [ ] `$NOM="Alice"`

> Pas d'espace autour du `=`, et pas de `$` à l'affectation : il ne sert qu'à lire la valeur.
:::
