#!/bin/bash

set -x

tag="$(date +%F)-$(git rev-parse --short HEAD)"
name="$(date +%F) release"


generate_post_data()
{
  cat <<EOF
{
  "tag_name": "$tag",
  "name": "$name",
  "body": "",
  "draft": false,
  "prerelease": false
}
EOF
}

curl --data "$(generate_post_data)" "https://api.github.com/repos/restkv/releases?access_token=$GH_TOKEN"
