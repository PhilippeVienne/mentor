---
id: tests-et-build
titre: "Tests et build de production"
resume: "Écrire un premier test, produire la version de production et comprendre comment elle est déployée."
duree: 40
objectifs:
  - Expliquer ce qu'est un test unitaire et lire un fichier `*.spec.ts`
  - Tester un composant avec `TestBed` et un service avec `HttpTestingController`
  - Lancer `ng build` et dire ce qui change en production
  - Suivre le chemin du Dockerfile et du script `start.sh` d'Adhésion
---

Tu viens de modifier un écran. Comment être sûr·e de ne pas avoir cassé le reste ? Et comment ton code devient-il un site que les bénévoles utilisent ? Cette leçon répond aux deux questions : les **tests** vérifient le code, le **build** (la construction, qui transforme ton code en fichiers prêts à être publiés) le prépare pour la production.

## Un test unitaire, c'est quoi ?

Un test unitaire est un petit programme qui exécute une partie du code (une fonction, un composant) et **vérifie le résultat attendu**. Tu le relances à chaque modification : si quelque chose casse, il devient rouge tout de suite, avant que les utilisatrices et utilisateurs ne s'en aperçoivent.

Jusqu'à Angular 20 (c'est le cas d'Adhésion), un projet créé avec la commande `ng new` utilise par défaut **Jasmine** pour écrire les tests et **Karma** pour les lancer dans un vrai navigateur. Ce sont les outils déclarés dans les `devDependencies` d'Adhésion (`jasmine-core`, `karma`, `karma-jasmine`). Les versions plus récentes d'Angular peuvent utiliser **Vitest** à la place, et c'est lui que le labo de cette leçon emploie (ton terminal n'a pas de navigateur). Rassure-toi : `describe`, `it`, `expect(…).toBe(…)` et `TestBed` s'écrivent de la même façon avec les deux. Les fichiers de test s'appellent `*.spec.ts`, à côté du code testé.

```ts
describe('calculerTotal', () => {
  it('additionne les prix', () => {
    expect(calculerTotal([5, 10])).toBe(15);
  });
});
```

- `describe` regroupe des tests sous un titre.
- `it` décrit **un** comportement attendu, en une phrase.
- `expect(valeur).toBe(attendu)` compare le résultat obtenu et celui qu'on espérait. Si les deux diffèrent, le test échoue et affiche l'écart.
- `calculerTotal` est une fonction fictive, utilisée ici seulement pour montrer la forme d'un test.

## Tester un composant avec TestBed

Un composant a besoin d'Angular pour vivre (gabarit, injection). `TestBed` fabrique un mini-module de test dans lequel on crée le composant :

```ts
import { Component } from '@angular/core';
import { ComponentFixture, TestBed } from '@angular/core/testing';

@Component({
  selector: 'app-titre',
  template: `<h1>{{ titre }}</h1>`,
})
class TitreComponent {
  titre = 'Mes adhérents';
}

describe('TitreComponent', () => {
  let fixture: ComponentFixture<TitreComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({ imports: [TitreComponent] }).compileComponents();
    fixture = TestBed.createComponent(TitreComponent);
    fixture.detectChanges();
  });

  it('affiche le titre', () => {
    const h1: HTMLElement = fixture.nativeElement.querySelector('h1');
    expect(h1.textContent).toBe('Mes adhérents');
  });
});
```

1. `beforeEach` s'exécute avant **chaque** test, pour repartir d'un état propre.
2. `configureTestingModule` décrit l'environnement du test. Ici le composant est autonome, donc on l'écrit dans `imports`.
3. `createComponent` instancie le composant ; `detectChanges()` demande à Angular de calculer le gabarit.
4. On lit le HTML produit (`nativeElement`) et on vérifie son contenu.

Adhésion contient quelques fichiers de ce genre (par exemple `erreur.component.spec.ts`). Comme ses composants sont dans un `NgModule` (`standalone: false`), le test les écrit dans `declarations: [ErreurComponent]` plutôt que dans `imports`. Ce fichier utilise aussi `async` importé de `@angular/core/testing`, un ancien nom remplacé par `waitForAsync` : ne le recopie pas.

## Tester un service HTTP sans réseau

On ne veut pas qu'un test appelle la vraie API. Angular fournit un faux serveur, `HttpTestingController` : on y vérifie la requête envoyée et on **simule** la réponse.

```ts
import { provideHttpClient } from '@angular/common/http';
import { HttpTestingController, provideHttpClientTesting } from '@angular/common/http/testing';
import { TestBed } from '@angular/core/testing';

describe('MembresService', () => {
  it('appelle la recherche avec le terme', () => {
    TestBed.configureTestingModule({
      providers: [provideHttpClient(), provideHttpClientTesting()],
    });
    const service = TestBed.inject(MembresService);
    const http = TestBed.inject(HttpTestingController);

    let recu: unknown;
    service.rechercher('camille').subscribe((page) => (recu = page));

    const requete = http.expectOne('https://example.org/api/membres/?search=camille');
    expect(requete.request.method).toBe('GET');
    requete.flush({ count: 0, results: [] }); // réponse simulée

    expect(recu).toEqual({ count: 0, results: [] });
    http.verify(); // aucune requête imprévue
  });
});
```

(`MembresService` est celui de la leçon 4.) C'est la leçon 2 qui s'applique : on remplace le vrai client HTTP par un faux grâce à l'injection de dépendances. `flush` envoie la réponse simulée, et `verify` échoue si le code a envoyé une requête que le test n'attendait pas.

:::info Dans Adhésion
`package.json` définit les scripts `ng test` et `ng lint`, mais `angular.json` ne déclare pas de cible `test` ni `lint` (seulement `build`, `serve` et `extract-i18n`). Avant de compter sur ces commandes, vérifie avec l'équipe qu'elles fonctionnent sur ta machine et dans la CI (l'intégration continue : un robot qui relance les tests à chaque modification du code).
:::

## Le build de production

En développement, `ng serve` lance un serveur local (`http://localhost:4200/`) qui recharge la page à chaque modification. En production, on veut des fichiers **statiques**, petits et rapides. C'est le rôle de :

```bash
npm run build
```

`npm run build` demande à `npm` (le gestionnaire de paquets de Node.js) d'exécuter le script nommé `build` du fichier `package.json`. Celui d'Adhésion exécute `ng build --configuration production`. Les fichiers sont écrits dans `dist/` (de « distribution » : le dossier de ce qui sera publié). La configuration `production` d'`angular.json` change plusieurs choses :

| Option | Effet |
| --- | --- |
| `optimization: true` | Le code est minifié (raccourci) et les parties inutilisées sont retirées |
| `aot: true` | Les gabarits sont compilés à l'avance, pas dans le navigateur |
| `outputHashing: "all"` | Un condensat est ajouté aux noms de fichiers : le cache du navigateur se renouvelle à chaque version |
| `sourceMap: false` | Pas de cartes pour déboguer le code d'origine |
| `fileReplacements` | `environment.ts` est remplacé par `environment.prod.ts` |

Le dernier point est important : le fichier `environment.prod.ts` d'Adhésion ne contient pas de vraies adresses, mais des marqueurs comme `'__ADHESION_API_URL__'` et `'__KEYCLOAK_REALM__'`.

## Du build au conteneur

Un **Dockerfile** est une recette qui décrit comment fabriquer une *image* (un paquet autonome contenant le site et ce qu'il faut pour l'exécuter) ; un *conteneur* est cette image en cours de fonctionnement. Celui d'Adhésion a deux étapes :

1. **Construction** : à partir de `node:20`, il copie le projet, installe les dépendances (`npm install --force`) et lance `npm run build`.
2. **Service** : à partir de `nginx:alpine` (un petit serveur web), il copie uniquement le résultat de la construction dans `/usr/share/nginx/html/`, avec la configuration `nginx/default.conf` et le script `nginx/start.sh`. Il définit aussi des valeurs par défaut pour des variables d'environnement (`ENV KEYCLOAK_REALM="exemple"`, etc.).

Une application Angular étant du code pré-compilé, on ne peut pas lire de variables d'environnement (des réglages que le système transmet à un programme, comme `KEYCLOAK_REALM=exemple`) à l'exécution. Le dépôt contourne le problème (le `README.md` l'explique) : au démarrage du conteneur, `start.sh` remplace les marqueurs par les vraies valeurs avec `sed`, un outil de remplacement de texte.

```bash
sed -i -e 's#__KEYCLOAK_REALM__#'"$KEYCLOAK_REALM"'#g' /usr/share/nginx/html/*.js
```

- `-i` modifie le fichier en place ; `s#ancien#nouveau#g` remplace toutes les occurrences (`#` sert de séparateur).
- `"$KEYCLOAK_REALM"` est la valeur de la variable d'environnement du conteneur.
- `/usr/share/nginx/html/*.js` désigne tous les fichiers JavaScript du site.

Une même image peut ainsi servir en test et en production. Le script compresse ensuite les fichiers puis lance nginx.

Enfin, `nginx/default.conf` contient `try_files $uri $uri/ /index.html;`. Une application Angular n'a qu'une page HTML : l'adresse `/members/12` n'existe pas comme fichier. Cette ligne dit à nginx : « si le fichier demandé n'existe pas, renvoie `index.html` » ; le routeur d'Angular prend alors le relais dans le navigateur.

:::warning Tout ce qui est dans le build est public
Les valeurs remplacées par `sed` se retrouvent dans des fichiers JavaScript que n'importe qui peut télécharger. N'y mets jamais de secret (mot de passe, clé privée) : seulement des adresses et des identifiants publics, comme l'identifiant de client Keycloak.
:::

## Entraîne-toi

:::labo
moteur: reel
intro: |
  Le projet de `/workspace` contient deux choses : du code à tester (`src/app/`) et le résultat d'un **vrai build de production** de l'application (`dist/club/browser/`, des fichiers JavaScript minifiés), fait une fois pour toutes quand l'environnement a été préparé. Le conteneur n'a que 512 Mo de mémoire : `ng build` n'y tient pas toujours, et tu n'as pas besoin de le relancer pour comprendre ce qui se passe après.

  Pour les tests, `tester` joue les `*.spec.ts` avec **Vitest** (et non Karma, voir la leçon) : leur écriture (`describe`, `it`, `expect`, `TestBed`) est la même que dans un projet Angular classique. `tester` contrôle ton travail sur une copie du projet : un test qui ne vérifie rien, ou dont tu as retiré des lignes, n'est pas accepté.
commandes:
  - cp -R /opt/exercices/07-tests-et-build/. .
  - /opt/angular/preparer
etapes:
  - texte: >-
      Écris ton premier test : crée `src/app/total.spec.ts` pour la fonction `calculerTotal` de `src/app/total.ts`, avec au moins deux `it(…)` (par exemple `[5, 10]` donne `15` et la liste vide donne `0`). Lance `tester total` : au moins deux tests doivent réussir.
    indice: >-
      `import { calculerTotal } from './total';` puis `describe('calculerTotal', () => { it('additionne les prix', () => { expect(calculerTotal([5, 10])).toBe(15); }); … });`
    verif:
      - commande-reussit: tester total
      - commande-reussit: controler-ecrit src/app/total.spec.ts /opt/exercices/07-tests-et-build/src/app/total.ts /opt/angular/mutants/total
    solution:
      - ecrire:
          src/app/total.spec.ts: |
            import { calculerTotal } from './total';

            describe('calculerTotal', () => {
              it('additionne les prix', () => {
                expect(calculerTotal([5, 10])).toBe(15);
              });

              it('renvoie 0 pour une liste vide', () => {
                expect(calculerTotal([])).toBe(0);
              });
            });
  - texte: >-
      Lance `tester titre` : le test d'un composant échoue (`expected '' to be 'Mes adhérents'`) parce que le gabarit n'a jamais été calculé. Dans `src/app/titre.spec.ts`, remplace le commentaire `À FAIRE` par `fixture.detectChanges();`, qui demande à Angular de calculer le gabarit. Relance : le test passe.
    indice: >-
      L'appel va juste après `fixture = TestBed.createComponent(TitreComponent);`, dans le `beforeEach`.
    apres: [1]
    verif:
      - commande-reussit: tester titre
      - commande-reussit: contient src/app/titre.spec.ts 'fixture\.detectChanges\s*\(\s*\)'
    solution:
      - sed -i 's|    // À FAIRE (étape 2).*|    fixture.detectChanges();|' src/app/titre.spec.ts
  - texte: >-
      Dans `src/app/membres.service.spec.ts`, le test d'un service HTTP oublie de **simuler la réponse** du serveur. Remplace le commentaire `À FAIRE` par `requete.flush({ count: 0, results: [] });` : `flush` envoie la réponse simulée. Vérifie avec `tester membres.service`.
    indice: >-
      `requete` est la requête interceptée par `HttpTestingController` ; `flush(…)` lui donne la réponse qu'on veut.
    apres: [2]
    verif:
      - commande-reussit: tester membres.service
      - commande-reussit: contient src/app/membres.service.spec.ts 'requete\.flush\s*\('
    solution:
      - 'sed -i ''s|    // À FAIRE (étape 3).*|    requete.flush({ count: 0, results: [] });|'' src/app/membres.service.spec.ts'
  - texte: >-
      Passe au conteneur. Les fichiers de `dist/club/browser/` contiennent des marqueurs comme `__ADHESION_API_URL__` (essaie `grep -o __ADHESION_API_URL__ dist/club/browser/*.js`). Écris un script `start.sh` qui les remplace, avec `sed -i`, par les valeurs des variables d'environnement `ADHESION_API_URL`, `KEYCLOAK_URL`, `KEYCLOAK_REALM` et `KEYCLOAK_CLIENT_ID`, dans tous les `dist/club/browser/*.js`. Puis lance-le : `ADHESION_API_URL=https://adhesion.example.org KEYCLOAK_URL=https://keycloak.example.org KEYCLOAK_REALM=exemple KEYCLOAK_CLIENT_ID=adhesion-frontend sh start.sh`.
    indice: >-
      Une option `-e 's#__ADHESION_API_URL__#'"$ADHESION_API_URL"'#g'` par marqueur, puis le chemin `dist/club/browser/*.js` à la fin de la commande `sed -i`.
    apres: [3]
    verif:
      - commande-reussit: verifier-start
      - commande-echoue: grep -rqE '__(ADHESION_API_URL|KEYCLOAK_URL|KEYCLOAK_REALM|KEYCLOAK_CLIENT_ID)__' dist/club/browser
      - commande-reussit: grep -rq 'https://adhesion\.example\.org' dist/club/browser
    solution:
      - ecrire:
          start.sh: |
            #!/bin/sh
            sed -i -e 's#__ADHESION_API_URL__#'"$ADHESION_API_URL"'#g' \
              -e 's#__KEYCLOAK_URL__#'"$KEYCLOAK_URL"'#g' \
              -e 's#__KEYCLOAK_REALM__#'"$KEYCLOAK_REALM"'#g' \
              -e 's#__KEYCLOAK_CLIENT_ID__#'"$KEYCLOAK_CLIENT_ID"'#g' dist/club/browser/*.js
      - ADHESION_API_URL=https://adhesion.example.org KEYCLOAK_URL=https://keycloak.example.org KEYCLOAK_REALM=exemple KEYCLOAK_CLIENT_ID=adhesion-frontend sh start.sh
  - texte: >-
      Dernier maillon : le serveur web. Dans `nginx/default.conf`, remplace le commentaire `À FAIRE` du bloc `location /` par la ligne `try_files $uri $uri/ /index.html;` : si l'adresse demandée n'est pas un fichier, nginx renvoie `index.html` et le routeur d'Angular prend le relais dans le navigateur.
    indice: >-
      Une seule ligne dans le bloc `location / { … }`, terminée par un point-virgule : `try_files $uri $uri/ /index.html;`.
    verif:
      - commande-reussit: contient nginx/default.conf 'location\s+/\s*\{[^}]*try_files\s+\$uri\s+\$uri/\s+/index\.html\s*;'
    solution:
      - 'sed -i ''s|        # À FAIRE.*|        try_files $uri $uri/ /index.html;|'' nginx/default.conf'
:::

## Vérifie tes acquis

:::quiz
Pourquoi utilise-t-on `HttpTestingController` pour tester un service ?

- [ ] Pour accélérer le vrai serveur
- [ ] Pour vérifier que l'API est bien en ligne
- [x] Pour contrôler la requête envoyée et simuler la réponse, sans réseau
- [ ] Pour générer automatiquement le service

> Les tests unitaires doivent être rapides et reproductibles : on remplace le vrai réseau par un faux.
:::

:::quiz
Que fait la configuration `production` d'`angular.json` avec `environment.ts` ?

- [ ] Elle supprime le fichier
- [ ] Elle le chiffre
- [x] Elle le remplace par `environment.prod.ts` lors du build
- [ ] Elle le déplace dans `dist/` sans le modifier

> `fileReplacements` substitue un fichier par un autre à la compilation.
:::

:::quiz
À quoi servent les marqueurs `__KEYCLOAK_URL__` remplacés par `start.sh` ?

- [x] À configurer une même image Docker avec les valeurs de chaque environnement, au démarrage du conteneur
- [ ] À masquer des mots de passe dans le code
- [ ] À activer la compression gzip
- [ ] À déclarer les rôles Keycloak

> Le build ne connaît pas encore l'environnement cible. Les marqueurs sont substitués au démarrage avec les variables d'environnement. Ce ne sont pas des secrets : le résultat reste public.
:::

:::quiz
Pourquoi `nginx/default.conf` contient-il `try_files $uri $uri/ /index.html` ?

- [ ] Pour accélérer le chargement des images
- [ ] Pour interdire l'accès aux fichiers inconnus
- [x] Pour renvoyer `index.html` quand l'adresse n'est pas un fichier, afin que le routeur d'Angular prenne le relais
- [ ] Pour compresser les réponses

> L'application n'a qu'une page HTML ; les adresses comme `/members/12` sont gérées dans le navigateur.
:::
