---
id: websocket-jwt
title: "WebSocket : une connexion qui reste ouverte"
summary: "Garder une connexion ouverte avec un navigateur (WebSocket), vérifier d'où il vient (Origin) et diffuser un message à tout le monde."
minutes: 45
objectives:
  - Expliquer la différence entre une requête HTTP et une connexion WebSocket
  - Expliquer ce que sont une origine, l'en-tête `Origin` et le CORS, et pourquoi un serveur WebSocket doit vérifier l'origine
  - Diffuser un message à plusieurs connexions sans conflit grâce à un mutex
  - Lire une fonction anonyme passée en paramètre de configuration
---

Avec HTTP, c'est toujours le client qui commence : « donne-moi la page », puis le serveur répond, et la conversation s'arrête. Mais comment un serveur peut-il *prévenir* toutes les pages ouvertes d'un coup, par exemple « la maintenance commence » ? Il lui faut une ligne ouverte en permanence. C'est ce que fait `mgmt` dans l'équipe.

## À quoi ça sert, et pourquoi

Un **WebSocket** est une connexion qui reste ouverte entre le navigateur et le serveur, et dans laquelle chacun peut envoyer un message quand il veut. Elle démarre comme une requête HTTP, puis le serveur accepte de « monter en gamme » : c'est l'**upgrade** (« mise à niveau »). Dans `mgmt`, des pages d'administration et des pages publiques se connectent en WebSocket, et le serveur leur pousse des évènements (`maintenance`, `connected`…) sans qu'elles aient à redemander.

Dans cette leçon, tu écris le cœur d'un petit « salon de discussion » de la ludothèque : chaque personne connectée envoie un message, et le serveur le renvoie à tout le monde. Tu utilises la bibliothèque `github.com/gorilla/websocket` (une **bibliothèque** est du code écrit par d'autres que l'on importe ; dans un module, `go get` ou `go mod tidy` la télécharge, mais dans le labo elle est déjà installée).

## D'où vient la connexion ? L'origine et le CORS

Une page web appartient toujours à un site, par exemple `https://ludotheque.example.org`. Cette adresse réduite à son **schéma** (`https`), son **domaine** (`ludotheque.example.org`) et son **port** s'appelle l'**origine** de la page. Quand le navigateur ouvre une connexion, il ajoute automatiquement un **en-tête** (une ligne d'information jointe à la requête) nommé **`Origin`**, qui dit de quelle origine vient la page qui demande. Le JavaScript de la page ne peut pas le falsifier.

Pourquoi est-ce important ? Imagine que tu sois connecté·e à la ludothèque, puis que tu ouvres un site pirate dans un autre onglet. Ce site peut demander à ton navigateur d'ouvrir un WebSocket vers la ludothèque, et le navigateur y joindra ta session. Le serveur doit donc regarder `Origin` et **refuser** toute origine qu'il ne connaît pas.

Tu entendras parler de **CORS** (*Cross-Origin Resource Sharing*, « partage de ressources entre origines différentes »). C'est la règle du navigateur pour les requêtes HTTP ordinaires d'un site vers un autre : le navigateur bloque la réponse, sauf si le serveur répond avec des en-têtes qui l'autorisent explicitement. **Les WebSocket ne sont pas protégés par le CORS** : le navigateur envoie la demande quoi qu'il arrive. C'est donc au serveur de faire lui-même la vérification de l'en-tête `Origin`.

## Le code du salon

Voici un serveur complet (un seul `main.go` ici ; le labo le découpe en plusieurs fichiers).

```go
package main

import (
	"fmt"
	"log"
	"net/http"
	"sync"

	"github.com/gorilla/websocket"
)

// OrigineAttendue est le seul site autorisé à ouvrir un WebSocket vers ce serveur.
const OrigineAttendue = "https://ludotheque.example.org"

// mise transforme une requête HTTP en WebSocket.
var mise = websocket.Upgrader{
	CheckOrigin: func(r *http.Request) bool {
		return r.Header.Get("Origin") == OrigineAttendue
	},
}

// Salle garde la liste des connexions ; le mutex protège la map.
type Salle struct {
	mu    sync.Mutex
	conns map[*websocket.Conn]string
}

func NouvelleSalle() *Salle {
	return &Salle{conns: map[*websocket.Conn]string{}}
}

func (s *Salle) Ajoute(c *websocket.Conn, nom string) {
	s.mu.Lock()
	defer s.mu.Unlock()
	s.conns[c] = nom
}

func (s *Salle) Retire(c *websocket.Conn) {
	s.mu.Lock()
	defer s.mu.Unlock()
	delete(s.conns, c)
}

func (s *Salle) Diffuse(msg []byte) {
	s.mu.Lock()
	defer s.mu.Unlock()
	for c := range s.conns {
		if err := c.WriteMessage(websocket.TextMessage, msg); err != nil {
			log.Println("écriture :", err)
		}
	}
}

// Ws est le handler de la route /ws?nom=Camille
func (s *Salle) Ws(w http.ResponseWriter, r *http.Request) {
	nom := r.URL.Query().Get("nom")
	if nom == "" {
		http.Error(w, "nom obligatoire", http.StatusBadRequest)
		return
	}
	conn, err := mise.Upgrade(w, r, nil)
	if err != nil {
		return // Upgrade a déjà répondu au client
	}
	defer conn.Close()

	s.Ajoute(conn, nom)
	defer s.Retire(conn)

	for {
		_, msg, err := conn.ReadMessage()
		if err != nil {
			return // la personne a fermé sa page
		}
		s.Diffuse([]byte(fmt.Sprintf("%s : %s", nom, msg)))
	}
}

func main() {
	salle := NouvelleSalle()
	http.HandleFunc("/ws", salle.Ws)
	log.Fatal(http.ListenAndServe(":8080", nil))
}
```

### Le contrôle de l'origine

`websocket.Upgrader` est une structure de configuration : on la remplit avec des champs, comme une `Jeu`. Son champ `CheckOrigin` attend une **fonction** qui reçoit la requête et répond `true` (accepter) ou `false` (refuser). Comme dans la leçon précédente, on lui passe une **fonction anonyme** écrite à l'endroit même où on en a besoin :

```go
CheckOrigin: func(r *http.Request) bool {
	return r.Header.Get("Origin") == OrigineAttendue
},
```

Ligne à ligne : `func(r *http.Request) bool` est une fonction qui reçoit la requête `r` et renvoie un booléen ; `r.Header.Get("Origin")` lit la valeur de l'en-tête `Origin` ; `==` la compare à l'origine attendue ; le résultat est renvoyé avec `return`. Si le serveur reçoit une demande d'un autre site, `Upgrade` répond `403` (interdit) et la connexion n'a jamais lieu. `mgmt` fait de même, en comparant l'origine à son port de frontal (`FrontendPort`).

### La salle et le mutex

- `type Salle struct { mu sync.Mutex; conns map[*websocket.Conn]string }` : la salle garde une **map** dont les clés sont les connexions (`*websocket.Conn`, un pointeur vers chaque connexion) et les valeurs, les noms.
- `Ajoute` et `Retire` modifient cette map. Chaque connexion est traitée par sa propre **goroutine** (la tâche légère vue dans la leçon 3), donc plusieurs goroutines peuvent y toucher en même temps. Sans `sync.Mutex`, deux écritures simultanées dans une map font **planter** le programme (« concurrent map writes »). `Lock` prend le verrou, `defer s.mu.Unlock()` le rend à la sortie de la fonction.
- `delete(s.conns, c)` retire une clé d'une map.
- `Diffuse` parcourt les connexions (`for c := range s.conns` : sur une map, `range` donne les clés) et écrit le message dans chacune avec `WriteMessage`. La bibliothèque `gorilla/websocket` précise qu'une connexion n'accepte qu'**un seul écrivain à la fois** : garder le verrou pendant toute la boucle l'assure.

### Le handler `Ws`

- `r.URL.Query().Get("nom")` lit le paramètre `nom` dans l'adresse (`/ws?nom=Camille`). Sans nom, on répond `400` avant même de monter en gamme.
- `mise.Upgrade(w, r, nil)` transforme la requête HTTP en WebSocket et renvoie `conn`, l'objet pour lire et écrire.
- `defer conn.Close()` et `defer s.Retire(conn)` **nettoient toujours** à la sortie : fermer la connexion et la retirer de la salle, quelle que soit la façon dont la fonction se termine. Les `defer` s'exécutent dans l'ordre inverse de leur écriture.
- `for { … }` est une boucle sans condition : elle tourne jusqu'au `return`. `conn.ReadMessage()` attend un message et renvoie trois choses : son type (inutile ici, d'où le `_`), son contenu `msg` (une suite d'octets, de type `[]byte`) et une erreur. Quand la lecture échoue, c'est que la personne a fermé sa page : on quitte la boucle.
- `[]byte(fmt.Sprintf("%s : %s", nom, msg))` fabrique le texte « Camille : salut » (`%s` accepte aussi bien un texte qu'une suite d'octets) puis le convertit en `[]byte` pour `Diffuse`.

:::danger Un point à surveiller dans le code de l'équipe
Dans les fichiers de `mgmt` que nous avons lus, les listes `loggers`, `watchers` et `customers` sont des variables globales parcourues et modifiées par plusieurs goroutines, et nous n'y avons pas trouvé de `sync.Mutex`. Avec ce que tu as appris, tu sais que c'est un risque.
:::

## Entraîne-toi

:::lab
engine: real
intro: |
  Tu écris le salon de discussion. `salle.go` (la structure `Salle` et le handler `Ws`) est déjà écrit. Il te reste trois morceaux, chacun dans son fichier : le contrôle de l'origine (`origine.go`), l'enregistrement des connexions (`registre.go`) et la diffusion (`diffusion.go`). Remplace les `panic("à écrire")`. Les tests (`salle_test.go`) démarrent un vrai serveur sur ta machine, ouvrent de vraies connexions WebSocket et envoient de vrais messages. Lance-les avec `go test ./...`. Les tests fournis sont vérifiés tels quels.
commands:
  - cp -R /opt/exercices/04-websocket/. .
  - go vet ./... >/dev/null 2>&1 || true
steps:
  - text: 'Dans `origine.go`, complète la **fonction anonyme** `CheckOrigin` : elle renvoie `true` seulement si l''en-tête `Origin` vaut `OrigineAttendue`. `TestOrigine` doit passer (une autre origine reçoit `403`)'
    hint: 'Remplace les deux lignes du corps de la fonction par `return r.Header.Get("Origin") == OrigineAttendue`.'
    checks:
      - command-succeeds: "verifier-go tests 04-websocket . TestOrigine"
    solution:
      - |
        cat > origine.go <<'EOF'
        package main

        import (
        	"net/http"

        	"github.com/gorilla/websocket"
        )

        // OrigineAttendue est le seul site autorisé à ouvrir un WebSocket vers ce serveur.
        const OrigineAttendue = "https://ludotheque.example.org"

        // mise transforme une requête HTTP en WebSocket (« upgrade »).
        var mise = websocket.Upgrader{
        	CheckOrigin: func(r *http.Request) bool {
        		return r.Header.Get("Origin") == OrigineAttendue
        	},
        }
        EOF
  - text: 'Dans `registre.go`, écris `Ajoute` et `Retire`, protégées par le mutex. `TestRegistre` doit passer (il lance 200 goroutines en même temps)'
    hint: 'Dans chaque méthode : `s.mu.Lock()` puis `defer s.mu.Unlock()`. Ensuite `s.conns[c] = nom` pour `Ajoute`, et `delete(s.conns, c)` pour `Retire`.'
    checks:
      - command-succeeds: "verifier-go tests 04-websocket . TestRegistre"
    solution:
      - |
        cat > registre.go <<'EOF'
        package main

        import "github.com/gorilla/websocket"

        // Ajoute enregistre la connexion et le nom de la personne.
        func (s *Salle) Ajoute(c *websocket.Conn, nom string) {
        	s.mu.Lock()
        	defer s.mu.Unlock()
        	s.conns[c] = nom
        }

        // Retire supprime la connexion de la salle.
        func (s *Salle) Retire(c *websocket.Conn) {
        	s.mu.Lock()
        	defer s.mu.Unlock()
        	delete(s.conns, c)
        }
        EOF
  - text: 'Dans `diffusion.go`, écris `Diffuse` : elle envoie le message à toutes les connexions de la salle, sous verrou. `TestDiffuse` doit passer (deux personnes se connectent, l''une écrit, les deux reçoivent « Camille : salut »)'
    hint: 'Verrou et `defer`, puis `for c := range s.conns { if err := c.WriteMessage(websocket.TextMessage, msg); err != nil { log.Println("écriture :", err) } }`. Imports : `log` et `github.com/gorilla/websocket`.'
    after: [1, 2]
    checks:
      - command-succeeds: "verifier-go tests 04-websocket . TestDiffuse"
    solution:
      - |
        cat > diffusion.go <<'EOF'
        package main

        import (
        	"log"

        	"github.com/gorilla/websocket"
        )

        // Diffuse envoie le message à toutes les connexions de la salle.
        func (s *Salle) Diffuse(msg []byte) {
        	s.mu.Lock()
        	defer s.mu.Unlock()
        	for c := range s.conns {
        		if err := c.WriteMessage(websocket.TextMessage, msg); err != nil {
        			log.Println("écriture :", err)
        		}
        	}
        }
        EOF
  - text: 'Vérifie l''ensemble : compile le salon dans un exécutable `salon` avec `go build -o salon .`, puis `go vet ./...` ne signale rien (il repère par exemple un mutex copié par erreur) et `go test ./...` passe en entier'
    hint: 'Lance `go build -o salon . && go vet ./... && go test ./...`.'
    after: [1, 2, 3]
    checks:
      - command-succeeds: 'verifier-go binaire 04-websocket salon'
      - command-succeeds: 'verifier-go tout 04-websocket'
    solution:
      - go build -o salon . && go vet ./... && go test ./...
:::

## Vérifie tes acquis

:::quiz
Quel est l'intérêt d'un WebSocket par rapport à une requête HTTP classique ?

- [ ] Il chiffre automatiquement les messages
- [ ] Il est plus rapide à écrire en Go
- [ ] Il évite d'avoir un serveur
- [x] La connexion reste ouverte et le serveur peut envoyer un message sans que le client le demande

> HTTP est « question puis réponse » ; un WebSocket garde le canal ouvert dans les deux sens.
:::

:::quiz
Que contient l'en-tête `Origin` envoyé par un navigateur quand une page ouvre un WebSocket ?

- [ ] Le nom et le mot de passe de la personne connectée
- [ ] L'adresse IP du serveur qui répond
- [x] L'origine (schéma, domaine et port) du site dont vient la page
- [ ] La date d'expiration de la session

> Le navigateur remplit lui-même `Origin` et la page ne peut pas le modifier : c'est ce qui permet au serveur de refuser un site inconnu.
:::

:::quiz
Pourquoi un serveur WebSocket doit-il vérifier lui-même l'en-tête `Origin` ?

- [ ] Parce que Go refuse de compiler un serveur sans cette vérification
- [ ] Parce que l'en-tête `Origin` est chiffré et doit être déchiffré
- [x] Parce que le CORS du navigateur ne protège pas les WebSocket : sans contrôle, n'importe quel site peut ouvrir la connexion
- [ ] Parce que sans `Origin`, le navigateur ne sait pas quel port utiliser

> Le CORS encadre les requêtes HTTP ordinaires entre sites, pas les WebSocket : la vérification se fait côté serveur.
:::

:::quiz
Pourquoi `Diffuse` garde-t-elle le verrou du mutex pendant toute la boucle d'écriture ?

- [ ] Pour que les messages soient chiffrés
- [x] Pour que la map ne soit pas modifiée pendant le cours et qu'une seule goroutine écrive à la fois dans les connexions
- [ ] Pour accélérer l'envoi des messages
- [ ] Parce que `range` exige un verrou sur toutes les maps

> Une map modifiée pendant qu'elle est parcourue ou lue par une autre goroutine fait planter le programme ; et `gorilla/websocket` n'autorise qu'un écrivain à la fois par connexion.
:::
