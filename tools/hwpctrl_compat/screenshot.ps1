# 화면을 찍어 PNG 로 남긴다 — 대화상자 때문에 오라클이 멈췄을 때 **무엇이 떴는지 보려는 것**이다.
#
# 왜 필요한가: 액션이 멈추면 "대화상자"라고만 알 뿐 어느 대화상자인지, 눌러야 할 것이 무엇인지
# 모른다. 찍어서 보면 그 자리에서 갈래가 갈린다(파일 대화상자인지, 서식 대화상자인지, 오류인지).
#
#   powershell -File tools/hwpctrl_compat/screenshot.ps1 -Out output/poc/hwpctrl/shot.png
param(
  [string]$Out = "output/poc/hwpctrl/screen.png"
)

Add-Type -AssemblyName System.Windows.Forms
Add-Type -AssemblyName System.Drawing

$bounds = [System.Windows.Forms.SystemInformation]::VirtualScreen
$bmp = New-Object System.Drawing.Bitmap($bounds.Width, $bounds.Height)
$gfx = [System.Drawing.Graphics]::FromImage($bmp)
$gfx.CopyFromScreen($bounds.Location, [System.Drawing.Point]::Empty, $bounds.Size)

$dir = Split-Path -Parent $Out
if ($dir -and -not (Test-Path $dir)) { New-Item -ItemType Directory -Force -Path $dir | Out-Null }
$bmp.Save($Out, [System.Drawing.Imaging.ImageFormat]::Png)
$gfx.Dispose()
$bmp.Dispose()
Write-Output "찍었다: $Out ($($bounds.Width)x$($bounds.Height))"
