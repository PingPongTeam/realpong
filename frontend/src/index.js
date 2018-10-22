import "phaser";

const gameWidth = window.innerWidth;
const gameHeight = window.innerHeight;

const RealPong = new Phaser.Class({
  Extends: Phaser.Scene,

  initialize: function RealPong() {
    Phaser.Scene.call(this, { key: "realpong" });

    this.paddle;
    this.ball;
    this.caught = false;
    this.paddleTimer;
  },

  preload: function() {
    this.load.image("ball", "assets/bomb.png");
    this.load.image("paddle", "assets/platform.png");
  },

  create: function() {
    this.physics.world.setBoundsCollision(false, false, true, false);

    this.ball = this.physics.add
      .image(100, 100, "ball")
      .setScale(window.devicePixelRatio);

    this.ball.setVelocity(0, 200);
    this.ball.setBounce(1, 1);
    this.ball.setCollideWorldBounds(true);

    this.paddle = this.physics.add
      .image(0, 0, "paddle")
      .setImmovable()
      .setVisible(false);

    this.physics.add.overlap(this.ball, this.paddle, this.hit, null, this);

    this.input.on("pointerdown", pointer => {
      this.paddle.x = pointer.x;
      this.paddle.y = pointer.y;
      this.paddle.visible = true;
      this.paddleTimer = setTimeout(() => {
        this.ball.body.moves = true;
        this.resetPaddle();
      }, 200);
    });
    this.input.on("pointerup", pointer => {
      if (this.caught) {
        this.ball.setVelocity(pointer.x - this.ball.x, pointer.y - this.ball.y);
        this.ball.body.moves = true;
      }
      this.resetPaddle();
    });
  },

  update: function() {
    if (
      this.ball.y > gameHeight ||
      this.ball.x > gameWidth ||
      this.ball.x < 0
    ) {
      this.resetBall();
    }
    if (this.ball.y < 10) {
      this.ball.setVelocityX(0);
    }
  },

  resetPaddle: function() {
    clearTimeout(this.paddleTimer);
    this.paddle.x = -200;
    this.caught = false;
  },

  hit: function(ball, paddle) {
    ball.body.moves = false;
    this.caught = true;
  },

  resetBall: function(ball) {
    this.ball.setVelocity(0, -200);
    this.ball.setPosition(gameWidth / 2, 100);
  }
});

var config = {
  type: Phaser.AUTO,
  width: gameWidth,
  height: gameHeight,
  physics: {
    default: "arcade",
    arcade: {
      debug: false,
      gravity: { y: 0 }
    }
  },
  scene: [RealPong]
};

var game = new Phaser.Game(config);
