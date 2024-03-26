# Livequiz

This is a very simple livequiz based on MD files with a live view of what the students answer.
It allows to send a link to students, who can answer questions.
The admin view shows all students, and what they answered already.

# TODO

- bugs:
  - l'interface admin peut "perdre" des noms après le redémarrage du système
    - probably due to clients only sending updates to questions
    - -> on startup, should send "sendAllStats" in "getStats"
  - when changing quiz, /admin still shows the attempts
    - backend stores answers globally, instead of per-quiz
- admin view:
  - route to /student if it's not an admin
- student view:
- correction view:
- general
  - admin can upload quizzes
  - admin can choose which quizz is shown
  - modes
    - exam mode: only once the admin switch is flicked will the students see if they answered correctly
    - live mode: after every question, students see if they answered correctly
      - updatable: students can update their questions
    - modes are a combination of switches:
      - students directly see if they answered correctly or not
      - stop students from changing their answers
- backend
  - accept directory for quizzes, and offer all .md files as quizzes
    - store answers per quiz
  - store user data on disk

# CHANGELOG

2024-03-26:
- Make toggles in admin view reflect what is set in the backend

2024-03-21:
- don't reset choices in multi
- correction: add number of answers for choices
- correction: fix several wrong calculations of choices

2024-03-20:
- add questions which are verified with regular expressions
  - use `~` instead of `=`, followed by the regular expression for search and replace
  - more than one `-` is allowed, and any matching regex solves the question
- fix small bugs in `/corrections`

2024-03-14:
- add statistics to each column of how many answered correctly
- like student view, but show the answers with the least success first

2024-03-12:
- admin: automatically update view every few seconds
- frontend: use rxjs to propagate state changes like quizState (showResult, frozen, ...) and studentAnswers (for admin)
- add a quiz-id, so that students only see their results
- show number of answers needed

2024-03-07:
- admin: 0 a 12, students: 1 a 13
- footer peut cacher les points
- footer with copyright and link to github
- student automatically updates with 'showView'

2024-03-06:
- added first real questionnaires

2024-03-04:
- add docker-compose.yaml with Dockerfile

2024-03-03:
- student_view: when results are shown, show the chosen and the correct result
- student_view: store the answers in the local storage for later retrieval
- backend: store quizzes on disk
- backend: get the quiz from the backend instead of having it in the frontend:
- admin_view: "show results" actually works: only if it's enabled should the correct answers be shown
- first version with frontend and backend

# Installation

Launch the docker-compose.yml