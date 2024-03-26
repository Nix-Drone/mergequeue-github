# trunk-merge-demos

A repository for simulating different scenarios with Trunk's
[Merge product](https://www.mergegraph.io/). These demos will create and attempt to merge PRs in a
realistic manner to simulate different possible scenarios Trunk Merge might encounter in your
workspace. That way, you can see how Merge reacts for yourself.

## Prerequisites

### To Learn

To get the most out of these demos, make sure you are familiar with the following:

1. [Trunk Merge](https://docs.trunk.io/merge) and
   [the problem it solves](https://trunk.io/blog?post=trunk-merge)
2. [What Impacted Targets Are](https://docs.trunk.io/merge-graph/impacted-targets)

### To Do

1. Ensure the machine you will run the demo on has `Python3` installed.
2. **Fork this repo** into an organization that you have permission to install GitHub apps in (or
   already has the Trunk GitHub app installed with access to all repos)
3. To ensure that PRs can only be merged by Trunk, we will make some branch protection changes -
   these changes are the same as what you'd do when adding Trunk Merge to an actual repo. Add these
   rules to `main`:
   1. "Require a pull request before mergeing". Do not require any approvals or enable other
      sub-settings.
   2. "Require status checks to pass before merging". From there, require specifically the "Compute
      Impacted Targets" status check (this won't show up until the first PR is created).
   3. If you are restricting who can push to `main`, then explicitly give `trunk-io` permission to
      push to the branch, or else it will not be able to merge PRs.
4. If you have not done so already, create a Trunk Account [here](https://app.trunk.io/signup) -
   [instructions](https://docs.trunk.io/web-app)
5. Ensure the Trunk GitHub app is installed in the repo and that you can see it in the Trunk UI
6. Select the `trunk-merge-demos` repo in the Trunk UI, then
   [create a merge graph](https://docs.trunk.io/merge/getting-started). You only need to follow the
   steps up to and including the "Set Up Trunk Merge" screen (using a concurrency of 30 (or less if
   you want to see what happens)) and the branch `main` - all other steps specific to repo setup
   have already been done.
7. TO:DO - add steps about configuring the repo API token.

## Demo Repo Setup

This repo is set up like a typical Bazel project in order to give the most realistic demo possible.
This repo also uses Trunk's [GitHub Action](https://github.com/trunk-io/merge-action) for getting
the list of impacted targets for a PR. Additionally, the
[.trunk/trunk.yaml file](https://github.com/trunk-io/trunk-merge-demos/blob/main/.trunk/trunk.yaml#L24)
has already been configured to support Trunk Merge.

## How to Set Up

Running these demos requires set up both on your local side and on GitHub's side (since we need to
raise PRs).

### Preparing a GitHub Access Token

PRs are raised using a provided GitHub access token. Whoever the token belongs to should have
permission to access the forked `trunk-merge-demos` repo.

To get a GitHub access token, follow the steps
[here](https://docs.github.com/en/authentication/keeping-your-account-and-data-secure/managing-your-personal-access-tokens).
The token must have `repo:status`, `public_repo`, and the `read:org` permissions. Once it is
created, save it.

### Preparing the Demo Repo Locally

To prepare the repo, do the following:

1. Fork this repo to your own organization, then clone it
2. Run `pip install -r requirements.txt`
3. Put your token as the value of the `GH_ACCESS_TOKEN` constant in `src/constants.py`. Do NOT ever
   push this token to the demo repo.

## Current Demos And How To Run Them

### What Happens When A Demo Is Run

When a demo is run, the following will happen:

1. All PRs necessary for the demo will be raised using the provided GitHub access token
2. The demo will wait until the "Compute Impacted Targets" action has run on all PRs
3. The demo will begin queueing PRs by commenting `/trunk merge` on them using the provided GitHub
   access token

Then from there, watch the magic on the "Graph" tab for the Merge Graph in the Trunk UI!

| Demo         | What It Does                                                                        | How To Run                 |
| ------------ | ----------------------------------------------------------------------------------- | -------------------------- |
| `fan_prs.py` | Will queue multiple PRs in a fan formation (meaning multiple branches in the graph) | `python3 ./src/fan_prs.py` |

## Cleaning the Test Repo

TODO: Add script to clean up the test repo

## Contributing

TODO: Add notes about contributions and guidelines.
