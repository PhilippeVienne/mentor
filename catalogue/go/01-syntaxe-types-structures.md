---
id: syntaxe-types-structures
titre: "Premiers pas : syntaxe, types et structures"
resume: "Lancer un premier programme Go et décrire des données avec des types, des structures, des listes et des tables, dans un vrai terminal."
duree: 45
objectifs:
  - Expliquer ce qu'est un langage compilé et à quoi sert Go dans l'équipe
  - Lancer un programme avec `go run` et lire sa structure ligne à ligne
  - Déclarer des variables, des structures, des listes (slices) et des tables (maps)
  - Choisir entre un récepteur valeur et un récepteur pointeur
---

Tu as peut-être déjà vu un message du type « le serveur est en maintenance » sur le site d'adhésion : derrière ce message, il y a un petit programme qui tourne en continu et qui prévient les pages web ouvertes. Dans l'équipe, ce genre de programme est écrit en **Go**. Dans cette première leçon, tu n'as besoin de rien connaître : on part de zéro.

:::info Un vrai environnement
Le labo de cette leçon te prête un conteneur Linux (un petit ordinateur virtuel jetable, rien que pour toi) avec Go 1.25 déjà installé, sans droits administrateur et **sans accès à Internet**. À l'arrêt, ton dossier de travail est **effacé** : tout ce que tu écris ici est un brouillon d'entraînement.
:::

## À quoi ça sert, et pourquoi Go ?

Un ordinateur ne comprend pas le texte que tu écris. Il faut le **traduire** en instructions qu'il sait exécuter. Il existe deux grandes manières de le faire :

:::cartes
### Langage interprété

Un traducteur lit ton texte pendant que le programme tourne (Python, par exemple). Pour lancer le programme ailleurs, il faut y installer le traducteur.

### Langage compilé

Un outil, le **compilateur**, traduit tout ton texte *une fois pour toutes* en un fichier exécutable. Ce fichier se lance seul, sans rien installer autour. C'est le cas de Go.
:::

Go a été créé pour écrire des **services** : des programmes qui tournent en permanence et répondent à d'autres programmes (par exemple, un navigateur qui demande une page). Il est apprécié pour trois raisons :

- il est **simple** : peu de mots-clés, une seule façon habituelle d'écrire les choses ;
- il produit **un seul fichier exécutable**, facile à mettre dans une image Docker (une « boîte » qui contient un programme et tout ce qu'il lui faut, on y revient dans la dernière leçon) ;
- il sait faire **plusieurs choses à la fois** sans effort (on y revient dans les leçons 3 et 4).

Trois projets de l'équipe l'utilisent : `adhesion/mgmt` (le service qui gère le mode maintenance), `event-planner-api` et `billetterie`. Leurs fichiers `go.mod` annoncent des versions de Go très différentes : **1.16** pour `mgmt`, **1.14** pour `billetterie`, **1.23** pour `event-planner-api`. Ce sont des versions sorties entre 2020 et 2024. L'équipe de Go ne maintient que les deux versions les plus récentes ([politique de support](https://go.dev/doc/devel/release#policy)) : les trois sont donc anciennes aujourd'hui. On signalera les différences qui comptent au fil du parcours.

## Le terminal et les premières commandes

Un **terminal** est une fenêtre où tu tapes des **commandes** : des lignes de texte qui demandent une action à l'ordinateur (créer un dossier, lancer un programme…). Tu valides chaque ligne avec la touche Entrée. Dans le labo, le terminal est déjà ouvert ; sur ton propre poste, installe Go depuis [go.dev/dl](https://go.dev/dl/) puis ouvre un terminal.

```bash
go version
```

Cette commande affiche la version installée, par exemple `go version go1.25.14 linux/amd64`. Si le terminal répond « commande introuvable », l'installation n'est pas terminée.

Un projet Go s'appelle un **module** : c'est un dossier qui contient un fichier `go.mod`, la carte d'identité du projet (son nom, la version de Go visée et la liste de ses dépendances, c'est-à-dire les bibliothèques écrites par d'autres).

```bash
mkdir ludotheque
cd ludotheque
go mod init ludotheque
```

- `mkdir ludotheque` crée un dossier (*make directory*) ;
- `cd ludotheque` entre dedans (*change directory*) ;
- `go mod init ludotheque` crée le fichier `go.mod` avec le nom `ludotheque`.

## Un premier programme, ligne à ligne

Imaginons une petite **ludothèque** (un prêt de jeux de société) : ce domaine fictif nous servira dans tout le parcours. Crée un fichier `main.go` avec ce contenu :

```go
package main

import "fmt"

// Jeu décrit un jeu de société de la ludothèque.
type Jeu struct {
	Titre   string
	Joueurs int
	Dispo   bool
}

// Description a un récepteur « valeur » : elle lit le jeu sans le modifier.
func (j Jeu) Description() string {
	return fmt.Sprintf("%s (%d joueur·se·s)", j.Titre, j.Joueurs)
}

// Emprunter a un récepteur « pointeur » : elle modifie le jeu d'origine.
func (j *Jeu) Emprunter() {
	j.Dispo = false
}

func main() {
	catalogue := []Jeu{
		{Titre: "Catane", Joueurs: 4, Dispo: true},
		{Titre: "Dixit", Joueurs: 6, Dispo: true},
	}

	catalogue[0].Emprunter()

	parTitre := map[string]Jeu{}
	for _, j := range catalogue {
		parTitre[j.Titre] = j
	}

	var inconnu Jeu // valeur zéro : "", 0, false
	fmt.Println(inconnu.Titre == "", inconnu.Joueurs, inconnu.Dispo)

	if dixit, ok := parTitre["Dixit"]; ok {
		fmt.Println(dixit.Description(), "disponible :", dixit.Dispo)
	}
	fmt.Println(len(catalogue), "jeux dont", catalogue[0].Titre, "emprunté :", !catalogue[0].Dispo)

	for _, j := range catalogue {
		j.Emprunter() // piège : j est une copie
	}
	fmt.Println(catalogue[1].Dispo)

	for i := range catalogue {
		catalogue[i].Emprunter()
	}
	fmt.Println(catalogue[1].Dispo)
}
```

Lance-le avec `go run .` : cette commande compile le dossier courant (le `.` veut dire « ici ») et exécute le résultat. Tu dois voir :

```console
true 0 false
Dixit (6 joueur·se·s) disponible : true
2 jeux dont Catane emprunté : true
true
false
```

Prenons le code par morceaux.

### Le squelette

- `package main` : chaque fichier Go déclare à quel **paquet** il appartient (un paquet est un groupe de fichiers qui travaillent ensemble). Le paquet `main` est spécial : c'est celui qui devient un programme exécutable.
- `import "fmt"` : on demande à utiliser `fmt`, un paquet fourni avec Go qui sait afficher et formater du texte (le nom vient de *format*).
- `// Jeu décrit…` : tout ce qui suit `//` sur une ligne est un **commentaire**. Go l'ignore, il est écrit pour les humains.
- `func main() { … }` : une **fonction** est un bloc d'instructions nommé, écrit entre accolades `{ }`. `main` est celle que Go lance au démarrage.
- `fmt.Println(…)` : affiche ses arguments dans le terminal, séparés par des espaces, puis passe à la ligne (*print line*). Le point dans `fmt.Println` veut dire « la chose `Println` du paquet `fmt` ».

### Les types et les structures

Un **type** dit quelle sorte de valeur on manipule : `string` pour du texte, `int` pour un nombre entier, `bool` pour vrai ou faux (`true` ou `false`).

Une **structure** (`struct`) regroupe plusieurs valeurs sous un même nom, comme une fiche : ici, la fiche d'un `Jeu` a un `Titre`, un nombre de `Joueurs` et un indicateur `Dispo`. Chaque ligne de la structure est un **champ** : un nom, puis son type. Le projet `mgmt` en contient une du même genre dans `webconfig.go` : `WebConfig`, avec trois champs (deux booléens et un message) qui décrivent l'état de la maintenance.

Pour fabriquer un jeu, on écrit `Jeu{Titre: "Catane", Joueurs: 4, Dispo: true}` : le type, puis entre accolades la valeur de chaque champ, sous la forme `nom: valeur`. On lit un champ avec un point : `j.Titre`.

Une majuscule au début d'un nom (`Jeu`, `Titre`) le rend **visible depuis les autres paquets** ; une minuscule le garde privé. C'est la seule « visibilité » de Go, il n'y a pas de mot-clé `public`.

### Les variables

- `catalogue := []Jeu{…}` : `:=` déclare une variable *et* lui donne une valeur ; Go devine son type. Ici, `[]Jeu` est une **slice** : une liste de `Jeu` dont la taille peut varier. On lit un élément avec `catalogue[0]` (le premier, la numérotation commence à 0) et la taille avec `len(catalogue)`.
- `parTitre := map[string]Jeu{}` : une **map** est une table qui associe une clé (ici un titre, de type `string`) à une valeur (un `Jeu`), comme un annuaire. On écrit `parTitre[j.Titre] = j` pour ranger une valeur et `parTitre["Dixit"]` pour la relire.
- `var inconnu Jeu` : une variable déclarée sans valeur reçoit sa **valeur zéro** : `""` pour un texte, `0` pour un nombre, `false` pour un booléen. Il n'y a jamais de variable « vide et dangereuse ».
- `inconnu.Titre == ""` : `==` **compare** deux valeurs et donne `true` ou `false` (un seul `=` serait une affectation, qui range une valeur). `!catalogue[0].Dispo` utilise `!`, qui **inverse** un booléen.
- `if dixit, ok := parTitre["Dixit"]; ok { … }` : lire une map renvoie deux choses, la valeur et un booléen `ok` qui dit si la clé existait. Le `;` sépare l'instruction préparatoire (`dixit, ok := …`) de la condition (`ok`) : le bloc entre accolades ne s'exécute que si `ok` vaut `true`.
- `for _, j := range catalogue` : parcourt la liste ; à chaque tour, `j` vaut l'élément suivant. Le `_` signifie « je n'ai pas besoin de la position ».

### Formater du texte avec `Sprintf`

`fmt.Sprintf("%s (%d joueur·se·s)", j.Titre, j.Joueurs)` fabrique un texte à partir d'un **modèle** (le premier argument) dans lequel des repères commençant par `%` sont remplacés, dans l'ordre, par les valeurs suivantes :

| Repère | Remplacé par | Exemple |
| --- | --- | --- |
| `%s` | un texte (*string*) | `Dixit` |
| `%d` | un nombre entier (*decimal*) | `6` |
| `%q` | un texte entre guillemets | `"Dixit"` |
| `%v` | n'importe quelle valeur, au format par défaut | `{Dixit 6 true}` |

Ici, `%s` reçoit `j.Titre` et `%d` reçoit `j.Joueurs` : le résultat est `Dixit (6 joueur·se·s)`. Le `S` de `Sprintf` veut dire que le texte est **renvoyé** (`return`) au lieu d'être affiché ; `fmt.Printf` fait la même chose mais affiche directement.

### Les méthodes et les pointeurs

Une **méthode** est une fonction attachée à un type : `func (j Jeu) Description() string` se lit « pour un `Jeu` appelé `j`, voici `Description`, qui renvoie un texte ». La partie `(j Jeu)` s'appelle le **récepteur** : c'est l'objet sur lequel on appelle la méthode, avec `jeu.Description()`. Le mot `string` après les parenthèses est le type de ce que la fonction renvoie avec `return`.

Le choix entre `(j Jeu)` et `(j *Jeu)` est important :

- avec `(j Jeu)`, Go donne à la méthode une **copie** du jeu. La méthode peut la lire, mais pas modifier l'original ;
- avec `(j *Jeu)`, la méthode reçoit un **pointeur**, c'est-à-dire l'adresse de l'original (l'étoile `*` se lit « pointeur vers »). Elle modifie la vraie fiche. C'est le cas de `Emprunter`.

:::warning Le piège de la copie dans `range`
Dans `for _, j := range catalogue`, la variable `j` est une **copie** de chaque jeu. Appeler `j.Emprunter()` modifie la copie, puis la jette : le catalogue ne change pas (la quatrième ligne affichée vaut encore `true`). Pour modifier les vraies fiches, passe par la position : `catalogue[i].Emprunter()`.
:::

## Entraîne-toi

:::labo
moteur: reel
intro: |
  Dans ton dossier de travail, `jeu.go` décrit un `Jeu`. Les méthodes et fonctions qui s'en servent sont **à écrire**, chacune dans son fichier (`description.go`, `emprunt.go`…) : remplace la ligne `panic("à écrire")` par ton code. Les tests sont dans `jeu_test.go`. Ouvre un fichier avec `nano` ou VS Code, enregistre, puis lance `go test ./...` pour voir ce qui passe. Le premier lancement est un peu long : Go compile ses propres paquets. Les tests fournis sont vérifiés tels quels.
commandes:
  - cp -R /opt/exercices/01-syntaxe/. .
  - go vet ./... >/dev/null 2>&1 || true
etapes:
  - texte: 'Écris la méthode `Description()` dans `description.go` : elle renvoie par exemple « Azul (4 joueur·se·s) ». Le test `TestDescription` doit passer'
    indice: 'Remplace `panic("à écrire")` par `return fmt.Sprintf("%s (%d joueur·se·s)", j.Titre, j.Joueurs)` et ajoute `import "fmt"` sous la ligne `package main`. Teste avec `go test -run TestDescription .`'
    verif:
      - commande-reussit: "verifier-go tests 01-syntaxe . TestDescription"
    solution:
      - |
        cat > description.go <<'EOF'
        package main

        import "fmt"

        // Description renvoie le jeu sous la forme « Titre (N joueur·se·s) ».
        func (j Jeu) Description() string {
        	return fmt.Sprintf("%s (%d joueur·se·s)", j.Titre, j.Joueurs)
        }
        EOF
  - texte: 'Écris la méthode `Emprunter()` dans `emprunt.go` avec un **récepteur pointeur** : elle met `Dispo` à `false` sur le jeu d''origine. `TestEmprunter` doit passer'
    indice: 'Le récepteur est déjà écrit `(j *Jeu)` : il te reste une ligne, `j.Dispo = false`.'
    verif:
      - commande-reussit: "verifier-go tests 01-syntaxe . TestEmprunter"
    solution:
      - |
        cat > emprunt.go <<'EOF'
        package main

        // Emprunter marque le jeu comme non disponible.
        func (j *Jeu) Emprunter() {
        	j.Dispo = false
        }
        EOF
  - texte: 'Écris la fonction `Disponibles` dans `disponibles.go` : elle renvoie une nouvelle slice avec les jeux dont `Dispo` vaut `true`, dans le même ordre. `TestDisponibles` doit passer'
    indice: 'Déclare `var res []Jeu`, parcours `catalogue` avec `for _, j := range catalogue`, et fais `res = append(res, j)` quand `j.Dispo` est vrai. Termine par `return res`.'
    verif:
      - commande-reussit: "verifier-go tests 01-syntaxe . TestDisponibles"
    solution:
      - |
        cat > disponibles.go <<'EOF'
        package main

        // Disponibles renvoie, dans le même ordre, les jeux du catalogue dont Dispo vaut true.
        func Disponibles(catalogue []Jeu) []Jeu {
        	var res []Jeu
        	for _, j := range catalogue {
        		if j.Dispo {
        			res = append(res, j)
        		}
        	}
        	return res
        }
        EOF
  - texte: 'Écris `IndexParTitre` dans `index.go` : elle renvoie une **map** qui associe chaque titre à son jeu. `TestIndexParTitre` doit passer'
    indice: 'Crée la table avec `index := map[string]Jeu{}`, remplis-la dans une boucle avec `index[j.Titre] = j`, puis renvoie-la.'
    verif:
      - commande-reussit: "verifier-go tests 01-syntaxe . TestIndexParTitre"
    solution:
      - |
        cat > index.go <<'EOF'
        package main

        // IndexParTitre renvoie une table qui associe chaque titre au jeu correspondant.
        func IndexParTitre(catalogue []Jeu) map[string]Jeu {
        	index := map[string]Jeu{}
        	for _, j := range catalogue {
        		index[j.Titre] = j
        	}
        	return index
        }
        EOF
  - texte: 'Écris `EmprunterTous` dans `tous.go` : elle doit modifier **les vrais jeux** du catalogue reçu. Attention au piège de la copie dans `range`. `TestEmprunterTous` doit passer'
    indice: 'Parcours les positions avec `for i := range catalogue` et appelle `catalogue[i].Emprunter()`. Avec `for _, j := range`, tu modifierais une copie.'
    apres: [2]
    verif:
      - commande-reussit: "verifier-go tests 01-syntaxe . TestEmprunterTous"
    solution:
      - |
        cat > tous.go <<'EOF'
        package main

        // EmprunterTous emprunte tous les jeux du catalogue.
        func EmprunterTous(catalogue []Jeu) {
        	for i := range catalogue {
        		catalogue[i].Emprunter()
        	}
        }
        EOF
  - texte: 'Compile le programme en un exécutable nommé `ludotheque` avec `go build -o ludotheque .`, puis lance-le avec `./ludotheque` : la dernière ligne affichée doit être `0 jeu disponible sur 2`'
    indice: '`main.go` est déjà écrit : il utilise toutes tes fonctions. Quand les cinq étapes précédentes sont faites, `go build -o ludotheque .` crée le fichier `ludotheque`, que tu lances avec `./ludotheque`. (`go run .` ferait les deux d''un coup, sans garder le fichier.)'
    apres: [1, 2, 3, 4, 5]
    verif:
      - commande-reussit: 'verifier-go binaire 01-syntaxe ludotheque'
      - commande-reussit: "verifier-go lancer 01-syntaxe '^0 jeu disponible sur 2$'"
    solution:
      - go build -o ludotheque .
      - ./ludotheque
:::

## Vérifie tes acquis

:::quiz
Qu'est-ce qui distingue un langage compilé comme Go d'un langage interprété ?

- [ ] Il n'a pas besoin de système d'exploitation pour fonctionner
- [x] Un compilateur le traduit en amont en un fichier exécutable qui se lance sans traducteur
- [ ] Il ne peut pas afficher de texte à l'écran
- [ ] Il s'exécute uniquement dans un navigateur

> Le compilateur produit un fichier exécutable autonome, ce qui simplifie le déploiement (une image Docker minimale, par exemple).
:::

:::quiz
Pourquoi `j.Emprunter()` dans `for _, j := range catalogue` ne change-t-il pas le catalogue ?

- [ ] Parce que `Emprunter` est une fonction privée
- [ ] Parce que `range` trie la liste avant de la parcourir
- [ ] Parce que la slice est en lecture seule
- [x] Parce que `j` est une copie de l'élément, pas l'élément lui-même

> Pour modifier l'élément d'origine, on écrit `catalogue[i].Emprunter()`.
:::

:::quiz
À quoi sert le second résultat `ok` dans `valeur, ok := maTable["clé"]` ?

- [x] À savoir si la clé existait dans la table
- [ ] À vérifier que la table n'est pas trop grande
- [ ] À copier la valeur dans une autre variable
- [ ] À supprimer la clé après lecture

> Sans `ok`, une clé absente donnerait la valeur zéro, impossible à distinguer d'une vraie valeur.
:::

:::quiz
Que produit `fmt.Sprintf("%s a %d ans", "Camille", 20)` ?

- [ ] Rien : `Sprintf` affiche directement dans le terminal
- [ ] Une erreur, car les repères `%s` et `%d` sont réservés à `Printf`
- [x] Le texte `Camille a 20 ans`, renvoyé par la fonction
- [ ] Le texte `%s a %d ans`, sans changement

> `%s` est remplacé par le premier argument (un texte) et `%d` par le second (un entier) ; `Sprintf` renvoie le résultat au lieu de l'afficher.
:::
