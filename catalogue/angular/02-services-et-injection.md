---
id: services-et-injection
titre: "Services et injection de dépendances"
resume: "Sortir la logique des composants dans des services et laisser Angular les fournir."
duree: 35
objectifs:
  - Expliquer à quoi sert un service et ce qu'est l'injection de dépendances
  - Écrire un service avec `@Injectable` et l'injecter avec `inject()` ou le constructeur
  - "Savoir où un service est fourni (`providedIn: 'root'` ou `providers` d'un module)"
  - Remplacer un service par un autre grâce à `{ provide, useClass }`
---

Deux écrans de ton application ont besoin de la même liste de membres, ou du même titre de page. Si chaque composant la recharge ou la stocke à sa façon, les copies se désynchronisent. La solution d'Angular : un **service**, un objet unique que les composants se partagent.

## Un service, c'est une classe

Analogie : dans une association, plusieurs personnes ont besoin du classeur des adhésions. Plutôt que d'en photocopier un par personne (avec des versions qui divergent), on en garde **un seul**, accessible à tous. Un **service** joue ce rôle dans l'application : c'est un objet unique, sans affichage, qui contient de la logique (appeler l'**API**, c'est-à-dire interroger par Internet le serveur qui détient les données d'Adhésion ; calculer) ou un état partagé (la liste chargée, le titre courant).

Techniquement, c'est une classe ordinaire marquée par le décorateur `@Injectable` (une étiquette qui dit à Angular « tu peux fabriquer et distribuer cette classe »).

Le frontend d'Adhésion en a un exemple minimal, `UiService`. Il permet à n'importe quel écran de changer le titre affiché par `AppComponent` :

```ts
@Injectable()
export class UiService {
  private subject = new Subject<any>();

  setTitle(message: string) {
    this.subject.next(message);
  }

  getTitle(): Observable<any> {
    return this.subject.asObservable();
  }
}
```

(Extrait simplifié de `src/app/data/ui.service.ts`.)

- `@Injectable()` marque la classe comme un service qu'Angular peut fabriquer.
- `private subject` est un canal interne, invisible de l'extérieur (`private`) ; `Subject<any>` veut dire qu'il transporte des valeurs de n'importe quel type (`any`).
- `setTitle` envoie un nouveau titre dans le canal (`next`).
- `getTitle` permet aux autres d'**écouter** le canal : `asObservable()` leur donne une vue en lecture seule, sans qu'ils puissent y envoyer de valeur.

Les `Subject` et `Observable` (des canaux de valeurs dans le temps) sont expliqués à la leçon 4 ; pour l'instant, retiens qu'un écran appelle `setTitle` et que l'écran racine écoute `getTitle` pour afficher le titre.

## L'injection de dépendances

Comment un composant obtient-il le service ? Un composant ne crée **jamais** ses services avec `new`. Il **demande** ce dont il a besoin et Angular le lui fournit : c'est l'**injection de dépendances** (DI). Il existe deux écritures.

```ts
import { Component, inject } from '@angular/core';

@Component({
  selector: 'app-liste-membres',
  template: `<p>Page : {{ titre }}</p>`,
})
export class ListeMembresComponent {
  // Écriture moderne : la fonction inject()
  private readonly ui = inject(UiService);
  titre = 'Membres';

  constructor() {
    this.ui.setTitle(this.titre);
  }
}
```

```ts
// Écriture historique : paramètre du constructeur
constructor(private ui: UiService) {}
```

Ligne par ligne : `inject(UiService)` demande à Angular l'instance du service ; `private readonly` veut dire qu'elle n'est visible que dans la classe et qu'on ne la remplace pas. Dans l'écriture historique, déclarer le paramètre `private ui: UiService` dans le constructeur (la fonction appelée à la création du composant) fait la même demande.

Les deux donnent le même résultat. Le code d'Adhésion mélange les deux : `MembersService` et la plupart des composants utilisent le constructeur (`constructor(private members: MembersService)`), tandis que `UserComponent` et `AppComponent` utilisent `inject(Keycloak)` (**Keycloak** est le serveur qui gère les connexions des utilisateur·rice·s : leçon 6). Dans un fichier existant, reste cohérent avec ce qui s'y trouve.

## Où le service est-il fourni ?

Pour qu'Angular sache fabriquer un service, il faut le **fournir** quelque part : lui dire « voici la recette de cet objet, et son périmètre ». Un `providers` est cette liste de recettes. Deux manières :

:::cartes
### `providedIn: 'root'`

```ts
@Injectable({ providedIn: 'root' })
export class MembresService { /* … */ }
```

Une seule instance pour toute l'application, sans rien déclarer d'autre. C'est la façon la plus simple, et celle de `AuthGuard` dans Adhésion.

### Liste `providers`

```ts
@NgModule({
  providers: [UiService, MembersService],
})
export class AppModule {}
```

Le service est déclaré à la main dans le module (ou dans `bootstrapApplication`). C'est le cas de `UiService`, `MembersService` ou `WeiService` dans `app.module.ts`.
:::

Dans les deux cas, tous les composants qui demandent le service reçoivent **la même instance** : c'est ce qui permet le partage d'état.

## Remplacer une implémentation

Le tableau `providers` accepte aussi une forme longue : « quand quelqu'un demande `X`, donne-lui `Y` ». Adhésion s'en sert pour traduire les libellés du paginateur de Material :

```ts
{ provide: MatPaginatorIntl, useClass: FrenchMatPaginatorIntl },
```

`MatPaginatorIntl` est une classe de la bibliothèque Material (leçon 5) qui contient les textes du bas des tableaux paginés. Ici, `FrenchMatPaginatorIntl` étend la classe d'origine en changeant les textes (« Page suivante », « Éléments par page : »). Le même mécanisme sert dans les tests, pour fournir un faux service à la place du vrai (leçon 7).

```ts
@Injectable({ providedIn: 'root' })
export class Notifications {
  prevenir(message: string) {
    console.log(message);
  }
}

@Injectable({ providedIn: 'root' })
export class NotificationsSilencieuses extends Notifications {
  override prevenir(_message: string) {
    /* ne fait rien */
  }
}

// Dans la configuration : tout le monde reçoit désormais la version silencieuse
// providers: [{ provide: Notifications, useClass: NotificationsSilencieuses }]
```

Lecture du code :

- `Notifications.prevenir` écrit le message dans la console du navigateur (`console.log`, l'outil de débogage que tu ouvres avec F12).
- `NotificationsSilencieuses extends Notifications` : la deuxième classe **hérite** de la première, c'est-à-dire qu'elle est aussi une `Notifications`, mais en remplaçant ce qu'elle veut.
- `override prevenir(_message: string)` redéfinit la méthode ; `override` est obligatoire dans ce cas. Le `_` devant `_message` dit « ce paramètre ne sert pas ».
- Les deux classes sont `providedIn: 'root'` : Angular sait fabriquer l'une et l'autre. La recette en commentaire dit laquelle donner quand on demande `Notifications`.

:::warning NullInjectorError : « No provider for … »
Si tu injectes un service qui n'est pas fourni, l'application plante au chargement de l'écran avec `NullInjectorError: No provider for MonService`. Deux causes classiques : le service est marqué `@Injectable()` sans `providedIn` et tu as oublié de l'ajouter aux `providers` du module (comme `UiService` dans Adhésion) ; ou bien le service est fourni dans un autre module que celui de l'écran concerné.
:::

## Entraîne-toi

:::labo
moteur: reel
intro: |
  Même principe qu'à la leçon précédente : un petit projet Angular est prêt dans `/workspace`, avec des tests (`*.spec.ts`) que tu ne modifies pas (`tester` remet de toute façon les tests d'origine dans une copie : les modifier ne servirait à rien). Tu écris les services et les réglages pour les faire passer. Lance-les avec `tester` suivi d'une partie du nom du fichier de test (par exemple `tester entete`) ; sans argument, `tester` joue tous les tests du projet.

  Les tests demandent des services à Angular comme le ferait l'application : ils ne fabriquent rien avec `new`.
commandes:
  - cp -R /opt/exercices/02-services-et-injection/. .
  - /opt/angular/preparer
etapes:
  - texte: >-
      Crée `src/app/compteur-visites.service.ts` : une classe `CompteurVisites`, décorée par `@Injectable({ providedIn: 'root' })`, avec une propriété `visites` (qui vaut `0` au départ) et une méthode `enregistrer()` qui ajoute 1 à `visites` puis **renvoie** la nouvelle valeur. Vérifie avec `tester compteur-visites`.
    indice: >-
      `visites = 0;` puis `enregistrer(): number { this.visites = this.visites + 1; return this.visites; }`. N'oublie pas `import { Injectable } from '@angular/core';`.
    verif:
      - commande-reussit: tester compteur-visites
      - commande-reussit: contient src/app/compteur-visites.service.ts '@Injectable\s*\(\s*\{\s*providedIn\s*:\s*[\x27\"]root[\x27\"]'
    solution:
      - ecrire:
          src/app/compteur-visites.service.ts: |
            import { Injectable } from '@angular/core';

            @Injectable({ providedIn: 'root' })
            export class CompteurVisites {
              visites = 0;

              enregistrer(): number {
                this.visites = this.visites + 1;
                return this.visites;
              }
            }
  - texte: >-
      Dans `src/app/entete.ts`, demande le service à Angular avec `inject(CompteurVisites)` (sans `new`), puis, dans le constructeur du composant, appelle `enregistrer()` et garde le résultat dans `visites`. Deux composants doivent se partager le même compteur. Vérifie avec `tester entete`.
    indice: >-
      `private readonly compteur = inject(CompteurVisites);` puis `constructor() { this.visites = this.compteur.enregistrer(); }`. Importe `inject` depuis `@angular/core`.
    apres: [1]
    verif:
      - commande-reussit: tester entete
      - commande-reussit: contient src/app/entete.ts 'inject\s*\(\s*CompteurVisites\s*\)'
      - commande-reussit: contient -v src/app/entete.ts 'new\s+CompteurVisites'
    solution:
      - ecrire:
          src/app/entete.ts: |
            import { Component, inject } from '@angular/core';
            import { CompteurVisites } from './compteur-visites.service';

            @Component({
              selector: 'app-entete',
              template: `<p>Visites : {{ visites }}</p>`,
            })
            export class EnteteComponent {
              private readonly compteur = inject(CompteurVisites);
              visites = 0;

              constructor() {
                this.visites = this.compteur.enregistrer();
              }
            }
  - texte: >-
      Dans `src/app/fournisseurs.ts`, ajoute à la liste la recette `{ provide: Notifications, useClass: NotificationsSilencieuses }` (importe les deux classes depuis `./notifications`) : quand on demande `Notifications`, Angular doit fabriquer la version silencieuse. Vérifie avec `tester notifications`.
    indice: >-
      `export const fournisseurs: Provider[] = [{ provide: Notifications, useClass: NotificationsSilencieuses }];`
    apres: [2]
    verif:
      - commande-reussit: tester notifications
      - commande-reussit: contient src/app/fournisseurs.ts 'provide\s*:\s*Notifications\s*,\s*useClass\s*:\s*NotificationsSilencieuses'
    solution:
      - ecrire:
          src/app/fournisseurs.ts: |
            import { Provider } from '@angular/core';
            import { Notifications, NotificationsSilencieuses } from './notifications';

            export const fournisseurs: Provider[] = [{ provide: Notifications, useClass: NotificationsSilencieuses }];
  - texte: >-
      Lance `tester ecran` : tu obtiens une `NullInjectorError: No provider for UiService`. `UiService` est un `@Injectable()` simple, et personne ne l'a fourni. Corrige `src/app/ui.service.ts` pour qu'Angular le fournisse tout seul à toute l'application, puis relance `tester ecran`.
    indice: >-
      Remplace `@Injectable()` par `@Injectable({ providedIn: 'root' })` dans `src/app/ui.service.ts`.
    apres: [3]
    verif:
      - commande-reussit: tester ecran
      - commande-reussit: contient src/app/ui.service.ts '@Injectable\s*\(\s*\{[^}]*providedIn'
    solution:
      - ecrire:
          src/app/ui.service.ts: |
            import { Injectable } from '@angular/core';
            import { Observable, Subject } from 'rxjs';

            @Injectable({ providedIn: 'root' })
            export class UiService {
              private subject = new Subject<string>();

              setTitle(message: string) {
                this.subject.next(message);
              }

              getTitle(): Observable<string> {
                return this.subject.asObservable();
              }
            }
  - texte: >-
      Modernise `src/app/ecran.ts` : remplace le paramètre de constructeur `private ui: UiService` par `private readonly ui = inject(UiService);` et garde un constructeur vide de paramètre qui appelle `this.ui.setTitle('Membres')`. `tester ecran` doit rester vert et le fichier ne doit plus contenir `constructor(private`.
    indice: >-
      `private readonly ui = inject(UiService);` puis `constructor() { this.ui.setTitle('Membres'); }` ; importe `inject` depuis `@angular/core`.
    apres: [4]
    verif:
      - commande-reussit: tester ecran
      - commande-reussit: contient src/app/ecran.ts 'inject\(UiService\)'
      - commande-reussit: contient -v src/app/ecran.ts 'constructor\s*\(\s*private'
    solution:
      - ecrire:
          src/app/ecran.ts: |
            import { Component, inject } from '@angular/core';
            import { UiService } from './ui.service';

            @Component({
              selector: 'app-ecran-membres',
              template: `<h2>Membres</h2>`,
            })
            export class EcranMembresComponent {
              private readonly ui = inject(UiService);

              constructor() {
                this.ui.setTitle('Membres');
              }
            }
:::

## Vérifie tes acquis

:::quiz
Pourquoi n'écrit-on pas `new MembersService()` dans un composant ?

- [ ] Parce que TypeScript l'interdit pour les classes avec décorateur
- [ ] Parce que ce serait plus lent à l'exécution
- [x] Parce qu'Angular doit fournir l'instance : elle est partagée et ses propres dépendances (comme `HttpClient`) sont injectées
- [ ] Parce que les services sont des fonctions et non des classes

> Avec `new`, tu contournes la DI : plus de partage d'instance, plus de dépendances injectées, et les tests ne peuvent plus substituer un faux service.
:::

:::quiz
Quelle est la différence entre `@Injectable()` et `@Injectable({ providedIn: 'root' })` ?

- [ ] Aucune : les deux se comportent exactement pareil
- [ ] Avec `providedIn`, il y a une instance par composant
- [x] Avec `providedIn: 'root'`, le service est fourni automatiquement ; avec `@Injectable()` seul, il faut l'ajouter à une liste `providers`
- [ ] `@Injectable()` seul rend le service accessible à toutes les applications

> `UiService` d'Adhésion est un `@Injectable()` simple, listé dans les `providers` d'`AppModule`. `AuthGuard` utilise `providedIn: 'root'`.
:::

:::quiz
Que signifie `{ provide: MatPaginatorIntl, useClass: FrenchMatPaginatorIntl }` ?

- [x] Quand un composant demande `MatPaginatorIntl`, Angular lui donne une instance de `FrenchMatPaginatorIntl`
- [ ] `FrenchMatPaginatorIntl` est supprimé au profit de `MatPaginatorIntl`
- [ ] Les deux classes sont fusionnées en une seule
- [ ] Le service n'est créé qu'une fois que l'utilisateur change de langue

> `provide` est la clé demandée, `useClass` est la classe réellement instanciée. C'est ainsi qu'on personnalise une bibliothèque ou qu'on injecte un faux service en test.
:::
