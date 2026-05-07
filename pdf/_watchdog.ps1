$log = 'C:\Users\planet\rhwp\pdf\_convert.log'
$idleLimit = 60  # seconds without log update before kill

while ($true) {
  Start-Sleep -Seconds 10
  if (-not (Test-Path -LiteralPath $log)) { continue }
  $age = (Get-Date) - (Get-Item -LiteralPath $log).LastWriteTime
  if ($age.TotalSeconds -gt $idleLimit) {
    $hwp = Get-Process -Name 'Hwp' -ErrorAction SilentlyContinue
    if ($hwp) {
      $msg = "[{0}] WATCHDOG: log idle {1:N0}s, killing Hwp PID(s) {2}" -f (Get-Date -Format 'HH:mm:ss'), $age.TotalSeconds, ($hwp.Id -join ',')
      Add-Content -LiteralPath $log -Value $msg -Encoding UTF8
      $hwp | ForEach-Object { Stop-Process -Id $_.Id -Force -ErrorAction SilentlyContinue }
    }
  }
  # Self-exit when conversion is done
  $tail = Get-Content -LiteralPath $log -Tail 2 -ErrorAction SilentlyContinue
  if ($tail -match 'DONE: ok=') { break }
}
