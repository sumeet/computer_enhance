#!/bin/bash
set -e

rustc ./decoder.rs 
diff -u <(cat listing_0037_single_register_mov.asm | grep -v '^$' | grep -v '^;') <(./decoder listing_0037_single_register_mov)
diff -u <(cat listing_0038_many_register_mov.asm | grep -v '^$' | grep -v '^;') <(./decoder listing_0038_many_register_mov)
diff -u <(cat listing_0039_more_movs.asm | grep -v '^$' | grep -v '^;') <(./decoder listing_0039_more_movs)
echo "test succeeded"
