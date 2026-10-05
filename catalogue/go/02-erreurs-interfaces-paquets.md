---
id: erreurs-interfaces-paquets
title: "Erreurs, interfaces et paquets"
summary: "Gérer ce qui peut mal tourner avec les erreurs, décrire un comportement avec une interface et ranger le code en paquets."
minutes: 45
objectives:
  - Retourner, envelopper et tester une erreur avec `errors.Is` et `errors.As`
  - Expliquer ce qu'est une interface et pourquoi elle est « implicite » en Go
  - Découper un module en paquets et savoir ce qui est visible d'un paquet à l'autre
---

Une ludothèque prête des jeux : et si le jeu demandé est déjà sorti, ou si la personne a déjà emprunté deux jeux ? Un programme doit prévoir ces cas et le dire clairement, sans s'arrêter brutalement. Go a une réponse très particulière à cette question, et elle structure tout le code que tu liras dans l'équipe.

## À quoi ça sert, et pourquoi Go fait autrement

Dans beaucoup de langages, une erreur « saute » hors de la fonction (on parle d'**exception**) et un autre bout de code, parfois très loin, la rattrape. Go refuse ce mécanisme caché : une fonction qui peut échouer **renvoie l'erreur comme un résultat ordinaire**, et c'est à celui qui l'appelle de la regarder. Une fonction peut renvoyer plusieurs résultats : en Go, l'erreur vient en dernier.

```go
err := p.Preter("Catane", "Camille")
if err != nil {
    // quelque chose s'est mal passé : on réagit ici
}
```

`error` est un type fourni par Go. La valeur `nil` signifie « rien » (ici : « pas d'erreur »). Le test `err != nil` (`!=` veut dire « est différent de ») revient des centaines de fois dans le code de l'équipe : c'est normal, c'est le style de Go.

## Créer, envelopper et reconnaître une erreur

Voici un petit paquet `pret` qui gère les emprunts. Crée un dossier `pret/` dans ton module et un fichier `pret/pret.go` :

```go
// Package pret gère les emprunts de la ludothèque.
package pret

import (
	"errors"
	"fmt"
)

// ErrIndisponible est une erreur « sentinelle » : on la compare avec errors.Is.
var ErrIndisponible = errors.New("jeu indisponible")

// ErreurQuota est une erreur typée : on la récupère avec errors.As.
type ErreurQuota struct {
	Membre string
	Max    int
}

func (e *ErreurQuota) Error() string {
	return fmt.Sprintf("%s a déjà %d jeux empruntés", e.Membre, e.Max)
}

// Preteur est une interface : toute valeur qui a cette méthode la satisfait.
type Preteur interface {
	Preter(titre, membre string) error
}

// Stock est une implémentation en mémoire.
type Stock struct {
	dispo    map[string]bool
	emprunts map[string]int
}

func NouveauStock(titres ...string) *Stock {
	s := &Stock{dispo: map[string]bool{}, emprunts: map[string]int{}}
	for _, t := range titres {
		s.dispo[t] = true
	}
	return s
}

func (s *Stock) Preter(titre, membre string) error {
	if !s.dispo[titre] {
		return fmt.Errorf("prêt de %q : %w", titre, ErrIndisponible)
	}
	if s.emprunts[membre] >= 2 {
		return &ErreurQuota{Membre: membre, Max: 2}
	}
	s.dispo[titre] = false
	s.emprunts[membre]++
	return nil
}
```

Et le programme principal, dans `main.go` à la racine du module (celui-ci s'appelle `ludotheque`, d'après `go mod init ludotheque`) :

```go
package main

import (
	"errors"
	"fmt"

	"ludotheque/pret"
)

func main() {
	var p pret.Preteur = pret.NouveauStock("Catane", "Dixit", "Azul")

	fmt.Println(p.Preter("Catane", "Camille"))
	err := p.Preter("Catane", "Sam")
	fmt.Println(err)
	fmt.Println(errors.Is(err, pret.ErrIndisponible))

	_ = p.Preter("Dixit", "Camille")
	err = p.Preter("Azul", "Camille")

	var quota *pret.ErreurQuota
	if errors.As(err, &quota) {
		fmt.Println("quota atteint pour", quota.Membre)
	}
}
```

Avec `go run .`, tu obtiens :

```console
<nil>
prêt de "Catane" : jeu indisponible
true
quota atteint pour Camille
```

### Le stock, ligne à ligne

Avant les erreurs, trois détails de syntaxe nouveaux dans `NouveauStock` :

- `titres ...string` : les trois points devant le type rendent le paramètre **variadique**. On peut appeler la fonction avec autant de titres qu'on veut, séparés par des virgules (`NouveauStock("Catane", "Dixit", "Azul")`), voire aucun. À l'intérieur de la fonction, `titres` est une slice de `string` (la liste de la leçon 1) que l'on parcourt avec `for _, t := range titres`.
- `&Stock{dispo: map[string]bool{}, …}` : `Stock{…}` fabrique une structure, comme `Jeu{…}` dans la leçon 1. Le `&` devant donne **l'adresse** de cette structure, c'est-à-dire un pointeur (`*Stock`) vers elle, au lieu d'une copie. C'est ce que renvoie la fonction : tout le monde partage alors le même stock. Les méthodes `Preter` ont d'ailleurs un récepteur pointeur `(s *Stock)` pour pouvoir le modifier.
- `map[string]bool{}` et `map[string]int{}` : deux tables vides. `dispo` dit si chaque titre est disponible (`true`/`false`) ; `emprunts` compte combien de jeux chaque membre a empruntés. Lire une clé absente d'une map donne la valeur zéro (`false`, `0`), ce qui est exactement ce qu'on veut ici. `s.emprunts[membre]++` ajoute 1 au compteur.

Et dans `Error()` ou `Errorf`, les repères `%s` (un texte), `%d` (un entier) et `%q` (un texte entre guillemets) fonctionnent comme dans la leçon précédente avec `Sprintf`.

### Les erreurs, ligne à ligne

- `errors.New("jeu indisponible")` fabrique une erreur à partir d'un texte. Stockée dans une variable globale `ErrIndisponible`, elle sert de **repère** : tout le monde peut la comparer. Le projet `event-planner-api` fait pareil dans `api/event/service/errors.go` (`IllegalNameError`, `AlreadyJoinedError`…).
- `fmt.Errorf("prêt de %q : %w", …, ErrIndisponible)` crée une nouvelle erreur qui **enveloppe** l'ancienne : `%q` insère le titre entre guillemets, `%w` (*wrap*, « envelopper ») garde l'erreur d'origine à l'intérieur. On ajoute du contexte sans perdre l'identité de l'erreur.
- `errors.Is(err, pret.ErrIndisponible)` répond « cette erreur, ou l'une de celles qu'elle enveloppe, est-elle celle-ci ? ». Les handlers d'`event-planner-api` l'utilisent pour choisir un code de réponse (`errors.Is(err, eventservice.IllegalNameError)`).
- `ErreurQuota` est un type à part entière : il porte des informations (qui, quelle limite). Toute valeur qui a une méthode `Error() string` est une erreur. `errors.As(err, &quota)` cherche une erreur de ce type dans la chaîne et la range dans `quota` (le `&` donne l'adresse de `quota` pour que la fonction puisse la remplir). Le même motif apparaît dans `event_handler.go` avec `*ent.NotFoundError`.
- `_ = p.Preter("Dixit", "Camille")` : le `_` jette le résultat, ici parce qu'on sait que ce prêt-là réussit.

## Les interfaces

Une **interface** décrit un comportement par ses méthodes, sans dire comment il est réalisé. Pense à une prise électrique : peu importe l'appareil, s'il a la bonne fiche, il se branche.

Ici, `Preteur` exige une seule méthode, `Preter(titre, membre string) error` (`titre, membre string` est l'écriture courte de `titre string, membre string`). `*Stock` possède cette méthode, donc il **est** un `Preteur`. Il n'y a nulle part de mot-clé « implements » : l'interface est **implicite**, la correspondance des méthodes suffit. C'est pourquoi `var p pret.Preteur = pret.NouveauStock(…)` est accepté.

Pourquoi c'est utile ? On peut écrire du code qui dépend de `Preteur` et le tester avec un faux prêteur, ou changer l'implémentation (un stock en base de données) sans toucher au reste. Le projet `event-planner-api` suit ce schéma : l'interface `IEventService` liste les opérations sur les évènements, et `EventHandler` ne connaît que cette interface.

## Les paquets et les modules

Un **paquet** regroupe les fichiers d'un même dossier ; tous commencent par la même ligne `package pret`. Un **module** regroupe plusieurs paquets sous un même `go.mod`. Pour utiliser un paquet, on l'importe avec le nom du module suivi du chemin du dossier : `"ludotheque/pret"`.

```text
ludotheque/
├── go.mod          ← module ludotheque
├── main.go         ← package main
└── pret/
    └── pret.go     ← package pret
```

Dans `event-planner-api`, le `go.mod` commence par `module eventplanner` et le code importe donc `"eventplanner/ent"` ou `"eventplanner/utils"`. Pour utiliser un nom d'un autre paquet, on le préfixe : `pret.NouveauStock`. Et il faut qu'il commence par une majuscule, sinon le compilateur refuse : c'est la règle de visibilité vue dans la leçon précédente.

:::warning `panic` n'est pas une gestion d'erreur
Go a aussi `panic(err)`, qui arrête brutalement le programme. Il est réservé aux situations où continuer n'a aucun sens (une erreur de programmation, une configuration indispensable absente au démarrage). Dans `mgmt`, une fonction `he(err)` appelle `panic` dès qu'elle reçoit une erreur ; dans `event_service.go`, la création d'un évènement fait `panic(err)` si la base répond mal. Dans un service qui sert des centaines de personnes, une erreur de base de données devrait plutôt être renvoyée à l'appelant et traduite en réponse « 500 », pas faire tomber la requête (ni le service, selon la configuration).
:::

## Entraîne-toi

:::lab
engine: real
intro: |
  Tu travailles sur le paquet `pret` de la ludothèque (dossier `pret/`). Les types d'erreurs sont déjà dans `pret/pret.go` ; il manque `NouveauStock`, `Preter` et l'interface `Preteur`. Chaque morceau à écrire est dans son fichier, avec un commentaire qui décrit ce qu'il doit faire : remplace `panic("à écrire")` par ton code. Les tests sont dans `pret/pret_test.go` ; lance-les avec `go test ./...`. Les tests fournis sont vérifiés tels quels.
commands:
  - cp -R /opt/exercices/02-erreurs-interfaces/. .
  - go vet ./... >/dev/null 2>&1 || true
steps:
  - text: 'Écris `NouveauStock` dans `pret/nouveau.go` : un stock où tous les titres reçus (paramètre variadique) sont disponibles. `TestNouveauStock` doit passer'
    hint: 'Reprends le code de la leçon : `s := &Stock{dispo: map[string]bool{}, emprunts: map[string]int{}}`, une boucle `for _, t := range titres` qui fait `s.dispo[t] = true`, puis `return s`.'
    checks:
      - command-succeeds: "verifier-go tests 02-erreurs-interfaces pret TestNouveauStock"
    solution:
      - |
        cat > pret/nouveau.go <<'EOF'
        package pret

        // NouveauStock crée un stock où tous les titres donnés sont disponibles.
        func NouveauStock(titres ...string) *Stock {
        	s := &Stock{dispo: map[string]bool{}, emprunts: map[string]int{}}
        	for _, t := range titres {
        		s.dispo[t] = true
        	}
        	return s
        }
        EOF
  - text: 'Dans `pret/preter.go`, fais d''abord échouer `Preter` quand le titre n''est pas disponible : renvoie une erreur qui **enveloppe** `ErrIndisponible` et cite le titre avec `%q`. `TestIndisponible` doit passer'
    hint: 'Commence par `if !s.dispo[titre] { return fmt.Errorf("prêt de %q : %w", titre, ErrIndisponible) }`, puis `return nil` pour le reste. N''oublie pas `import "fmt"`.'
    after: [1]
    checks:
      - command-succeeds: "verifier-go tests 02-erreurs-interfaces pret TestIndisponible"
    solution:
      - |
        cat > pret/preter.go <<'EOF'
        package pret

        import "fmt"

        // Preter prête le titre au membre (première version : seulement le test de disponibilité).
        func (s *Stock) Preter(titre, membre string) error {
        	if !s.dispo[titre] {
        		return fmt.Errorf("prêt de %q : %w", titre, ErrIndisponible)
        	}
        	return nil
        }
        EOF
  - text: 'Complète `Preter` pour qu''un prêt réussi retire le titre du stock (`dispo` à `false`) et compte l''emprunt du membre. `TestPretReussi` doit passer'
    hint: 'Avant le `return nil` final : `s.dispo[titre] = false` puis `s.emprunts[membre]++`.'
    after: [2]
    checks:
      - command-succeeds: "verifier-go tests 02-erreurs-interfaces pret TestPretReussi"
    solution:
      - |
        cat > pret/preter.go <<'EOF'
        package pret

        import "fmt"

        // Preter prête le titre au membre.
        func (s *Stock) Preter(titre, membre string) error {
        	if !s.dispo[titre] {
        		return fmt.Errorf("prêt de %q : %w", titre, ErrIndisponible)
        	}
        	s.dispo[titre] = false
        	s.emprunts[membre]++
        	return nil
        }
        EOF
  - text: 'Ajoute la limite de **deux jeux par membre** : au troisième prêt, `Preter` renvoie un `*ErreurQuota` (avec `Membre` et `Max: 2`) et ne retire rien du stock. `TestQuota` doit passer'
    hint: 'Après le test de disponibilité : `if s.emprunts[membre] >= 2 { return &ErreurQuota{Membre: membre, Max: 2} }`.'
    after: [3]
    checks:
      - command-succeeds: "verifier-go tests 02-erreurs-interfaces pret TestQuota"
    solution:
      - |
        cat > pret/preter.go <<'EOF'
        package pret

        import "fmt"

        // Preter prête le titre au membre.
        func (s *Stock) Preter(titre, membre string) error {
        	if !s.dispo[titre] {
        		return fmt.Errorf("prêt de %q : %w", titre, ErrIndisponible)
        	}
        	if s.emprunts[membre] >= 2 {
        		return &ErreurQuota{Membre: membre, Max: 2}
        	}
        	s.dispo[titre] = false
        	s.emprunts[membre]++
        	return nil
        }
        EOF
  - text: 'Déclare l''interface `Preteur` (une seule méthode : `Preter(titre, membre string) error`) dans un nouveau fichier `pret/interface.go`. Puis lance `go run .` : `main.go` utilise ton interface et doit afficher `quota atteint pour Camille`'
    hint: 'Le fichier commence par `package pret`, puis `type Preteur interface { Preter(titre, membre string) error }`. Aucun `implements` : `*Stock` la satisfait déjà.'
    after: [1, 2, 3, 4]
    checks:
      - command-succeeds: "verifier-go lancer 02-erreurs-interfaces 'quota atteint pour Camille'"
    solution:
      - |
        cat > pret/interface.go <<'EOF'
        package pret

        // Preteur est une interface : toute valeur qui a cette méthode la satisfait.
        type Preteur interface {
        	Preter(titre, membre string) error
        }
        EOF
  - text: 'Vérifie l''ensemble : compile le programme dans un exécutable `prets` avec `go build -o prets .`, puis `go vet ./...` ne signale rien et `go test ./...` passe en entier'
    hint: 'Lance `go build -o prets . && go vet ./... && go test ./...`.'
    after: [1, 2, 3, 4, 5]
    checks:
      - command-succeeds: 'verifier-go binaire 02-erreurs-interfaces prets'
      - command-succeeds: 'verifier-go tout 02-erreurs-interfaces'
    solution:
      - go build -o prets . && go vet ./... && go test ./...
:::

## Vérifie tes acquis

:::quiz
Quel est l'intérêt de `%w` dans `fmt.Errorf("…%w", err)` ?

- [ ] Il affiche l'erreur en rouge dans le terminal
- [x] Il ajoute du contexte tout en gardant l'erreur d'origine reconnaissable par `errors.Is`
- [ ] Il transforme l'erreur en avertissement
- [ ] Il supprime l'erreur d'origine

> `%w` enveloppe l'erreur : `errors.Is` et `errors.As` la retrouvent en parcourant la chaîne.
:::

:::quiz
Un type `*Stock` possède une méthode `Preter(titre, membre string) error`. Que faut-il écrire pour qu'il soit accepté comme `Preteur` ?

- [ ] `type Stock implements Preteur`
- [ ] Rien d'autre qu'un import du paquet `interfaces`
- [x] Rien : avoir les bonnes méthodes suffit, l'interface est implicite
- [ ] Une méthode `Register()` appelée au démarrage

> En Go, un type satisfait une interface dès qu'il en possède toutes les méthodes.
:::

:::quiz
Dans un paquet `pret`, une fonction s'appelle `nouveauStock` (minuscule). Que se passe-t-il depuis `main` ?

- [ ] Elle est utilisable comme `pret.nouveauStock`
- [x] Le compilateur refuse : un nom en minuscule n'est pas visible hors de son paquet
- [ ] Elle est utilisable, mais seulement en lecture
- [ ] Go affiche un avertissement mais compile

> Seuls les noms qui commencent par une majuscule sont exportés.
:::

:::quiz
Que signifient les trois points dans `func NouveauStock(titres ...string)` ?

- [ ] Que la fonction renvoie une liste de `string`
- [x] Que la fonction accepte autant de titres qu'on veut, même aucun, et les reçoit sous forme de slice
- [ ] Que le dernier titre est facultatif et vaut `""` s'il manque
- [ ] Que la fonction est appelée en parallèle pour chaque titre

> Un paramètre variadique (`...type`) se comporte dans la fonction comme une slice de ce type.
:::
