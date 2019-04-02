#!/usr/bin/env bash

set -u

trap "exit 1" TERM
export TOP_PID=$$

: "${APPNAME:=deploy}"
: "${CIRCLE_FULL_ENDPOINT:=https://circleci.com/api/v1.1/project/github/$CIRCLE_PROJECT_USERNAME/$CIRCLE_PROJECT_REPONAME}"
: "${CIRCLE_TAG:=}"

main() {
    need_cmd curl
    need_cmd jq
    need_cmd mkdir
    need_cmd grep
    need_cmd cat
    need_cmd head
    need_cmd sed
    need_cmd basename
    need_cmd file

    if [ -z "$CIRCLE_TAG" ]; then
        say "Deploying only when CIRCLE_TAG is defined"
        exit 0
    fi
    
    # Get the list with CircleCI build numbers for the current tagged release
    local _builds
    local _filter
    local _build_nums
    
    _builds=$(ensure circle "$CIRCLE_FULL_ENDPOINT")
    _filter='.[] | select(.vcs_tag == "'$CIRCLE_TAG'" and .vcs_revision == "'$CIRCLE_SHA1'" and .workflows.job_name != "deploy") | .build_num'
    _build_nums=$(ensure jq "$_filter" <<< "$_builds")

    if [ -z "$_build_nums" ]; then
        err "No builds for tagged release $CIRCLE_TAG"
    fi

    ensure mkdir -p /tmp/artifacts

    IFS=$'\n'
    for build_num in $_build_nums; do
        say "Downloading artifacts for #${build_num}..."

        local _artifacts_json
        local _artifacts

        _artifacts_json=$(circle "$CIRCLE_FULL_ENDPOINT/$build_num/artifacts")
        _artifacts=$(ensure jq -r '.[] | .url' <<< "$_artifacts_json")

        for artifact in $_artifacts; do
            say "$artifact"
            (ensure cd /tmp/artifacts; ensure curl -sSOL --retry 3 "$artifact")
        done
    done

    local _body
    local _payload
    local _response
    local _upload_url

    # Grab latest release notes from the Changelog
    _body=$(
        ensure cat CHANGELOG.md \
        | ensure grep -Pzo '##.*\n\n\K\X*?(?=\n##|$)' \
        | ensure tr '\0' '\n' \
        | ensure head -n1
    )

    _payload=$(
        jq --null-input \
            --arg tag "$CIRCLE_TAG" \
            --arg name "$CIRCLE_TAG" \
            --arg body "$_body" \
            '{ tag_name: $tag, name: $name, body: $body, draft: false }'
    )

    _response=$(
        curl -sSL -X POST "https://api.github.com/repos/$CIRCLE_PROJECT_USERNAME/$CIRCLE_PROJECT_REPONAME/releases" \
            -H "Accept: application/vnd.github.v3+json" \
            -H "Authorization: token $GITHUB_TOKEN" \
            -H "Content-Type: application/json" \
            -d "$_payload"
    )

    _upload_url=$(
        echo "$_response" \
        | ensure jq -r .upload_url \
        | ensure sed -e "s/{?name,label}//"
    )

    for _file in /tmp/artifacts/*; do
        local _basename
        local _mimetype
        local _response
        local _state

        _basename=$(ensure basename "$_file")
        _mimetype=$(ensure file --mime-type -b "$_file") 

        say "Uploading $_basename..."
        
        _response=$(
            curl -sSL -X POST "$_upload_url?name=$_basename" \
                -H "Accept: application/vnd.github.manifold-preview" \
                -H "Authorization: token $GITHUB_TOKEN" \
                -H "Content-Type: $_mimetype" \
                --data-binary "@$_file"
        )

        _state=$(ensure jq -r '.state' <<< "$_response")

        if [ "$_state" != "uploaded" ]; then
            err "Artifact not uploaded: $_basename"
        fi
    done
}

circle() {
    curl "${1}?circle-token=$CIRCLE_TOKEN" \
        -sS --retry 3 \
        -H "Accept: application/json"
}

say() {
    printf '\33[1m%s:\33[0m %s\n' "$APPNAME" "$1"
}

err() {
    printf '\33[1;31m%s:\33[0m %s\n' "$APPNAME" "$1" >&2
    kill -s TERM $TOP_PID
}

need_cmd() {
    if ! command -v "$1" > /dev/null 2>&1; then
        err "need '$1' (command not found)"
    fi
}

ensure() {
    "$@"
    if [ $? != 0 ]; then
        err "command failed: $*";
    fi
}

main "$@" || exit 1
