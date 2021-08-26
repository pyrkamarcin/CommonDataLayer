#!/usr/bin/env bash

has_param() {
    local term="$1"
    shift

    for arg; do
        if [[ $arg == "$term" ]]; then
            echo 1
            return
        fi
    done
    echo 0
}

DRY_RUN=$(has_param '--dry-run' "$@")

Try() {
    if [ "$DRY_RUN" -eq "1" ]; then
        echo "+ " "$@"
    else
        echo "+ " "$@"
        eval "$@"
    fi
}

VERSION_LONG="$1";
VERSION_RELEASE=false
echo "version: ${VERSION_LONG}"

if [[ $VERSION_LONG =~ ^([0-9]+\.[0-9]+)\.[0-9]+$ ]]; then
    echo "Its release!"
    VERSION_RELEASE=true
    VERSION_SHORT="${BASH_REMATCH[1]}"
    echo "short: $VERSION_SHORT"
else
    echo "Its RC!"
fi

Build() {
    crate="$1"
    echo "building ${crate}"

    IMAGE="epiphanyplatform/cdl-${crate}"
    LONG_IMAGE="${IMAGE}:${VERSION_LONG}"

    # shellcheck disable=SC2068 # Intended splitting of @
    Try docker build . -t "$LONG_IMAGE" ${@:2}
    Try docker push "$LONG_IMAGE"
    if [ $VERSION_RELEASE = true ] ; then
        SHORT_IMAGE="${IMAGE}:${VERSION_SHORT}"

        Try docker tag "$LONG_IMAGE" "$SHORT_IMAGE"
        Try docker push "$SHORT_IMAGE"
    fi
}

crates=(data-router command-service query-router query-service query-service-ts schema-registry api edge-registry partial-update-engine object-builder materializer-general materializer-ondemand)
for crate in "${crates[@]}"
do
    Build "${crate}" --build-arg BIN="$crate"
done

cd web-admin || exit

Build "web-admin"
