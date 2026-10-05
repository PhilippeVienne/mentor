---
id: base-de-donnees
title: "Accéder à une base de données"
summary: "Garder les données quand le programme s'arrête : cacher la base derrière une interface, utiliser le contexte et lire un schéma Ent."
minutes: 45
objectives:
  - Expliquer pourquoi un service a besoin d'une base de données et ce qu'est un ORM
  - Écrire une interface de dépôt (repository) avec un `context.Context`
  - Lire un schéma Ent et une requête générée dans `event-planner-api`
  - Repérer ce qui change entre Ent (PostgreSQL) et le MongoDB d'`billetterie`
---

Le serveur de la leçon 3 oublie tout dès qu'on l'arrête : ses données vivent dans la mémoire du programme. Pour une liste d'adhérent·e·s ou d'évènements, ce n'est pas acceptable. Il faut ranger les données dans une **base de données**, un programme séparé dont c'est le métier de les conserver.

## À quoi ça sert, et ce que font les projets de l'équipe

Une base de données stocke des informations de façon durable et permet de les retrouver vite. Il en existe de deux grandes familles : les bases **relationnelles** (des tables avec des lignes et des colonnes, interrogées en langage SQL, comme PostgreSQL) et les bases **documents** (des fiches JSON, comme MongoDB). Les trois projets Go de l'équipe ne font pas le même choix. Dans le tableau, un **pilote** (*driver*) est la bibliothèque qui sait « parler » à un type de base de données précis :

| Projet | Où sont les données ? | Outil Go |
| --- | --- | --- |
| `mgmt` | En mémoire seulement (listes de connexions, configuration) : tout est perdu au redémarrage | aucun |
| `event-planner-api` | PostgreSQL, via le pilote `pgx` | **Ent** (ORM) |
| `billetterie` | MongoDB, pour les fiches d'adhérent·e·s | `mogo` (au-dessus de `mgo`) |

Un **ORM** (*Object-Relational Mapping*) est une bibliothèque qui te laisse manipuler des lignes de base de données comme des structures du langage (la fiche `Jeu` de la leçon 1), sans écrire de SQL à la main. Les bibliothèques `mogo` et `mgo` d'`billetterie` datent de l'époque de Go 1.14 (son `go.mod` l'indique) : il faut les considérer comme anciennes avant de les réutiliser dans un nouveau projet.

## Cacher la base derrière une interface

Comme dans la leçon 2, on décrit ce dont le programme a besoin dans une interface, sans dire comment c'est fait. On ajoute aussi un **contexte** (`context.Context`) : un objet qu'on passe de fonction en fonction et qui transporte un délai maximal ou un signal d'annulation. Si la personne ferme sa page ou si la requête dépasse deux secondes, le contexte est annulé et la base n'a plus besoin de continuer.

```go
package main

import (
	"context"
	"errors"
	"fmt"
	"time"
)

type Jeu struct {
	ID    int
	Titre string
}

var ErrIntrouvable = errors.New("jeu introuvable")

// DepotJeux cache la base de données derrière une interface.
// Le contexte permet d'annuler la requête (client parti, délai dépassé).
type DepotJeux interface {
	Creer(ctx context.Context, titre string) (*Jeu, error)
	Trouver(ctx context.Context, id int) (*Jeu, error)
}

// depotMemoire sert aux tests et aux démonstrations.
type depotMemoire struct {
	jeux    map[int]*Jeu
	suivant int
}

func nouveauDepotMemoire() *depotMemoire {
	return &depotMemoire{jeux: map[int]*Jeu{}, suivant: 1}
}

func (d *depotMemoire) Creer(ctx context.Context, titre string) (*Jeu, error) {
	if err := ctx.Err(); err != nil {
		return nil, err
	}
	j := &Jeu{ID: d.suivant, Titre: titre}
	d.jeux[j.ID] = j
	d.suivant++
	return j, nil
}

func (d *depotMemoire) Trouver(ctx context.Context, id int) (*Jeu, error) {
	if err := ctx.Err(); err != nil {
		return nil, err
	}
	j, ok := d.jeux[id]
	if !ok {
		return nil, fmt.Errorf("jeu %d : %w", id, ErrIntrouvable)
	}
	return j, nil
}

func main() {
	var depot DepotJeux = nouveauDepotMemoire()

	ctx, annule := context.WithTimeout(context.Background(), 2*time.Second)
	defer annule()

	j, err := depot.Creer(ctx, "Azul")
	if err != nil {
		panic(err)
	}
	fmt.Println("créé :", j.ID, j.Titre)

	if _, err := depot.Trouver(ctx, 42); errors.Is(err, ErrIntrouvable) {
		fmt.Println("erreur attendue :", err)
	}

	annule() // on annule : la suite échoue immédiatement
	_, err = depot.Trouver(ctx, 1)
	fmt.Println(errors.Is(err, context.Canceled))
}
```

```console
créé : 1 Azul
erreur attendue : jeu 42 : jeu introuvable
true
```

Ce code se lit ainsi :

- `type DepotJeux interface { … }` est une interface (leçon 2) : elle liste les opérations dont le reste du programme a besoin. `(*Jeu, error)` signifie que `Creer` renvoie deux résultats, un pointeur vers le jeu créé et une erreur. `ctx context.Context` est le premier paramètre de chaque méthode, par convention.
- `depotMemoire` garde les jeux dans une map `map[int]*Jeu` (le numéro vers un pointeur sur le jeu) et retient dans `suivant` le numéro du prochain jeu. `nouveauDepotMemoire` la prépare avec une map vide et `suivant` à 1.
- Dans `Creer`, `&Jeu{ID: d.suivant, Titre: titre}` fabrique le jeu et en garde l'adresse (leçon 2), puis `d.suivant++` ajoute 1 au compteur : le jeu suivant aura le numéro suivant.
- Dans `Trouver`, `j, ok := d.jeux[id]` est le motif « valeur et `ok` » de la leçon 1 : si le numéro n'existe pas, on renvoie une erreur qui enveloppe `ErrIntrouvable` (leçon 2) avec `%w`.
- Dans `main`, `2*time.Second` vaut deux secondes ; `panic(err)` arrête le programme, acceptable dans une démonstration.
- `context.Background()` est le contexte vide, point de départ ; `context.WithTimeout(…, 2*time.Second)` en fabrique un qui s'annule tout seul après deux secondes. `annule` est la fonction qui l'annule à la main. On la diffère avec `defer annule()` pour libérer les ressources.
- `ctx.Err()` vaut `nil` tant que tout va bien, puis `context.Canceled` ou `context.DeadlineExceeded` une fois annulé. Un vrai dépôt SQL passe `ctx` à la base, qui interrompt alors la requête.
- Ici, `depotMemoire` remplace la vraie base : un test (leçon 7) peut ainsi tourner sans PostgreSQL. Le jour où l'on branche la vraie base, on écrit une autre structure qui offre les mêmes méthodes, et `main` ne change presque pas.

## Ne pas écrire les secrets dans le code

Pour se connecter à la vraie base, il faut un nom d'utilisateur et un mot de passe. On ne les écrit **jamais** dans le code (il est versionné et lu par toute l'équipe) : on les lit dans des **variables d'environnement**, des valeurs fournies au programme par le système au moment où on le lance. Voici une fonction qui compose l'adresse de la base :

```go
func ConfigDepuisEnv() (string, error) {
	hote := os.Getenv("DB_HOST")
	if hote == "" {
		hote = "localhost"
	}
	utilisateur := os.Getenv("DB_USER")
	motDePasse := os.Getenv("DB_PASSWORD")
	if utilisateur == "" || motDePasse == "" {
		return "", errors.New("DB_USER et DB_PASSWORD sont obligatoires")
	}
	return fmt.Sprintf("postgres://%s:%s@%s/ludotheque", utilisateur, motDePasse, hote), nil
}
```

`os.Getenv("DB_HOST")` lit une variable (texte vide si elle n'existe pas). `||` signifie « ou » : l'erreur est renvoyée si l'un des deux est vide. Le nom du serveur a une valeur par défaut raisonnable (`localhost`), mais **pas le mot de passe** : sans lui, le programme ne doit pas démarrer.

## Lire un schéma Ent

**Ent** est un ORM qui part d'une description des données en Go et **génère** le code d'accès. Dans `event-planner-api`, le dossier `ent/schema/` contient une structure par table. Voici celle des évènements (`ent/schema/event.go`), réduite à ses champs :

```go
// Fields of the Event.
func (Event) Fields() []ent.Field {
	return []ent.Field{
		field.String("name"),
		field.String("description").Optional(),
		field.Time("start_date"),
		field.Time("end_date"),
		field.Time("invite_start_date").Optional(),
		field.Time("invite_end_date").Optional(),
		field.String("invite_token").Optional(),
	}
}
```

- `field.String("name")` déclare une colonne de texte obligatoire ; `.Optional()` la rend facultative.
- Les **liens** entre tables (*edges*) sont décrits à côté : `edge.To("manager", User.Type).Unique().Required()` dit qu'un évènement a exactement un responsable, qui est un `User`.
- Le fichier `ent/generate.go` contient la ligne `//go:generate go run -mod=mod entgo.io/ent/cmd/ent generate ./schema`. La commande `go generate ./ent` la lance et crée tout le code : `client.Event.Create()`, `client.User.Query()`, etc. Ce code généré n'est pas dans le dépôt : le Dockerfile exécute `RUN go generate ./ent` avant de compiler.

Une fois le code généré, on interroge la base avec des méthodes qui s'enchaînent. Voici l'extrait de `start.go`, qui cherche une permission et la crée si elle n'existe pas :

```go
_, err := client.UserPermission.Query().Where(userpermission.Name(permission)).Only(context.Background())
if err != nil {
	if ent.IsNotFound(err) {
		_, err := client.UserPermission.Create().
			SetName(permission).
			Save(context.Background())
		// …
	}
}
```

- `client.UserPermission.Query()` démarre une recherche dans la table des permissions ;
- `.Where(userpermission.Name(permission))` filtre sur la colonne `name` ;
- `.Only(ctx)` attend **exactement une** ligne : zéro ligne donne une erreur « introuvable », que `ent.IsNotFound(err)` reconnaît (le même motif que `errors.Is` de la leçon 2) ;
- `.Create().SetName(…).Save(ctx)` insère une ligne.

Pour **faire évoluer** la structure de la base sans tout perdre, le projet garde des fichiers de **migration** (des scripts SQL datés) dans `ent/migrate/migrations/`, par exemple `20240901230403_init.sql`, accompagnés d'un `atlas.sum` qui protège leur intégrité (Atlas est l'outil qui gère ces migrations).

:::info Retour sur les secrets
Dans `event-planner-api`, l'adresse de la base se compose à partir de variables d'environnement (`DB_USER`, `DB_PASSWORD`…) lues avec `utils.GetEnv`, et un fichier `.env.example` montre les noms attendus. C'est la bonne pratique : jamais de mot de passe écrit dans le code ni dans le dépôt.
:::

## Entraîne-toi

:::lab
engine: real
intro: |
  Tu écris un dépôt de jeux en mémoire qui respecte l'interface `DepotJeux`. `depot.go` (l'interface, la structure et l'erreur `ErrIntrouvable`) est déjà écrit. Les morceaux à écrire sont chacun dans leur fichier : `creer.go`, `trouver.go`, `lister.go` et `config.go`. Remplace `panic("à écrire")`. Les tests sont dans `depot_test.go` ; lance-les avec `go test ./...`. Aucune vraie base n'est utilisée : c'est justement l'intérêt de l'interface. Les tests fournis sont vérifiés tels quels.
commands:
  - cp -R /opt/exercices/06-base-de-donnees/. .
  - go vet ./... >/dev/null 2>&1 || true
steps:
  - text: 'Écris `Creer` dans `creer.go` : elle enregistre le jeu sous le prochain numéro (1, puis 2…) et le renvoie. `TestCreer` doit passer'
    hint: 'Crée `j := &Jeu{ID: d.suivant, Titre: titre}`, range-le avec `d.jeux[j.ID] = j`, ajoute 1 avec `d.suivant++` et renvoie `j, nil`.'
    checks:
      - command-succeeds: "verifier-go tests 06-base-de-donnees . TestCreer"
    solution:
      - |
        cat > creer.go <<'EOF'
        package main

        import "context"

        // Creer enregistre un jeu avec le prochain numéro et le renvoie.
        func (d *depotMemoire) Creer(ctx context.Context, titre string) (*Jeu, error) {
        	j := &Jeu{ID: d.suivant, Titre: titre}
        	d.jeux[j.ID] = j
        	d.suivant++
        	return j, nil
        }
        EOF
  - text: 'Écris `Trouver` dans `trouver.go` : elle renvoie le jeu de ce numéro, ou une erreur qui **enveloppe** `ErrIntrouvable` et cite le numéro (« jeu 42 : jeu introuvable »). `TestTrouver` doit passer'
    hint: 'Utilise `j, ok := d.jeux[id]` ; si `!ok`, renvoie `nil, fmt.Errorf("jeu %d : %w", id, ErrIntrouvable)`.'
    after: [1]
    checks:
      - command-succeeds: "verifier-go tests 06-base-de-donnees . TestTrouver"
    solution:
      - |
        cat > trouver.go <<'EOF'
        package main

        import (
        	"context"
        	"fmt"
        )

        // Trouver renvoie le jeu qui a ce numéro.
        func (d *depotMemoire) Trouver(ctx context.Context, id int) (*Jeu, error) {
        	j, ok := d.jeux[id]
        	if !ok {
        		return nil, fmt.Errorf("jeu %d : %w", id, ErrIntrouvable)
        	}
        	return j, nil
        }
        EOF
  - text: 'Respecte le **contexte** : au début de `Creer` et de `Trouver`, si `ctx.Err()` n''est pas `nil`, renvoie cette erreur sans rien faire. `TestContexte` doit passer'
    hint: 'Première instruction de chaque méthode : `if err := ctx.Err(); err != nil { return nil, err }`.'
    after: [1, 2]
    checks:
      - command-succeeds: "verifier-go tests 06-base-de-donnees . TestContexte"
    solution:
      - |
        cat > creer.go <<'EOF'
        package main

        import "context"

        // Creer enregistre un jeu avec le prochain numéro et le renvoie.
        func (d *depotMemoire) Creer(ctx context.Context, titre string) (*Jeu, error) {
        	if err := ctx.Err(); err != nil {
        		return nil, err
        	}
        	j := &Jeu{ID: d.suivant, Titre: titre}
        	d.jeux[j.ID] = j
        	d.suivant++
        	return j, nil
        }
        EOF
        cat > trouver.go <<'EOF'
        package main

        import (
        	"context"
        	"fmt"
        )

        // Trouver renvoie le jeu qui a ce numéro.
        func (d *depotMemoire) Trouver(ctx context.Context, id int) (*Jeu, error) {
        	if err := ctx.Err(); err != nil {
        		return nil, err
        	}
        	j, ok := d.jeux[id]
        	if !ok {
        		return nil, fmt.Errorf("jeu %d : %w", id, ErrIntrouvable)
        	}
        	return j, nil
        }
        EOF
  - text: 'Écris `Lister` dans `lister.go` : tous les jeux, triés par numéro croissant. `TestLister` doit passer'
    hint: 'Rassemble les jeux de `d.jeux` dans une slice (`append`), puis trie-la avec `sort.Slice(res, func(i, k int) bool { return res[i].ID < res[k].ID })` : la fonction anonyme dit si l''élément `i` doit passer avant l''élément `k`. L''ordre d''une map est aléatoire, c''est pour cela qu''il faut trier.'
    after: [1]
    checks:
      - command-succeeds: "verifier-go tests 06-base-de-donnees . TestLister"
    solution:
      - |
        cat > lister.go <<'EOF'
        package main

        import (
        	"context"
        	"sort"
        )

        // Lister renvoie tous les jeux, triés par numéro croissant.
        func (d *depotMemoire) Lister(ctx context.Context) ([]*Jeu, error) {
        	var res []*Jeu
        	for _, j := range d.jeux {
        		res = append(res, j)
        	}
        	sort.Slice(res, func(i, k int) bool { return res[i].ID < res[k].ID })
        	return res, nil
        }
        EOF
  - text: 'Écris `ConfigDepuisEnv` dans `config.go` : l''adresse de la base se compose avec `DB_HOST` (`localhost` par défaut), `DB_USER` et `DB_PASSWORD` (obligatoires, sinon une erreur). `TestConfigDepuisEnv` doit passer'
    hint: 'Reprends la fonction de la leçon : `os.Getenv`, un `if` pour la valeur par défaut, un `if utilisateur == "" || motDePasse == ""` pour l''erreur, et `fmt.Sprintf("postgres://%s:%s@%s/ludotheque", …)`.'
    checks:
      - command-succeeds: "verifier-go tests 06-base-de-donnees . TestConfigDepuisEnv"
    solution:
      - |
        cat > config.go <<'EOF'
        package main

        import (
        	"errors"
        	"fmt"
        	"os"
        )

        // ConfigDepuisEnv compose l'adresse de connexion à la base à partir des variables d'environnement.
        func ConfigDepuisEnv() (string, error) {
        	hote := os.Getenv("DB_HOST")
        	if hote == "" {
        		hote = "localhost"
        	}
        	utilisateur := os.Getenv("DB_USER")
        	motDePasse := os.Getenv("DB_PASSWORD")
        	if utilisateur == "" || motDePasse == "" {
        		return "", errors.New("DB_USER et DB_PASSWORD sont obligatoires")
        	}
        	return fmt.Sprintf("postgres://%s:%s@%s/ludotheque", utilisateur, motDePasse, hote), nil
        }
        EOF
  - text: 'Compile le programme avec `go build -o depot .`, puis lance-le avec `./depot` : il doit lister « 1 Azul » et « 2 Catane », puis afficher l''erreur attendue. `go vet ./...` et `go test ./...` passent aussi'
    hint: 'Lance `go build -o depot .` puis `./depot`, et enfin `go vet ./... && go test ./...`.'
    after: [1, 2, 3, 4, 5]
    checks:
      - command-succeeds: 'verifier-go binaire 06-base-de-donnees depot'
      - command-succeeds: "verifier-go lancer 06-base-de-donnees '^2 Catane$'"
      - command-succeeds: 'verifier-go tout 06-base-de-donnees'
    solution:
      - go build -o depot .
      - ./depot
:::

## Vérifie tes acquis

:::quiz
Pourquoi `mgmt` perd-il son état quand on le redémarre ?

- [ ] Parce qu'il utilise MongoDB
- [ ] Parce que Go efface les variables globales à l'arrêt
- [x] Parce qu'il garde ses listes de connexions uniquement dans la mémoire du programme
- [ ] Parce qu'il n'utilise pas de contexte

> Sans base de données, tout ce qui est stocké dans des variables disparaît avec le processus.
:::

:::quiz
À quoi sert un `context.Context` passé à une fonction qui interroge la base ?

- [ ] À choisir le type de base de données
- [ ] À chiffrer la requête
- [ ] À stocker les résultats en cache
- [x] À pouvoir annuler l'opération ou lui imposer un délai maximal

> Le contexte transporte annulation et délais ; une requête abandonnée n'a plus à occuper la base.
:::

:::quiz
Que fait `client.UserPermission.Query().Where(…).Only(ctx)` quand aucune ligne ne correspond ?

- [ ] Il renvoie une ligne vide sans erreur
- [x] Il renvoie une erreur que `ent.IsNotFound` reconnaît
- [ ] Il crée automatiquement la ligne
- [ ] Il arrête le programme

> `Only` exige exactement un résultat : zéro ou plusieurs donnent une erreur.
:::

:::quiz
Pourquoi le Dockerfile d'`event-planner-api` exécute-t-il `go generate ./ent` ?

- [ ] Pour télécharger PostgreSQL
- [ ] Pour lancer les tests
- [x] Pour produire le code Ent à partir des schémas, avant la compilation
- [ ] Pour créer les tables dans la base

> Le code d'accès (`client.Event`, `client.User`…) est généré à partir de `ent/schema/`.
:::
