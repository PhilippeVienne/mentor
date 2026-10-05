---
id: tests-docker
title: "Tests et compilation dans une image Docker minimale"
summary: "Vérifier ton code avec go test et go vet, puis le livrer dans une image Docker légère construite en deux étapes."
minutes: 50
objectives:
  - Écrire un test avec le paquet `testing` et `httptest`, en tableau de cas
  - Lancer `go vet` et `go test ./...` et lire leur résultat
  - Expliquer un Dockerfile en deux étapes et pourquoi `CGO_ENABLED=0` aide
  - Repérer les différences entre les Dockerfile de l'équipe et une version moderne
---

Tu as écrit un serveur qui marche « sur ta machine ». Deux questions restent : comment être sûr·e qu'il marchera encore après ta prochaine modification, et comment le livrer à l'équipe d'infrastructure ? La première question se règle avec des **tests**, la seconde avec une **image Docker**. Docker est un outil qui range un programme et tout ce dont il a besoin dans une « boîte » standard, l'**image**, qu'on peut lancer à l'identique sur n'importe quel serveur ; un **conteneur** est une image en train de tourner.

## À quoi ça sert, et pourquoi

Un **test automatique** est un petit programme qui appelle ton code avec des cas connus et vérifie le résultat. Tu le relances à chaque changement : s'il devient rouge, tu sais tout de suite que tu as cassé quelque chose, avant que quelqu'un d'autre ne le découvre. Go fournit tout ce qu'il faut dans la bibliothèque standard, sans outil à installer.

Une **image Docker** est cette boîte : elle contient ton programme et tout ce qu'il lui faut pour tourner. Dans l'équipe, c'est ce que l'équipe Infra déploie. La **CI** (*intégration continue*) est un robot qui, à chaque modification du code, lance les tests et construit l'image ; le parcours *CI/CD avec GitLab* explique comment ce pipeline est construit.

## Écrire un test

Par convention, un fichier de test se termine par `_test.go` et vit à côté du code testé. Une fonction de test s'appelle `TestQuelqueChose` et reçoit un `*testing.T`, l'objet qui sert à signaler un échec. Voici un test du serveur de la leçon 3 (fichier `main_test.go`, même dossier que `main.go`). Il utilise deux notions de Go à connaître : une **structure anonyme** (`struct{ … }` écrite sans lui donner de nom, pratique pour un type utilisé une seule fois) et une **fonction anonyme** (leçon 3) passée à `t.Run` :

```go
package main

import (
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"
)

func TestAjoute(t *testing.T) {
	cas := []struct {
		nom    string
		corps  string
		statut int
	}{
		{"jeu valide", `{"titre":"Azul","joueurs":4}`, http.StatusCreated},
		{"titre manquant", `{"joueurs":4}`, http.StatusBadRequest},
		{"JSON cassé", `{"titre":`, http.StatusBadRequest},
	}
	for _, c := range cas {
		t.Run(c.nom, func(t *testing.T) {
			s := &serveur{}
			req := httptest.NewRequest("POST", "/jeux", strings.NewReader(c.corps))
			rec := httptest.NewRecorder()

			s.ajoute(rec, req)

			if rec.Code != c.statut {
				t.Errorf("statut = %d, attendu %d", rec.Code, c.statut)
			}
		})
	}
}
```

- `cas := []struct{ nom string; corps string; statut int }{ … }` est une slice (leçon 1) dont chaque élément est une petite structure sans nom. C'est une liste de **cas de test** : un nom, un corps de requête à envoyer, le statut attendu. Les apostrophes inversées autour du JSON permettent d'y écrire des guillemets sans les protéger. Cette forme, appelée **test en tableau**, est la plus courante en Go : pour ajouter un cas, on ajoute une ligne.
- `httptest.NewRequest` fabrique une fausse requête, **sans ouvrir de port réseau**. `httptest.NewRecorder` joue le rôle de `w` et enregistre ce que le handler écrit (le statut dans `rec.Code`, le corps dans `rec.Body`).
- `s.ajoute(rec, req)` appelle directement le handler, comme le ferait le routeur.
- `t.Run(c.nom, func(t *testing.T) { … })` lance un sous-test par cas : Go appelle la fonction anonyme que tu lui passes, et en cas d'échec il indique lequel. `strings.NewReader` fabrique un « corps de requête » à partir d'un texte.
- `t.Errorf` note l'échec et continue ; `t.Fatalf` note l'échec et arrête ce test. Comme `Sprintf`, ils acceptent les repères `%d` (entier) et `%s` (texte).

Lance tous les tests du module (`./...` veut dire « ce dossier et tous ses sous-dossiers »), puis l'analyseur statique :

```bash
go test ./...
go vet ./...
```

```console
ok  	ludotheque	0.002s
```

`go vet` ne lance pas ton code : il le **lit** et signale des constructions suspectes (un `Printf` avec un mauvais format, par exemple). Ajoute `-v` à `go test` pour voir chaque test et sous-test.

:::warning Un test qui échoue toujours ne protège de rien
Le projet `billetterie` contient un fichier `main_test.go` dont la fonction `Test0` se réduit à `t.FailNow()`. C'est un gabarit laissé en l'état : `go test` échouerait si on le lançait. Dans les fichiers `.gitlab-ci.yml` de `mgmt` et d'`billetterie`, nous n'avons pas trouvé de job qui lance `go test` (le stage `test` n'y contient que les analyses de sécurité incluses par modèle). Écrire de vrais tests et les brancher à la CI serait un bon premier chantier.
:::

## Livrer dans une image Docker légère

Un programme Go se compile en un fichier unique. L'image finale n'a donc besoin ni du compilateur ni des sources : seulement de ce fichier. Pour y arriver, on utilise un Dockerfile **en deux étapes** (*multi-stage*) : la première compile, la seconde ne garde que le résultat.

```dockerfile
# Étape 1 : compiler (image lourde, avec tout Go)
FROM golang:1.25 AS build
WORKDIR /app
COPY go.mod ./
RUN go mod download
COPY *.go ./
RUN CGO_ENABLED=0 go build -o /ludotheque

# Étape 2 : l'image finale ne contient que le programme
FROM alpine:3.22
RUN adduser -D appli
USER appli
COPY --from=build /ludotheque /ludotheque
EXPOSE 8080
ENTRYPOINT ["/ludotheque"]
```

Ligne à ligne :

- `FROM golang:1.25 AS build` démarre à partir d'une image qui contient Go (le nombre après les deux-points est le **tag**, ici la version) et la nomme `build`.
- `WORKDIR /app` fixe le dossier de travail ; `COPY go.mod ./` puis `RUN go mod download` téléchargent les dépendances **avant** de copier le code : tant que `go.mod` ne change pas, Docker réutilise cette étape en cache et accélère les constructions suivantes. (Si ton projet a des dépendances, copie aussi `go.sum`.)
- `COPY *.go ./` copie les sources, `RUN CGO_ENABLED=0 go build -o /ludotheque` compile. `CGO_ENABLED=0` demande un programme **100 % Go**, qui ne dépend d'aucune bibliothèque du système : il tourne donc dans n'importe quelle image, même minuscule.
- `FROM alpine:3.22` démarre une seconde image, très petite. **Alpine** est une distribution Linux (un système d'exploitation) volontairement minimaliste : quelques Mo seulement, juste de quoi lancer un programme. `RUN adduser -D appli` crée un utilisateur nommé `appli`, et `USER appli` fait tourner le programme sous cet utilisateur sans pouvoir particulier, au lieu de `root` (l'administrateur, qui peut tout faire : un programme compromis ne doit pas l'être).
- `COPY --from=build /ludotheque /ludotheque` ne récupère **que** le programme compilé.
- `EXPOSE 8080` documente le port ; `ENTRYPOINT` indique la commande lancée au démarrage.

L'image finale ne pèse que quelques dizaines de Mo (Alpine plus ton programme), alors que l'image `golang` seule en pèse plusieurs centaines : c'est ce qu'on gagne avec les deux étapes. Pour la construire, on lancerait `docker build -t ludotheque .` ; ce conteneur de labo n'a pas Docker, tu écriras donc seulement le Dockerfile.

## Les Dockerfile de l'équipe

Les trois projets n'ont pas le même niveau :

| Projet | Étapes | Image de départ | Remarque |
| --- | --- | --- | --- |
| `event-planner-api` | 2 (`build-stage`, `build-release-stage`) | `golang:1.23.0` puis `alpine:3.12.12` | Image finale légère. Alpine 3.12 est une version ancienne, à mettre à jour. |
| `billetterie` | 2 | `golang:1.14` puis `alpine:3.11` | Idem, avec des images très anciennes. |
| `mgmt` | 1 | `golang:1.16` | Le second étage `alpine` est présent mais **mis en commentaire** : l'image livrée contient donc tout Go. |

Les Dockerfile de `mgmt` et `billetterie` compilent avec `go build -a -installsuffix cgo`, des options héritées d'anciennes versions de Go qui ne sont plus utiles avec `CGO_ENABLED=0`. Ils lancent aussi `go mod tidy` dans l'image, ce qui peut modifier les dépendances à la construction ; avec `go mod download` et un `go.sum` versionné, la construction est plus prévisible.

:::tip Mettre à jour Go
Passer le `go.mod` de `mgmt` de 1.16 à une version récente demande peu de changement de code, mais Macaron (version 1.3.5 dans son `go.mod`) mérite une décision à part : regarde d'abord sur le dépôt du framework s'il est toujours maintenu. Teste avec `go test ./...` et `go vet ./...` avant et après chaque étape.
:::

## Entraîne-toi

:::lab
engine: real
intro: |
  Tu reprends le serveur de la leçon 3 (`serveur.go` et `main.go`), sans aucun test. À toi de le tester puis de préparer sa livraison. Les tests vont dans un fichier `main_test.go` que tu crées toi-même (copie-colle l'exemple de la leçon et complète-le). Ce conteneur n'a pas Docker : tu écris le `Dockerfile`, mais tu ne le construis pas. Tes tests doivent réussir sur le serveur correct **et** échouer sur une version défectueuse : ils seront essayés sur les deux.
commands:
  - cp -R /opt/exercices/07-tests-docker/. .
  - go vet ./... >/dev/null 2>&1 || true
steps:
  - text: 'Crée `main_test.go` avec un test `TestAjoute` **en tableau** d''au moins trois cas (jeu valide `201`, titre manquant `400`, JSON cassé `400`) lancés avec `t.Run`. `go test -v` doit afficher trois sous-tests réussis'
    hint: 'Reprends l''exemple de la leçon : `package main`, les imports (`net/http`, `net/http/httptest`, `strings`, `testing`) et la fonction `TestAjoute`. Lance ensuite `go test -v ./...`.'
    checks:
      - command-succeeds: 'verifier-go tests-ecrits 07-tests-docker TestAjoute 3'
    solution:
      - |
        cat > main_test.go <<'EOF'
        package main

        import (
        	"net/http"
        	"net/http/httptest"
        	"strings"
        	"testing"
        )

        func TestAjoute(t *testing.T) {
        	cas := []struct {
        		nom    string
        		corps  string
        		statut int
        	}{
        		{"jeu valide", `{"titre":"Azul","joueurs":4}`, http.StatusCreated},
        		{"titre manquant", `{"joueurs":4}`, http.StatusBadRequest},
        		{"JSON cassé", `{"titre":`, http.StatusBadRequest},
        	}
        	for _, c := range cas {
        		t.Run(c.nom, func(t *testing.T) {
        			s := &serveur{}
        			req := httptest.NewRequest("POST", "/jeux", strings.NewReader(c.corps))
        			rec := httptest.NewRecorder()

        			s.ajoute(rec, req)

        			if rec.Code != c.statut {
        				t.Errorf("statut = %d, attendu %d", rec.Code, c.statut)
        			}
        		})
        	}
        }
        EOF
  - text: 'Dans un second fichier `liste_test.go`, écris `TestListe` : ajoute un jeu à un `serveur`, appelle `liste` avec un `httptest.NewRecorder()`, et vérifie le statut `200` et la présence du titre dans `rec.Body.String()`. `go test -v` doit afficher `--- PASS: TestListe`'
    hint: 'Fabrique `s := &serveur{jeux: []Jeu{{Titre: "Azul", Joueurs: 4}}}`, appelle `s.liste(rec, httptest.NewRequest("GET", "/jeux", nil))`, puis teste `rec.Code` et `strings.Contains(rec.Body.String(), "Azul")`.'
    checks:
      - command-succeeds: 'verifier-go tests-ecrits 07-tests-docker TestListe 0'
    solution:
      - |
        cat > liste_test.go <<'EOF'
        package main

        import (
        	"net/http/httptest"
        	"strings"
        	"testing"
        )

        func TestListe(t *testing.T) {
        	s := &serveur{jeux: []Jeu{{Titre: "Azul", Joueurs: 4}}}
        	rec := httptest.NewRecorder()

        	s.liste(rec, httptest.NewRequest("GET", "/jeux", nil))

        	if rec.Code != 200 {
        		t.Errorf("statut = %d, attendu 200", rec.Code)
        	}
        	if !strings.Contains(rec.Body.String(), "Azul") {
        		t.Errorf("le corps %q devrait contenir Azul", rec.Body.String())
        	}
        }
        EOF
  - text: 'Lance `go vet ./...` : il signale un défaut dans `serveur.go` (un mutex copié par un récepteur « valeur »). Corrige-le pour que `go vet ./...` ne dise plus rien'
    hint: 'Lis le message de `go vet` : la méthode `liste` a un récepteur `(s serveur)` qui copie le mutex. Passe-la en `(s *serveur)`, comme `ajoute`.'
    checks:
      - command-succeeds: 'verifier-go vet 07-tests-docker'
    solution:
      - sed -i 's/func (s serveur) liste/func (s *serveur) liste/' serveur.go
  - text: 'Compile un exécutable **100 % Go** nommé `ludotheque` avec `CGO_ENABLED=0 go build -o ludotheque .`. Le fichier `ludotheque` doit exister'
    hint: 'Tape exactement `CGO_ENABLED=0 go build -o ludotheque .` (dans ce labo, `CGO_ENABLED=0` est déjà le réglage par défaut, mais il faut savoir l''écrire).'
    after: [3]
    checks:
      - command-succeeds: 'verifier-go binaire 07-tests-docker ludotheque'
    solution:
      - CGO_ENABLED=0 go build -o ludotheque .
  - text: 'Écris un `Dockerfile` **en deux étapes** : une première étape nommée `build` (`FROM golang:1.25 AS build`) qui compile avec `CGO_ENABLED=0`, une seconde sur `alpine:3.22` qui copie le programme avec `COPY --from=build` et le lance avec un utilisateur non-root (`USER`)'
    hint: 'Reprends le Dockerfile de la leçon. Il doit contenir deux lignes `FROM`, `CGO_ENABLED=0`, `COPY --from=build` et `USER appli`.'
    checks:
      - command-succeeds: 'verifier-go dockerfile 07-tests-docker'
    solution:
      - |-
        cat > Dockerfile <<'EOF'
        # Étape 1 : compiler (image lourde, avec tout Go)
        FROM golang:1.25 AS build
        WORKDIR /app
        COPY go.mod ./
        RUN go mod download
        COPY *.go ./
        RUN CGO_ENABLED=0 go build -o /ludotheque

        # Étape 2 : l'image finale ne contient que le programme
        FROM alpine:3.22
        RUN adduser -D appli
        USER appli
        COPY --from=build /ludotheque /ludotheque
        EXPOSE 8080
        ENTRYPOINT ["/ludotheque"]
        EOF
:::

## Vérifie tes acquis

:::quiz
Quel nom doit porter un fichier pour que `go test` le reconnaisse comme fichier de test ?

- [ ] `test_main.go`
- [ ] `main.test.go`
- [ ] `tests/main.go`
- [x] `main_test.go`

> Les fichiers de test se terminent par `_test.go` et leurs fonctions commencent par `Test`.
:::

:::quiz
Pourquoi utilise-t-on `httptest.NewRecorder()` dans le test d'un handler ?

- [x] Pour capturer la réponse du handler sans ouvrir de port réseau
- [ ] Pour enregistrer la requête dans la base de données
- [ ] Pour chiffrer la réponse
- [ ] Pour lancer un navigateur

> Le recorder joue le rôle du `ResponseWriter` et conserve le statut, les en-têtes et le corps.
:::

:::quiz
Quel est l'avantage d'un Dockerfile en deux étapes pour un programme Go ?

- [ ] Il compile deux fois plus vite
- [ ] Il permet de lancer deux programmes
- [x] L'image finale ne contient que le programme compilé, sans le compilateur ni les sources
- [ ] Il évite d'écrire un `go.mod`

> L'étape de compilation est jetée ; seul le fichier exécutable est copié dans l'image finale, qui reste petite.
:::

:::quiz
Que demande `CGO_ENABLED=0` au moment de compiler ?

- [ ] De désactiver les tests
- [ ] De compiler en mode débogage
- [ ] D'ignorer les dépendances
- [x] Un programme purement Go, sans dépendance aux bibliothèques C du système

> Un exécutable statique tourne dans une image minimale comme `alpine`.
:::
