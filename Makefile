# Publishing the website.
#
# https://hl7-rust.github.io is an *organization* GitHub Pages site, which
# GitHub only ever serves from a repository named `hl7-rust.github.io`. So the
# site cannot be published from this workspace directly, even though
# `hl7-rust.github.io/` here is where its source belongs.
#
# `make publish` closes that gap: it splits that directory out of this
# workspace's history and pushes the result to the `hl7-rust.github.io`
# repository, where that directory's own `.github/workflows/deploy.yml` —
# arriving at the repository root — builds it and deploys it to Pages.
#
# Building and testing the crates is cargo's job, not this file's:
#
#     cargo build       cargo test       cargo build -p hl7-2

WEBSITE_PREFIX := hl7-rust.github.io
WEBSITE_REMOTE := website
WEBSITE_URL    := git@github.com:hl7-rust/hl7-rust.github.io.git
WEBSITE_BRANCH := main

# Same destination, a second name: spec/monorepo-github-pages/index.md names
# this the family's shared convention for the sibling repos in this posture
# (er7-rust, fhir-rust, snomed-rust, openehr-rust), so `make github-pages`
# exists there too, pointed at each repo's own <name>.github.io. Kept as its
# own remote name rather than reusing $(WEBSITE_REMOTE) so a clone that only
# knows the family convention (`git remote -v` showing `github-pages`, not
# `website`) still finds the right command.
GITHUB_PAGES_REMOTE := github-pages

.DEFAULT_GOAL := help
.PHONY: help publish website-remote github-pages github-pages-remote

help:
	@echo 'make publish       Push $(WEBSITE_PREFIX)/ to $(WEBSITE_URL), forced'
	@echo 'make github-pages  The same push, via git subtree push, not forced'

# Add the remote if this clone does not have it yet, so a fresh clone can
# publish without a separate setup step.
website-remote:
	@git remote get-url $(WEBSITE_REMOTE) >/dev/null 2>&1 \
	  || git remote add $(WEBSITE_REMOTE) $(WEBSITE_URL)

github-pages-remote:
	@git remote get-url $(GITHUB_PAGES_REMOTE) >/dev/null 2>&1 \
	  || git remote add $(GITHUB_PAGES_REMOTE) $(WEBSITE_URL)

# The push is forced because the two histories are unrelated: the website
# repository grew on its own before this directory existed, and its history
# before that point is kept on its `archive/standalone` branch. Forcing is
# also what makes a rewrite of this workspace's history publishable at all.
# The cost is that a commit made directly in the website repository is
# overwritten rather than reported — so do not make one.
publish: website-remote
	git push $(WEBSITE_REMOTE) \
	  "$$(git subtree split --prefix=$(WEBSITE_PREFIX))":refs/heads/$(WEBSITE_BRANCH) \
	  --force

# `git subtree push` is `split` plus a plain (non-forced) push in one step,
# reusing its own cached split mapping so a second run only ships what
# changed since the last one. It only fast-forwards, which after the first,
# force-pushed `publish` is normally true here too — subtree split extends
# its own prior rewritten history, and nothing else ever commits to the far
# branch (see the `publish` comment above) — but unlike `publish`, it
# refuses outright rather than silently overwriting if that ever stops
# being true, so prefer this once a repository's history is established.
#
# The command itself lives in bin/make-github-pages (a POSIX shell script,
# not inlined here) — the same three-argument script is the version of this
# target the sibling repos in the family run too, each with its own prefix.
github-pages: github-pages-remote
	bin/make-github-pages $(WEBSITE_PREFIX) $(GITHUB_PAGES_REMOTE) $(WEBSITE_BRANCH)
