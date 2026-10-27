#!/usr/bin/env bash
# Applies the repository settings OpenTBOI wants on GitHub, with the `gh` CLI (logged in as the owner):
#
#   scripts/github-setup.sh OWNER/REPOSITORY
#
# Everything here can be run again. Branch protection needs a public repository (or GitHub Pro); on a
# private repository of a free account GitHub answers 403, which is reported and skipped.
set -euo pipefail

if [[ $# -ne 1 || "$1" != */* ]]; then
  echo "usage: $0 OWNER/REPOSITORY" >&2
  exit 2
fi
repository="$1"

DESCRIPTION="An open-source engine for the original The Binding of Isaac that reads its art and sound from your own Steam copy"
TOPICS=(rust game-engine binding-of-isaac sdl3 bevy flash steam fan-project)

gh repo edit "${repository}" --description "${DESCRIPTION}" --enable-wiki=false --enable-projects=false \
  --delete-branch-on-merge --enable-merge-commit=false --enable-squash-merge --enable-rebase-merge
for topic in "${TOPICS[@]}"; do
  gh repo edit "${repository}" --add-topic "${topic}"
done

# A squash merge takes the pull request's title (which carries the semver tag) and description.
gh api -X PATCH "repos/${repository}" -f squash_merge_commit_title=PR_TITLE -f squash_merge_commit_message=PR_BODY > /dev/null

# Workflows get a read-only token unless a job asks for more (the release job asks for contents: write).
gh api -X PUT "repos/${repository}/actions/permissions/workflow" \
  -f default_workflow_permissions=read -F can_approve_pull_request_reviews=false >/dev/null

# Alerts and update pull requests for vulnerable dependencies.
gh api -X PUT "repos/${repository}/vulnerability-alerts" >/dev/null
gh api -X PUT "repos/${repository}/automated-security-fixes" >/dev/null

# Secret scanning with push protection is free on public repositories only.
gh api -X PATCH "repos/${repository}" \
  -f 'security_and_analysis[secret_scanning][status]=enabled' \
  -f 'security_and_analysis[secret_scanning_push_protection][status]=enabled' >/dev/null 2>&1 \
  || echo "secret scanning was not enabled (it needs a public repository)" >&2

# main: changes arrive only through pull requests (no direct pushes, administrators included), once the
# checks of CI pass, with a linear history, and no force pushes or deletions.
if ! gh api -X PUT "repos/${repository}/branches/main/protection" --input - >/dev/null <<'JSON'
{
  "required_status_checks": {
    "strict": true,
    "contexts": [
      "Source build, Fedora 44, system SDL3",
      "Source build, Ubuntu 24.04, SDL3 built from source",
      "Licence list stays generatable",
      "Shell scripts are lint-clean",
      "Pull request title has a semver tag"
    ]
  },
  "enforce_admins": true,
  "required_pull_request_reviews": {
    "required_approving_review_count": 0
  },
  "restrictions": null,
  "required_linear_history": true,
  "allow_force_pushes": false,
  "allow_deletions": false
}
JSON
then
  echo "main was not protected (needs a public repository or GitHub Pro)" >&2
fi
