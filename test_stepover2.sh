#!/bin/bash
cli="target/release/rde-cli.exe"
debuggee="target/release/rust_app_example.exe"

{
    sleep 0.5
    echo "break height"
    sleep 0.5
    echo "continue"
    sleep 1
    echo "next"
    sleep 2
    echo "quit"
} | $cli "$debuggee" --demo insert-sequence > test_so2_stdout.txt 2> test_so2_stderr.txt &
pid=$!

sleep 8
if kill -0 $pid 2>/dev/null; then
    kill -9 $pid 2>/dev/null
fi

echo "=== STDOUT ==="
cat test_so2_stdout.txt 2>/dev/null | grep -E "^(rde>|\[Breakpoint|\[Step|\[Single|Erro)" | tail -20
echo ""
echo "=== STDERR (last 20 non-timestamp lines) ==="
cat test_so2_stderr.txt 2>/dev/null | grep -v "^\[2m2026" | tail -20
