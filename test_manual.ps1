$cli = "C:\workspace\rde\target\release\rde-cli.exe"
$debuggee = "C:\workspace\rde\target\release\rust_app_example.exe"

$p = Start-Process -FilePath $cli -ArgumentList @($debuggee, "--demo", "insert-sequence") -RedirectStandardInput "test_stdin.txt" -RedirectStandardOutput "test_stdout.txt" -RedirectStandardError "test_stderr.txt" -NoNewWindow -PassThru

Start-Sleep -Seconds 1

$stdin = @"
break insert
continue
step
step
step
bt
quit
"@

$stdin | Out-File -FilePath "test_stdin.txt" -Encoding ASCII -NoNewline

Start-Sleep -Seconds 10

if (!$p.HasExited) {
    Stop-Process -Id $p.Id -Force
}

Write-Host "=== STDOUT ==="
Get-Content "test_stdout.txt" -ErrorAction SilentlyContinue
Write-Host "=== STDERR ==="
Get-Content "test_stderr.txt" -ErrorAction SilentlyContinue
