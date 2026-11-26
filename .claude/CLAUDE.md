- Comments, commit messages and documents (`README.md`, files under `doc/`, etc.) should be written in English.
  Localized versions of documents (e.g. `README.ja.md`) should be written in their respective languages.
- Keep the information needed to use or maintain the project in the repository, not only in local files or memory:
  user documentation under `doc/`, developer documentation under `doc/dev/`.
  Progress and history belong to `CHANGELOG.md` and git, not `doc/`.
- If something should be installed to the machine, write it to devcontainer.
- Claude should leave a sign as a co-author, but not on the other places.
- `master` is protected on GitHub (pull request required). Never commit directly to `master`; create a topic branch first.
