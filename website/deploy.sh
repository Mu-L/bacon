# This script is dedicated to the official documentation site at https://dystroy.org/bacon

# build the documentation site
ddoc

# copy the site to the deployement stage
cp -r site/* $HOME/dev/www/dystroy/bacon/

# build the config schema
bacon --generate-config-schema > $HOME/dev/www/dystroy/bacon/.bacon.schema.json

# deploy on dystroy.org
$HOME/dev/www/dystroy/deploy.sh
