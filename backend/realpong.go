package main

import (
	"log"
	"net/http"
	"os"
)

func main() {
	http_port := os.Getenv("HTTP_PORT")
	http_root := os.Getenv("HTTP_ROOT")
	if len(http_root) == 0 {
		http_root = "../../frontend"
	}
	if len(http_port) == 0 {
		http_port = "8080"
	}
	log.Printf("Serve static from '" + http_root + "'")
	fs := http.FileServer(http.Dir(http_root))
	http.Handle("/", fs)

	log.Printf("Listening for connections at port " + http_port)
	log.Fatal(http.ListenAndServe(":"+http_port, nil))
}
