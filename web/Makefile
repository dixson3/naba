PELICAN ?= pelican
PELICANOPTS =

BASEDIR = $(CURDIR)
INPUTDIR = $(BASEDIR)/content
OUTPUTDIR = $(BASEDIR)/output
VALIDATEDIR = $(BASEDIR)/output-validate
CONFFILE = $(BASEDIR)/pelicanconf.py
PUBLISHCONF = $(BASEDIR)/publishconf.py

S3_BUCKET = naba.ysapp.net

# CF_DISTRIBUTION (and other captured ids) are written by scripts/provision_aws.sh to
# aws-config.mk, which is gitignored (it holds account-specific resource ids). Absent
# before provisioning — the CloudFront invalidation steps no-op until it exists.
-include $(BASEDIR)/aws-config.mk

DEBUG ?= 0
ifeq ($(DEBUG), 1)
	PELICANOPTS += -D
endif

PORT ?= 0
ifneq ($(PORT), 0)
	PELICANOPTS += -p $(PORT)
endif

help:
	@echo 'Makefile for the naba website (Pelican + S3 + CloudFront)              '
	@echo '                                                                       '
	@echo 'Usage:                                                                 '
	@echo '   make html               (re)generate the site into output/         '
	@echo '   make clean              remove the generated files                  '
	@echo '   make devserver [PORT=8000]  serve + regenerate on change (local)    '
	@echo '   make validate           production build into a throwaway dir       '
	@echo '   make publish            generate using production settings          '
	@echo '   make sync_installer     stage + upload install.sh (short TTL)       '
	@echo '   make s3_upload          publish + sync site to S3 + invalidate      '
	@echo '   make invalidate         invalidate the mutable CloudFront paths     '
	@echo '   make provision          run the gated AWS provisioning script       '
	@echo '   make deploy             s3_upload + sync_installer (full deploy)     '

html:
	"$(PELICAN)" "$(INPUTDIR)" -o "$(OUTPUTDIR)" -s "$(CONFFILE)" $(PELICANOPTS)

clean:
	[ ! -d "$(OUTPUTDIR)" ] || rm -rf "$(OUTPUTDIR)"
	[ ! -d "$(VALIDATEDIR)" ] || rm -rf "$(VALIDATEDIR)"
	[ ! -d "__pycache__" ] || rm -rf "__pycache__"

devserver:
	"$(PELICAN)" -lr "$(INPUTDIR)" -o "$(OUTPUTDIR)" -s "$(CONFFILE)" $(PELICANOPTS)

# Production build. Depends on stage_installer so install.sh is always in output/ before
# the build — a phony target runs only ONCE per make invocation, so staging must precede
# (not merely be a sibling of) publish, otherwise s3_upload's `sync --delete` would drop
# install.sh and sync_installer would find no output/install.sh.
publish: stage_installer
	"$(PELICAN)" "$(INPUTDIR)" -o "$(OUTPUTDIR)" -s "$(PUBLISHCONF)" $(PELICANOPTS)

# Production build into an isolated dir so the dev output/ tree is never disturbed.
validate:
	[ ! -d "$(VALIDATEDIR)" ] || rm -rf "$(VALIDATEDIR)"
	"$(PELICAN)" "$(INPUTDIR)" -o "$(VALIDATEDIR)" -s "$(PUBLISHCONF)" $(PELICANOPTS)
	rm -rf "$(VALIDATEDIR)"
	@echo "Build validation passed."

# Stage the hosted install.sh (mirrors cargo-dist's naba-installer.sh from the pinned
# release tag; fail-safe placeholder when no release exists yet) into content/extra/,
# so it is part of the normal build/output and tree-wide sync.
stage_installer:
	./scripts/sync_installer.sh

# Re-upload ONLY install.sh with an explicit short Cache-Control (the tree-wide `s3 sync`
# does not set per-key cache headers) and invalidate that one path. `publish` already staged
# it into output/ (publish depends on stage_installer), so no separate stage step here.
sync_installer: publish
	aws s3 cp "$(OUTPUTDIR)/install.sh" "s3://$(S3_BUCKET)/install.sh" \
		--cache-control "max-age=300" --content-type "text/x-shellscript"
ifneq ($(CF_DISTRIBUTION),)
	aws cloudfront create-invalidation --distribution-id $(CF_DISTRIBUTION) --paths "/install.sh"
endif

s3_upload: publish
	aws s3 sync "$(OUTPUTDIR)"/ "s3://$(S3_BUCKET)" --delete
ifneq ($(CF_DISTRIBUTION),)
	aws cloudfront create-invalidation --distribution-id $(CF_DISTRIBUTION) --paths "/*"
endif

# Invalidate the mutable paths only (small site; the tree-wide s3_upload already does /*).
invalidate:
ifneq ($(CF_DISTRIBUTION),)
	aws cloudfront create-invalidation --distribution-id $(CF_DISTRIBUTION) --paths "/install.sh" "/index.html" "/*"
else
	@echo "CF_DISTRIBUTION not set (run make provision first)."
endif

# Full deploy: site tree, then the short-TTL install.sh override.
deploy: s3_upload sync_installer

provision:
	./scripts/provision_aws.sh

.PHONY: help html clean devserver publish validate stage_installer sync_installer \
	s3_upload invalidate deploy provision
