var connectionStatus;
var messageHandler;
var gameCreator;
window.onload = () => {
  connectionStatus = new ConnectionStatus();
  messageHandler = new MessageHandler();
  const createGame = document.getElementById("createGame");

  const socket = new WebSocket("ws://localhost");

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

class MessageHandler {
  constructor() {
    this.messageBox = document.getElementById("messages");
    this.print("awaiting further instructions from backend");
  }

  print(message) {
    const li = document.createElement("li");
    li.textContent = message;
    this.messageBox.appendChild(li);
  }
}
