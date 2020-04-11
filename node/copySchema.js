const fs = require("fs");
const path = require("path");

fs.copyFileSync(
    path.join(path.dirname(__dirname), "playlist.schema.json"),
    path.join(__dirname, "src", "playlist.schema.json")
);
