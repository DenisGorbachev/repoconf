#!/usr/bin/env -S usage bash
#USAGE arg "<template>" help="The name of the package & repo"
#USAGE arg "<visibility>" {
#USAGE   choices "public" "private"
#USAGE }
#USAGE arg "<name>" help="The name of the package & repo"

set -xeuo pipefail

template=${usage_template:?}
name=${usage_name:?}
repo_full_name_new="DenisGorbachev/$name"
visibility=${usage_visibility:?}
dir="$HOME/workspace/$name"

gh repo create --template "$template" "--$visibility" "$repo_full_name_new"
gh repo clone "$repo_full_name_new" "$dir"

git remote add template "$template"

if [[ -f "$dir/setup.sh" ]]; then
  . "$dir/setup.sh"
else
  echo "setup.sh not found in $dir"
fi
