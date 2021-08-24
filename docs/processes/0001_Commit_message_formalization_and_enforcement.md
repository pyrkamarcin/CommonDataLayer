# Commit message formalization and enforcement

## Commit message formalization 
In CDL repository all issues titles, pull request titles and commit messages must have prefixed with a tag.

Currently we allow tags:
* `chore` - dependency updates, ci updates, refactorings, other changes that don't affect `CDL` functionality
* `test` - adding or removing tests; it does not include changes to deployment or CI. These go under `chore.`
* `fix` - fixing a bug in **code**
* `feat` - adding new feature to **CDL**
* `docs` - adding documentation (readmes, mdbook, plantuml etc.)

On merge, we squash commits and use one of 5 tags. We are not using the scope as it's basically useless - most of our changes affect everything.
The summary should describe which components were changed.

As for the long commit message:

```
parent-tag: summary

* tag: summary
* tag: summary
```

Parent tag is the main goal of the work, and the list contains all things we did during implementation. E.g.:

```
feat: add lazers to CDL

* chore: add deployment target on the moon
* docs: document usage of lazers (and how to keep them away from cats)
```

## Commit message enforcement
Commit message format is enforced via one of GitHub Actions.
