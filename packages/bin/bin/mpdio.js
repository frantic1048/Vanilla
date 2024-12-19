#!/usr/bin/env node

const net = require("node:net");
const { exec } = require("node:child_process");
const process = require("node:process");

const host = "localhost";
const port = 6600;

const _prompt = "> ";

const debuginfo = (info) => process.stderr.write(`--- ${info}\n`);

const idlePattern = /^changed:/;
const statePattern = /^state: (\w+)/m;

// TODO: reconnecting logic, or just let systemd do the restart?
const client = net.createConnection(port, host, () => {
  debuginfo("doc: https://mpd.readthedocs.io/en/latest/protocol.html");
  debuginfo(`connected to mpd server: ${host}:${port}`);
  client.setEncoding("utf8");
  // process.stdin.pipe(client);
  client.write("status\n");
});

client.on("data", (data) => {
  // process.stdout.write(`${data}${prompt}`);
  // process.stdout.write(data);

  if (idlePattern.test(data)) {
    client.write("status\n");
  } else if (statePattern.test(data)) {
    const state = statePattern.exec(data)[1];
    debuginfo(`state: ${state}`);
    if (state === "play") {
      // TODO: check profile before setting
      // MEMO: pactl list short cards
      exec(
        "pactl set-card-profile alsa_card.usb-TEAC_Corporation_UD-505-00 off"
      );
    } else {
      exec(
        "pactl set-card-profile alsa_card.usb-TEAC_Corporation_UD-505-00 pro-audio"
      );
    }
    client.write("idle player\n");
  }
});

client.on("end", () => {
  console.debug("disconnected from mpd server");
});

process.on("SIGINT", () => {
  client.end();
});
