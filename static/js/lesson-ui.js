/* Page de leçon : amorce le labo, le quiz, les boutons de commande, les fichiers à créer, les schémas Mermaid
   et l'envoi de la progression (XP, niveaux, badges) à l'API Django. */
(function (root) {
    'use strict';

    const doc = root.document;
    // Mermaid est embarqué dans static/vendor/ (fonctionne hors ligne) ; le CDN n'est qu'un secours.
    const SCRIPT_SRC = doc.currentScript ? doc.currentScript.src : '';
    const MERMAID_LOCAL = SCRIPT_SRC ? new URL('../vendor/mermaid/mermaid.min.js', SCRIPT_SRC).href : '';

    const readJson = (id) => {
        const el = doc.getElementById(id);
        if (!el) return null;
        try {
            return JSON.parse(el.textContent);
        } catch (e) {
            console.error(`JSON invalide dans #${id}`, e);
            return null;
        }
    };

    const el = (tag, cls, text) => {
        const node = doc.createElement(tag);
        if (cls) node.className = cls;
        if (text !== undefined) node.textContent = text;
        return node;
    };

    /* ── Toasts ─────────────────────────────────────────────────────────── */
    function toastHost() {
        let host = doc.getElementById('toasts');
        if (!host) {
            host = el('div', 'toasts');
            host.id = 'toasts';
            host.setAttribute('aria-live', 'polite');
            doc.body.append(host);
        }
        return host;
    }

    function toast(kind, title, text, link) {
        const node = el('div', `toast toast--${kind}`);
        node.setAttribute('role', 'status');
        node.append(el('strong', 'toast__title', title));
        if (text) node.append(el('span', 'toast__text', text));
        if (link) {
            const a = el('a', 'toast__link', link.label);
            a.href = link.href;
            node.append(a);
        }
        const close = el('button', 'toast__close', '×');
        close.type = 'button';
        close.setAttribute('aria-label', 'Fermer la notification');
        close.addEventListener('click', () => node.remove());
        node.append(close);
        toastHost().append(node);
        root.setTimeout(() => node.remove(), link ? 12000 : 6000);
    }

    /* ── Progression ────────────────────────────────────────────────────── */
    function updateXp(level) {
        if (!level) return;
        const chip = doc.getElementById('xp-chip');
        if (chip) {
            chip.dataset.level = level.level;
            chip.dataset.xp = level.xp;
            const text = chip.querySelector('.xp-chip__text');
            const label = `Niv. ${level.level} · ${level.xp} XP`;
            if (text) text.textContent = label;
            else chip.textContent = label;
        }
        doc.querySelectorAll('[data-xp-bar]').forEach((bar) => { bar.style.width = `${level.percent}%`; });
    }

    /* Étapes de labo validées côté serveur : le quiz de la leçon s'en sert pour se déverrouiller. */
    const lessonState = { data: null, listeners: [] };

    function setTasksDone(indices) {
        if (!lessonState.data) return;
        const merged = new Set([...(lessonState.data.lesson.tasksDone || []), ...indices]);
        lessonState.data.lesson.tasksDone = [...merged].sort((x, y) => x - y);
        lessonState.listeners.forEach((fn) => fn());
    }

    class Progress {
        constructor(data) {
            this.data = data;
            this.queue = Promise.resolve();
        }

        send(payload) {
            const { data } = this;
            if (!data.apiUrl) return this.queue;
            this.queue = this.queue.then(() => root.fetch(data.apiUrl, {
                method: 'POST',
                credentials: 'same-origin',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ course: data.course.slug, lesson: data.lesson.slug, ...payload }),
            }).then((r) => (r.ok ? r.json() : r.json().catch(() => ({})).then((body) => Promise.reject(new Error(body.error || `HTTP ${r.status}`)))))
                .then((res) => this.announce(res))
                .catch((e) => {
                    console.error('Progression non enregistrée', e);
                    toast('error', 'Progression non enregistrée', e.message);
                }));
            return this.queue;
        }

        announce(res) {
            updateXp(res.level);
            if (res.progress && res.progress.tasks_done) setTasksDone(res.progress.tasks_done);
            if (res.xp_gained) toast('xp', `+${res.xp_gained} XP`, res.level ? `Niveau ${res.level.level} · ${res.level.title}` : '');
            if (res.level_up) toast('level', `Niveau ${res.level.level} atteint !`, res.level.title);
            (res.new_badges || []).forEach((b) => toast('badge', `${b.emoji} Badge débloqué : ${b.name}`, b.description));
            if (res.lesson_completed) {
                const { nextUrl } = this.data.lesson;
                toast('done', res.course_completed ? 'Parcours terminé, bravo !' : 'Leçon terminée !', '', nextUrl ? { href: nextUrl, label: 'Continuer →' } : null);
            }
        }
    }

    /* ── Quiz de leçon : verrouillé par le labo, une question à la fois ───── */
    const LOCK_ICON = '<svg viewBox="0 0 24 24" width="26" height="26" aria-hidden="true" focusable="false"><rect x="5" y="11" width="14" height="10" rx="2" fill="none" stroke="currentColor" stroke-width="2"/><path d="M8 11V8a4 4 0 0 1 8 0v3" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"/></svg>';

    class QuizUI {
        /**
         * @param {HTMLElement} host conteneur #quiz-root
         * @param {Array} questions [{question, options:[{html, correct}], explanation}]
         * @param {object} opts { lock: () => {done, total, complete}, onScore(score), best, onGoLab() }
         */
        constructor(host, questions, opts) {
            this.host = host;
            this.opts = opts;
            this.flow = new root.QuizLogic.QuizFlow(questions);
            this.best = opts.best || 0;
            this.unlocked = false;
            host.classList.add('quiz');
            this.refresh();
        }

        /* Réévalue le verrouillage (à chaque étape de labo validée). Une fois débloqué, le quiz le reste. */
        refresh() {
            if (this.unlocked) return;
            const lock = this.opts.lock();
            if (!lock.complete) {
                this.renderLocked(lock);
                return;
            }
            const wasLocked = this.shown === 'locked';
            this.unlocked = true;
            this.renderQuestion();
            if (wasLocked) toast('info', 'Quiz débloqué !', 'Tu as terminé le labo : tu peux répondre aux questions.');
        }

        renderLocked(lock) {
            this.shown = 'locked';
            const card = el('div', 'quiz__locked');
            card.setAttribute('role', 'status');
            const icon = el('span', 'quiz__lock');
            icon.innerHTML = LOCK_ICON;
            const text = el('div', 'quiz__locked-text');
            text.append(el('p', 'quiz__locked-title', 'Quiz verrouillé'));
            text.append(el('p', '', `Termine d'abord le labo (${lock.done}/${lock.total} étape${lock.total > 1 ? 's' : ''}) pour débloquer le quiz.`));
            const bar = el('div', 'quiz__bar');
            bar.setAttribute('role', 'progressbar');
            bar.setAttribute('aria-label', 'Étapes du labo validées');
            bar.setAttribute('aria-valuemin', '0');
            bar.setAttribute('aria-valuemax', String(lock.total));
            bar.setAttribute('aria-valuenow', String(lock.done));
            bar.append(el('span'));
            bar.firstChild.style.width = `${lock.total ? Math.round((lock.done * 100) / lock.total) : 0}%`;
            text.append(bar);
            if (this.opts.onGoLab) {
                const go = el('button', 'lab__btn', 'Aller au labo');
                go.type = 'button';
                go.addEventListener('click', () => this.opts.onGoLab());
                text.append(go);
            }
            card.append(icon, text);
            this.host.replaceChildren(card);
        }

        header() {
            const { flow } = this;
            const head = el('div', 'quiz__head');
            const count = el('span', 'quiz__count', flow.label);
            count.setAttribute('aria-live', 'polite');
            head.append(count);
            if (this.best) head.append(el('span', 'quiz__best', `Meilleur score : ${this.best}/${flow.total}`));
            const bar = el('div', 'quiz__bar');
            bar.setAttribute('role', 'progressbar');
            bar.setAttribute('aria-label', 'Progression du quiz');
            bar.setAttribute('aria-valuemin', '0');
            bar.setAttribute('aria-valuemax', String(flow.total));
            bar.setAttribute('aria-valuenow', String(flow.index));
            bar.append(el('span'));
            bar.firstChild.style.width = `${Math.round((flow.index * 100) / flow.total)}%`;
            return [head, bar];
        }

        renderQuestion() {
            this.shown = 'question';
            const { flow } = this;
            const q = flow.current;
            const card = el('div', 'quiz__card');
            const fieldset = el('fieldset', 'quiz__question');
            fieldset.tabIndex = -1;
            const legend = el('legend', 'quiz__legend');
            legend.innerHTML = `<span class="quiz__num">${flow.index + 1}</span> ${q.question}`;
            fieldset.append(legend);
            const feedback = el('div', 'quiz__feedback');
            feedback.hidden = true;
            feedback.setAttribute('role', 'status');
            const next = el('button', 'lab__btn lab__btn--primary quiz__next', flow.isLast ? 'Voir mon résultat' : 'Question suivante');
            next.type = 'button';
            next.disabled = true;

            const labels = q.options.map((opt, oi) => {
                const label = el('label', 'quiz__option');
                const input = doc.createElement('input');
                input.type = 'radio';
                input.name = `quiz-q${flow.index}`;
                input.value = oi;
                const text = el('span', 'quiz__text');
                text.innerHTML = opt.html;
                label.append(input, text);
                input.addEventListener('change', () => {
                    const result = flow.answer(oi);
                    if (!result) return;
                    fieldset.querySelectorAll('input').forEach((i) => { i.disabled = true; });
                    labels[result.correctIndex].classList.add('is-correct');
                    if (!result.correct) label.classList.add('is-wrong');
                    feedback.hidden = false;
                    feedback.className = `quiz__feedback ${result.correct ? 'is-ok' : 'is-ko'}`;
                    feedback.innerHTML = `<strong>${result.correct ? 'Bonne réponse !' : 'Pas tout à fait.'}</strong> ${q.explanation || ''}`;
                    next.disabled = false;
                    next.focus();
                });
                return label;
            });
            fieldset.append(...labels, feedback);

            next.addEventListener('click', () => {
                if (!flow.next()) return;
                if (flow.finished) this.renderSummary();
                else {
                    this.renderQuestion();
                    this.host.querySelector('.quiz__question').focus({ preventScroll: false });
                }
            });
            const actions = el('div', 'quiz__actions');
            actions.append(next);
            card.append(...this.header(), fieldset, actions);
            this.host.replaceChildren(card);
        }

        renderSummary() {
            this.shown = 'summary';
            const { flow } = this;
            const summary = el('div', 'quiz__summary');
            summary.setAttribute('role', 'status');
            summary.tabIndex = -1;
            const perfect = flow.score === flow.total;
            summary.append(el('strong', 'quiz__score', `Score : ${flow.score}/${flow.total}`));
            summary.append(el('span', '', perfect ? ' Sans faute, bravo !' : ' Relis les explications puis retente ta chance.'));
            if (this.best > flow.score) summary.append(el('span', 'quiz__best', `Ton meilleur score reste ${this.best}/${flow.total}.`));
            const retry = el('button', 'lab__btn', 'Refaire le quiz');
            retry.type = 'button';
            retry.addEventListener('click', () => {
                flow.reset();
                this.renderQuestion();
                this.host.querySelector('.quiz__question').focus();
            });
            summary.append(retry);
            this.host.replaceChildren(summary);
            summary.focus();
            this.best = Math.max(this.best, flow.score);
            this.opts.onScore(flow.score);
        }
    }

    /* ── Mermaid (chargé seulement si la page contient un schéma) ───────── */
    function loadScript(src) {
        return new Promise((resolve, reject) => {
            const tag = doc.createElement('script');
            tag.src = src;
            tag.onload = resolve;
            tag.onerror = () => reject(new Error(`chargement impossible : ${src}`));
            doc.head.append(tag);
        });
    }

    function mountMermaid() {
        const nodes = [...doc.querySelectorAll('pre.mermaid')];
        if (!nodes.length) return;
        const ready = root.mermaid ? Promise.resolve()
            : loadScript(MERMAID_LOCAL); /* no third-party fallback: a diagram stays as text if the script is not installed */
        ready.then(() => {
            root.mermaid.initialize({
                startOnLoad: false,
                theme: 'dark',
                securityLevel: 'strict',
                themeVariables: {
                    primaryColor: '#1e293b', primaryTextColor: '#e2e8f0', lineColor: '#94a3b8', primaryBorderColor: (getComputedStyle(document.documentElement).getPropertyValue('--primary') || '').trim() || '#6366F1'
                },
                gitGraph: { useMaxWidth: true, rotateCommitLabel: false },
            });
            return root.mermaid.run({ nodes });
        }).catch((e) => {
            console.warn('Schémas Mermaid indisponibles : le code est affiché à la place.', e);
            nodes.forEach((n) => n.classList.add('mermaid--failed'));
        });
    }

    /* Évite de couper une option courte (« --name web ») sur deux lignes dans les tableaux. */
    function keepShortCodeTogether() {
        doc.querySelectorAll('#lesson-body td code').forEach((code) => {
            if (code.textContent.length <= 24) code.classList.add('is-short');
        });
    }

    /* ── Sections vides (« Entraîne-toi » : le labo est dans le panneau voisin) ─ */
    function hintEmptySections(hasLab) {
        const body = doc.getElementById('lesson-body');
        if (!body || !hasLab) return;
        body.querySelectorAll('h2').forEach((h2) => {
            const next = h2.nextElementSibling;
            if (!next || next.tagName !== 'H2') return;
            const hint = doc.createElement('p');
            hint.className = 'labo-hint';
            hint.textContent = 'Les étapes du labo sont dans le panneau « Labo » : à droite de la leçon sur grand écran, juste en dessous sur mobile. Chaque étape validée te rapporte de l\'XP.';
            h2.after(hint);
        });
    }

    /* ── Amorçage ───────────────────────────────────────────────────────── */
    function init() {
        const data = readJson('lesson-data');
        const labHost = doc.getElementById('lab-root');
        const quizHost = doc.getElementById('quiz-root');
        let lab = null;
        const progress = data ? new Progress(data) : null;
        lessonState.data = data;
        let quizUi = null;
        const lockState = () => {
            const done = new Set((data.lesson.tasksDone) || []);
            if (lab && lab.session && lab.session.done) lab.session.done.forEach((i) => done.add(i));
            const required = data.lesson.labRequired !== false;
            return root.QuizLogic.labProgress(required ? data.lesson.tasks : 0, [...done]);
        };

        if (data && data.lab && labHost && root.LabCore && root.LabUI) {
            const session = new root.LabCore.LabSession(data.lab.engine || data.course.engine, data.lab);
            lab = new root.LabUI(labHost, session, {
                title: 'Labo',
                onTasksDone: (indices) => {
                    indices.forEach((i) => progress.send({ type: 'task', task: i }));
                    if (quizUi) quizUi.refresh();
                },
            });
        } else if (labHost) {
            labHost.hidden = true;
        }

        if (data && quizHost && data.quiz && data.quiz.length) {
            quizUi = new QuizUI(quizHost, data.quiz, {
                lock: lockState,
                best: data.lesson.quizBest,
                onScore: (score) => progress.send({ type: 'quiz', score }),
                onGoLab: () => {
                    const target = labHost || doc.getElementById('env-root') || doc.querySelector('.lesson-lab');
                    if (!target) return;
                    target.scrollIntoView({ behavior: 'smooth', block: 'start' });
                    if (lab && lab.input) lab.input.focus({ preventScroll: true });
                },
            });
            lessonState.listeners.push(() => quizUi.refresh());
        }

        doc.addEventListener('click', (event) => {
            const run = event.target.closest('.cmd__run[data-cmd]');
            if (run) {
                if (!lab) { toast('info', 'Pas de labo ici', 'Cette leçon n\'a pas de terminal : tu peux copier la commande.'); return; }
                const rect = lab.root.getBoundingClientRect();
                if (rect.top < 0 || rect.top > root.innerHeight * 0.7) lab.root.scrollIntoView({ behavior: 'smooth', block: 'start' });
                lab.exec(run.dataset.cmd);
                lab.input.focus({ preventScroll: true });
                return;
            }
            const create = event.target.closest('.filebox__create');
            if (create) {
                const box = create.closest('.filebox');
                if (!box || !lab) { toast('info', 'Pas de labo ici', 'Cette leçon n\'a pas de dossier de travail.'); return; }
                lab.writeFile(box.dataset.file, box.dataset.content || '');
                toast('info', `Fichier ${box.dataset.file} créé`, 'Il est dans le dossier de travail du labo.');
            }
        });

        root.lessonLab = lab;
        hintEmptySections(!!lab);
        keepShortCodeTogether();
        mountMermaid();
    }

    if (doc && doc.readyState === 'loading') doc.addEventListener('DOMContentLoaded', init);
    else if (doc) init();

    root.LessonUI = { toast, setTasksDone };
}(typeof window !== 'undefined' ? window : globalThis));
