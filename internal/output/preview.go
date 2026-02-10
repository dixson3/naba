package output

import (
	"os/exec"
	"runtime"
)

// Preview opens a file in the system's default viewer.
func Preview(path string) error {
	var cmd string
	switch runtime.GOOS {
	case "darwin":
		cmd = "open"
	case "windows":
		cmd = "start"
	default:
		cmd = "xdg-open"
	}
	return exec.Command(cmd, path).Start()
}
