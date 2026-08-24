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

.DEFAULT_GOAL := help
.PHONY: help publish website-remote

help:
	@echo 'make publish   Push $(WEBSITE_PREFIX)/ to $(WEBSITE_URL)'

# Add the remote if this clone does not have it yet, so a fresh clone can
# publish without a separate setup step.
website-remote:
	@git remote get-url $(WEBSITE_REMOTE) >/dev/null 2>&1 \
	  || git remote add $(WEBSITE_REMOTE) $(WEBSITE_URL)

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
