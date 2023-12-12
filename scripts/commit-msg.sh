# mv to .git/hooks/commit-msg
#!/bin/sh

set -e

if ! head -1 "$1" | grep -E "^(feat|fix|chore|docs|test|style|refactor|perf|build|ci|revert)(\(.+?\))?: .{1,}$"; then
    echo "Aborting commit. Invalid commit message." >&2
    exit 1
fi
if ! head -1 "$1" | grep -qE "^.{1,88}$"; then
    echo "Aborting commit. Commit message is too long." >&2
    exit 1
fi
