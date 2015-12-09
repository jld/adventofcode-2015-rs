#!/bin/sed -f
/ -> b$/!p
s/[a-z][a-z]*/&_/g
$aa_ -> b
