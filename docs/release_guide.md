# IVISS Release Guide

> **Who is this for?**
> This document is for anyone who needs to understand how new versions of IVISS are created, numbered, and published — including project managers, team leads, and technical stakeholders. No deep programming knowledge is required.

---

## 1. What is a "Release"?

A **release** is a numbered snapshot of the application at a specific point in time. Think of it like a software edition — version `1.2.0` is a specific, stable edition of IVISS that can be referenced, downloaded, or rolled back to at any time.

Each release:
- Gets a unique version number (e.g. `v1.2.0`)
- Has an automatically generated list of changes (a **changelog**)
- Is published on GitHub so the team can see exactly what changed and when

---

## 2. How Version Numbers Work

IVISS uses a widely adopted standard called **Semantic Versioning** (SemVer). Every version number has three parts:

```
v  MAJOR . MINOR . PATCH
      1  .   2   .   3
```

| Part | When it changes | What it means |
|---|---|---|
| **MAJOR** | A breaking change | Something fundamental changed — old behaviour no longer works the same way |
| **MINOR** | A new feature | New functionality was added, but nothing existing was broken |
| **PATCH** | A bug fix | Something that was broken was fixed |

**Examples:**
- `v1.0.0` → `v1.0.1` — a small bug was fixed
- `v1.0.1` → `v1.1.0` — a new feature was added
- `v1.1.0` → `v2.0.0` — a major change that affects how the system works

---

## 3. How Releases Are Created Automatically

IVISS uses a tool called **Semantic Release** that reads the commit messages written by developers and decides automatically:

- Whether a new release is needed
- What the new version number should be
- What to include in the changelog

This means **no human has to manually decide the version number** — it is determined entirely by the content of the code changes.

### How it reads commit messages

Developers follow a specific format when writing commit messages:

| Commit message starts with | Type of change | Version impact |
|---|---|---|
| `feat:` | New feature | MINOR bump (`1.0.0` → `1.1.0`) |
| `fix:` | Bug fix | PATCH bump (`1.0.0` → `1.0.1`) |
| `feat!:` or `BREAKING CHANGE:` | Breaking change | MAJOR bump (`1.0.0` → `2.0.0`) |
| `chore:`, `docs:`, `style:`, `refactor:` | Maintenance | **No release** |

### What happens when multiple commits are pushed at once?

Semantic Release reads **all commits** since the last release — not just the most recent one. It then picks the **highest impact** change to determine the version bump.

**Example:** If a push contains these 3 commits:
- `fix: correct login timeout` → would be a patch
- `feat: add export to PDF` → would be a minor
- `feat!: redesign authentication flow` → would be a major

The result is a **single MAJOR release** that includes all three changes in the changelog.

---

## 4. When Does a Release Happen?

Releases are **only created when code is merged into the `main` branch**.

```
feature branch  →  dev branch  →  main branch  →  RELEASE CREATED
```

- Pushing to `dev` → **no release** (this is the testing/staging branch)
- Merging `dev` into `main` → **release is created automatically**

This means the team can work freely on `dev`, test everything, and only trigger a release when they are confident the changes are ready for production.

---

## 5. What Gets Published in a Release

When a release is triggered, the following happens automatically:

1. **Version number is calculated** based on commit messages
2. **CHANGELOG.md is updated** with a human-readable list of all changes
3. **A GitHub Release is created** — visible on the GitHub repository page with release notes
4. **Docker images are tagged** with the new version number and pushed to the container registry

---

## 6. Resetting the Version Number

If the team decides to restart versioning from a specific number (for example, to start fresh at `v0.1.0` for a client handover), the following steps are needed:

> ⚠️ This should only be done by the technical lead. It affects the entire release history.

```bash
# Step 1 — Delete all existing version tags locally
git tag -l | xargs git tag -d

# Step 2 — Delete all existing version tags from GitHub
git tag -l | xargs -I {} git push origin --delete {}

# Step 3 — Create the new starting tag
git tag v0.1.0
git push origin v0.1.0
```

After this, the next release will start from `v0.1.0` and increment normally from there.

---

## 7. Viewing Past Releases

All releases are visible on GitHub:

1. Go to the IVISS repository on GitHub
2. Click **"Releases"** on the right side of the page
3. Each release shows the version number, date, and a full list of changes

---

## 8. Summary

| Question | Answer |
|---|---|
| Who creates releases? | Automatically — no manual action needed |
| When is a release created? | Only when code is merged into `main` |
| How is the version number decided? | By the type of commit messages in the push |
| Where can I see releases? | On the GitHub repository under "Releases" |
| Can we reset the version? | Yes — by deleting and recreating Git tags |
