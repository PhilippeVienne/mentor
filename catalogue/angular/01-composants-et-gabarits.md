---
id: composants-et-gabarits
title: "Composants, modules et gabarits"
summary: "Lire et écrire un composant Angular : classe, gabarit, liaisons et blocs de contrôle."
minutes: 45
objectives:
  - Décrire les trois parties d'un composant (classe, gabarit, styles)
  - Utiliser les liaisons `{{ }}`, `[ ]`, `( )` et `[( )]`
  - Écrire un gabarit avec `@if` et `@for`
  - Distinguer un composant autonome (*standalone*) d'un composant déclaré dans un `NgModule`
---

Tu ouvres un écran de l'administration d'**Adhésion** (le site web que l'équipe utilise pour gérer les adhérent·e·s des associations étudiantes) et tu veux comprendre d'où vient chaque bouton, chaque liste, chaque message. Dans Angular, tout écran est un arbre de **composants**. Si tu sais en lire un, tu sais lire tous les autres.

## À quoi sert Angular, et pourquoi ?

Un site web classique renvoie une nouvelle page **HTML** (le langage qui décrit le contenu d'une page web) à chaque clic. Une application comme l'administration d'Adhésion (recherche instantanée, fenêtres, tableaux qui se mettent à jour) ressemble plus à un logiciel : la page se charge **une seule fois** puis du code JavaScript la modifie. On parle d'application « monopage » (*Single Page Application*).

Écrire ce code à la main devient vite un enchevêtrement. **Angular** est un *framework* (une boîte à outils avec des règles d'organisation, développée par Google) qui impose une structure commune : des composants pour l'affichage, des services pour la logique, un routeur pour les adresses. Résultat : quand tu arrives sur un projet, tu sais où chercher.

Tu auras besoin de la **CLI Angular**, la commande `ng` : par exemple `ng serve` lance l'application en local et `ng generate component nom` crée les fichiers d'un composant. Et tu écris en **TypeScript**, vu dans le cours précédent.

Image mentale pour la suite : un composant, c'est une **brique Lego**. Une page est un assemblage de briques ; chaque brique sait afficher sa partie et réagir à ses propres clics.

## Anatomie d'un composant

Un composant, c'est une **classe TypeScript** décorée par `@Component`, qui contient trois choses : des données et des méthodes (la classe), du HTML (le **gabarit**, *template*) et des styles (écrits en **CSS**, le langage qui décrit l'apparence d'une page : couleurs, tailles, marges).

```ts
import { Component } from '@angular/core';

@Component({
  selector: 'app-bienvenue',
  template: `<h2>Bienvenue au {{ nomClub }} !</h2>`,
  styles: `h2 { color: #E32618; }`,
})
export class BienvenueComponent {
  nomClub = 'Club photo';
}
```

Ligne par ligne :

- `import { Component } from '@angular/core';` va chercher l'outil `Component` dans la bibliothèque Angular.
- `@Component({ … })` est un **décorateur** : une étiquette placée au-dessus de la classe qui lui donne un rôle (ici « cette classe est un composant ») et sa configuration.
- `selector` est le nom de la balise HTML que tu écris pour afficher le composant : `<app-bienvenue />`. Le préfixe `app-` est une convention d'Adhésion pour éviter les conflits avec les vraies balises HTML.
- `template` contient le HTML ; sur un vrai projet on le sépare avec `templateUrl: './fichier.component.html'`.
- `styles` contient le CSS, limité à ce composant : ce `h2` (un titre de page) n'affecte pas le reste de la page. Dans `h2 { color: #E32618; }`, `h2` désigne les éléments visés et `color` leur couleur, écrite en code hexadécimal.
- `export class BienvenueComponent` est la classe : elle porte l'état. Ici `nomClub`, que le gabarit lit avec `{{ nomClub }}` (les doubles accolades affichent la valeur).
- Si `nomClub` change dans la classe, l'affichage se met à jour tout seul.

Le projet Adhésion sépare toujours les trois fichiers, par exemple `list.component.ts`, `list.component.html` et `list.component.scss`. **SCSS** est une version enrichie du CSS : on y écrit les mêmes règles de style avec quelques facilités en plus (par exemple imbriquer une règle dans une autre), puis un outil le convertit en CSS pour le navigateur. Ce découpage en trois fichiers est la convention de la commande `ng generate component`.

## Les quatre liaisons

Le gabarit parle à la classe avec quatre syntaxes. Retiens-les par la forme des symboles.

| Syntaxe | Sens | Exemple |
| --- | --- | --- |
| `{{ valeur }}` | classe vers affichage (texte) | `{{ membre.prenom }}` |
| `[propriete]="valeur"` | classe vers propriété d'un élément | `[disabled]="envoiEnCours"` |
| `(evenement)="action()"` | élément vers classe (évènement) | `(click)="enregistrer()"` |
| `[(ngModel)]="champ"` | les deux sens (champ de saisie) | `[(ngModel)]="recherche"` |

Dans le dépôt d'Adhésion, la page de recherche d'adhérent·e·s (`members/list/list.component.html`) en utilise plusieurs :

```html
<input (keyup)="term$.next($event.target.value)" [(ngModel)]="searchbox" matInput type="search" />
...
<div class="name">{{ member.first_name }} {{ member.last_name | uppercase }}</div>
```

Dans la première ligne :

- `<input … type="search" />` est un champ de saisie HTML pour une recherche ; `matInput` est un attribut d'Angular Material qui l'habille (leçon 5).
- `(keyup)="term$.next(…)"` : à chaque touche relâchée, appelle du code de la classe (`term$` est un objet qui transmet les valeurs saisies, expliqué à la leçon 4) ; `$event` est l'objet décrivant l'évènement (ici, il donne accès au champ de saisie).
- `[(ngModel)]="searchbox"` : garde la variable `searchbox` et le contenu du champ synchronisés dans les deux sens.

La seconde ligne utilise un **tube** (*pipe*) : `| uppercase` transforme la valeur avant l'affichage, sans toucher aux données.

## Conditions et boucles : les blocs de contrôle

Depuis Angular 17, le gabarit dispose de blocs `@if`, `@for` et `@switch`, plus lisibles que les anciennes directives. Voici un composant complet, avec une liste de membres d'un club fictif.

```ts
import { UpperCasePipe } from '@angular/common';
import { Component } from '@angular/core';

interface Membre {
  id: number;
  prenom: string;
  nom: string;
  cotisationPayee: boolean;
}

@Component({
  selector: 'app-liste-membres',
  imports: [UpperCasePipe],
  template: `
    <h2>Membres ({{ membres.length }})</h2>

    @if (membres.length === 0) {
      <p>Aucun membre pour l'instant.</p>
    } @else {
      <ul>
        @for (m of membres; track m.id) {
          <li>
            {{ m.prenom }} {{ m.nom | uppercase }}
            @if (!m.cotisationPayee) {
              <strong>(cotisation à régler)</strong>
            }
          </li>
        }
      </ul>
    }

    <button type="button" (click)="ajouter()">Ajouter un membre</button>
  `,
})
export class ListeMembresComponent {
  membres: Membre[] = [
    { id: 1, prenom: 'Camille', nom: 'Durand', cotisationPayee: true },
    { id: 2, prenom: 'Sam', nom: 'Martin', cotisationPayee: false },
  ];

  ajouter() {
    const id = this.membres.length + 1;
    this.membres = [...this.membres, { id, prenom: 'Nouveau', nom: 'Membre', cotisationPayee: false }];
  }
}
```

Lecture du code :

- `interface Membre` décrit la forme d'un objet (ses champs et leurs types) ; elle n'existe que pour TypeScript.
- `imports: [UpperCasePipe]` rend le tube `uppercase` utilisable dans le gabarit.
- `{{ membres.length }}` affiche le nombre d'éléments du tableau.
- `@if (condition) { … } @else { … }` affiche l'un ou l'autre bloc.
- `@for (m of membres; track m.id) { … }` répète le bloc pour chaque élément, en nommant l'élément courant `m`.
- `(click)="ajouter()"` appelle la méthode `ajouter` de la classe au clic.

Trois remarques :

- `@for` **exige** `track` : Angular s'en sert pour savoir quels éléments ont changé et ne redessiner que ceux-là. Prends un identifiant stable (`m.id`).
- `@if … @else` remplace le couple `*ngIf` / `else`.
- On remplace le tableau par une nouvelle copie (`[...this.membres, …]`) plutôt que de le modifier en place : c'est plus simple à raisonner.

Une **directive** est une instruction ajoutée à un élément HTML pour modifier son comportement. Les anciennes directives `*ngIf` et `*ngFor` existent toujours, et **le code d'Adhésion les utilise encore** (`*ngIf="loading"`, `*ngFor="let member of lastSearchResult.results"`). Quand tu modifies un fichier existant, garde son style ; pour un nouvel écran, préfère `@if` et `@for`.

## Données entrantes et sortantes

Une page est un emboîtement : un composant « parent » contient des composants « enfants » (une liste contient des cartes). Le parent **donne** des données à l'enfant, et l'enfant le **prévient** par des évènements, comme une brique qui reçoit une consigne et renvoie un signal.

```ts
import { Component, input, output } from '@angular/core';

@Component({
  selector: 'app-carte-membre',
  template: `
    <article>
      <h3>{{ prenom() }}</h3>
      <button type="button" (click)="selectionne.emit(prenom())">Voir la fiche</button>
    </article>
  `,
})
export class CarteMembreComponent {
  readonly prenom = input.required<string>();
  readonly selectionne = output<string>();
}
```

Ligne par ligne : `input.required<string>()` déclare une valeur que le parent **doit** fournir ; `output<string>()` déclare un évènement que l'enfant peut émettre avec `.emit(valeur)` ; dans le gabarit, on lit une entrée en l'appelant comme une fonction : `prenom()`.

Le parent écrit `<app-carte-membre [prenom]="m.prenom" (selectionne)="ouvrir($event)" />`. Les fonctions `input()` et `output()` sont l'écriture moderne ; le code d'Adhésion utilise encore les décorateurs équivalents `@Input()` et `@Output() … = new EventEmitter<…>()`, par exemple dans `VaChequeReceivedBtnComponent`.

## Composants autonomes ou NgModule

Angular doit savoir quelles balises sont connues dans un gabarit : sinon `<app-carte-membre>` serait pour lui une balise HTML inconnue. Il existe deux façons de le lui dire.

:::cards
### Composant autonome (*standalone*)

Le composant déclare lui-même ses dépendances dans `imports: [...]`. C'est le **comportement par défaut depuis Angular 19** : tu n'as rien à écrire de plus.

### NgModule

Un **NgModule** (« module Angular ») est un classeur qui liste ses composants dans `declarations` et ses dépendances dans `imports`. Le composant porte alors `standalone: false`.
:::

Le frontend d'Adhésion est en Angular 20, mais il est resté **organisé en modules** : `main.ts` démarre `AppModule` avec `bootstrapModule`, et `app.module.ts` liste dans `declarations` tous les composants de l'application (`AppComponent`, `ListComponent`, `MemberViewComponent`…). C'est pour cela que chacun porte `standalone: false` :

```ts
@Component({
  selector: 'app-list',
  templateUrl: './list.component.html',
  styleUrls: ['./list.component.scss'],
  standalone: false,
})
export class ListComponent implements OnInit { /* … */ }
```

- `selector: 'app-list'` : la balise `<app-list>` affiche ce composant.
- `templateUrl` et `styleUrls` : le gabarit et les styles sont dans des fichiers à part (au lieu de `template` et `styles` écrits dans la classe).
- `standalone: false` : le composant n'est pas autonome, il doit être déclaré dans un `NgModule`.
- `implements OnInit` : la classe promet de fournir une méthode `ngOnInit()` qu'Angular appelle une fois, à la création du composant, pour charger ses données.

Dans ce dépôt, quelques éléments tiers sont eux autonomes et se placent dans `imports` d'`AppModule` : `QRCodeComponent` (`angularx-qrcode`) et `BaseChartDirective` (`ng2-charts`).

:::warning Un composant oublié dans `declarations`
Si tu crées un composant dans Adhésion sans l'ajouter à `declarations` d'`AppModule`, la compilation échoue avec une erreur du type « 'app-mon-composant' is not a known element ». La commande `ng generate component` ajoute la ligne pour toi : utilise-la plutôt que de créer les fichiers à la main.
:::

## Entraîne-toi

:::lab
engine: real
intro: |
  Tu travailles dans un petit projet Angular déjà prêt (le dossier `/workspace`) : un club photo avec une liste de membres. Ton travail : écrire le **gabarit** (le HTML d'un composant) de `src/app/liste-membres.html`, puis compléter une carte de membre.

  Les fichiers `*.spec.ts` sont des **tests** : de petits programmes qui vérifient ce que l'écran affiche. Ne les modifie pas, fais-les passer (`tester` remet de toute façon les tests d'origine dans une copie : les modifier ne servirait à rien). La commande `tester` les lance : elle compile ton code avec le compilateur d'Angular en mode strict, puis joue les tests dans un faux navigateur en mémoire (il n'y a pas de vrai navigateur dans ce terminal). Une ligne `✓` signifie « test réussi », une ligne `×` « test échoué ». Édite les fichiers avec `nano`.
commands:
  - cp -R /opt/exercices/01-composants-et-gabarits/. .
  - /opt/angular/preparer
steps:
  - text: >-
      Dans `src/app/liste-membres.html`, fais afficher le nombre de membres dans le titre avec une liaison `{{ }}` : le titre doit devenir `Membres (2)`. Vérifie avec `tester liste-membres -t "nombre de membres"` : la ligne du test doit afficher ✓.
    hint: >-
      `<h2>Membres ({{ membres.length }})</h2>` : `membres` est le tableau de la classe `ListeMembresComponent`, `length` donne sa taille.
    checks:
      - command-succeeds: tester liste-membres -t 'nombre de membres'
      - command-succeeds: contient src/app/liste-membres.html '\{\{\s*membres\.length\s*\}\}'
    solution:
      - write:
          src/app/liste-membres.html: |
            <h2>Membres ({{ membres.length }})</h2>
            <button type="button">Ajouter un membre</button>
  - text: >-
      Affiche la liste : une balise `<ul>` qui contient, grâce à un bloc `@for` (avec `track m.id`), un `<li>` par membre au format `Camille DURAND` (le nom en majuscules avec le tube `uppercase`). Un tube doit être **importé** : dans `liste-membres.ts`, ajoute `UpperCasePipe` (de `@angular/common`) à `imports`. Sans cela, `tester` répond « No pipe found with name 'uppercase' ». Vérifie avec `tester liste-membres -t majuscules`.
    hint: >-
      `@for (m of membres; track m.id) { <li>{{ m.prenom }} {{ m.nom | uppercase }}</li> }` dans une balise `<ul>`, et `imports: [UpperCasePipe]` dans le décorateur.
    after: [1]
    checks:
      - command-succeeds: tester liste-membres -t majuscules
      - command-succeeds: contient src/app/liste-membres.html '@for\s*\(.*track\s+m\.id'
      - command-succeeds: contient src/app/liste-membres.html '\|\s*uppercase'
      - command-succeeds: contient src/app/liste-membres.ts 'imports\s*:\s*\[[^\]]*UpperCasePipe'
    solution:
      - write:
          src/app/liste-membres.ts: |
            import { UpperCasePipe } from '@angular/common';
            import { Component } from '@angular/core';
            import { Membre } from './membre';

            @Component({
              selector: 'app-liste-membres',
              imports: [UpperCasePipe],
              templateUrl: './liste-membres.html',
            })
            export class ListeMembresComponent {
              membres: Membre[] = [
                { id: 1, prenom: 'Camille', nom: 'Durand', cotisationPayee: true },
                { id: 2, prenom: 'Sam', nom: 'Martin', cotisationPayee: false },
              ];

              ajouter() {
                const id = this.membres.length + 1;
                this.membres = [...this.membres, { id, prenom: 'Nouveau', nom: 'Membre', cotisationPayee: false }];
              }
            }
      - write:
          src/app/liste-membres.html: |
            <h2>Membres ({{ membres.length }})</h2>
            <ul>
              @for (m of membres; track m.id) {
                <li>{{ m.prenom }} {{ m.nom | uppercase }}</li>
              }
            </ul>
            <button type="button">Ajouter un membre</button>
  - text: >-
      Avec un bloc `@if`, ajoute dans chaque `<li>` la mention `<strong>(cotisation à régler)</strong>`, seulement pour les membres dont `cotisationPayee` est faux. Vérifie avec `tester liste-membres -t cotisation`.
    hint: >-
      Après le nom, dans le `<li>` : `@if (!m.cotisationPayee) { <strong>(cotisation à régler)</strong> }`. Le `!` veut dire « n'est pas ».
    after: [2]
    checks:
      - command-succeeds: tester liste-membres -t cotisation
      - command-succeeds: contient src/app/liste-membres.html '@if\s*\(.*cotisationPayee'
    solution:
      - write:
          src/app/liste-membres.html: |
            <h2>Membres ({{ membres.length }})</h2>
            <ul>
              @for (m of membres; track m.id) {
                <li>
                  {{ m.prenom }} {{ m.nom | uppercase }}
                  @if (!m.cotisationPayee) {
                    <strong>(cotisation à régler)</strong>
                  }
                </li>
              }
            </ul>
            <button type="button">Ajouter un membre</button>
  - text: >-
      Gère la liste vide : s'il n'y a aucun membre, affiche `<p>Aucun membre pour l'instant.</p>` à la place de la liste (un `@if (membres.length === 0) { … } @else { … }` autour du `<ul>`). Vérifie avec `tester liste-membres -t vide`.
    hint: >-
      `@if (membres.length === 0) { <p>Aucun membre pour l'instant.</p> } @else { <ul>…</ul> }` : le `<ul>` existant passe dans le bloc `@else`.
    after: [3]
    checks:
      - command-succeeds: tester liste-membres -t vide
      - command-succeeds: contient src/app/liste-membres.html '@if\s*\(.*membres\.length'
      - command-succeeds: contient src/app/liste-membres.html '@else'
    solution:
      - write:
          src/app/liste-membres.html: |
            <h2>Membres ({{ membres.length }})</h2>
            @if (membres.length === 0) {
              <p>Aucun membre pour l'instant.</p>
            } @else {
              <ul>
                @for (m of membres; track m.id) {
                  <li>
                    {{ m.prenom }} {{ m.nom | uppercase }}
                    @if (!m.cotisationPayee) {
                      <strong>(cotisation à régler)</strong>
                    }
                  </li>
                }
              </ul>
            }
            <button type="button">Ajouter un membre</button>
  - text: >-
      Fais réagir le bouton : sur la balise `<button>`, ajoute une liaison d'évènement qui appelle la méthode `ajouter()` de la classe au clic. Le test clique sur le bouton et attend trois membres. Vérifie avec `tester liste-membres -t "au clic"`.
    hint: >-
      `<button type="button" (click)="ajouter()">Ajouter un membre</button>` : les parenthèses désignent un évènement.
    after: [4]
    checks:
      - command-succeeds: tester liste-membres -t 'au clic'
      - command-succeeds: contient src/app/liste-membres.html '\(click\)\s*=\s*.\s*ajouter\(\)'
    solution:
      - write:
          src/app/liste-membres.html: |
            <h2>Membres ({{ membres.length }})</h2>
            @if (membres.length === 0) {
              <p>Aucun membre pour l'instant.</p>
            } @else {
              <ul>
                @for (m of membres; track m.id) {
                  <li>
                    {{ m.prenom }} {{ m.nom | uppercase }}
                    @if (!m.cotisationPayee) {
                      <strong>(cotisation à régler)</strong>
                    }
                  </li>
                }
              </ul>
            }
            <button type="button" (click)="ajouter()">Ajouter un membre</button>
  - text: >-
      Ouvre `src/app/carte-membre.ts` et fais de la carte un composant qui dialogue avec son parent : une entrée **obligatoire** `prenom` (`input.required<string>()`), une sortie `selectionne` (`output<string>()`), un `<h3>` qui affiche le prénom (en l'appelant comme une fonction : `prenom()`) et un bouton qui émet le prénom au clic (`selectionne.emit(prenom())`). Pense à importer `input` et `output` de `@angular/core`. Termine par `tester` sans argument : tout le projet doit être vert.
    hint: >-
      Dans la classe : `readonly prenom = input.required<string>();` et `readonly selectionne = output<string>();`. Dans le gabarit : `<h3>{{ prenom() }}</h3>` et `(click)="selectionne.emit(prenom())"`.
    after: [5]
    checks:
      - command-succeeds: tester
      - command-succeeds: contient src/app/carte-membre.ts 'input\.required\s*<\s*string\s*>'
      - command-succeeds: contient src/app/carte-membre.ts '\boutput\s*<\s*string\s*>\s*\('
    solution:
      - write:
          src/app/carte-membre.ts: |-
            import { Component, input, output } from '@angular/core';

            @Component({
              selector: 'app-carte-membre',
              template: `
                <article>
                  <h3>{{ prenom() }}</h3>
                  <button type="button" (click)="selectionne.emit(prenom())">Voir la fiche</button>
                </article>
              `,
            })
            export class CarteMembreComponent {
              readonly prenom = input.required<string>();
              readonly selectionne = output<string>();
            }
:::

## Vérifie tes acquis

:::quiz
Que fait `[disabled]="envoiEnCours"` sur un bouton ?

- [ ] Il affiche le texte « envoiEnCours » dans le bouton
- [ ] Il appelle une méthode `envoiEnCours()` au clic
- [x] Il lie la propriété `disabled` du bouton à la valeur de `envoiEnCours` dans la classe
- [ ] Il crée une variable `disabled` dans la classe

> Les crochets désignent une liaison de propriété : la valeur de la classe est recopiée dans la propriété de l'élément, à chaque changement.
:::

:::quiz
Pourquoi `@for (m of membres; track m.id)` impose-t-il `track` ?

- [x] Pour qu'Angular identifie chaque élément et ne redessine que ceux qui ont changé
- [ ] Pour trier la liste par identifiant
- [ ] Pour empêcher l'ajout de doublons dans le tableau
- [ ] Pour afficher l'identifiant devant chaque ligne

> `track` donne une identité stable à chaque élément. Sans elle, Angular ne pourrait pas distinguer un élément déplacé d'un élément remplacé.
:::

:::quiz
Dans `app.module.ts` d'Adhésion, les composants sont listés dans `declarations` et portent `standalone: false`. Que faut-il faire pour un nouveau composant du même style ?

- [ ] Rien : Angular détecte tous les fichiers `*.component.ts`
- [x] L'ajouter à `declarations` d'`AppModule`
- [ ] L'ajouter à `providers`
- [ ] Ajouter `standalone: true` et l'importer dans `main.ts`

> Un composant non autonome doit appartenir à un module. `ng generate component` fait cet ajout automatiquement.
:::

:::quiz
Quelle écriture est la forme moderne d'une donnée reçue du parent ?

- [ ] `@Input() prenom: string;` uniquement, car c'est la seule forme qui existe
- [ ] `prenom = new EventEmitter<string>();`
- [x] `readonly prenom = input.required<string>();`

> `input()` et `output()` sont les fonctions modernes. Les décorateurs `@Input()` et `@Output()` fonctionnent toujours et restent présents dans Adhésion.
:::
