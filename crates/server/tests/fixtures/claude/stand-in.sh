#!/bin/bash
fixtures=$(dirname "$0")

show() {
    local title lines
    {
        IFS= read -r title
        mapfile -t lines
    } <"$fixtures/$1.txt"
    printf '\033]0;%s\007\033[H\033[2J' "$title"
    printf 'agent %s\r\n' "${GBAND_WINDOW:-}"
    printf '%s\r\n' "${lines[@]}"
}

show start
while IFS= read -rsn1 key; do
    case $key in
    w) show working ;;
    b) show permission ;;
    i) show finished ;;
    s) show start ;;
    l)
        show working
        sleep 1
        show finished
        ;;
    q) exit 0 ;;
    esac
done
