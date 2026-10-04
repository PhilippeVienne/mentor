#!/usr/bin/env node
'use strict';
/*
 * verifier-web : vérifications « pristines » des labos du parcours HTML, CSS et JavaScript.
 *
 * Usage : verifier-web <leçon> <contrôle>     (lancé depuis /workspace ; ex. « verifier-web 04 clic »)
 *
 * Principe : l'apprenant·e peut modifier tous les fichiers de son dossier de travail (y compris les tests et
 * `index.html`), on ne leur fait donc pas confiance pour juger son travail.
 *   1. le dossier de travail est copié dans un dossier temporaire ;
 *   2. les fichiers d'origine (pris dans /opt/exercices, non modifiable) que la consigne ne demande pas de
 *      modifier sont remis par-dessus : `index.html`, `page.html`, les tests, les données du faux serveur ;
 *   3. on exige des PREUVES : sortie exacte, résultat dans le DOM (jsdom) sur plusieurs jeux de données, nombre
 *      de tests exécutés et réussis ; le code est analysé avec un vrai analyseur JavaScript (acorn), pas avec
 *      des motifs de texte, et les commentaires sont ignorés.
 * Sortie : lignes « OK » ou « ÉCHEC » ; code de retour 0 seulement si tout est vrai.
 */
const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');
const { spawnSync } = require('node:child_process');

const OUTILS = process.env.MENTOR_OUTILS || '/opt/outils';
const OPT = process.env.MENTOR_EXERCICES || '/opt/exercices';
const PAQUETS = process.env.MENTOR_PAQUETS || '/opt/paquets';
const acorn = require(path.join(OUTILS, 'node_modules', 'acorn'));

const DOSSIERS = {
  '01': '01-structure-html', '02': '02-css', '03': '03-bases-javascript',
  '04': '04-dom-et-evenements', '05': '05-api-fetch', '06': null,
};

// ── Utilitaires ──────────────────────────────────────────────────────────────
const problemes = [];
const infos = [];
const ko = (m) => problemes.push(m);
const ok = (m) => infos.push(m);
const norme = (s) => String(s).replace(/\s+/g, ' ').trim();

function fin() {
  for (const i of infos) console.log('OK     ' + i);
  for (const p of problemes) console.log('ÉCHEC  ' + p);
  process.exit(problemes.length ? 1 : 0);
}

function fichiersOrigine(dir, base = dir) {
  const sortie = [];
  for (const e of fs.readdirSync(dir, { withFileTypes: true })) {
    const p = path.join(dir, e.name);
    if (e.isDirectory()) sortie.push(...fichiersOrigine(p, base));
    else sortie.push(path.relative(base, p));
  }
  return sortie;
}

// Copie « pristine » : le travail de l'apprenant·e + les fichiers d'origine (sauf les `cibles`).
function preparer(lecon, cibles) {
  const tmp = fs.mkdtempSync(path.join(os.tmpdir(), 'verif-web-'));
  const cwd = process.cwd();
  fs.cpSync(cwd, tmp, { recursive: true, force: true, filter: (s) => !/(^|\/)\.git(\/|$)/.test(path.relative(cwd, s)) });
  const dossier = DOSSIERS[lecon];
  const origine = dossier ? path.join(OPT, dossier) : null;
  if (origine) {
    for (const rel of fichiersOrigine(origine)) {
      if (cibles.includes(rel)) continue;
      fs.mkdirSync(path.dirname(path.join(tmp, rel)), { recursive: true });
      fs.copyFileSync(path.join(origine, rel), path.join(tmp, rel));
    }
  }
  return { tmp, origine, cwd };
}

function lancer(ctx, cmd, args, delai = 9000) {
  const r = spawnSync(cmd, args, {
    cwd: ctx.tmp, encoding: 'utf8', timeout: delai,
    env: { ...process.env, NO_COLOR: '1', FORCE_COLOR: '0' },
  });
  return { code: r.status, sortie: r.stdout || '', erreur: r.stderr || '', delai: !!(r.error && r.error.code === 'ETIMEDOUT') };
}

// ── Analyse JavaScript ───────────────────────────────────────────────────────
function analyser(ctx, rel, module = false) {
  const chemin = path.join(ctx.tmp, rel);
  if (!fs.existsSync(chemin)) { ko(`le fichier ${rel} n'existe pas`); return null; }
  const texte = fs.readFileSync(chemin, 'utf8');
  const commentaires = [];
  try {
    const ast = acorn.parse(texte, { ecmaVersion: 'latest', sourceType: module ? 'module' : 'script', onComment: commentaires, allowHashBang: true });
    let code = texte;
    for (const c of [...commentaires].reverse()) code = code.slice(0, c.start) + ' '.repeat(c.end - c.start) + code.slice(c.end);
    return { ast, texte, code: norme(code), vide: norme(code) === '' };
  } catch (e) {
    ko(`${rel} contient une erreur de syntaxe : ${e.message}`);
    return null;
  }
}

function parcourir(noeud, f) {
  if (!noeud || typeof noeud.type !== 'string') return;
  f(noeud);
  for (const cle of Object.keys(noeud)) {
    const v = noeud[cle];
    if (Array.isArray(v)) v.forEach((x) => x && typeof x.type === 'string' && parcourir(x, f));
    else if (v && typeof v.type === 'string') parcourir(v, f);
  }
}
const contient = (a, pred) => { let r = false; parcourir(a.ast, (n) => { if (pred(n)) r = true; }); return r; };
const nom = (n) => (n.type === 'Identifier' ? n.name : n.type === 'MemberExpression' && !n.computed ? n.property.name : n.type === 'MemberExpression' && n.property.type === 'Literal' ? String(n.property.value) : '');
const appelle = (a, methode) => contient(a, (n) => n.type === 'CallExpression' && nom(n.callee) === methode);

// ── Exécution ────────────────────────────────────────────────────────────────
function executer(ctx, fichier, attendu, etiquette = '', prefixe = false) {
  const r = lancer(ctx, process.execPath, [fichier]);
  if (r.delai) { ko(`\`node ${fichier}\` met trop de temps à s'exécuter`); return false; }
  if (r.code !== 0) { ko(`\`node ${fichier}\` échoue : ${norme(r.erreur).slice(0, 220)}`); return false; }
  if (prefixe ? !r.sortie.startsWith(attendu) : r.sortie !== attendu) {
    ko(`\`node ${fichier}\`${etiquette} devrait afficher ${JSON.stringify(attendu.replace(/\n$/, ''))} mais affiche ${JSON.stringify(r.sortie.replace(/\n$/, '').slice(0, 200))}`);
    return false;
  }
  return true;
}

// Mutation : on change une valeur du code de l'apprenant·e et on exige que la sortie suive. Une valeur écrite
// « en dur » dans un `console.log` ne suit pas.
function muter(ctx, fichier, motif, remplacement, attendu, message, prefixe = false) {
  const chemin = path.join(ctx.tmp, fichier);
  const original = fs.readFileSync(chemin, 'utf8');
  if (!motif.test(original)) { ko(`${message} (impossible à contrôler : « ${motif.source.slice(0, 40)} » est introuvable dans ${fichier})`); return; }
  fs.writeFileSync(chemin, original.replace(motif, remplacement));
  const r = lancer(ctx, process.execPath, [fichier]);
  fs.writeFileSync(chemin, original);
  if (r.code === 0 && (prefixe ? r.sortie.startsWith(attendu) : r.sortie === attendu)) ok(message);
  else ko(`${message} : la sortie ne suit pas quand on change la donnée (affiche ${JSON.stringify(r.sortie.replace(/\n$/, '').slice(0, 120))}) ; calcule la valeur au lieu de l'écrire en dur`);
}

// ── verifier-page (jsdom) sur la copie pristine ──────────────────────────────────────────────────────────────
function page(ctx, fichier, options, message) {
  const r = lancer(ctx, process.execPath, [path.join(OUTILS, 'verifier-page.js'), fichier, ...options], 14000);
  if (r.delai) { ko(`${message} : la page met trop de temps à réagir`); return false; }
  if (r.code === 0) { ok(message); return true; }
  const echecs = r.sortie.split('\n').filter((l) => /^(ÉCHEC|\[ERREUR)/.test(l)).slice(0, 3);
  ko(`${message} : ${echecs.join(' | ') || norme(r.erreur).slice(0, 200)}`);
  return false;
}

// La consigne n'autorise que du JavaScript : le HTML doit être identique à l'original.
function htmlIntact(ctx, rel = 'index.html') {
  const reel = path.join(ctx.cwd, rel);
  const attendu = path.join(ctx.origine, rel);
  if (!fs.existsSync(reel)) { ko(`${rel} n'existe plus`); return false; }
  if (fs.readFileSync(reel, 'utf8').replace(/\r\n/g, '\n') !== fs.readFileSync(attendu, 'utf8')) {
    ko(`${rel} a été modifié : dans ce labo, tu n'écris que du JavaScript, dans \`app.js\` (remets \`${rel}\` comme au départ)`);
    return false;
  }
  return true;
}
function scriptNonVide(ctx) {
  const a = analyser(ctx, 'app.js');
  if (!a) return null;
  if (a.vide) { ko("`app.js` est vide : écris ton code JavaScript dedans"); return null; }
  return a;
}

// ── Contrôles par leçon ──────────────────────────────────────────────────────────────────────────────────────
const CONTROLES = {};

// 01 : réparer une page abîmée (le contenu d'origine doit rester)
CONTROLES['01'] = {
  cibles: { reparer: ['a-corriger.html'] },
  reparer(ctx) {
    const JSDOM = require(path.join(OUTILS, 'node_modules', 'jsdom')).JSDOM;
    const lire = (f) => new JSDOM(fs.readFileSync(f, 'utf8')).window.document;
    const avant = lire(path.join(ctx.origine, 'a-corriger.html'));
    const apres = lire(path.join(ctx.tmp, 'a-corriger.html'));
    const textes = (d, sel) => [...d.querySelectorAll(sel)].map((e) => norme(e.textContent));
    const ensembles = [['title', 'le titre'], ['h1,h2,h3,h4,h5,h6', 'les titres de la page'], ['button', 'le bouton'], ['label', 'le libellé du champ']];
    for (const [sel, quoi] of ensembles) {
      if (JSON.stringify(textes(avant, sel)) !== JSON.stringify(textes(apres, sel))) ko(`${quoi} ne doit pas changer de texte : répare la page sans la vider`);
    }
    const attrs = (d, sel, a) => [...d.querySelectorAll(sel)].map((e) => e.getAttribute(a));
    if (JSON.stringify(attrs(avant, 'img', 'src')) !== JSON.stringify(attrs(apres, 'img', 'src'))) ko("l'image doit garder son `src`");
    if (!apres.querySelector('form input#email[name=email][type=email]')) ko('le champ `#email` du formulaire doit rester');
    if (!problemes.length) ok('le contenu de la page est resté intact');
    page(ctx, 'a-corriger.html', ['--accessible'], 'la page passe les contrôles d\'accessibilité');
  },
};

// 02 : CSS (page.html est restauré : seule style.css compte)
const CSS = {
  regle: [['--style', 'h1', 'color', 'rgb(227, 38, 24)']],
  carte: [['--style', '.carte', 'width', '300px'], ['--style', '.carte', 'padding-top', '16px'], ['--style', '.carte', 'border-top-width', '2px'], ['--style', '.carte', 'box-sizing', 'border-box']],
  flex: [['--style', '.menu', 'display', 'flex'], ['--style', '.menu', 'justify-content', 'space-between'], ['--style', '.menu', 'gap', '1rem']],
  grille: [['--style', '.galerie', 'display', 'grid'], ['--style', '.galerie', 'grid-template-columns', 'repeat(3, 1fr)'], ['--style', '.galerie', 'gap', '1rem']],
};
CONTROLES['02'] = {
  cibles: { regle: ['style.css'], carte: ['style.css'], flex: ['style.css'], grille: ['style.css'], adaptatif: ['style.css'] },
  regle: (c) => css(c, CSS.regle),
  carte: (c) => css(c, CSS.carte),
  flex: (c) => css(c, CSS.flex),
  grille: (c) => css(c, CSS.grille),
  adaptatif(c) {
    css(c, [['--largeur', '400', '--style', '.galerie', 'grid-template-columns', '1fr']], 'sur un écran de téléphone (400 px)');
    css(c, [['--largeur', '767', '--style', '.galerie', 'grid-template-columns', '1fr']], 'juste sous 768 px');
    css(c, [['--largeur', '768', '--style', '.galerie', 'grid-template-columns', 'repeat(3, 1fr)']], 'à partir de 768 px');
    css(c, [['--largeur', '1280', '--style', '.galerie', 'grid-template-columns', 'repeat(3, 1fr)']], 'sur un grand écran');
  },
};
function css(ctx, groupes, quand = '') {
  const css = path.join(ctx.tmp, 'style.css');
  if (!fs.existsSync(css) || norme(fs.readFileSync(css, 'utf8').replace(/\/\*[\s\S]*?\*\//g, '')) === '') { ko('`style.css` est vide : écris-y tes règles'); return; }
  for (const g of groupes) {
    page(ctx, 'page.html', g, `la page vérifie ${g.slice(g[0] === '--largeur' ? 2 : 0).join(' ').replace(/^--style /, '')}${quand ? ' ' + quand : ''}`);
  }
}

// 03 : bases de JavaScript
CONTROLES['03'] = {
  cibles: { boutique: ['boutique.js'], prix: ['prix.js'], evenements: ['evenements.js'], copie: ['copie.js'], stats: ['statistiques.js'] },
  boutique(ctx) {
    const a = analyser(ctx, 'boutique.js');
    if (!a) return;
    const modele = contient(a, (n) => n.type === 'TemplateLiteral' && n.expressions.length >= 2);
    if (!modele) ko('affiche le texte avec un texte à backticks (`...${nom}...${places}...`)');
    if (!contient(a, (n) => n.type === 'VariableDeclaration' && n.kind === 'const' && n.declarations.some((d) => d.id.name === 'nom'))) ko('déclare une constante `nom`');
    if (!contient(a, (n) => n.type === 'VariableDeclaration' && n.kind === 'let' && n.declarations.some((d) => d.id.name === 'places'))) ko('déclare une variable `places` avec `let`');
    if (!executer(ctx, 'boutique.js', 'Club Photo : 19 places restantes\n')) return;
    ok('`node boutique.js` affiche le bon texte');
    muter(ctx, 'boutique.js', /\b20\b/, '37', 'Club Photo : 36 places restantes\n', 'le nombre de places est bien calculé');
    muter(ctx, 'boutique.js', /(["'])Club Photo\1/, '"Zorglub"', 'Zorglub : 19 places restantes\n', 'le nom est bien lu dans la constante `nom`');
  },
  prix(ctx) {
    const a = analyser(ctx, 'prix.js');
    if (!a) return;
    if (!contient(a, (n) => n.type === 'FunctionDeclaration' && n.id.name === 'prixTotal')) ko('écris la fonction `prixTotal` avec `function`');
    if (!contient(a, (n) => n.type === 'VariableDeclarator' && n.id.name === 'prixAvecRemise' && n.init && n.init.type === 'ArrowFunctionExpression')) ko('écris `prixAvecRemise` comme une fonction fléchée (`const prixAvecRemise = (…) => …`)');
    if (!executer(ctx, 'prix.js', '9\n40.5\n')) return;
    ok('`node prix.js` affiche 9 puis 40.5');
    fs.copyFileSync(path.join(OUTILS, 'controles', '03', 'prix.controle.js'), path.join(ctx.tmp, 'prix.controle.js'));
    const r = lancer(ctx, process.execPath, ['prix.controle.js']);
    const m = r.sortie.trim().split('\n').pop().match(/^CONTROLES-REUSSIS (\d+)$/);
    if (r.code === 0 && m && Number(m[1]) === 6) ok('6 contrôles réussis sur `prixTotal` et `prixAvecRemise`');
    else ko('les fonctions `prixTotal` et `prixAvecRemise` ne donnent pas les bons résultats : ' + norme(r.erreur || r.sortie).slice(0, 220));
  },
  evenements(ctx) {
    const a = analyser(ctx, 'evenements.js');
    if (!a) return;
    const original = fs.readFileSync(path.join(ctx.origine, 'evenements.js'), 'utf8');
    const attendu = norme(original.replace(/\/\/[^\n]*/g, ''));
    if (!a.code.startsWith(attendu)) ko('le tableau `evenements` (données fournies) ne doit pas être modifié : écris ton code en dessous');
    if (!appelle(a, 'filter') || !appelle(a, 'map')) ko('utilise `filter` puis `map` pour construire `disponibles`');
    if (!appelle(a, 'find')) ko('utilise `find` pour trouver `grand`');
    if (!executer(ctx, 'evenements.js', "[ 'Sortie photo', 'Exposition' ]\nExposition\n")) return;
    ok('`node evenements.js` affiche les bons résultats');
    muter(ctx, 'evenements.js', /places: 20\b/, 'places: 40', "[ 'Sortie photo', 'Exposition' ]\nSortie photo\n", '`grand` est bien le premier événement de plus de 30 places');
    muter(ctx, 'evenements.js', /places: 0\b/, 'places: 5', "[ 'Sortie photo', 'Atelier retouche', 'Exposition' ]\nExposition\n", '`disponibles` suit les places des événements');
  },
  copie(ctx) {
    const a = analyser(ctx, 'copie.js');
    if (!a) return;
    if (!contient(a, (n) => n.type === 'VariableDeclarator' && n.id.type === 'ObjectPattern')) ko('utilise une déstructuration (`const { titre, places } = evenement;`)');
    if (!contient(a, (n) => n.type === 'ObjectExpression' && n.properties.some((p) => p.type === 'SpreadElement'))) ko("fais la copie avec l'opérateur de décomposition (`{ ...evenement, places: 19 }`)");
    if (!executer(ctx, 'copie.js', 'Sortie photo 20\n19 20\n')) return;
    ok('`node copie.js` affiche les bons résultats');
    muter(ctx, 'copie.js', /\b20\b/, '35', 'Sortie photo 35\n19 35\n', 'la déstructuration lit bien les valeurs de `evenement`');
  },
  stats(ctx) {
    const a = analyser(ctx, 'statistiques.js');
    if (!a) return;
    if (a.vide) { ko('`statistiques.js` est vide'); return; }
    const test = 'statistiques.controle.test.js';
    fs.copyFileSync(path.join(OUTILS, 'controles', '03', 'statistiques.test.js'), path.join(ctx.tmp, test));
    // Le test d'origine du dossier de travail est remplacé par la copie d'origine : on lance la copie.
    const r = lancer(ctx, process.execPath, ['--test', '--test-reporter=tap', test], 14000);
    const compte = (cle) => { const m = r.sortie.match(new RegExp('^# ' + cle + ' (\\d+)$', 'm')); return m ? Number(m[1]) : null; };
    const total = compte('tests'), reussis = compte('pass'), echecs = compte('fail'), ignores = compte('skipped');
    if (total !== null && total >= 6 && reussis === total && echecs === 0 && !ignores) ok(`${reussis} tests exécutés, ${reussis} réussis`);
    else {
      ko(`les tests ne passent pas (exécutés : ${total}, réussis : ${reussis}, échecs : ${echecs})`);
      const detail = r.sortie.split('\n').filter((l) => /^\s*(not ok|# Subtest|error:)/.test(l)).slice(0, 3).map(norme).join(' | ');
      if (detail) ko('   ' + detail);
    }
  },
};

// 04 : DOM et événements (index.html est remis à l'original ET doit être intact)
CONTROLES['04'] = {
  cibles: { titre: ['app.js'], compteur: ['app.js'], plancher: ['app.js'], liste: ['app.js'], formulaire: ['app.js'] },
  titre(ctx) {
    if (!htmlIntact(ctx) || !scriptNonVide(ctx)) return;
    page(ctx, 'index.html', ['--egal', '#titre', "Club Photo du campus"], 'le titre est remplacé par le script');
  },
  compteur(ctx) {
    if (!htmlIntact(ctx) || !scriptNonVide(ctx)) return;
    page(ctx, 'index.html', ['--clic', '#inscription', '--clic', '#inscription', '--egal', '#compteur', '18'], 'deux clics donnent 18');
    page(ctx, 'index.html', ['--clics', '#inscription', '5', '--egal', '#compteur', '15'], 'cinq clics donnent 15');
    page(ctx, 'index.html', ['--egal', '#compteur', '20'], 'sans clic, le compteur reste à 20');
  },
  plancher(ctx) {
    if (!htmlIntact(ctx) || !scriptNonVide(ctx)) return;
    page(ctx, 'index.html', ['--clics', '#inscription', '25', '--egal', '#compteur', '0'], 'vingt-cinq clics laissent 0');
    page(ctx, 'index.html', ['--clics', '#inscription', '20', '--egal', '#compteur', '0'], 'vingt clics laissent 0');
    page(ctx, 'index.html', ['--clics', '#inscription', '19', '--egal', '#compteur', '1'], 'dix-neuf clics laissent 1');
  },
  liste(ctx) {
    if (!htmlIntact(ctx)) return;
    const a = scriptNonVide(ctx);
    if (!a) return;
    if (contient(a, (n) => n.type === 'MemberExpression' && ['innerHTML', 'outerHTML', 'insertAdjacentHTML'].includes(nom(n)))) ko("n'utilise pas `innerHTML` : crée les éléments avec `createElement`");
    if (!appelle(a, 'createElement')) ko('crée les `<li>` avec `document.createElement("li")`');
    page(ctx, 'index.html', ['--egal', '#liste li:nth-child(1)', 'Camille', '--egal', '#liste li:nth-child(2)', 'Noé', '--egal', '#liste li:nth-child(3)', 'Inès', '--absent', '#liste li:nth-child(4)'], 'la liste contient Camille, Noé et Inès');
    // Mutation : la liste vient bien du tableau écrit dans le script
    const f = path.join(ctx.tmp, 'app.js');
    const original = fs.readFileSync(f, 'utf8');
    if (/(["'`])Camille\1/.test(original)) {
      fs.writeFileSync(f, original.replace(/(["'`])Camille\1/, '"Zoé"'));
      page(ctx, 'index.html', ['--egal', '#liste li:nth-child(1)', 'Zoé'], 'la liste suit le tableau de prénoms du script');
      fs.writeFileSync(f, original);
    } else ko('les prénoms doivent être écrits dans le script (un tableau `["Camille", "Noé", "Inès"]`)');
  },
  formulaire(ctx) {
    if (!htmlIntact(ctx) || !scriptNonVide(ctx)) return;
    page(ctx, 'index.html', ['--saisie', '#email', 'camille@example.org', '--envoyer', '#formulaire', '--egal', '#message', 'Inscription de camille@example.org'], 'le message accueille camille@example.org');
    page(ctx, 'index.html', ['--saisie', '#email', 'noe@example.org', '--envoyer', '#formulaire', '--egal', '#message', 'Inscription de noe@example.org'], 'le message suit l\'adresse saisie (noe@example.org)');
  },
};

// 05 : fetch (index.html et faux serveur d'origine ; un second faux serveur change les données)
const API_ALT = [{ id: 7, titre: "Tir à l'arc", places: 3 }, { id: 8, titre: 'Yoga', places: 12 }, { id: 9, titre: 'Complet', places: 0 }];
function api2(ctx) {
  fs.mkdirSync(path.join(ctx.tmp, 'api-alt'), { recursive: true });
  fs.writeFileSync(path.join(ctx.tmp, 'api-alt', 'evenements.json'), JSON.stringify(API_ALT));
}
CONTROLES['05'] = {
  cibles: { liste: ['app.js'], statut: ['app.js'], erreurs: ['app.js'], envoi: ['app.js'] },
  liste(ctx) {
    if (!htmlIntact(ctx) || !scriptNonVide(ctx)) return;
    page(ctx, 'index.html', ['--egal', '#evenements li:nth-child(1)', 'Sortie photo (20 places)', '--egal', '#evenements li:nth-child(2)', 'Atelier retouche (0 places)', '--appelle', 'GET', '/api/evenements'], 'la liste affiche les événements du serveur');
    api2(ctx);
    page(ctx, 'index.html', ['--api', 'api-alt', '--egal', '#evenements li:nth-child(1)', "Tir à l'arc (3 places)", '--egal', '#evenements li:nth-child(3)', 'Complet (0 places)', '--absent', '#evenements li:nth-child(4)'], 'la liste suit les données reçues (autre jeu de données)');
  },
  statut(ctx) {
    if (!htmlIntact(ctx) || !scriptNonVide(ctx)) return;
    page(ctx, 'index.html', ['--vide', '#statut', '--existe', '#evenements li'], 'le message de chargement est effacé');
    api2(ctx);
    page(ctx, 'index.html', ['--api', 'api-alt', '--vide', '#statut', '--existe', '#evenements li:nth-child(3)'], 'le message est effacé avec un autre jeu de données');
  },
  erreurs(ctx) {
    if (!htmlIntact(ctx)) return;
    const a = scriptNonVide(ctx);
    if (!a) return;
    if (!contient(a, (n) => n.type === 'TryStatement' && n.handler)) ko('entoure le chargement d\'un `try … catch`');
    page(ctx, 'index.html', ['--erreur', '500', '--contient', '#statut', 'Impossible de charger les événements', '--absent', '#evenements li'], 'une erreur 500 est gérée');
    page(ctx, 'index.html', ['--erreur', '404', '--contient', '#statut', 'Impossible de charger les événements', '--absent', '#evenements li'], 'une erreur 404 est gérée');
    page(ctx, 'index.html', ['--panne', '--contient', '#statut', 'Impossible de charger les événements', '--absent', '#evenements li'], 'une panne réseau est gérée');
    page(ctx, 'index.html', ['--vide', '#statut', '--existe', '#evenements li'], 'sans erreur, la liste s\'affiche et le message disparaît');
  },
  envoi(ctx) {
    if (!htmlIntact(ctx)) return;
    const a = scriptNonVide(ctx);
    if (!a) return;
    if (!contient(a, (n) => n.type === 'CallExpression' && nom(n.callee) === 'stringify')) ko('construis le corps avec `JSON.stringify`');
    for (const email of ['camille@example.org', 'noe@example.org']) {
      const motif = '^\\{"email":"' + email.replace(/\./g, '\\.') + '"\\}$';
      page(ctx, 'index.html', ['--saisie', '#email', email, '--envoyer', '#inscription', '--appelle', 'POST', '/api/inscriptions', '--corps', motif, '--entete', 'content-type', 'application/json', '--contient', '#statut', 'Inscription enregistrée'], `l'inscription de ${email} est envoyée en POST avec un corps JSON`);
    }
  },
};

// 06 : projet Node.js (pas d'exercice d'origine : on contrôle le résultat des commandes)
function pkg(ctx) {
  try { return JSON.parse(fs.readFileSync(path.join(ctx.tmp, 'package.json'), 'utf8')); } catch { ko("`package.json` est absent ou invalide : lance `npm init -y`"); return null; }
}
CONTROLES['06'] = {
  cibles: { modules: [], script: [], paquet: [], utiliser: [] },
  modules(ctx) {
    const p = pkg(ctx);
    if (!p) return;
    if (p.type !== 'module') ko('`package.json` doit contenir `"type": "module"`');
    const prix = analyser(ctx, 'prix.js', true);
    const app = analyser(ctx, 'app.js', true);
    if (!prix || !app) return;
    const exportsNommes = new Set();
    parcourir(prix.ast, (n) => {
      if (n.type === 'ExportNamedDeclaration') {
        if (n.declaration && n.declaration.id) exportsNommes.add(n.declaration.id.name);
        if (n.declaration && n.declaration.declarations) n.declaration.declarations.forEach((d) => exportsNommes.add(d.id.name));
        (n.specifiers || []).forEach((s) => exportsNommes.add(s.exported.name));
      }
    });
    for (const e of ['prixTotal', 'TVA']) if (!exportsNommes.has(e)) ko(`\`prix.js\` doit exporter \`${e}\` (export nommé)`);
    if (!contient(prix, (n) => n.type === 'ExportDefaultDeclaration')) ko('`prix.js` doit avoir un export par défaut (`formater`)');
    if (!contient(app, (n) => n.type === 'ImportDeclaration' && n.source.value === './prix.js')) ko('`app.js` doit importer `./prix.js`');
    fs.copyFileSync(path.join(OUTILS, 'controles', '06', 'prix.controle.mjs'), path.join(ctx.tmp, 'prix.controle.mjs'));
    const r = lancer(ctx, process.execPath, ['prix.controle.mjs']);
    const m = r.sortie.trim().split('\n').pop().match(/^CONTROLES-REUSSIS (\d+)$/);
    if (r.code === 0 && m && Number(m[1]) === 4) ok('4 contrôles réussis sur le module `prix.js`');
    else ko('le module `prix.js` ne se comporte pas comme demandé : ' + norme(r.erreur || r.sortie).slice(0, 200));
    if (!executer(ctx, 'app.js', '13.50 €\n16.20 €\n', '', true)) return;
    ok('`node app.js` affiche 13.50 € puis 16.20 €');
    muter(ctx, 'app.js', /\b4\.5\b/, '10', '30.00 €\n36.00 €\n', 'le total est bien calculé par `prixTotal` et mis en forme par `formater`', true);
  },
  script(ctx) {
    const p = pkg(ctx);
    if (!p) return;
    const s = p.scripts && p.scripts.demo;
    if (typeof s === 'string' && /^node app\.js$/.test(s.trim())) ok('le script `demo` lance `node app.js`'); else ko('le script `demo` doit valoir `node app.js`');
    const r = lancer(ctx, 'npm', ['run', '--silent', 'demo']);
    if (r.code === 0 && r.sortie.startsWith('13.50 €\n16.20 €\n')) ok('`npm run demo` affiche les deux montants'); else ko('`npm run demo` doit afficher 13.50 € puis 16.20 € : ' + norme(r.erreur || r.sortie).slice(0, 160));
  },
  paquet(ctx) {
    const p = pkg(ctx);
    if (!p) return;
    const dep = p.dependencies && p.dependencies['mini-date'];
    if (typeof dep === 'string' && /mini-date/.test(dep)) ok('`mini-date` est dans les `dependencies`'); else ko('`mini-date` doit être dans la section `dependencies` de `package.json` : `npm install /opt/paquets/mini-date`');
    const installe = path.join(ctx.tmp, 'node_modules', 'mini-date', 'index.js');
    const modele = path.join(PAQUETS, 'mini-date', 'index.js');
    if (!fs.existsSync(installe)) ko('`node_modules/mini-date` est absent');
    else if (fs.readFileSync(installe, 'utf8') !== fs.readFileSync(modele, 'utf8')) ko('`node_modules/mini-date` ne contient pas le paquet attendu');
    else ok('`node_modules/mini-date` contient le vrai paquet');
  },
  utiliser(ctx) {
    const app = analyser(ctx, 'app.js', true);
    if (!app) return;
    if (!contient(app, (n) => n.type === 'ImportDeclaration' && n.source.value === 'mini-date' && n.specifiers.some((s) => s.imported && s.imported.name === 'dateFr'))) ko('`app.js` doit contenir `import { dateFr } from "mini-date";`');
    const installe = path.join(ctx.tmp, 'node_modules', 'mini-date', 'index.js');
    if (!fs.existsSync(installe) || fs.readFileSync(installe, 'utf8') !== fs.readFileSync(path.join(PAQUETS, 'mini-date', 'index.js'), 'utf8')) { ko('installe d\'abord le vrai paquet : `npm install /opt/paquets/mini-date`'); return; }
    if (!executer(ctx, 'app.js', '13.50 €\n16.20 €\n03/10/2026\n')) return;
    ok('`node app.js` affiche les trois lignes');
    muter(ctx, 'app.js', /Date\.UTC\(\s*2026\s*,\s*9\s*,\s*3\s*\)/, 'Date.UTC(2027, 0, 15)', '13.50 €\n16.20 €\n15/01/2027\n', 'la date est bien formatée par `dateFr` du paquet');
  },
};

// ── Programme principal ──────────────────────────────────────────────────────────────────────────────────────
const [lecon, controle] = process.argv.slice(2);
if (!(lecon in DOSSIERS) || !CONTROLES[lecon] || typeof CONTROLES[lecon][controle] !== 'function') {
  console.error('Usage : verifier-web <leçon> <contrôle>');
  process.exit(2);
}
const ctx = preparer(lecon, CONTROLES[lecon].cibles[controle] || []);
try {
  CONTROLES[lecon][controle](ctx);
} catch (e) {
  ko("erreur de l'outil de vérification : " + (e && e.stack ? e.stack.split('\n').slice(0, 3).join(' ') : e));
} finally {
  try { fs.rmSync(ctx.tmp, { recursive: true, force: true }); } catch { /* sans importance */ }
}
fin();
