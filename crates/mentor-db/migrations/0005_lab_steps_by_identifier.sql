-- Lab steps are referred to by identifier, no longer by position.
--
-- `tasks_done` held zero-based positions of the validated steps. Reordering the steps of a lab, or adding one
-- in the middle, silently relabelled what learners had done. It now holds the identifiers of the steps (the
-- `id` an author gave, or a digest of the step's text), which the catalogue compiler produces.
--
-- Existing values are kept as text ("0", "1"…). They match no step, since an identifier starts with a
-- letter: a lesson that was completed stays completed, a lab that was half done has to be done again. Nothing
-- but the import of v1 data could have written such rows; `mentor import-v1` now translates positions itself.

ALTER TABLE lesson_progress ALTER COLUMN tasks_done DROP DEFAULT;
ALTER TABLE lesson_progress ALTER COLUMN tasks_done TYPE text[] USING tasks_done::text[];
ALTER TABLE lesson_progress ALTER COLUMN tasks_done SET DEFAULT '{}';
