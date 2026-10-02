package main

import (
	"os"
	"path/filepath"
	"testing"
)

func TestRun(t *testing.T) {
	src, dst, single := t.TempDir(), t.TempDir(), t.TempDir()
	tool := filepath.Join(src, "tool")
	if err := os.WriteFile(tool, []byte("bin"), 0o755); err != nil {
		t.Fatal(err)
	}
	if err := os.Chmod(tool, os.ModeSetuid|0o755); err != nil { // setuid must not survive the copy
		t.Fatal(err)
	}

	for _, c := range []struct{ src, dst string }{{src, dst}, {tool, single}} {
		if err := run(c.src, c.dst); err != nil {
			t.Fatal(err)
		}
		fi, err := os.Stat(filepath.Join(c.dst, "tool"))
		if err != nil {
			t.Fatal(err)
		}
		if m := fi.Mode(); m&0o111 != 0o111 || m&os.ModeSetuid != 0 {
			t.Errorf("run(%s): mode %v, want executable without setuid", c.src, m)
		}
	}
	if err := run(tool, single); err == nil {
		t.Error("rerun: want error, existing files must not be overwritten")
	}
	if err := run(filepath.Join(src, "missing"), dst); err == nil {
		t.Error("missing source: want error")
	}
}
