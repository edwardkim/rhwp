# 멈춘 한글이 **어떤 창을 띄우고 있는지** 열거한다.
#
# 왜 필요한가: 액션이 안 끝날 때 "대화상자겠지" 하고 원인을 붙이면 틀린다(실제로 틀렸다).
# 창 목록을 기계로 뽑으면 69개를 눈으로 보지 않고도 전수 판정할 수 있고, 대화상자가 있으면
# **제목까지** 나와서 어느 대화상자인지 바로 안다.
#
#   powershell -File tools/hwpctrl_compat/hang_windows.ps1
#
# 출력: 한 줄에 `핸들<TAB>클래스<TAB>제목<TAB>보임여부`.

$sig = @'
using System;
using System.Collections.Generic;
using System.Runtime.InteropServices;
using System.Text;

public class WinList {
  [DllImport("user32.dll")] static extern bool EnumWindows(EnumProc cb, IntPtr p);
  [DllImport("user32.dll")] static extern int GetWindowText(IntPtr h, StringBuilder s, int n);
  [DllImport("user32.dll")] static extern int GetClassName(IntPtr h, StringBuilder s, int n);
  [DllImport("user32.dll")] static extern bool IsWindowVisible(IntPtr h);
  [DllImport("user32.dll")] static extern uint GetWindowThreadProcessId(IntPtr h, out uint pid);
  delegate bool EnumProc(IntPtr h, IntPtr p);

  public static List<string> ForProcess(uint want) {
    var rows = new List<string>();
    EnumWindows((h, p) => {
      uint pid; GetWindowThreadProcessId(h, out pid);
      if (pid != want) return true;
      var title = new StringBuilder(512); GetWindowText(h, title, 512);
      var cls = new StringBuilder(256); GetClassName(h, cls, 256);
      rows.Add(h.ToString() + "\t" + cls + "\t" + title + "\t" + (IsWindowVisible(h) ? "보임" : "숨김"));
      return true;
    }, IntPtr.Zero);
    return rows;
  }
}
'@

Add-Type -TypeDefinition $sig -Language CSharp

$procs = Get-Process -Name Hwp -ErrorAction SilentlyContinue
if (-not $procs) { Write-Output "한글 프로세스 없음"; exit 0 }
foreach ($p in $procs) {
  foreach ($row in [WinList]::ForProcess([uint32]$p.Id)) { Write-Output $row }
}
