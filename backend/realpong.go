package main

import (
	"crypto/sha256"
	"encoding/base64"
	"fmt"
	"github.com/gorilla/websocket"
	"log"
	"math/rand"
	"net/http"
	"os"
	"time"
	"unsafe"
)

type Game struct {
	id      string
	created time.Time
	player1 *Player
	player2 *Player
}

type Player struct {
	id            string
	created       time.Time
	last_activity time.Time
	game          *Game
	score         uint32
}

type ServerContext struct {
	id_seq_cnt uint64
	players    map[string]*Player
}

// Generate an unique identifier (sha256 base64 string)
func game_gen_uuid(ctx ServerContext) string {
	ctx.id_seq_cnt += 1
	id := sha256.Sum256((*[8]byte)(unsafe.Pointer(&ctx.id_seq_cnt))[:])
	return base64.URLEncoding.EncodeToString(id[:])
}

// Setup ticker to destroy clients which have been inactive for some time
func game_start_gc(ctx ServerContext) {

	// Setup timer to run every 2 seconds.
	run_interval := 1000 * time.Millisecond

	ticker := time.NewTicker(run_interval)
	go func() {
		for range ticker.C {
			now := time.Now()
			for id, player := range ctx.players {
				inactivity_time := now.Sub(player.last_activity) //.Sub(now)
				if inactivity_time >= 2000*time.Millisecond {
					log.Println("Player '" + id + "' is inactive - Destroy!")
					delete(ctx.players, id)
				}
			}
		}
	}()

	return
}

// Creat new client
func game_new_player(ctx ServerContext) *Player {
	now := time.Now()

	// Generate client ID
	player_id := game_gen_uuid(ctx)
	game_id := game_gen_uuid(ctx)
	game := &Game{game_id, now, nil, nil}

	ctx.players[player_id] = &Player{player_id, now, now, game, 0}

	game.player1 = ctx.players[player_id]

	log.Println("New player created (" + player_id + ")")
	return ctx.players[player_id]
}

var upgrader = websocket.Upgrader{
	ReadBufferSize:  1024,
	WriteBufferSize: 1024,
}

func main() {

	rand.Seed(time.Now().UTC().UnixNano())

	// Some backend config setup
	http_port := os.Getenv("HTTP_PORT")
	http_root := os.Getenv("HTTP_ROOT")
	if len(http_root) == 0 {
		http_root = "../frontend"
	}
	if len(http_port) == 0 {
		http_port = "8080"
	}

	// Create the server context where we map connections to players and games
	ctx := ServerContext{rand.Uint64(), make(map[string]*Player)}

	game_new_player(ctx)

	// Setup garbage collecting of inactive clients
	game_start_gc(ctx)

	// Setup http server to serve static files
	log.Printf("Serve static from '" + http_root + "'")
	fs := http.FileServer(http.Dir(http_root))
	http.Handle("/", fs)

	// Setup http server to listen for websocket connections
	http.HandleFunc("/ws", func(w http.ResponseWriter, r *http.Request) {

		// For now - Echo whatever we receive to client
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
