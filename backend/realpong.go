package main

import (
	"fmt"
	"github.com/gorilla/websocket"
	"log"
	"net/http"
	"os"
)

var upgrader = websocket.Upgrader{
	ReadBufferSize:  1024,
	WriteBufferSize: 1024,
}

func main() {
	http_port := os.Getenv("HTTP_PORT")
	http_root := os.Getenv("HTTP_ROOT")
	if len(http_root) == 0 {
		http_root = "../frontend"
	}
	if len(http_port) == 0 {
		http_port = "8080"
	}
	log.Printf("Serve static from '" + http_root + "'")
	fs := http.FileServer(http.Dir(http_root))
	http.Handle("/", fs)

	http.HandleFunc("/ws", func(w http.ResponseWriter, r *http.Request) {
		conn, err := upgrader.Upgrade(w, r, nil)
		if err != nil {
			log.Println("upgrade: ", err)
			return
		}
		defer conn.Close()
		for {
			// Read message from browser
			msgType, msg, err := conn.ReadMessage()
			if err != nil {
				log.Println("read: ", err)
				return
			}

			// Print the message to the console
			fmt.Printf("%s sent: %s\n", conn.RemoteAddr(), string(msg))

			// Write message back to browser
			if err = conn.WriteMessage(msgType, msg); err != nil {
				log.Println("write: ", err)
				return
			}
		}
	})

	log.Printf("Listening for connections at port " + http_port)
	log.Fatal(http.ListenAndServe(":"+http_port, nil))
}
