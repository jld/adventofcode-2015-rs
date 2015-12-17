#!/usr/bin/awk -f
{g[NF]++}END{for(k in g){print k,g[k]}}
