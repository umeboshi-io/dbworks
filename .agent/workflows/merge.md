---
description: How to commit, create a PR, and squash merge into main
---

# Commit & Squash Merge Workflow

## 1. Stage and commit changes on the feature branch

```bash
git add -A
git commit -m "<type>: <description>"
```

Commit message types: `feat`, `fix`, `refactor`, `test`, `docs`, `chore`

// turbo

## 2. Push the feature branch

```bash
git push origin <branch-name>
```

## 3. Create a GitHub Pull Request

```bash
gh pr create --base main --head <branch-name> \
  --title "<type>: <description>" \
  --body "<detailed description of changes>"
```

## 4. Squash merge via GitHub CLI

```bash
gh pr merge <pr-number> --squash \
  --subject "<type>: <description> (#<pr-number>)"
```

// turbo

## 5. Pull the squashed commit into local main

```bash
git checkout main
git pull origin main
```
