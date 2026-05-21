<div align="center">
  <h1>MANNager</h1>
  <img style="width: 256px" 
    src="https://github.com/user-attachments/assets/561f1a01-9f2a-4bf3-bc10-18ebd21db2da"
    />
  <h4>Easily manage, install, configure, update, and launch Source Engine dedicated servers.</h4>
  <p style="margin-bottom: 0.5ex;">
    <img
      src="https://img.shields.io/github/downloads/tsuza/mannager-source/total?color=ff69b4"
      />
    <a href="https://github.com/tsuza/mannager-source/blob/main/LICENSE.txt">
    <img alt="GitHub license" src="https://img.shields.io/github/license/tsuza/mannager-source">
    </a>
    <a href="https://github.com/tsuza/mannager-source/stargazers">
    <img alt="GitHub stars" src="https://img.shields.io/github/stars/tsuza/mannager-source?color=yellow&label=Project%20Stars">
    </a>
    <img
      src="https://img.shields.io/github/actions/workflow/status/tsuza/mannager-source/release.yml"
      />
    <a href="https://github.com/iced-rs/iced">
    <img alt="GitHub stars" src="https://iced.rs/badge.svg">
    </a>

  </p>
  
  <a href="#installation">Installation</a> •
  <a href="#configuration">Configuration</a> •
  <a href="#features">Features</a> •
  <a href="#contributors">Contributors</a> •
  <a href="https://apps.tsuza.net/mannager">Website</a>

</div>

<img width="3468" height="1307" alt="overview(1)" src="https://github.com/user-attachments/assets/1916610d-f604-4b68-b52c-ccb888e84ff6" />

# Features
## Supported Games
- Team Fortress 2
- Half Life 2: Deathmatch
- Left 4 Dead 1
- Left 4 Dead 2
- Counter-Strike: Source
- Counter-Strike: Global Offensive
- Counter-Strike 2
- Deadlock
- No More Room In Hell

Additional games can be requred through Github issues.

## Server Management
- Install and update dedicated server directly through the app
- Easily change important things such as the map, max players, hostname, SDR / Port Forwarding, etc...
- Native terminal
- Quick access to servers folders

## Modding Support
- Install stable or development builds of SourceMod.
- ...a plugin downloader is planned!

# Installation
1. Go to the [latest release](https://github.com/tsuza/mannager-source/releases/latest).
2. If you're on Windows, pick the msi installer. If you're on Linux, install the AppImage ( use GearLever to easily manage AppImages if you don't already ).
3. Enjoy!

# Configuration
Everything server-related that can be edited, can be done through the server creation / editing buttons in-app. If you wish to customize something more, just press the menu ( the gear icon ) on the server entry, press "Open Folder", and do whatever you want.

The server list file, a file that contains all of the servers being tracked by the app, is called `servers_list.toml`, and can reside in one of the paths, based on priority:
- In the same folder of the executable.
- ( DEFAULT ) `$HOME/.config/mannager-source` on Linux & `%APPDATA%/Roaming/mannager-source` on Windows

# Building
If you wish to build it yourself, just run `cargo build --release`. If you need to test it, `cargo run`.

# Packaging
This project utilizes [Velopack](https://docs.velopack.io/) for packaging.

## Install Velopack
Requires the `.NTE SDK`:

```bash
dotnet tool update -g vpk
```

## Package the Application
```bash
cargo build --release

cp target/release/mannager publish/mannager

vpk pack \
  -u net.tsuza.mannager \
  -v {version} \
  -p ./publish \
  -o ./Releases \
  -e mannager
```

For additional packaging options, see the worfklow file.

# Contributors
<a href="https://github.com/tsuza/mannager-source/graphs/contributors">
  <img src="https://contrib.rocks/image?repo=tsuza/mannager-source" alt="contrib.rocks image" />
</a>

