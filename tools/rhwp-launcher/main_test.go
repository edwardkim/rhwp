package main

import "testing"

func TestBuildLauncherCommandUsesExePathAndQuotedFile(t *testing.T) {
	exePath := `D:\vibe-coding\rhwp\tools\RHWPLauncher.exe`
	filePath := `D:\docs\sample file.hwp`

	got := buildLauncherCommand(exePath, filePath)
	want := `"` + exePath + `" "` + filePath + `"`

	if got != want {
		t.Fatalf("expected %q, got %q", want, got)
	}
}

func TestScriptPathFromExeUsesSiblingPowerShellScript(t *testing.T) {
	exePath := `D:\vibe-coding\rhwp\tools\RHWPLauncher.exe`
	got := scriptPathFromExe(exePath)
	want := `D:\vibe-coding\rhwp\tools\rhwp_launcher.ps1`

	if got != want {
		t.Fatalf("expected %q, got %q", want, got)
	}
}
