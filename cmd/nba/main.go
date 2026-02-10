package main

import (
	"fmt"
	"os"

	"github.com/dixson3/nba/internal/cli"
)

func main() {
	if err := cli.Execute(); err != nil {
		code := 1
		if ec, ok := err.(interface{ ExitCode() int }); ok {
			code = ec.ExitCode()
		}
		fmt.Fprintln(os.Stderr, err)
		os.Exit(code)
	}
}
