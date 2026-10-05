/* Logique pure (sans DOM) des quiz de leçon et de l'examen de validation : testable avec Jest. */
(function (root) {
    'use strict';

    /* Le quiz d'une leçon n'est débloqué qu'une fois toutes les étapes du labo validées (0 étape = pas de labo). */
    function labProgress(total, doneIndices) {
        const done = new Set(doneIndices || []);
        const count = Math.min(done.size, total);
        return { done: count, total, complete: total === 0 || count >= total };
    }

    /* « 03:05 » à partir d'un nombre de secondes. */
    function formatClock(seconds) {
        const s = Math.max(0, Math.floor(seconds));
        const mm = String(Math.floor(s / 60)).padStart(2, '0');
        const ss = String(s % 60).padStart(2, '0');
        return `${mm}:${ss}`;
    }

    /* « 3 min 05 s » (annonces vocales et libellés). */
    function formatRemaining(seconds) {
        const s = Math.max(0, Math.ceil(seconds));
        if (s < 60) return `${s} s`;
        const m = Math.floor(s / 60);
        const r = s % 60;
        return r ? `${m} min ${String(r).padStart(2, '0')} s` : `${m} min`;
    }

    /* Quiz de leçon : une question à la fois, dans l'ordre, sans retour en arrière. */
    class QuizFlow {
        constructor(questions) {
            this.questions = questions;
            this.reset();
        }

        reset() {
            this.index = 0;
            this.score = 0;
            this.answered = false; // la question courante a-t-elle reçu une réponse ?
            this.finished = false;
            this.lastCorrect = null;
        }

        get total() {
            return this.questions.length;
        }

        get current() {
            return this.finished ? null : this.questions[this.index];
        }

        get label() {
            return `Question ${Math.min(this.index + 1, this.total)} / ${this.total}`;
        }

        get canAdvance() {
            return this.answered && !this.finished;
        }

        get isLast() {
            return this.index === this.total - 1;
        }

        /* Répond à la question courante (une seule fois). Retourne { correct, correctIndex } ou null. */
        answer(optionIndex) {
            if (this.finished || this.answered) return null;
            const question = this.current;
            const option = question.options[optionIndex];
            if (!option) return null;
            this.answered = true;
            this.lastCorrect = !!option.correct;
            if (option.correct) this.score += 1;
            return { correct: this.lastCorrect, correctIndex: question.options.findIndex((o) => o.correct) };
        }

        /* Passe à la suite ; impossible tant que la question courante n'a pas de réponse. */
        next() {
            if (!this.canAdvance) return false;
            if (this.isLast) {
                this.finished = true;
            } else {
                this.index += 1;
                this.answered = false;
                this.lastCorrect = null;
            }
            return true;
        }
    }

    /* Examen : réponses modifiables avant soumission, navigation libre entre les questions. */
    class ExamFlow {
        constructor(questions, saved) {
            this.questions = questions;
            this.index = 0;
            this.chosen = {};
            const ids = new Set(questions.map((q) => q.id));
            Object.entries(saved || {}).forEach(([id, pos]) => {
                if (ids.has(id) && Number.isInteger(pos)) this.chosen[id] = pos;
            });
        }

        get total() {
            return this.questions.length;
        }

        get current() {
            return this.questions[this.index];
        }

        get label() {
            return `Question ${this.index + 1} / ${this.total}`;
        }

        select(questionId, position) {
            const q = this.questions.find((item) => item.id === questionId);
            if (!q || !Number.isInteger(position) || position < 0 || position >= q.options.length) return false;
            this.chosen[questionId] = position;
            return true;
        }

        goTo(i) {
            if (!Number.isInteger(i) || i < 0 || i >= this.total) return false;
            this.index = i;
            return true;
        }

        next() {
            return this.goTo(this.index + 1);
        }

        prev() {
            return this.goTo(this.index - 1);
        }

        isAnswered(i) {
            return this.chosen[this.questions[i].id] !== undefined;
        }

        unanswered() {
            return this.questions.map((q, i) => i).filter((i) => !this.isAnswered(i));
        }

        get answeredCount() {
            return this.total - this.unanswered().length;
        }

        /* Corps de la soumission : { id: position choisie } */
        answers() {
            return { ...this.chosen };
        }
    }

    const api = {
        labProgress, formatClock, formatRemaining, QuizFlow, ExamFlow
    };
    if (typeof module !== 'undefined' && module.exports) module.exports = api;
    else root.QuizLogic = api;
}(typeof window !== 'undefined' ? window : globalThis));
