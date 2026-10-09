#!/usr/bin/env node
// verifier-ts : vérifications « pristines » des labos du cours TypeScript.
//
// Usage : verifier-ts <leçon> <contrôle>      (lancé depuis /workspace ; ex. « verifier-ts 03 premier »)
//
// Principe : on ne fait JAMAIS confiance aux fichiers que l'apprenant·e peut modifier pour juger son travail.
//   1. le dossier de travail est copié dans un dossier temporaire ;
//   2. tous les fichiers d'origine (pris dans /opt/exercices, non modifiable) que la consigne ne demande pas
//      de modifier sont remis par-dessus ;
//   3. TypeScript est lancé avec son API sur cette copie, en forçant `strict` (le tsconfig de l'apprenant·e ne
//      peut donc pas affaiblir le contrôle) ; `@ts-nocheck`, `@ts-ignore` et tout `@ts-expect-error` en plus de
//      ceux de l'exercice sont refusés, ainsi que `any` ;
//   4. le code est exécuté avec tsx et on exige des PREUVES : sortie exacte attendue, ou contrôles d'un fichier de
//      test d'origine (/opt/verif/controles) qui doit annoncer le nombre exact de contrôles réussis.
// Sortie : lignes « OK » ou « ÉCHEC » ; code de retour 0 seulement si tout est vrai.
import { createRequire } from 'node:module';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { spawnSync } from 'node:child_process';

const require = createRequire('/opt/outils/package.json');
const ts = require('typescript');

const OPT = process.env.MENTOR_EXERCICES || '/opt/exercices';
const CTL = process.env.MENTOR_CONTROLES || '/opt/verif/controles';
const DOSSIERS = {
  '01': '01-pourquoi-typer',
  '02': '02-types-de-base',
  '03': '03-fonctions-generiques',
  '04': '04-tsconfig',
  '05': '05-typer-api',
  '06': '06-lint-formatage',
};
const FAMILLE_STRICT = [
  'noImplicitAny', 'strictNullChecks', 'strictFunctionTypes', 'strictBindCallApply',
  'strictPropertyInitialization', 'noImplicitThis', 'useUnknownInCatchVariables', 'alwaysStrict',
];

// ── Utilitaires ──────────────────────────────────────────────────────────────
const problemes = [];
const infos = [];
const ko = (m) => problemes.push(m);
const ok = (m) => infos.push(m);
function fin() {
  for (const i of infos) console.log('OK     ' + i);
  for (const p of problemes) console.log('ÉCHEC  ' + p);
  process.exit(problemes.length ? 1 : 0);
}
const norme = (s) => s.replace(/\s+/g, ' ').trim();

function copier(src, dst, filtre) {
  fs.cpSync(src, dst, { recursive: true, force: true, filter: filtre, dereference: false });
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

// Prépare la copie « pristine » : travail de l'apprenant·e + fichiers d'origine (sauf les `cibles`).
function preparer(dossier, cibles) {
  const tmp = fs.mkdtempSync(path.join(os.tmpdir(), 'verif-ts-'));
  const cwd = process.cwd();
  copier(cwd, tmp, (s) => !/(^|\/)(node_modules|dist|\.git)(\/|$)/.test(path.relative(cwd, s)));
  const origine = path.join(OPT, dossier);
  for (const rel of fichiersOrigine(origine)) {
    if (cibles.includes(rel)) continue;
    fs.mkdirSync(path.dirname(path.join(tmp, rel)), { recursive: true });
    fs.copyFileSync(path.join(origine, rel), path.join(tmp, rel));
  }
  fs.symlinkSync('/opt/outils/node_modules', path.join(tmp, 'node_modules'));
  return { tmp, origine, cibles, dossier };
}

function nettoyer(ctx) {
  try { fs.rmSync(ctx.tmp, { recursive: true, force: true }); } catch { /* sans importance */ }
}

function lancer(ctx, cmd, args, opts = {}) {
  const r = spawnSync(cmd, args, {
    cwd: ctx.tmp, encoding: 'utf8', timeout: opts.delai || 9000, input: opts.entree || '',
    env: { ...process.env, NO_COLOR: '1', FORCE_COLOR: '0' },
  });
  return { code: r.status, sortie: r.stdout || '', erreur: r.stderr || '', delai: r.error && r.error.code === 'ETIMEDOUT' };
}
const bin = (ctx, nom) => path.join(ctx.tmp, 'node_modules', '.bin', nom);

// ── Analyse du code (jamais de regex brute sur les commentaires ou les chaînes) ───────────────────────────────
function analyser(ctx, rel) {
  const chemin = path.join(ctx.tmp, rel);
  if (!fs.existsSync(chemin)) return null;
  const texte = fs.readFileSync(chemin, 'utf8');
  return ts.createSourceFile(rel, texte, ts.ScriptTarget.Latest, true, ts.ScriptKind.TS);
}

function commentaires(sf) {
  const trouves = new Map();
  const visiter = (n) => {
    for (const r of ts.getLeadingCommentRanges(sf.text, n.getFullStart()) || []) trouves.set(r.pos, r);
    for (const r of ts.getTrailingCommentRanges(sf.text, n.getEnd()) || []) trouves.set(r.pos, r);
    n.getChildren(sf).forEach(visiter);
  };
  visiter(sf);
  return [...trouves.values()].sort((a, b) => a.pos - b.pos).map((r) => ({ ...r, texte: sf.text.slice(r.pos, r.end) }));
}

function codeSansCommentaires(sf) {
  let t = sf.text;
  for (const c of commentaires(sf).reverse()) t = t.slice(0, c.pos) + ' '.repeat(c.end - c.pos) + t.slice(c.end);
  return norme(t);
}

function parcourir(noeud, f) {
  f(noeud);
  ts.forEachChild(noeud, (e) => parcourir(e, f));
}
const contient = (sf, pred) => { let r = false; parcourir(sf, (n) => { if (pred(n)) r = true; }); return r; };
const trouver = (sf, pred) => { const l = []; parcourir(sf, (n) => { if (pred(n)) l.push(n); }); return l; };
const K = ts.SyntaxKind;

function nbDirectives(sf) {
  const c = { 'ts-nocheck': 0, 'ts-ignore': 0, 'ts-expect-error': 0, 'eslint-disable': 0 };
  for (const cm of commentaires(sf)) {
    for (const nom of Object.keys(c)) if (cm.texte.includes('@' + nom) || (nom === 'eslint-disable' && /eslint-disable/.test(cm.texte))) c[nom]++;
  }
  return c;
}

// ── Vérification des types, sur la copie pristine, `strict` imposé ─────────────────────────────────────────────
function optionsDepuis(ctx, tsconfig = 'tsconfig.json') {
  const cfg = path.join(ctx.tmp, tsconfig);
  const brut = ts.readConfigFile(cfg, ts.sys.readFile);
  if (brut.error) { ko(`${tsconfig} : ${ts.flattenDiagnosticMessageText(brut.error.messageText, ' ')}`); return null; }
  const parse = ts.parseJsonConfigFileContent(brut.config, ts.sys, ctx.tmp);
  const erreurs = parse.errors.filter((e) => e.code !== 18003); // 18003 : aucun fichier à compiler
  for (const e of erreurs) ko(`${tsconfig} : ${ts.flattenDiagnosticMessageText(e.messageText, ' ')}`);
  const o = { ...parse.options, strict: true, noEmit: true, skipLibCheck: true, noCheck: false };
  for (const k of FAMILLE_STRICT) o[k] = true;
  delete o.suppressImplicitAnyIndexErrors;
  return { options: o, fichiers: parse.fileNames.map((f) => path.relative(ctx.tmp, f)), config: brut.config };
}

function programme(ctx, racines, options) {
  return ts.createProgram(racines.map((r) => path.join(ctx.tmp, r)), options);
}

const sourcesApprenant = (ctx, prog) => prog.getSourceFiles()
  .map((sf) => ({ sf, rel: path.relative(ctx.tmp, sf.fileName) }))
  .filter(({ rel }) => !rel.startsWith('..') && !rel.includes('node_modules'));

function typer(ctx, racines, { options, filtreFichier, sansAny = true, ignorerCodes = [] } = {}) {
  const opt = options || optionsDepuis(ctx);
  if (!opt) return null;
  for (const r of racines) {
    if (!fs.existsSync(path.join(ctx.tmp, r))) { ko(`le fichier ${r} n'existe pas`); return null; }
  }
  const prog = programme(ctx, racines, opt.options || opt);
  const o = opt.options || opt;
  // 1. diagnostics TypeScript
  const diags = ts.getPreEmitDiagnostics(prog).filter((d) => !ignorerCodes.includes(d.code));
  const retenus = diags.filter((d) => !d.file || !path.relative(ctx.tmp, d.file.fileName).includes('node_modules'));
  const vus = [];
  for (const d of retenus) {
    const rel = d.file ? path.relative(ctx.tmp, d.file.fileName) : '(global)';
    if (filtreFichier && d.file && !filtreFichier(rel)) continue;
    const pos = d.file ? d.file.getLineAndCharacterOfPosition(d.start) : null;
    vus.push({ rel, code: d.code, texte: `${rel}${pos ? `(${pos.line + 1},${pos.character + 1})` : ''} : TS${d.code} ${ts.flattenDiagnosticMessageText(d.messageText, ' ').slice(0, 160)}` });
  }
  // 2. directives d'inhibition et `any`
  const ficApp = sourcesApprenant(ctx, prog);
  for (const { sf, rel } of ficApp) {
    const att = path.join(ctx.origine, rel);
    const avant = fs.existsSync(att) ? nbDirectives(ts.createSourceFile(rel, fs.readFileSync(att, 'utf8'), ts.ScriptTarget.Latest, true)) : { 'ts-nocheck': 0, 'ts-ignore': 0, 'ts-expect-error': 0 };
    const apres = nbDirectives(sf);
    for (const nom of ['ts-nocheck', 'ts-ignore', 'ts-expect-error']) {
      if (apres[nom] > avant[nom]) ko(`${rel} : « @${nom} » désactive la vérification des types, retire-le et corrige l'erreur`);
    }
    const aCorriger = ctx.cibles.includes(rel) || !fs.existsSync(att);
    if (sansAny && aCorriger && contient(sf, (n) => n.kind === K.AnyKeyword)) ko(`${rel} : le type explicite \`any\` désactive la vérification, utilise un vrai type`);
    if (aCorriger && sf.text.trim() !== '' && codeSansCommentaires(sf) === '') ko(`${rel} : le fichier ne contient plus de code`);
  }
  return { prog, diags: vus, fichiers: ficApp.map((x) => x.rel) };
}

// Exige zéro diagnostic (hors codes ignorés) et en liste quelques-uns.
function exigerSansErreur(res, quoi = 'TypeScript') {
  if (!res) return;
  if (res.diags.length) {
    ko(`${quoi} signale encore ${res.diags.length} erreur(s) :`);
    for (const d of res.diags.slice(0, 4)) ko('   ' + d.texte);
  } else ok(`${quoi} ne signale aucune erreur (mode strict)`);
}

function exigerAbsenceCode(res, codes, message) {
  if (!res) return;
  const trouves = res.diags.filter((d) => codes.includes(d.code));
  if (trouves.length) { ko(message + ' : elle est toujours signalée'); ko('   ' + trouves[0].texte); } else ok(message);
}

// ── Preuves d'exécution ─────────────────────────────────────────────────────────────────────────────────────
function executer(ctx, fichier, attendu, { regexp } = {}) {
  const r = lancer(ctx, bin(ctx, 'tsx'), [fichier]);
  if (r.delai) { ko(`${fichier} met trop de temps à s'exécuter`); return r; }
  if (r.code !== 0) {
    ko(`\`npx tsx ${fichier}\` échoue : ${norme(r.erreur).slice(0, 200)}`);
    return r;
  }
  if (attendu !== undefined) {
    const bon = regexp ? new RegExp(attendu).test(r.sortie.trim()) : r.sortie === attendu;
    if (bon) ok(`\`npx tsx ${fichier}\` affiche ce qui est attendu`);
    else ko(`\`npx tsx ${fichier}\` devrait afficher ${JSON.stringify(attendu.replace(/\n$/, ''))} mais affiche ${JSON.stringify(r.sortie.replace(/\n$/, '').slice(0, 200))}`);
  }
  return r;
}

// Lance un fichier de contrôle d'origine (copié dans la copie pristine sous le nom `_controle.ts`).
function controle(ctx, nom, nbAttendu, cibleTexte) {
  const src = path.join(CTL, ctx.dossier.slice(0, 2), nom);
  const dst = path.join(ctx.tmp, '_controle.ts');
  fs.copyFileSync(src, dst);
  const r = lancer(ctx, bin(ctx, 'tsx'), ['_controle.ts']);
  const lignes = r.sortie.trim().split('\n');
  const derniere = lignes[lignes.length - 1] || '';
  const m = derniere.match(/^CONTROLES-REUSSIS (\d+)$/);
  if (r.code === 0 && m && Number(m[1]) === nbAttendu) ok(`${nbAttendu} contrôles réussis (${cibleTexte})`);
  else {
    ko(`${cibleTexte} ne passe pas les contrôles`);
    const detail = norme(r.erreur || r.sortie).slice(0, 300);
    if (detail) ko('   ' + detail);
  }
}

function exige(ctx, rel, regexp, message) {
  const sf = analyser(ctx, rel);
  if (!sf) { ko(`le fichier ${rel} n'existe pas`); return; }
  if (regexp.test(codeSansCommentaires(sf))) ok(message);
  else ko(message);
}

// ── Contrôles par leçon ──────────────────────────────────────────────────────────────────────────────────────
const CONTROLES = {
  // 01 : lire et corriger les erreurs de evenements.ts
  '01': {
    propriete(c) {
      const r = typer(c, ['evenements.ts']);
      exigerAbsenceCode(r, [2551, 2339], 'La propriété inconnue de la ligne 3 est corrigée');
      exige(c, 'evenements.ts', /console\.log\(evenement\.titre\.toUpperCase\(\)\);/, 'Le `console.log` lit bien `evenement.titre`');
      exige(c, 'evenements.ts', /const evenement = \{ titre: "Soirée d'intégration", lieu: "Amphi Chappe", places: 120 \};/, "L'objet `evenement` est intact");
    },
    appel(c) {
      const r = typer(c, ['evenements.ts']);
      exigerAbsenceCode(r, [2345], "L'appel de `reserver` est corrigé");
      exige(c, 'evenements.ts', /function reserver\(places: number, demandees: number\): number \{ return places - demandees; \}/, 'La fonction `reserver` garde ses types');
      exige(c, 'evenements.ts', /reserver\(120, -?\d+\);/, "`reserver` est appelée avec des nombres");
    },
    compteur(c) {
      const r = typer(c, ['evenements.ts']);
      exigerAbsenceCode(r, [2322], "L'affectation de `compteur` est corrigée");
      exige(c, 'evenements.ts', /let compteur = 3;/, '`compteur` est toujours déduit comme un nombre');
      exige(c, 'evenements.ts', /compteur = -?\d+(\.\d+)?;/, 'On range un nombre dans `compteur`');
    },
    dist(c) {
      const r = typer(c, ['evenements.ts']);
      exigerSansErreur(r);
      const reel = path.join(process.cwd(), 'dist', 'evenements.js');
      if (!fs.existsSync(reel)) { ko("`dist/evenements.js` n'existe pas : lance `npx tsc --noEmit false --outDir dist`"); return; }
      if (r) {
        const o = { ...optionsDepuis(c).options, noEmit: false, outDir: path.join(c.tmp, '_dist') };
        const prog = programme(c, ['evenements.ts'], o);
        let attendu = null;
        prog.emit(undefined, (nom, data) => { if (nom.endsWith('evenements.js')) attendu = data; });
        const obtenu = fs.readFileSync(reel, 'utf8');
        if (attendu !== null && obtenu.trim() === attendu.trim()) ok('`dist/evenements.js` est exactement la sortie de `tsc`');
        else ko('`dist/evenements.js` ne correspond pas à ce que `tsc` produit pour `evenements.ts` : relance la commande');
        if (/:\s*number/.test(obtenu)) ko('`dist/evenements.js` contient encore des types');
      }
    },
  },

  // 02 : types de base
  '02': {
    statut(c) {
      const r = typer(c, ['statut.ts']);
      exigerSansErreur(r);
      exige(c, 'statut.ts', /type StatutInscription = "en_attente" \| "confirmee" \| "annulee";/, "L'union `StatutInscription` est intacte");
      exige(c, 'statut.ts', /let statut: StatutInscription = /, '`statut` est toujours de type `StatutInscription`');
      executer(c, 'statut.ts', 'confirmee\n');
    },
    evenement(c) {
      const r = typer(c, ['evenement.ts']);
      exigerSansErreur(r);
      exige(c, 'evenement.ts', /interface Evenement \{ id: number; titre: string; lieu: string \| null; places: number; description\?: string; \}/, "L'interface `Evenement` est intacte");
      executer(c, 'evenement.ts', "^Soirée d'intégration [0-9]+$", { regexp: true });
    },
    lieu(c) {
      const r = typer(c, ['lieu.ts']);
      exigerSansErreur(r);
      exige(c, 'lieu.ts', /console\.log\(afficherLieu\(\{ id: 1, titre: "Gala", lieu: null, places: 200 \}\)\);/, 'Le premier appel de test est intact');
      exige(c, 'lieu.ts', /console\.log\(afficherLieu\(\{ id: 2, titre: "Soirée", lieu: "Amphi Chappe", places: 120 \}\)\);/, 'Le second appel de test est intact');
      executer(c, 'lieu.ts', 'Lieu à confirmer\nAMPHI CHAPPE\n');
    },
    decrire(c) {
      const r = typer(c, ['decrire.ts']);
      exigerSansErreur(r);
      const sf = analyser(c, 'decrire.ts');
      if (sf && !contient(sf, (n) => n.kind === K.TypeOfExpression)) ko('utilise `typeof` pour distinguer les textes des nombres');
      exige(c, 'decrire.ts', /function decrire\(valeur: string \| number\): string \{/, 'La signature de `decrire` est intacte');
      exige(c, 'decrire.ts', /console\.log\(decrire\("gala"\)\);\s*console\.log\(decrire\(3\.5\)\);/, "Les appels de test sont intacts");
      executer(c, 'decrire.ts', 'GALA\n3.50\n');
    },
    inscription(c) {
      const r = typer(c, ['usage.ts'], { filtreFichier: () => true });
      exigerSansErreur(r);
      const sf = analyser(c, 'inscription.ts');
      if (!sf) { ko("le fichier inscription.ts n'existe pas"); return; }
      const itf = trouver(sf, (n) => n.kind === K.InterfaceDeclaration && n.name.text === 'Inscription')[0];
      if (!itf || !itf.modifiers?.some((m) => m.kind === K.ExportKeyword)) { ko('`inscription.ts` doit exporter une interface `Inscription`'); return; }
      const membre = (nom) => itf.members.find((m) => m.name && m.name.text === nom);
      const typeDe = (m) => (m && m.type ? m.type.getText(sf) : '');
      if (typeDe(membre('id')) !== 'number') ko('`id` doit être de type `number`');
      if (!/^StatutInscription$/.test(typeDe(membre('statut')))) ko('`statut` doit être de type `StatutInscription`');
      const com = membre('commentaire');
      if (!com || !com.questionToken || typeDe(com) !== 'string') ko('`commentaire` doit être facultatif (`commentaire?: string`)');
      const statut = trouver(sf, (n) => n.kind === K.TypeAliasDeclaration && n.name.text === 'StatutInscription')[0];
      const lits = statut ? statut.type.getText(sf).replace(/\s/g, '') : '';
      if (lits !== '"en_attente"|"confirmee"|"annulee"') ko('`StatutInscription` doit être l\'union fermée "en_attente" | "confirmee" | "annulee"');
      if (!problemes.length) ok('`Inscription` est correcte (champs et types)');
      executer(c, 'usage.ts', '0 2\n');
    },
  },

  // 03 : fonctions et génériques
  '03': {
    adherents(c) {
      const r = typer(c, ['adherents.ts']);
      exigerSansErreur(r);
      const sf = analyser(c, 'adherents.ts');
      const f = sf && trouver(sf, (n) => n.kind === K.FunctionDeclaration && n.name && n.name.text === 'ajouterAdherents')[0];
      if (!f) ko('la fonction `ajouterAdherents` doit exister');
      else {
        if (f.parameters.length !== 3 || f.parameters.some((p) => !p.type)) ko('chaque paramètre de `ajouterAdherents` doit avoir un type');
        if (!f.parameters[2] || !(f.parameters[2].questionToken || f.parameters[2].initializer)) ko('le troisième paramètre (`motif`) est facultatif');
        if (!f.type) ko('`ajouterAdherents` doit déclarer son type de retour');
      }
      exige(c, 'adherents.ts', /console\.log\(ajouterAdherents\(bde, 25\)\.adherents\);/, "L'appel de test est intact");
      executer(c, 'adherents.ts', 'sans motif\n825\n');
    },
    premier(c) {
      const r = typer(c, ['premier.ts']);
      exigerSansErreur(r);
      const sf = analyser(c, 'premier.ts');
      const f = sf && trouver(sf, (n) => n.kind === K.FunctionDeclaration && n.name && n.name.text === 'premier')[0];
      if (!f || !f.typeParameters || f.typeParameters.length < 1) ko('`premier` doit être générique (`premier<T>`)');
      exige(c, 'premier.ts', /const n = premier\(\[3, 4, 5\]\);\s*const s = premier\(\["a", "b"\]\);/, 'Les appels de test sont intacts');
    },
    modifier(c) {
      const r = typer(c, ['modifier.ts']);
      exigerSansErreur(r);
      const sf = analyser(c, 'modifier.ts');
      const f = sf && trouver(sf, (n) => n.kind === K.FunctionDeclaration && n.name && n.name.text === 'modifier')[0];
      const t = f && f.parameters[1] && f.parameters[1].type ? f.parameters[1].type.getText(sf).replace(/\s/g, '') : '';
      if (t !== 'Partial<Evenement>') ko('le deuxième paramètre de `modifier` doit être de type `Partial<Evenement>`');
      exige(c, 'modifier.ts', /console\.log\(modifier\(gala, \{ titre: "Gala 2027" \}\)\);/, "L'appel de test est intact");
      executer(c, 'modifier.ts', "{ id: 1, titre: 'Gala 2027', brouillon: true }\n");
    },
    filtre(c) {
      const r = typer(c, ['filtre.ts']);
      exigerSansErreur(r);
      const sf = analyser(c, 'filtre.ts');
      if (sf && !contient(sf, (n) => n.kind === K.TypePredicate)) ko('donne au filtre un prédicat de type (`e is Evenement`)');
      exige(c, 'filtre.ts', /console\.log\(evenements\[0\]\.titre\);/, "L'affichage est intact");
      executer(c, 'filtre.ts', 'Gala\n');
    },
    enveloppe(c) {
      const r = typer(c, ['usage-enveloppe.ts']);
      exigerSansErreur(r);
      const sf = analyser(c, 'enveloppe.ts');
      if (!sf) { ko("le fichier enveloppe.ts n'existe pas"); return; }
      const exporte = (n) => n.modifiers && n.modifiers.some((m) => m.kind === K.ExportKeyword);
      const itf = trouver(sf, (n) => n.kind === K.InterfaceDeclaration && n.name.text === 'Reponse')[0];
      if (!itf || !exporte(itf) || !itf.typeParameters || itf.typeParameters.length < 1) ko("`enveloppe.ts` doit exporter l'interface générique `Reponse<T>`");
      const f = trouver(sf, (n) => n.kind === K.FunctionDeclaration && n.name && n.name.text === 'envelopper')[0];
      if (!f || !exporte(f) || !f.typeParameters || f.typeParameters.length < 1) ko('`enveloppe.ts` doit exporter la fonction générique `envelopper<T>`');
      executer(c, 'usage-enveloppe.ts', '800 true\nundefined\n');
    },
  },

  // 04 : tsconfig
  '04': {
    strict(c) {
      const o = optionsDepuisApprenant(c);
      if (!o) return;
      if (o.brut.compilerOptions?.strict === true) ok('`strict` vaut `true`'); else ko('`strict` doit valoir `true` dans `compilerOptions`');
      for (const k of FAMILLE_STRICT) if (o.brut.compilerOptions?.[k] === false) ko(`l'option \`${k}\` ne doit pas être remise à \`false\``);
      for (const f of ['src/main.ts', 'src/lieu.ts', 'src/association.ts']) if (!o.fichiers.includes(f)) ko(`le tsconfig doit continuer à inclure ${f}`);
      if (o.brut.compilerOptions?.noCheck) ko('`noCheck` désactive la vérification');
    },
    lieu(c) {
      const o = optionsDepuis(c);
      const r = typer(c, ['src/lieu.ts'], { options: o });
      exigerSansErreur(r);
      controle(c, 'lieu.ts', 3, '`src/lieu.ts`');
    },
    association(c) {
      const o = optionsDepuis(c);
      const r = typer(c, ['src/association.ts'], { options: o });
      exigerSansErreur(r);
      controle(c, 'association.ts', 3, '`src/association.ts`');
    },
    alias(c) {
      const o = optionsDepuis(c);
      const r = typer(c, ['src/main.ts'], { options: o, filtreFichier: (rel) => rel === 'src/main.ts' });
      exigerSansErreur(r, "L'import `@/lieu` de `src/main.ts`");
      if (o) {
        const res = ts.resolveModuleName('@/lieu', path.join(c.tmp, 'src/main.ts'), o.options, ts.sys).resolvedModule;
        if (res && path.relative(c.tmp, res.resolvedFileName) === 'src/lieu.ts') ok('`@/` désigne bien le dossier `src/`');
        else ko('`@/lieu` doit désigner `src/lieu.ts`');
      }
    },
    script(c) {
      const o = optionsDepuis(c);
      const pkg = lirePackage(c);
      const s = pkg && pkg.scripts && pkg.scripts['ts-check'];
      if (typeof s === 'string' && /^tsc\s+--noEmit$/.test(s.trim())) ok('le script `ts-check` lance `tsc --noEmit`'); else ko('le script `ts-check` doit valoir `tsc --noEmit`');
      if (!o) return;
      for (const f of ['src/main.ts', 'src/lieu.ts', 'src/association.ts']) if (!o.fichiers.includes(f)) ko(`le tsconfig doit inclure ${f}`);
      const r = typer(c, o.fichiers.filter((f) => f.startsWith('src/')), { options: o });
      exigerSansErreur(r, 'Le projet');
    },
  },

  // 05 : typer une API
  '05': {
    brut(c) {
      const r = typer(c, ['brut.ts']);
      exigerSansErreur(r);
      const sf = analyser(c, 'brut.ts');
      if (sf && contient(sf, (n) => n.kind === K.AsExpression || n.kind === K.TypeAssertionExpression || n.kind === K.NonNullExpression)) ko("une affirmation de type (`as`) n'est pas une vérification : teste la valeur");
      exige(c, 'brut.ts', /const brut: unknown = JSON\.parse\('\{"a":1\}'\);/, '`brut` reste de type `unknown`');
      if (sf && !contient(sf, (n) => n.kind === K.IfStatement)) ko('vérifie la valeur avec un `if` avant de lire `a`');
      executer(c, 'brut.ts', '1\n');
    },
    garde(c) {
      const r = typer(c, ['garde.ts']);
      exigerSansErreur(r);
      controle(c, 'garde.ts', 14, 'le type guard `estEvenement`');
    },
    charger(c) {
      fs.copyFileSync(path.join(CTL, '05', 'garde-reference.ts'), path.join(c.tmp, 'garde.ts'));
      const r = typer(c, ['charger.ts']);
      exigerSansErreur(r);
      const sf = analyser(c, 'charger.ts');
      if (sf && contient(sf, (n) => n.kind === K.AsExpression)) ko("retire l'affirmation `as Evenement[]` : vérifie vraiment les données");
      if (sf && !contient(sf, (n) => n.kind === K.Identifier && n.text === 'estEvenement' && n.parent.kind !== K.ImportSpecifier)) ko('utilise `estEvenement` pour vérifier les données');
      if (sf && !contient(sf, (n) => n.kind === K.ThrowStatement)) ko('lève une erreur (`throw`) quand la réponse est mal formée');
      controle(c, 'charger.ts', 6, '`chargerEvenements`');
    },
    main(c) {
      fs.copyFileSync(path.join(CTL, '05', 'garde-reference.ts'), path.join(c.tmp, 'garde.ts'));
      fs.copyFileSync(path.join(CTL, '05', 'charger-reference.ts'), path.join(c.tmp, 'charger.ts'));
      const r = typer(c, ['main.ts']);
      exigerSansErreur(r);
      const sf = analyser(c, 'main.ts');
      const essai = sf && trouver(sf, (n) => n.kind === K.TryStatement)[0];
      if (!essai || !essai.catchClause) ko('entoure le chargement d\'un `try … catch`');
      else if (!contient(essai.tryBlock, (n) => n.kind === K.CallExpression && /chargerEvenements$/.test(n.expression.getText(sf)))) ko('le `try` doit contenir l\'appel de `chargerEvenements`');
      if (sf && contient(sf, (n) => n.kind === K.CallExpression && /process\.exit/.test(n.expression.getText(sf)))) ko("n'arrête pas le programme avec `process.exit`");
      executer(c, 'main.ts', '^Erreur[^\\n]*$', { regexp: true });
    },
  },

  // 06 : ESLint et Prettier
  '06': {
    config(c) {
      const config = path.join(c.tmp, 'eslint.config.mjs');
      if (!fs.existsSync(config)) { ko("`eslint.config.mjs` n'existe pas"); return; }
      const r = lancer(c, bin(c, 'eslint'), ['--format', 'json', 'src/asso.ts'], { delai: 14000 });
      let res;
      try { res = JSON.parse(r.sortie); } catch { ko("ESLint n'a pas pu lire ta configuration : " + norme(r.erreur).slice(0, 200)); return; }
      const regles = new Set(res.flatMap((f) => f.messages.map((m) => m.ruleId)));
      const fatals = res.flatMap((f) => f.messages.filter((m) => m.fatal));
      if (fatals.length) ko('erreur de configuration : ' + fatals[0].message);
      if (regles.has('@typescript-eslint/no-explicit-any')) ok('ESLint signale `no-explicit-any`'); else ko('ESLint doit signaler `no-explicit-any` : utilise les règles recommandées de `typescript-eslint`');
      if (regles.has('@typescript-eslint/no-unused-vars') || regles.has('no-unused-vars')) ok('ESLint signale les variables inutilisées'); else ko("ESLint doit signaler la variable inutilisée : ajoute aussi les règles recommandées d'ESLint");
    },
    asso(c) {
      installerConfigEslint(c);
      verifierAsso(c);
      const r = lancer(c, bin(c, 'eslint'), ['--max-warnings', '0', '--format', 'json', '.'], { delai: 14000 });
      verifierEslint(c, r);
    },
    prettierrc(c) {
      const f = path.join(c.tmp, '.prettierrc');
      if (!fs.existsSync(f)) { ko("`.prettierrc` n'existe pas"); return; }
      let j;
      try { j = JSON.parse(fs.readFileSync(f, 'utf8')); } catch { ko('`.prettierrc` doit contenir du JSON valide, par exemple `{ "printWidth": 100, "tabWidth": 4 }`'); return; }
      if (j.printWidth === 100) ok('`printWidth` vaut 100'); else ko('`printWidth` doit valoir 100 (un nombre)');
      if (j.tabWidth === 4) ok('`tabWidth` vaut 4'); else ko('`tabWidth` doit valoir 4 (un nombre)');
    },
    format(c) {
      installerPrettierrc(c);
      verifierPrettier(c);
      controle(c, 'asso.ts', 3, '`total`');
    },
    scripts(c) {
      installerConfigEslint(c);
      installerPrettierrc(c);
      const pkg = lirePackage(c);
      const s = (pkg && pkg.scripts) || {};
      if (/^eslint( \.)?( --max-warnings 0)?$/.test(String(s.lint || '').trim())) ok('le script `lint` lance ESLint'); else ko('le script `lint` doit lancer `eslint`');
      if (/^prettier --check src$/.test(String(s['prettier-check'] || '').trim())) ok('le script `prettier-check` lance Prettier'); else ko('le script `prettier-check` doit valoir `prettier --check src`');
      verifierAsso(c);
      verifierEslint(c, lancer(c, bin(c, 'eslint'), ['--max-warnings', '0', '--format', 'json', '.'], { delai: 14000 }));
      verifierPrettier(c);
    },
  },
};

function lirePackage(c) {
  try { return JSON.parse(fs.readFileSync(path.join(c.tmp, 'package.json'), 'utf8')); } catch { ko('`package.json` doit contenir du JSON valide'); return null; }
}
function optionsDepuisApprenant(c) {
  const cfg = path.join(c.tmp, 'tsconfig.json');
  const brut = ts.readConfigFile(cfg, ts.sys.readFile);
  if (brut.error) { ko('`tsconfig.json` est invalide : ' + ts.flattenDiagnosticMessageText(brut.error.messageText, ' ')); return null; }
  const parse = ts.parseJsonConfigFileContent(brut.config, ts.sys, c.tmp);
  const erreurs = parse.errors.filter((e) => e.code !== 18003);
  for (const e of erreurs) ko('`tsconfig.json` : ' + ts.flattenDiagnosticMessageText(e.messageText, ' '));
  return { brut: brut.config, fichiers: parse.fileNames.map((f) => path.relative(c.tmp, f)) };
}
function installerConfigEslint(c) {
  fs.copyFileSync(path.join(CTL, '06', 'eslint.config.mjs'), path.join(c.tmp, 'eslint.config.mjs'));
}
function installerPrettierrc(c) {
  fs.writeFileSync(path.join(c.tmp, '.prettierrc'), '{ "printWidth": 100, "tabWidth": 4 }\n');
}
function verifierAsso(c) {
  const sf = analyser(c, 'src/asso.ts');
  if (!sf) { ko("`src/asso.ts` n'existe pas"); return; }
  const d = nbDirectives(sf);
  if (d['eslint-disable'] || d['ts-ignore'] || d['ts-nocheck'] || d['ts-expect-error']) ko("les commentaires qui désactivent ESLint ou TypeScript ne corrigent rien : corrige le code");
  const r = typer(c, ['src/asso.ts'], { options: optionsDepuis(c) });
  exigerSansErreur(r);
  controle(c, 'asso.ts', 3, '`total`');
}
function verifierEslint(c, r) {
  let res;
  try { res = JSON.parse(r.sortie); } catch { ko('ESLint a échoué : ' + norme(r.erreur).slice(0, 200)); return; }
  const fichiers = res.filter((f) => /src\/asso\.ts$/.test(f.filePath));
  if (!fichiers.length) { ko('ESLint doit examiner `src/asso.ts`'); return; }
  const msgs = res.flatMap((f) => f.messages);
  if (msgs.length === 0) ok('ESLint ne signale plus rien');
  else { ko(`ESLint signale encore ${msgs.length} problème(s) :`); for (const m of msgs.slice(0, 3)) ko(`   ligne ${m.line} : ${m.message} (${m.ruleId})`); }
}
function verifierPrettier(c) {
  const r = lancer(c, bin(c, 'prettier'), ['--check', 'src'], { delai: 9000 });
  if (r.code === 0) ok('Prettier : le dossier `src` est bien formaté');
  else ko('`npx prettier --check src` signale un fichier mal formaté : ' + norme(r.sortie + r.erreur).slice(0, 160));
}

// ── Programme principal ──────────────────────────────────────────────────────────────────────────────────────
// Fichiers que l'apprenant·e doit modifier pour chaque contrôle (tout le reste est remis à l'identique).
const CIBLES = {
  '01': { propriete: ['evenements.ts'], appel: ['evenements.ts'], compteur: ['evenements.ts'], dist: ['evenements.ts'] },
  '02': { statut: ['statut.ts'], evenement: ['evenement.ts'], lieu: ['lieu.ts'], decrire: ['decrire.ts'], inscription: [] },
  '03': { adherents: ['adherents.ts'], premier: ['premier.ts'], modifier: ['modifier.ts'], filtre: ['filtre.ts'], enveloppe: [] },
  '04': {
    strict: ['tsconfig.json'], lieu: ['src/lieu.ts'], association: ['src/association.ts'],
    alias: ['tsconfig.json'], script: ['tsconfig.json', 'package.json', 'src/lieu.ts', 'src/association.ts'],
  },
  '05': { brut: ['brut.ts'], garde: ['garde.ts'], charger: ['charger.ts'], main: ['main.ts'] },
  '06': { config: ['eslint.config.mjs'], asso: ['src/asso.ts'], prettierrc: ['.prettierrc'], format: ['src/asso.ts'], scripts: ['package.json', 'src/asso.ts'] },
};

const [lecon, nom] = process.argv.slice(2);
if (!DOSSIERS[lecon] || !CONTROLES[lecon]?.[nom]) {
  console.error('Usage : verifier-ts <leçon> <contrôle> (leçons : ' + Object.keys(DOSSIERS).join(', ') + ')');
  process.exit(2);
}
const ctx = preparer(DOSSIERS[lecon], CIBLES[lecon][nom]);
try {
  CONTROLES[lecon][nom](ctx);
} catch (e) {
  ko("erreur de l'outil de vérification : " + (e && e.message ? e.message : e));
} finally {
  nettoyer(ctx);
}
fin();
