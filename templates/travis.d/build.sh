#!/usr/bin/env bash
# HACKGPROJECT VERSION: {{source_rev}}
set -euo pipefail
PROJECT_TYPE="{{project_type}}"
ORG_NAME="HackGT"
SOURCE_DIR=$(readlink -f "${BASH_SOURCE[0]}")
SOURCE_DIR=$(dirname "$SOURCE_DIR")
cd "${SOURCE_DIR}/.."
set -x

if ! hash docker &>/dev/null; then
    echo "Cannot find `docker`!" >&2
    exit 64
fi

docker=
if docker ps &>/dev/null; then
    docker=docker
else
    docker='sudo docker'
fi

remote=$(git remote -v | grep -Pio "${ORG_NAME}"'/[a-zA-Z0-9-_\.]*' | head -1)
image_name=$(basename "${remote%.*}")

build_project_source() {
    if [[ -f Dockerfile.build ]]; then
        local build_image_name="$(basename $(pwd))-build"
        $docker build -f Dockerfile.build --rm -t "$build_image_name" .
        $docker run -w '/src' -v "$(pwd):/src" "$build_image_name"
    fi
}

test_project_source() {
    if [[ -f Dockerfile.test ]]; then
        local test_image_name="$(basename $(pwd))-test"
        $docker build -f Dockerfile.test --rm -t "$test_image_name" .
        $docker run -w '/src' -v "$(pwd):/src" "$test_image_name"
    fi
}

build_project_container() {
    $docker build -f Dockerfile --rm -t "$image_name" .
}

publish_project_container() {
    local git_rev=$(git rev-parse HEAD)
    local push_image_name="${DOCKER_ID_USER}/${image_name}"
    docker login -u="${DOCKER_ID_USER}" -p="${DOCKER_PASSWORD}"
    docker tag "$image_name" "$push_image_name":"$git_rev"
    docker push "$push_image_name"
    docker tag "$push_image_name":"$git_rev" "$push_image_name":latest
    docker push "$push_image_name"
}

trigger_biodomes_build() {
    body='{
    "request": {
    "branch":"master"
    } }'

    curl -s -X POST \
       -H "Content-Type: application/json" \
       -H "Accept: application/json" \
       -H "Travis-API-Version: 3" \
       -H "Authorization: token ${TRAVIS_TOKEN}" \
       -d "$body" \
       https://api.travis-ci.org/repo/${ORG_NAME}%2Fbiodomes/requests
}

install_jekyll() {
    gem install jekyll
    bundle install
}

build_jekyll() {
    bundle exec jekyll build
}

commit_to_branch() {
    local branch="${1:-gh-pages}"
    local git_rev=$(git rev-parse --short HEAD)
    git config user.name 'Michael Eden'
    git config user.email 'themichaeleden@gmail.com'
    git fetch origin
    git reset "origin/$branch"
    git add -A .
    git status
    git commit -m "Automatic Travis deploy of ${git_rev}."
    git push -q origin "HEAD:${branch}"
}

set_cloudflare_dns() {
    local type="$1"
    local name="$2"
    local content="$3"
    local proxied="$4"

    # get all the dns records
    local dns_records=$(curl -X GET \
          -H "X-Auth-Email: ${CLOUDFLARE_EMAIL}" \
          -H "X-Auth-Key: ${CLOUDFLARE_AUTH}" \
          -H "Content-Type: application/json" \
          "https://api.cloudflare.com/client/v4/zones/${CLOUDFLARE_ZONE}/dns_records")

    # Check if we already set it
    local jq_exists=$(cat <<-END
        .result[]
        | select(.type == "${type}")
        | select(.name == "${name}")
        | select(.content == "${content}")
END
    )
    if [[ -n $(echo "${dns_records}" | jq "${jq_exists}") ]]; then
        echo "Record already set, not setting again."
        return
    fi

    # Check if there's a different one already set
    local duplicate_exists=$(echo "${dns_records}" \
        | jq '.result[] | select(.name == "${name}")')
    if [[ -n $duplicate_exists ]]; then
        echo "Record with the same host exists, will not overwrite!"
        exit 64
    fi

    # Set IT!
    local dns_record=$(cat <<-END
        {
            "type": "${type}",
            "name": "${name}",
            "content": "${content}",
            "proxied": $proxied
        }
END
    )
    local dns_success=$(curl -X POST \
         --data "$dns_record" \
         -H "X-Auth-Email: ${CLOUDFLARE_EMAIL}" \
         -H "X-Auth-Key: ${CLOUDFLARE_AUTH}" \
         -H "Content-Type: application/json" \
         "https://api.cloudflare.com/client/v4/zones/${CLOUDFLARE_ZONE}/dns_records")

    if [[ $dns_success != true ]]; then
        echo 'DNS Setting on cloudflare failed!!'
        echo 'CloudFlare output:'
        echo "$dns_success"
        exit 64
    fi
    echo DNS set! You\'ll have to wait a bit to see the changes!
}


deployment_project() {
    build_project_source
    test_project_source
    build_project_container

    if [[ ${TRAVIS_BRANCH:-} = master && ${TRAVIS_PULL_REQUEST:-} = false ]]; then
        publish_project_container
        trigger_biodomes_build
    fi
}

static_project() {
    # If there's anything we want to do to build it,
    # do it now. (if we're using something other than jekyll)
    if [[ -f build.sh ]]; then
        ./build.sh
    fi

    if [[ ${TRAVIS_BRANCH:-} = gh-pages && ${TRAVIS_PULL_REQUEST:-} = false ]]; then
        set_cloudflare_dns CNAME "$(cat CNAME)" "${ORG_NAME}.github.io" true
    fi
}



if [[ $PROJECT_TYPE = deployment ]]; then
    deployment_project
fi

if [[ $PROJECT_TYPE = static ]]; then
    static_project
fi
