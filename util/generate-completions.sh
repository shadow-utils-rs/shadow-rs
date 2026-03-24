#!/usr/bin/env bash
# Generate shell completions for all shadow-rs tools.
#
# Usage:
#   ./util/generate-completions.sh bash
#   ./util/generate-completions.sh zsh
#   ./util/generate-completions.sh fish
#
# For proper clap-based completions, build with clap_complete.
# This is a minimal placeholder until clap_complete is integrated.

set -euo pipefail

SHELL_TYPE="${1:-bash}"
TOOLS="passwd pwck useradd userdel usermod chpasswd chage groupadd groupdel groupmod grpck chfn chsh newgrp"

case "$SHELL_TYPE" in
  bash)
    for tool in $TOOLS; do
      echo "complete -W '--help' $tool"
    done
    ;;
  zsh)
    echo "#compdef shadow-rs"
    for tool in $TOOLS; do
      echo "complete -c $tool -s h -l help -d 'Show help'"
    done
    ;;
  fish)
    for tool in $TOOLS; do
      echo "complete -c $tool -s h -l help -d 'Show help'"
    done
    ;;
  *)
    echo "Usage: $0 {bash|zsh|fish}" >&2
    exit 1
    ;;
esac
