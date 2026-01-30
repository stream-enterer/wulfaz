package main

import (
	"log"
	"time"

	"github.com/hajimehoshi/ebiten/v2"

	"wulfaz/internal/app"
)

func main() {
	seed := time.Now().UnixNano()

	ebiten.SetWindowSize(800, 600)
	ebiten.SetWindowTitle("Wulfaz")

	if err := ebiten.RunGame(app.New(seed)); err != nil {
		log.Fatal(err)
	}
}
