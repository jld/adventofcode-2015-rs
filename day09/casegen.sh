#!/bin/sh
set -e -u
n=$1
nl="
"
# Warning: GNU-isms.
names=$(grep '^[A-Z][a-z]*$' /usr/share/dict/words | shuf | head -n $n)$nl
dists=$(seq $((n * n * 3)) | shuf | head -n $((n * (n - 1) / 2)))$nl

while [ -n "$names" ]; do
   name=${names%%${nl}*}
   names=${names#*${nl}}
   rnames=$names
   while [ -n "$rnames" ]; do
      rname=${rnames%%${nl}*}
      rnames=${rnames#*${nl}}
      dist=${dists%%${nl}*}
      dists=${dists#*${nl}}
      echo "$name to $rname = $dist"
   done
done
