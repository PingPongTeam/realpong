window.onload = () => {
  const createGameButton = document.getElementById("createGame");

  const socket = new WebSocket("ws://" + window.location.host + "/ws");

  socket.onopen = e => {
    connectionStatus.update({
      message: "Connection established!",
      status: "success"
    });
    console.log(e.currentTarget.url);
  };
  socket.onerror = error => {
    connectionStatus.update({ message: "Connection error", status: "danger" });
    console.warn(error);
  };
  socket.onmessage = event => {
    messageHandler.print(event.data);
  };

  createGameButton.addEventListener("click", () => {
    socket.send("gifv match plz!");
  });
};

class ConnectionStatus {
  constructor() {
    this.prompt = document.getElementById("connectionStatus");
  }

  update({ message, status }) {
    this.prompt.textContent = message;
    switch (status) {
      case "success":
        this.prompt.style.borderColor = "springgreen";
        break;
      case "warning":
        this.prompt.style.borderColor = "darkorange";
        break;
      case "danger":
        this.prompt.style.borderColor = "orangered";
        break;

      default:
        this.prompt.style.borderColor = "grey";
        break;
    }
  }
}
const connectionStatus = new ConnectionStatus();

class TakeOver {
  constructor() {
    this.takeOver = document.createElement("div");
    this.takeOver.id = "takeOver";
    document.body.appendChild(this.takeOver);
  }

  show(message) {
    this.takeOver.textContent = message;
    this.takeOver.style.display = "flex";
  }

  hide() {
    this.takeOver.style.display = "none";
  }
}
const takeOver = new TakeOver();

class MessageHandler {
  constructor() {
    this.messageBox = document.getElementById("messages");
    this.print("awaiting further instructions from backend");
  }

  interpret(message) {
    // wat?
  }

  print(message) {
    const li = document.createElement("li");
    li.textContent = message;
    this.messageBox.appendChild(li);
  }
}
const messageHandler = new MessageHandler();

class Canvas {
  constructor() {
    this.canvas = document.getElementById("canvas");
    if (!this.canvas.getContext) {
      takeOver.show(
        "Your browser must be very old?! I can't darw things with technology this old :("
      );
    }
  }
}
const canvas = new Canvas();
