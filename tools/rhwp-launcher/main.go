package main

import (
	"os"
	"os/exec"
	"path/filepath"
	"syscall"
)

func scriptPathFromExe(exePath string) string {
	return filepath.Join(filepath.Dir(exePath), "rhwp_launcher.ps1")
}

func buildLauncherCommand(exePath, filePath string) string {
	if filePath == "" {
		return `"` + exePath + `"`
	}
	return `"` + exePath + `" "` + filePath + `"`
}

func main() {
	exePath, err := os.Executable()
	if err != nil {
		os.Exit(1)
	}

	scriptPath := scriptPathFromExe(exePath)
	args := []string{
		"-NoProfile",
		"-ExecutionPolicy", "Bypass",
		"-WindowStyle", "Hidden",
		"-File", scriptPath,
	}

	if len(os.Args) > 1 {
		args = append(args, os.Args[1])
	}

	cmd := exec.Command("powershell.exe", args...)
	cmd.SysProcAttr = &syscall.SysProcAttr{HideWindow: true}
	if err := cmd.Start(); err != nil {
		os.Exit(1)
	}
}
