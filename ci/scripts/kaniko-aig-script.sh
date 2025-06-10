#!/usr/bin/env sh

# Exits as soon as any line fails.
set -xeuo pipefail

# ghcraddr="ghcr.io/risingwavelabs/risingwave"
# dockerhubaddr="risingwavelabs/risingwave"
aighubaddr="$ATOME_HARBOR_HOST/atome-data-mid-end/risingwave_script"
aighubcacheaddr="$ATOME_HARBOR_HOST/atome-data-mid-end/risingwave-build-cache"
arch="$(uname -m)"
CARGO_PROFILE=${CARGO_PROFILE:-production}
GIT_COMMIT_CODE=$CI_COMMIT_SHORT_SHA

auth=`echo -n $ATOME_HARBOR_USER:$ATOME_HARBOR_PASSWORD | base64`
mkdir -p /kaniko/.docker
cat <<EOF > /kaniko/.docker/config.json
{
  "auths": {
    "https://${ATOME_HARBOR_HOST}": {
      "auth": "${auth}"
    }
  }
}
EOF

/kaniko/executor \
  --context "${CI_PROJECT_DIR}" \
  --build-arg "GIT_SHA=${GIT_COMMIT_CODE}" \
  --build-arg "CARGO_PROFILE=${CARGO_PROFILE}" \
  --dockerfile "${CI_PROJECT_DIR}/docker/Dockerfile_script" \
  --destination "${aighubaddr}:${GIT_COMMIT_CODE}-${arch}" \
  --cache=true \
  --cache-repo "${aighubcacheaddr}"
