#!/usr/bin/env bash
#
# This script is dedicated to the official documentation site at https://dystroy.org/bacon

set -Eeuo pipefail
cd "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

command -v ddoc >/dev/null || { echo "ddoc not found — see https://dystroy.org/ddoc" >&2; exit 1; }
command -v bacon >/dev/null || { echo "bacon not found — needed for the config schema" >&2; exit 1; }

# build the documentation site
ddoc

# build the config schema, so TOML editors pick it up from the site
bacon --generate-config-schema > site/.bacon.schema.json

# deploy directly on the server: going through ~/dev/www/dystroy would republish
# that machine's stale copy of every other project
chmod -R a+rX site
rsync -av site/ dys@dystroy.org:prod/www.dystroy.org/bacon/
