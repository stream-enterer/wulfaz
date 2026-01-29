package main

import (
	"fmt"
	"time"

	"wulfaz/internal/tea"
)

func main() {
	seed := time.Now().UnixNano()
	runtime := tea.NewRuntime(seed)
	fmt.Println("Wulfaz MVP scaffold")
	fmt.Printf("Seed: %d\n", seed)
	_ = runtime // TODO: runtime.Run()
}
