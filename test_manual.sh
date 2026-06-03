#!/bin/bash
cli="target/release/rde-cli.exe"
debuggee="target/release/rust_app_example.exe"

# Create stdin file first
cat > test_stdin.txt << 'INNEREOF'
break insert
continue
step
step
step
bt
quit
INNEREOF

# Start process with redirected I/O
$cli "$debuggee" --demo insert-sequence < test_stdin.txt > test_stdout.txt 2> test_stderr.txt &
pid=$!

sleep 10

if kill -0 $pid 2>/dev/null; then
    kill -9 $pid 2>/dev/null
fi

echo "=== STDOUT ==="
cat test_stdout.txt 2>/dev/null || echo "(no stdout)"
echo ""
echo "=== STDERR ==="
cat test_stderr.txt 2>/dev/null || echo "(no stderr)"
