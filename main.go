// Command copy-tools copies binaries from its own image into a shared volume,
// for use as a Kubernetes init container in a distroless static image.
//
//	copy-tools SRC DSTDIR
//
// A file SRC is copied into DSTDIR; a directory SRC has its contents copied
// into DSTDIR. Any failure exits nonzero, so Kubernetes never starts the app
// container with missing binaries.
package main

import (
	"errors"
	"fmt"
	"io"
	"os"
	"path/filepath"
)

func main() {
	if len(os.Args) != 3 {
		fmt.Fprintln(os.Stderr, "usage: copy-tools SRC DSTDIR\n\nCopies file SRC, or the contents of directory SRC, into directory DSTDIR.")
		os.Exit(2)
	}
	src, dst := os.Args[1], os.Args[2]
	if err := run(src, dst); err != nil {
		fmt.Fprintf(os.Stderr, "copy-tools: copying %s to %s: %v\n", src, dst, err)
		os.Exit(1)
	}
	fmt.Printf("copy-tools: copied %s to %s\n", src, dst)
}

// run copies src into dst. Like os.CopyFS, it refuses to overwrite existing files.
func run(src, dst string) error {
	fi, err := os.Stat(src)
	if err != nil {
		return err
	}
	if fi.IsDir() {
		return os.CopyFS(dst, os.DirFS(src))
	}
	in, err := os.Open(src)
	if err != nil {
		return err
	}
	defer in.Close()
	out, err := os.OpenFile(filepath.Join(dst, filepath.Base(src)), os.O_WRONLY|os.O_CREATE|os.O_EXCL, fi.Mode().Perm())
	if err != nil {
		return err
	}
	_, err = io.Copy(out, in)
	return errors.Join(err, out.Close())
}
