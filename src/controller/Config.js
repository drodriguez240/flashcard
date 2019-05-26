const electron = require("electron");
const path = require("path");
const fs = require("fs");

const isDev = process.env.NODE_ENV === "development";
const userDataPath = isDev
  ? path.join((electron.app || electron.remote.app).getPath("userData"), "dev")
  : path.join(
      (electron.app || electron.remote.app).getPath("userData"),
      "user"
    );

if (!fs.existsSync(userDataPath)) {
  fs.mkdirSync(userDataPath);
}

const imagesPath = path.join(userDataPath, "images");

if (!fs.existsSync(imagesPath)) {
  fs.mkdirSync(imagesPath);
}

const backupsPath = path.join(userDataPath, "backups");

if (!fs.existsSync(backupsPath)) {
  fs.mkdirSync(backupsPath);
}

const defaultConfig = {
  windowBounds: {
    width: 800,
    height: 600
  },
  backup: path.join(userDataPath, "backup.zip")
};

function getConfigFile() {
  return path.join(userDataPath, "config.json");
}

function parseConfigFile() {
  try {
    return JSON.parse(fs.readFileSync(getConfigFile()));
  } catch (error) {
    return defaultConfig;
  }
}

let config = parseConfigFile();

function writeConfigFile() {
  fs.writeFileSync(getConfigFile(), JSON.stringify(config));
}

class Config {
  // TODO - Main and renderer process both create an instance. This should not happen.
  get(key) {
    config = parseConfigFile();
    return config[key];
  }

  set(key, val) {
    config[key] = val;
    writeConfigFile();
  }

  getUserDataPath() {
    return userDataPath;
  }

  getImagesPath() {
    return imagesPath;
  }

  getBackupsPath() {
    return backupsPath;
  }

  isDev() {
    return isDev;
  }
}

export default new Config();
