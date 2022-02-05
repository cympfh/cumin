#!/bin/bash

log-date() {
    date "+%Y/%m/%d %H:%M:%S" | tr -d '\r\n'
}
log-info() {
    printf "[$(log-date)] \e[32m$1\e[0m\n" >&2
}
log-error() {
    printf "[$(log-date)] \e[31m$1\e[0m\n" >&2
}

CUMINC=$(command -v cuminc)
if [ ! "$CUMINC" ]; then
    log-error "Not Found: cuminc"
    exit 2
fi

find_cumin() {
    f=${1%.json}.cumin
    g=${1%.fail}.cumin
    if [ -f "$f" ]; then
        echo $f
    elif [ -f "$g" ]; then
        echo $g
    else
        echo ""
    fi
}

find_env() {
    f=${1%.json}.env
    g=${1%.fail}.env
    if [ -f "$f" ]; then
        cat $f | tr "\n" " " | sed 's/ *$//'
    elif [ -f "$g" ]; then
        cat $g | tr "\n" " " | sed 's/ *$//'
    else
        echo ""
    fi
}

run_cumin() {
    CUMIN=$1
    ENV=$2
    echo env "$ENV" $CUMINC "$CUMIN" | env -i bash
}

for JSON in examples/*.json; do
    CUMIN=$(find_cumin "$JSON")
    ENV=$(find_env "$JSON")
    log-info "cumin=$CUMIN, json=$JSON, env=($ENV)"
    if [ -z "$CUMIN" ]; then
        log-error "Not Found cumin File for $JSON"
        exit 2
    fi
    if ! diff <( jq -cM . "$JSON" ) <( run_cumin "$CUMIN" "$ENV" | jq -cM ); then
        log-error "$CUMIN failed"
    fi
done

for FAIL in examples/*.fail; do
    CUMIN=$(find_cumin "$FAIL")
    ENV=$(find_env "$FAIL")
    log-info "Fail case: cumin=$CUMIN, env=($ENV)"
    run_cumin "$CUMIN" "$ENV" >/dev/null 2>&1
    if [ $? = 0 ]; then
        log-error "$CUMIN succeeded against your expectation"
    fi
done
