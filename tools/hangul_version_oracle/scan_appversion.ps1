# Extract authoring-app / format version for every document in the corpus.
#   .hwpx -> version.xml carries the exact authoring app ("application", "appVersion")
#   .hwp  -> only the HWP5 format version is recorded (FileHeader, signature + 32)
# Output TSV: relpath, kind, appVersion, formatVersion
[CmdletBinding()]
param(
  [Parameter(Mandatory = $true)][string]$ListPath,
  [Parameter(Mandatory = $true)][string]$OutPath,
  [string]$Root = 'D:\hwpdocs_10k_share'
)

$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.IO.Compression.FileSystem

$files = Get-Content -LiteralPath $ListPath -Encoding UTF8 | Where-Object { $_.Trim().Length -gt 0 }
$sig = [System.Text.Encoding]::ASCII.GetBytes('HWP Document File')
$writer = New-Object System.IO.StreamWriter($OutPath, $false, (New-Object System.Text.UTF8Encoding($false)))

function Get-Hwp5Version([string]$path) {
  $fs = [System.IO.File]::OpenRead($path)
  try {
    $len = [int][Math]::Min($fs.Length, 1048576)
    $buf = New-Object byte[] $len
    $read = $fs.Read($buf, 0, $len)
    for ($i = 0; $i -lt $read - $sig.Length - 36; $i++) {
      if ($buf[$i] -eq $sig[0]) {
        $ok = $true
        for ($k = 1; $k -lt $sig.Length; $k++) { if ($buf[$i + $k] -ne $sig[$k]) { $ok = $false; break } }
        if ($ok) {
          return ("{0}.{1}.{2}.{3}" -f $buf[$i + 35], $buf[$i + 34], $buf[$i + 33], $buf[$i + 32])
        }
      }
    }
    return ''
  } finally { $fs.Dispose() }
}

function Get-HwpxVersion([string]$path) {
  $z = $null
  try {
    $z = [System.IO.Compression.ZipFile]::OpenRead($path)
    $e = $z.Entries | Where-Object { $_.FullName -eq 'version.xml' } | Select-Object -First 1
    if (-not $e) { return @('', '') }
    $sr = New-Object System.IO.StreamReader($e.Open())
    $t = $sr.ReadToEnd()
    $sr.Close()
    $app = ''
    $ver = ''
    if ($t -match 'application="([^"]*)"') { $app = $Matches[1] }
    if ($t -match 'appVersion="([^"]*)"') { $ver = $Matches[1] }
    $fmt = ''
    if ($t -match 'major="(\d+)"\s+minor="(\d+)"\s+micro="(\d+)"\s+buildNumber="(\d+)"') {
      $fmt = "$($Matches[1]).$($Matches[2]).$($Matches[3]).$($Matches[4])"
    }
    return @("$app|$ver", $fmt)
  } catch {
    return @('(ERR)', '')
  } finally { if ($z) { $z.Dispose() } }
}

$n = 0
foreach ($f in $files) {
  $n++
  $rel = $f
  if ($f.StartsWith($Root, [System.StringComparison]::OrdinalIgnoreCase)) { $rel = $f.Substring($Root.Length).TrimStart('\') }
  $ext = [System.IO.Path]::GetExtension($f).ToLowerInvariant()
  try {
    if ($ext -eq '.hwpx') {
      $r = Get-HwpxVersion $f
      $writer.WriteLine("$rel`thwpx`t$($r[0])`t$($r[1])")
    } else {
      $v = Get-Hwp5Version $f
      $writer.WriteLine("$rel`thwp5`t`t$v")
    }
  } catch {
    $writer.WriteLine("$rel`tERR`t`t")
  }
  if (($n % 1000) -eq 0) { Write-Output "  $n / $($files.Count)" }
}
$writer.Dispose()
Write-Output "DONE $n"
