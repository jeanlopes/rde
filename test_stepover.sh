#!/bin/bash
cli="target/release/rde-cli.exe"
debuggee="target/release/rust_app_example.exe"

{
echo "break height"
echo "continue"
sleep 1
echo "next"
sleep 1
echo "quit"
} | $cli "$debuggee" --demo insert-sequence > test_so_stdout.txt 2> test_so_stderr.txt

echo "=== STDOUT ==="
cat test_so_stdout.txt 2>/dev/null
echo ""
echo "=== STDERR ==="
cat test_so_stderr.txt 2>/dev/null
