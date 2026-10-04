---
id: serveur-http-json
titre: "Un serveur HTTP qui parle JSON"
resume: "Écrire une petite API : recevoir une requête HTTP, répondre en JSON et ajouter un middleware, avec la bibliothèque standard."
duree: 50
objectifs:
  - Expliquer ce qu'est une API HTTP, une requête, une réponse et un code de statut
  - Écrire un handler qui lit et écrit du JSON avec `encoding/json`
  - Expliquer ce qu'est une fonction anonyme et une closure
  - Enchaîner un middleware autour d'un routeur et protéger des données partagées avec un mutex
---

Quand tu ouvres une page web, ton navigateur envoie une demande à un serveur, qui répond. Une **API** (interface de programmation) fonctionne de la même façon, mais le client est un autre programme : par exemple la page d'administration d'Adhésion qui demande à `mgmt` si le site public est en maintenance. Dans cette leçon, tu écris un tel serveur.

## À quoi ça sert, et le vocabulaire

- **HTTP** est le langage que parlent navigateurs et serveurs. Le client envoie une **requête** (*request*), le serveur renvoie une **réponse** (*response*).
- Une requête a une **méthode** qui dit l'intention : `GET` pour lire, `POST` pour envoyer des données, `DELETE` pour supprimer. Elle vise un **chemin** (`/jeux`).
- Une réponse a un **code de statut** : `200` tout va bien, `201` créé, `400` la demande est mal formée, `404` introuvable, `405` méthode non autorisée, `500` panne du serveur.
- **JSON** est un format de texte pour échanger des données : `{"titre": "Azul", "joueurs": 4}`. Les accolades décrivent un objet, avec des paires nom et valeur.

Les trois projets de l'équipe ont ce rôle. Dans `mgmt`, par exemple, la route `/webconfig` renvoie l'état de la maintenance sous forme de JSON, grâce à la ligne `ctx.JSON(200, GlobalWebConfig)`.

## Convertir une structure en JSON

Go sait transformer une structure en JSON et inversement, grâce au paquet `encoding/json`. Pour choisir le nom des champs dans le JSON, on ajoute à chaque champ une **étiquette** (*tag*) entre apostrophes inversées :

```go
type Jeu struct {
	Titre   string `json:"titre"`
	Joueurs int    `json:"joueurs"`
	Note    string `json:"note,omitempty"`
}
```

`json:"titre"` signifie « dans le JSON, ce champ s'appelle `titre` ». `omitempty` signifie « n'écris pas ce champ s'il est vide ». Tu retrouves ces étiquettes dans `webconfig.go` du projet `mgmt` (`json:"admin_frontend_enabled"`) et dans les modèles d'`event-planner-api`.

## Une notion nouvelle : les fonctions anonymes et les closures

Jusqu'ici, chaque fonction avait un nom. En Go, une fonction est aussi une **valeur** : on peut la ranger dans une variable, la passer en argument ou la renvoyer. Une fonction sans nom, écrite à l'endroit où on l'utilise, s'appelle une **fonction anonyme** :

```go
saluer := func(nom string) string {
	return "Bonjour " + nom
}
fmt.Println(saluer("Camille")) // Bonjour Camille
```

Ligne à ligne : `func(nom string) string { … }` est une fonction qui prend un texte et en renvoie un ; `saluer :=` la range dans une variable ; `saluer("Camille")` l'appelle ; le `+` colle deux textes ensemble.

Une fonction anonyme peut utiliser les variables qui existent autour d'elle, et elle les **garde en mémoire** même quand la fonction qui l'a créée est terminée. On parle de **closure** (« fermeture ») :

```go
func compteur() func() int {
	n := 0
	return func() int {
		n++
		return n
	}
}

suivant := compteur()
fmt.Println(suivant(), suivant(), suivant()) // 1 2 3
```

`compteur` renvoie une fonction (`func() int` est le type « fonction sans argument qui renvoie un entier »). Cette fonction se souvient de `n` : à chaque appel, elle l'augmente de 1 (`n++`) puis le renvoie. Garde cette idée en tête : le middleware ci-dessous est exactement une closure qui se souvient du handler qu'elle enveloppe.

## Le serveur complet

Crée un module (`go mod init ludotheque`) et le fichier `main.go` suivant.

```go
package main

import (
	"encoding/json"
	"log"
	"net/http"
	"sync"
)

// Jeu est la représentation JSON d'un jeu. Les étiquettes `json:"…"` fixent les noms des champs.
type Jeu struct {
	Titre   string `json:"titre"`
	Joueurs int    `json:"joueurs"`
	Note    string `json:"note,omitempty"`
}

type serveur struct {
	mu   sync.Mutex
	jeux []Jeu
}

func (s *serveur) liste(w http.ResponseWriter, r *http.Request) {
	s.mu.Lock()
	defer s.mu.Unlock()
	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(s.jeux)
}

func (s *serveur) ajoute(w http.ResponseWriter, r *http.Request) {
	var j Jeu
	if err := json.NewDecoder(r.Body).Decode(&j); err != nil {
		http.Error(w, `{"erreur":"JSON invalide"}`, http.StatusBadRequest)
		return
	}
	if j.Titre == "" {
		http.Error(w, `{"erreur":"titre obligatoire"}`, http.StatusBadRequest)
		return
	}
	s.mu.Lock()
	s.jeux = append(s.jeux, j)
	s.mu.Unlock()
	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(http.StatusCreated)
	json.NewEncoder(w).Encode(j)
}

// journal est un middleware : il enveloppe un http.Handler.
func journal(suivant http.Handler) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		log.Println(r.Method, r.URL.Path)
		suivant.ServeHTTP(w, r)
	})
}

func main() {
	s := &serveur{}
	mux := http.NewServeMux()
	mux.HandleFunc("GET /jeux", s.liste)
	mux.HandleFunc("POST /jeux", s.ajoute)

	log.Fatal(http.ListenAndServe(":8080", journal(mux)))
}
```

### Lecture ligne à ligne

- **Un handler** (« gestionnaire ») est une fonction `func(w http.ResponseWriter, r *http.Request)`. `r` représente la requête reçue (l'étoile `*` indique un pointeur, comme dans la leçon 1), `w` sert à écrire la réponse.
- `liste` : place l'en-tête `Content-Type: application/json` (une indication pour le client sur le format de la réponse), puis `json.NewEncoder(w).Encode(s.jeux)` écrit la liste en JSON directement dans la réponse.
- `ajoute` : `json.NewDecoder(r.Body).Decode(&j)` lit le corps de la requête (`r.Body`) et remplit `j`. Le `&` donne l'adresse de `j` pour que `Decode` puisse le modifier (le même mécanisme de pointeur que dans la leçon 1). Si le JSON est cassé ou si le titre manque, on répond `400` avec `http.Error` et on s'arrête avec `return`. La forme `if err := …; err != nil` déclare `err` juste pour ce `if`.
- `w.WriteHeader(http.StatusCreated)` envoie le statut `201`. Il faut l'appeler **avant** d'écrire le corps.
- **Le routeur** `http.NewServeMux()` associe un motif (`"GET /jeux"`) à un handler. Il répond tout seul `404` si le chemin n'existe pas et `405` si la méthode n'est pas permise. `s.liste` (sans parenthèses) désigne la méthode elle-même, qu'on passe au routeur sans l'appeler.
- `http.ListenAndServe(":8080", …)` démarre le serveur sur le port `8080` (un « numéro de guichet » de l'ordinateur) et ne rend la main que s'il tombe en panne : `log.Fatal` affiche alors l'erreur et quitte.

### Le middleware

Un **middleware** est une fonction qui enveloppe un handler pour ajouter un comportement commun avant ou après lui : ici, écrire une ligne de journal pour chaque requête. Regardons `journal` de près :

- `http.Handler` est une **interface** (leçon 2) qui exige une seule méthode, `ServeHTTP(w, r)`. Le routeur `mux` en est un.
- `http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) { … })` fait deux choses : `func(w …, r …) { … }` est une **fonction anonyme**, et `http.HandlerFunc(…)` la **convertit** en `http.Handler` (c'est un petit adaptateur fourni par Go : une simple fonction devient quelqu'un qui sait `ServeHTTP`).
- Cette fonction anonyme est une **closure** : elle utilise `suivant`, un paramètre de `journal`, et s'en souvient après la fin de `journal`. À chaque requête, elle écrit une ligne de journal (`log.Println`), puis passe la main (`suivant.ServeHTTP(w, r)`).

`journal(mux)` renvoie donc un nouveau handler qui note la requête puis appelle le vrai routeur. C'est exactement le rôle des middlewares de l'équipe : dans `mgmt`, `AuthenticationMiddleware` vérifie le jeton avant d'appeler le vrai traitement, `RBACMiddleWare` contrôle le rôle ; dans `billetterie`, `BureauMiddleware` réserve des pages au bureau.

### Pourquoi un `Mutex` ?

Go traite **chaque requête dans sa propre tâche** appelée *goroutine* (un fil d'exécution très léger). Deux requêtes peuvent donc modifier `s.jeux` au même instant, ce qui abîme les données. Un **mutex** (`sync.Mutex`, de l'anglais *mutual exclusion*) est un verrou : `Lock()` attend son tour et réserve l'accès, `Unlock()` le libère. `defer s.mu.Unlock()` signifie « exécute cette ligne à la sortie de la fonction », ce qui garantit que le verrou est rendu même en cas de `return` anticipé.

## Essayer le serveur

Lance `go run .` dans un terminal, puis ouvre un second terminal. `curl` est un outil qui envoie une requête HTTP depuis le terminal : `-s` le rend silencieux, `-X POST` choisit la méthode, `-d` donne le corps de la requête.

```bash
curl -s -X POST localhost:8080/jeux -d '{"titre":"Azul","joueurs":4}'
curl -s -X POST localhost:8080/jeux -d '{"joueurs":4}'
curl -s localhost:8080/jeux
```

```console
{"titre":"Azul","joueurs":4}
{"erreur":"titre obligatoire"}
[{"titre":"Azul","joueurs":4}]
```

La première commande crée un jeu, la deuxième est refusée (code `400`), la troisième liste les jeux. Les données sont gardées **en mémoire** : si tu arrêtes le programme, elles disparaissent (la leçon 6 y remédie).

:::info Quel framework choisir ?
Les projets de l'équipe n'utilisent pas tous la bibliothèque standard. Un **framework** est une bibliothèque plus large qui impose une façon d'organiser le code. `mgmt` et `billetterie` utilisent **Macaron** (versions 1.3.5 et 1.3.8 dans leurs `go.mod`), qui offre `ctx.JSON(…)` et des groupes de routes ; `event-planner-api` utilise **Echo** (version 4). Les idées sont les mêmes que ce que tu viens de voir : un handler, un routeur, des middlewares. Avant d'adopter un framework dans un nouveau projet, regarde sur son dépôt si son développement est toujours actif. Attention aussi à la version de Go : les motifs `"GET /jeux"` ci-dessus demandent **Go 1.22 ou plus**, ils ne marchent pas avec le Go 1.16 de `mgmt`.
:::

## Entraîne-toi

:::labo
moteur: reel
intro: |
  Tu écris le serveur de la ludothèque. `jeu.go` (la structure `Jeu` et le `serveur` avec son mutex) est déjà écrit. Les quatre morceaux à écrire sont chacun dans leur fichier : `liste.go`, `ajoute.go`, `journal.go` et `routeur.go`. Remplace `panic("à écrire")` par ton code ; les tests sont dans `serveur_test.go` et simulent des requêtes **sans ouvrir de port réseau**. Lance-les avec `go test ./...`. Les tests fournis sont vérifiés tels quels.
commandes:
  - cp -R /opt/exercices/03-serveur-http/. .
  - go vet ./... >/dev/null 2>&1 || true
etapes:
  - texte: 'Écris le handler `liste` dans `liste.go` : en-tête `Content-Type: application/json`, puis la liste `s.jeux` en JSON (sans oublier le verrou). `TestListe` doit passer'
    indice: 'Reprends les quatre lignes de `liste` de la leçon : `s.mu.Lock()`, `defer s.mu.Unlock()`, `w.Header().Set(...)`, `json.NewEncoder(w).Encode(s.jeux)`. Il faut importer `encoding/json` et `net/http`.'
    verif:
      - commande-reussit: "verifier-go tests 03-serveur-http . TestListe"
    solution:
      - |
        cat > liste.go <<'EOF'
        package main

        import (
        	"encoding/json"
        	"net/http"
        )

        // liste répond à GET /jeux : la liste des jeux en JSON.
        func (s *serveur) liste(w http.ResponseWriter, r *http.Request) {
        	s.mu.Lock()
        	defer s.mu.Unlock()
        	w.Header().Set("Content-Type", "application/json")
        	json.NewEncoder(w).Encode(s.jeux)
        }
        EOF
  - texte: 'Écris le handler `ajoute` dans `ajoute.go` pour le cas normal : décode le jeu, ajoute-le à `s.jeux` (sous verrou) et réponds `201` avec le jeu en JSON. `TestAjouteValide` et `TestAjouteConcurrent` doivent passer'
    indice: 'Dans cet ordre : `json.NewDecoder(r.Body).Decode(&j)`, `s.mu.Lock()`, `append`, `s.mu.Unlock()`, puis `w.Header().Set(...)`, `w.WriteHeader(http.StatusCreated)` et `json.NewEncoder(w).Encode(j)`.'
    verif:
      - commande-reussit: "verifier-go tests 03-serveur-http . TestAjouteValide TestAjouteConcurrent"
    solution:
      - |
        cat > ajoute.go <<'EOF'
        package main

        import (
        	"encoding/json"
        	"net/http"
        )

        // ajoute répond à POST /jeux : cas normal.
        func (s *serveur) ajoute(w http.ResponseWriter, r *http.Request) {
        	var j Jeu
        	json.NewDecoder(r.Body).Decode(&j)
        	s.mu.Lock()
        	s.jeux = append(s.jeux, j)
        	s.mu.Unlock()
        	w.Header().Set("Content-Type", "application/json")
        	w.WriteHeader(http.StatusCreated)
        	json.NewEncoder(w).Encode(j)
        }
        EOF
  - texte: 'Rends `ajoute` robuste : réponds `400` (avec `http.Error`) et ne retiens rien quand le JSON est cassé ou que le titre est vide. `TestAjouteInvalide` doit passer'
    indice: 'Mets le résultat de `Decode` dans `err` : `if err := json.NewDecoder(r.Body).Decode(&j); err != nil { http.Error(w, "...", http.StatusBadRequest); return }`. Fais pareil pour `j.Titre == ""`.'
    apres: [2]
    verif:
      - commande-reussit: "verifier-go tests 03-serveur-http . TestAjouteInvalide"
    solution:
      - |
        cat > ajoute.go <<'EOF'
        package main

        import (
        	"encoding/json"
        	"net/http"
        )

        // ajoute répond à POST /jeux.
        func (s *serveur) ajoute(w http.ResponseWriter, r *http.Request) {
        	var j Jeu
        	if err := json.NewDecoder(r.Body).Decode(&j); err != nil {
        		http.Error(w, `{"erreur":"JSON invalide"}`, http.StatusBadRequest)
        		return
        	}
        	if j.Titre == "" {
        		http.Error(w, `{"erreur":"titre obligatoire"}`, http.StatusBadRequest)
        		return
        	}
        	s.mu.Lock()
        	s.jeux = append(s.jeux, j)
        	s.mu.Unlock()
        	w.Header().Set("Content-Type", "application/json")
        	w.WriteHeader(http.StatusCreated)
        	json.NewEncoder(w).Encode(j)
        }
        EOF
  - texte: 'Écris le middleware `journal` dans `journal.go` : une **closure** (fonction anonyme convertie avec `http.HandlerFunc`) qui écrit « MÉTHODE CHEMIN » avec `log.Println`, puis appelle `suivant.ServeHTTP(w, r)`. `TestJournal` doit passer'
    indice: 'Le corps de la fonction est : `return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) { log.Println(r.Method, r.URL.Path); suivant.ServeHTTP(w, r) })` (une instruction par ligne).'
    verif:
      - commande-reussit: "verifier-go tests 03-serveur-http . TestJournal"
    solution:
      - |
        cat > journal.go <<'EOF'
        package main

        import (
        	"log"
        	"net/http"
        )

        // journal est un middleware : il enveloppe un http.Handler.
        func journal(suivant http.Handler) http.Handler {
        	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
        		log.Println(r.Method, r.URL.Path)
        		suivant.ServeHTTP(w, r)
        	})
        }
        EOF
  - texte: 'Écris `NouveauRouteur` dans `routeur.go` : un `http.NewServeMux()` avec les motifs `"GET /jeux"` et `"POST /jeux"`, enveloppé par `journal`. `TestRouteur` doit passer (`404` pour un chemin inconnu, `405` pour `DELETE /jeux`)'
    indice: '`mux := http.NewServeMux()`, `mux.HandleFunc("GET /jeux", s.liste)`, `mux.HandleFunc("POST /jeux", s.ajoute)`, puis `return journal(mux)`.'
    apres: [1, 2, 4]
    verif:
      - commande-reussit: "verifier-go tests 03-serveur-http . TestRouteur"
    solution:
      - |
        cat > routeur.go <<'EOF'
        package main

        import "net/http"

        // NouveauRouteur associe les motifs aux handlers de s, enveloppés par le middleware journal.
        func NouveauRouteur(s *serveur) http.Handler {
        	mux := http.NewServeMux()
        	mux.HandleFunc("GET /jeux", s.liste)
        	mux.HandleFunc("POST /jeux", s.ajoute)
        	return journal(mux)
        }
        EOF
  - texte: 'Vérifie l''ensemble : compile le serveur dans un exécutable `serveur` avec `go build -o serveur .`, puis `go vet ./...` ne signale rien et `go test ./...` passe en entier'
    indice: 'Lance `go build -o serveur . && go vet ./... && go test ./...`.'
    apres: [1, 2, 3, 4, 5]
    verif:
      - commande-reussit: 'verifier-go binaire 03-serveur-http serveur'
      - commande-reussit: 'verifier-go tout 03-serveur-http'
    solution:
      - go build -o serveur . && go vet ./... && go test ./...
:::

## Vérifie tes acquis

:::quiz
Que fait l'étiquette `json:"note,omitempty"` sur un champ de structure ?

- [x] Le champ s'appelle `note` dans le JSON et il est omis quand il est vide
- [ ] Le champ est chiffré avant d'être envoyé
- [ ] Le champ n'est lu qu'en entrée, jamais écrit en sortie
- [ ] Le champ devient obligatoire

> Le nom avant la virgule fixe la clé JSON ; `omitempty` retire le champ s'il vaut sa valeur zéro.
:::

:::quiz
Dans `func compteur() func() int { n := 0; return func() int { n++; return n } }`, pourquoi la fonction renvoyée peut-elle encore utiliser `n` après la fin de `compteur` ?

- [ ] Parce que `n` est une variable globale
- [ ] Parce que Go copie `n` dans le terminal
- [ ] Parce que `compteur` n'est jamais vraiment terminée
- [x] Parce que c'est une closure : elle garde en mémoire les variables qu'elle utilise

> Une fonction anonyme qui utilise des variables de son entourage les emporte avec elle ; c'est ce qui permet à `journal` de se souvenir de `suivant`.
:::

:::quiz
À quoi sert un middleware comme `journal` ?

- [ ] À remplacer le routeur
- [ ] À stocker les données en base
- [ ] À compiler le programme plus vite
- [x] À ajouter un traitement commun (journal, authentification…) autour d'un handler

> Un middleware reçoit un handler et en renvoie un autre, qui fait son travail puis appelle le suivant.
:::

:::quiz
Pourquoi protège-t-on `s.jeux` avec un mutex ?

- [ ] Parce que les slices Go sont toujours chiffrées
- [x] Parce que plusieurs requêtes s'exécutent en même temps et pourraient le modifier simultanément
- [ ] Parce que `json.Encode` exige un verrou
- [ ] Parce que sans mutex, le serveur ne démarre pas

> Chaque requête tourne dans sa propre goroutine : sans verrou, deux écritures simultanées peuvent corrompre les données.
:::
