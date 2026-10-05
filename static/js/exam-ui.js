/* Examen de validation d'un parcours : intro → questions (modifiables) → récapitulatif → résultats détaillés.
   Le serveur tire et mélange les questions, tient le chronomètre et note : ce script n'envoie que des positions. */
(function (root) {
    'use strict';

    const doc = root.document;
    const logic = root.QuizLogic;

    const readJson = (id) => {
        const node = doc.getElementById(id);
        if (!node) return null;
        try {
            return JSON.parse(node.textContent);
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

    const button = (label, cls, onClick) => {
        const b = el('button', `lab__btn ${cls || ''}`.trim(), label);
        b.type = 'button';
        if (onClick) b.addEventListener('click', onClick);
        return b;
    };

    /* ── Notifications ──────────────────────────────────────────────────── */
    function toast(kind, title, text) {
        let host = doc.getElementById('toasts');
        if (!host) {
            host = el('div', 'toasts');
            host.id = 'toasts';
            host.setAttribute('aria-live', 'polite');
            doc.body.append(host);
        }
        const node = el('div', `toast toast--${kind}`);
        node.setAttribute('role', 'status');
        node.append(el('strong', 'toast__title', title));
        if (text) node.append(el('span', 'toast__text', text));
        const close = el('button', 'toast__close', '×');
        close.type = 'button';
        close.setAttribute('aria-label', 'Fermer la notification');
        close.addEventListener('click', () => node.remove());
        node.append(close);
        host.append(node);
        root.setTimeout(() => node.remove(), 7000);
    }

    function updateXp(level) {
        if (!level) return;
        const chip = doc.getElementById('xp-chip');
        if (chip) {
            const text = chip.querySelector('.xp-chip__text');
            const label = `Niv. ${level.level} · ${level.xp} XP`;
            if (text) text.textContent = label;
        }
        doc.querySelectorAll('[data-xp-bar]').forEach((bar) => { bar.style.width = `${level.percent}%`; });
    }

    /* ── Appels serveur ─────────────────────────────────────────────────── */
    function post(url, body) {
        return root.fetch(url, {
            method: 'POST',
            credentials: 'same-origin',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(body || {}),
        }).then((r) => r.json().catch(() => ({})).then((json) => ({ status: r.status, ok: r.ok, json })));
    }

    const store = {
        get(key) {
            try {
                return JSON.parse(root.sessionStorage.getItem(key) || 'null');
            } catch (e) {
                return null;
            }
        },
        set(key, value) {
            try {
                if (value === null) root.sessionStorage.removeItem(key);
                else root.sessionStorage.setItem(key, JSON.stringify(value));
            } catch (e) { /* stockage indisponible : les réponses restent en mémoire */ }
        },
    };

    /* ── Examen ─────────────────────────────────────────────────────────── */
    class ExamUI {
        constructor(host, data) {
            this.host = host;
            this.data = data;
            this.flow = null;
            this.attempt = null;
            this.submitting = false;
            this.timerId = null;
            this.retryId = null;
            this.announced = new Set();
            this.live = el('div', 'sr-only');
            this.live.setAttribute('aria-live', 'polite');
            host.after(this.live);
            this.showIntro();
        }

        get storageKey() {
            return `exam:${this.data.course.slug}:${this.attempt}`;
        }

        clear() {
            if (this.retryId) root.clearInterval(this.retryId);
            this.retryId = null;
            this.host.replaceChildren();
        }

        say(text) {
            this.live.textContent = text;
        }

        /* Compte à rebours avant un nouvel essai : active le bouton quand le délai est écoulé. */
        countdown(seconds, label, btn) {
            let left = seconds;
            const render = () => {
                if (left > 0) {
                    label.textContent = `Nouvel essai possible dans ${logic.formatClock(left)}.`;
                    btn.disabled = true;
                } else {
                    label.textContent = 'Tu peux retenter l\'examen.';
                    btn.disabled = false;
                    if (this.retryId) root.clearInterval(this.retryId);
                }
            };
            render();
            if (left > 0) {
                this.retryId = root.setInterval(() => { left -= 1; render(); }, 1000);
            }
        }

        /* ── Introduction ──────────────────────────────────────────────── */
        showIntro(message) {
            this.clear();
            const { state } = this.data;
            const card = el('div', 'exam__card');
            card.append(el('h2', 'exam__title', state.open ? 'Une tentative est en cours' : 'Prêt·e ?'));
            card.append(el('p', '', state.open
                ? 'Tu peux la reprendre : le chronomètre a continué de tourner côté serveur.'
                : 'Le chronomètre démarre dès que tu cliques sur « Commencer ». Tu pourras revoir et modifier tes réponses avant de soumettre.'));
            if (message) {
                const alert = el('p', 'callout callout--warning', message);
                alert.setAttribute('role', 'alert');
                card.append(alert);
            }
            const start = button(state.open ? 'Reprendre l\'examen' : 'Commencer l\'examen', 'lab__btn--primary exam__start', () => this.start(start));
            card.append(start);
            if (state.retryAfter > 0 && !state.open) {
                const label = el('p', 'exam__retry');
                card.append(label);
                this.countdown(state.retryAfter, label, start);
            }
            this.host.append(card);
        }

        start(btn) {
            btn.disabled = true;
            post(this.data.startUrl).then(({ status, ok, json }) => {
                if (ok) {
                    this.begin(json);
                    return;
                }
                if (status === 429) {
                    this.data.state.retryAfter = json.retry_after || 60;
                    this.data.state.open = false;
                    this.showIntro('Tu pourras retenter l\'examen dans quelques instants.');
                    return;
                }
                if (status === 409) {
                    root.location.reload();
                    return;
                }
                btn.disabled = false;
                this.showIntro(json.error || 'Impossible de démarrer l\'examen.');
            }).catch(() => {
                btn.disabled = false;
                this.showIntro('Le serveur ne répond pas. Réessaie dans un instant.');
            });
        }

        begin(payload) {
            this.attempt = payload.attempt;
            this.endsAt = Date.now() + payload.seconds_left * 1000;
            this.flow = new logic.ExamFlow(payload.questions, store.get(this.storageKey));
            this.payload = payload;
            this.startTimer();
            this.showQuestion();
        }

        /* ── Chronomètre ───────────────────────────────────────────────── */
        remaining() {
            return Math.max(0, Math.round((this.endsAt - Date.now()) / 1000));
        }

        startTimer() {
            if (this.timerId) root.clearInterval(this.timerId);
            this.timerId = root.setInterval(() => this.tick(), 1000);
        }

        stopTimer() {
            if (this.timerId) root.clearInterval(this.timerId);
            this.timerId = null;
        }

        tick() {
            const left = this.remaining();
            const node = this.host.querySelector('.exam__timer-value');
            if (node) {
                node.textContent = logic.formatClock(left);
                const timer = node.closest('.exam__timer');
                timer.classList.toggle('is-low', left <= 300 && left > 60);
                timer.classList.toggle('is-critical', left <= 60);
            }
            [300, 60, 30].forEach((threshold) => {
                if (left <= threshold && left > 0 && !this.announced.has(threshold)) {
                    this.announced.add(threshold);
                    this.say(`Il te reste ${logic.formatRemaining(left)}.`);
                }
            });
            if (left <= 0 && !this.submitting) {
                this.say('Temps écoulé : envoi de tes réponses.');
                this.submit();
            }
        }

        timerBadge() {
            const timer = el('div', 'exam__timer');
            timer.setAttribute('role', 'timer');
            timer.setAttribute('aria-label', 'Temps restant');
            timer.append(el('span', 'exam__timer-label', 'Temps restant'));
            timer.append(el('strong', 'exam__timer-value', logic.formatClock(this.remaining())));
            return timer;
        }

        /* ── Questions ─────────────────────────────────────────────────── */
        header() {
            const { flow } = this;
            const head = el('div', 'exam__head-bar');
            const info = el('div', 'exam__counter');
            const count = el('span', 'exam__count', flow.label);
            count.setAttribute('aria-live', 'polite');
            info.append(count, el('span', 'exam__answered', `${flow.answeredCount} / ${flow.total} répondue${flow.answeredCount > 1 ? 's' : ''}`));
            head.append(info, this.timerBadge());
            return head;
        }

        dots() {
            const { flow } = this;
            const nav = el('ol', 'exam__dots');
            nav.setAttribute('aria-label', 'Aller à une question');
            flow.questions.forEach((q, i) => {
                const item = el('li');
                const dot = button(String(i + 1), `exam__dot${flow.isAnswered(i) ? ' is-answered' : ''}${i === flow.index ? ' is-current' : ''}`, () => {
                    flow.goTo(i);
                    this.showQuestion();
                });
                dot.setAttribute('aria-label', `Question ${i + 1}${flow.isAnswered(i) ? ', répondue' : ', sans réponse'}`);
                if (i === flow.index) dot.setAttribute('aria-current', 'step');
                item.append(dot);
                nav.append(item);
            });
            return nav;
        }

        showQuestion() {
            this.clear();
            const { flow } = this;
            const q = flow.current;
            const card = el('div', 'exam__card');
            card.append(this.header(), this.dots());
            const fieldset = el('fieldset', 'quiz__question exam__question');
            fieldset.tabIndex = -1;
            const legend = el('legend', 'quiz__legend');
            legend.innerHTML = `<span class="quiz__num">${flow.index + 1}</span> ${q.question}`;
            fieldset.append(legend);
            q.options.forEach((opt, pos) => {
                const label = el('label', 'quiz__option');
                const input = doc.createElement('input');
                input.type = 'radio';
                input.name = `exam-q${flow.index}`;
                input.value = pos;
                input.checked = flow.chosen[q.id] === pos;
                const text = el('span', 'quiz__text');
                text.innerHTML = opt.html;
                label.append(input, text);
                input.addEventListener('change', () => {
                    flow.select(q.id, pos);
                    store.set(this.storageKey, flow.answers());
                    this.refreshProgress();
                });
                fieldset.append(label);
            });
            card.append(fieldset);
            const actions = el('div', 'exam__actions');
            const prev = button('← Précédente', '', () => { flow.prev(); this.showQuestion(); });
            prev.disabled = flow.index === 0;
            const last = flow.index === flow.total - 1;
            const next = button(last ? 'Récapitulatif →' : 'Suivante →', 'lab__btn--primary', () => {
                if (last) this.showRecap();
                else { flow.next(); this.showQuestion(); }
            });
            actions.append(prev, next);
            card.append(actions);
            this.host.append(card);
            fieldset.focus({ preventScroll: true });
        }

        /* Met à jour le compteur et les pastilles sans redessiner la question (le focus reste sur la réponse). */
        refreshProgress() {
            const { flow } = this;
            const answered = this.host.querySelector('.exam__answered');
            if (answered) answered.textContent = `${flow.answeredCount} / ${flow.total} répondue${flow.answeredCount > 1 ? 's' : ''}`;
            this.host.querySelectorAll('.exam__dot').forEach((dot, i) => {
                dot.classList.toggle('is-answered', flow.isAnswered(i));
                dot.setAttribute('aria-label', `Question ${i + 1}${flow.isAnswered(i) ? ', répondue' : ', sans réponse'}`);
            });
        }

        /* ── Récapitulatif ─────────────────────────────────────────────── */
        showRecap() {
            this.clear();
            const { flow } = this;
            const card = el('div', 'exam__card');
            const head = el('div', 'exam__head-bar');
            head.append(el('h2', 'exam__title', 'Récapitulatif'), this.timerBadge());
            card.append(head);
            const missing = flow.unanswered();
            const lead = missing.length
                ? `${missing.length} question${missing.length > 1 ? 's' : ''} sans réponse : elle${missing.length > 1 ? 's' : ''} compter${missing.length > 1 ? 'ont' : 'a'} comme fausse${missing.length > 1 ? 's' : ''}.`
                : 'Tu as répondu à toutes les questions. Relis-les si tu veux, puis soumets.';
            const notice = el('p', missing.length ? 'callout callout--warning' : 'exam__lead', lead);
            notice.setAttribute('role', 'status');
            card.append(notice);
            const list = el('ol', 'exam__recap');
            flow.questions.forEach((q, i) => {
                const item = el('li', `exam__recap-item${flow.isAnswered(i) ? ' is-answered' : ' is-missing'}`);
                const excerpt = doc.createElement('div');
                excerpt.innerHTML = q.question;
                const text = (excerpt.textContent || '').trim();
                item.append(el('span', 'exam__recap-text', `${i + 1}. ${text.length > 110 ? `${text.slice(0, 107)}…` : text}`));
                item.append(el('span', 'exam__recap-state', flow.isAnswered(i) ? 'Répondue' : 'Sans réponse'));
                item.append(button('Modifier', 'lab__btn--ghost', () => { flow.goTo(i); this.showQuestion(); }));
                list.append(item);
            });
            card.append(list);
            const actions = el('div', 'exam__actions');
            actions.append(button('← Revenir aux questions', '', () => { flow.goTo(flow.total - 1); this.showQuestion(); }));
            const submit = button('Soumettre mes réponses', 'lab__btn--primary', () => this.submit());
            actions.append(submit);
            card.append(actions);
            this.host.append(card);
            card.querySelector('.exam__title').tabIndex = -1;
            card.querySelector('.exam__title').focus({ preventScroll: true });
        }

        /* ── Soumission et résultats ───────────────────────────────────── */
        submit() {
            if (this.submitting) return;
            this.submitting = true;
            this.stopTimer();
            this.host.querySelectorAll('button').forEach((b) => { b.disabled = true; });
            this.say('Envoi de tes réponses…');
            post(this.data.submitUrl, { attempt: this.attempt, answers: this.flow.answers() })
                .then(({ status, ok, json }) => {
                    if (ok || status === 410) {
                        store.set(this.storageKey, null);
                        this.showResult(json, status === 410);
                        return;
                    }
                    this.submitting = false;
                    if (status === 409 || status === 404) {
                        root.location.reload();
                        return;
                    }
                    this.startTimer();
                    this.showRecap();
                    toast('error', 'Envoi impossible', json.error || `Erreur ${status}`);
                })
                .catch(() => {
                    this.submitting = false;
                    this.startTimer();
                    this.showRecap();
                    toast('error', 'Envoi impossible', 'Le serveur ne répond pas : tes réponses sont conservées, réessaie.');
                });
        }

        announce(res) {
            updateXp(res.level);
            if (res.xp_gained) toast('xp', `+${res.xp_gained} XP`, 'Parcours validé par examen');
            if (res.level_up && res.level) toast('level', `Niveau ${res.level.level} atteint !`, res.level.title);
            (res.new_badges || []).forEach((b) => toast('badge', `${b.emoji} Badge débloqué : ${b.name}`, b.description));
            if (res.course_validated) toast('done', 'Parcours validé !', `${this.data.course.title} est validé : les parcours suivants sont débloqués.`);
        }

        showResult(res, expired) {
            this.clear();
            const passed = !!res.passed;
            const card = el('div', `exam__card exam__result ${passed ? 'is-passed' : 'is-failed'}`);
            const title = el('h2', 'exam__title', passed ? 'Examen réussi, bravo !' : (expired ? 'Temps écoulé' : 'Pas encore'));
            title.tabIndex = -1;
            card.append(title);
            if (expired) {
                card.append(el('p', 'exam__lead', 'Le temps imparti est écoulé : cette tentative est close sans note.'));
            } else {
                const score = el('p', 'exam__score');
                score.append(el('strong', '', `${res.score} / ${res.total}`), ` bonnes réponses · seuil de réussite : ${res.pass_mark} %`);
                card.append(score);
            }
            if (passed) {
                card.append(el('p', 'exam__lead', `Le parcours « ${this.data.course.title} » est validé${res.lessons_validated ? ` (${res.lessons_validated} leçon${res.lessons_validated > 1 ? 's' : ''} validée${res.lessons_validated > 1 ? 's' : ''} sans labo)` : ''}.`));
                const actions = el('div', 'exam__actions');
                const back = el('a', 'btn btn--primary', 'Retour au parcours');
                back.href = this.data.courseUrl;
                const badges = el('a', 'btn btn--ghost', 'Voir mes badges');
                badges.href = '/badges/';
                actions.append(back, badges);
                card.append(actions);
                this.announce(res);
            } else {
                const label = el('p', 'exam__retry');
                const retry = button('Retenter l\'examen', 'lab__btn--primary', () => root.location.reload());
                const review = el('a', 'btn btn--ghost', 'Revoir le parcours');
                review.href = this.data.courseUrl;
                const actions = el('div', 'exam__actions');
                actions.append(retry, review);
                card.append(label, actions);
                this.countdown(res.retry_after || 0, label, retry);
            }
            if (res.results) card.append(this.review(res.results));
            this.host.append(card);
            title.focus({ preventScroll: false });
            this.say(passed ? 'Examen réussi.' : 'Examen non validé.');
        }

        /* Correction détaillée : réponse choisie, bonne réponse, explication. */
        review(results) {
            const section = el('section', 'exam__review');
            section.append(el('h3', '', 'Correction détaillée'));
            const list = el('ol', 'exam__review-list');
            results.forEach((r, i) => {
                const item = el('li', `exam__review-item ${r.correct ? 'is-ok' : 'is-ko'}`);
                const head = el('div', 'exam__review-q');
                head.innerHTML = `<span class="quiz__num">${i + 1}</span> ${r.question}`;
                item.append(head);
                const options = el('ul', 'exam__review-options');
                r.options.forEach((html, pos) => {
                    const li = el('li');
                    if (pos === r.correct_position) li.classList.add('is-correct');
                    if (pos === r.chosen && pos !== r.correct_position) li.classList.add('is-wrong');
                    const mark = pos === r.correct_position ? '✓ ' : (pos === r.chosen ? '✗ ' : '');
                    li.innerHTML = `<span class="exam__mark" aria-hidden="true">${mark}</span>${html}`;
                    if (pos === r.chosen) li.append(el('span', 'sr-only', pos === r.correct_position ? ' (ta réponse, correcte)' : ' (ta réponse, incorrecte)'));
                    else if (pos === r.correct_position) li.append(el('span', 'sr-only', ' (bonne réponse)'));
                    options.append(li);
                });
                item.append(options);
                if (r.chosen === null) item.append(el('p', 'exam__missed', 'Pas de réponse.'));
                if (r.explanation) {
                    const why = el('p', 'exam__why');
                    why.innerHTML = `<strong>Explication :</strong> ${r.explanation}`;
                    item.append(why);
                }
                list.append(item);
            });
            section.append(list);
            return section;
        }
    }

    function init() {
        const data = readJson('exam-data');
        const host = doc.getElementById('exam-root');
        if (!data || !host || !logic) return;
        root.examUi = new ExamUI(host, data);
    }

    if (doc && doc.readyState === 'loading') doc.addEventListener('DOMContentLoaded', init);
    else if (doc) init();
}(typeof window !== 'undefined' ? window : globalThis));
