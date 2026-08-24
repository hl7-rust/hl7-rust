#!/bin/sh
git push website "$(git subtree split --prefix=hl7-rust.github.io)":refs/heads/main --force
