package main

import (
	"fmt"
	"runtime"
)

func main() {
	fmt.Printf("Hello from %s!\n", runtime.Version())
}
