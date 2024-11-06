<div align="center">
  <h1>MANNager</h1>
  <img style="width: 256px" 
    src="https://github.com/user-attachments/assets/561f1a01-9f2a-4bf3-bc10-18ebd21db2da"
    />
  <h4>A functional & easy-to-use Server Manager for Source games.</h4>
  <p style="margin-bottom: 0.5ex;">
    <img
      src="https://img.shields.io/github/downloads/tsuza/mannager-source/total?color=ff69b4"
      />
    <a href="https://github.com/tsuza/mannager-source/blob/main/LICENSE.txt">
    <img alt="GitHub license" src="https://img.shields.io/github/license/tsuza/mannager-source?color=ff69b4">
    </a>
    <a href="https://github.com/tsuza/mannager-source/stargazers">
    <img alt="GitHub stars" src="https://img.shields.io/github/stars/tsuza/mannager-source?color=yellow&label=Project%20Stars">
    </a>
    <a href="https://github.com/tsuza/mannager-source/issues">
    <img alt="GitHub issues" src="https://img.shields.io/github/issues/tsuza/mannager-source?color=brightgreen&label=issues">
    </a>
    <a href="https://github.com/tsuza/mannager-source/network">
    <img alt="GitHub forks" src="https://img.shields.io/github/forks/tsuza/mannager-source?color=9cf&label=forks">
    </a>
    <img
      src="https://img.shields.io/github/workflow/status/tsuza/mannager-source/release.yml?color=9cf&label=build"
      />
  </p>
  
  <a href="#installation">Installation</a> •
  <a href="#configuration">Configuration</a> •
  <a href="#features">Features</a> •
  <a href="#to-do">To-Do list</a> •
  <a href="#contributors">Contributors</a> •
  <a href="https://tsuza.github.io/mannager-source/">Website</a>

</div>

![overview](https://github.com/user-attachments/assets/a1ea7ce7-4d97-4457-97d4-6e59258af498)

## Features
- Easily download, update & setup a Source game's dedicated server from a wide range of selections.
  - Team Fortress 2
  - Half Life 2: Deathmatch
  - No More Room In Hell
  - Left 4 Dead 1
  - Left 4 Dead 2
- Easily download either the stable or dev branch of Sourcemod ( and Metamod ).
- Automatically port forward your server.
- Native terminal built into the app for server management.

## Installation
Not yet.

## Configuration
Everything server-related that can be edited, can be done through the server creation / editing buttons in-app. If you wish to customize something more, just press the menu ( the gear icon ) on the server entry, press "Open Folder", and do whatever you want.

The server list file, a file that contains all of the servers being tracked by the app, is called `servers_list.toml`, and can reside in one of the paths, based on priority:
- In the same folder of the executable.
- ( DEFAULT ) `$HOME/.config/mannager-source` on Linux & `%APPDATA%/Roaming/mannager-source` on Windows

## To-Do
The top priority for now is:
- [ ] Finalize the custom title bar for the terminal.
- [ ] Finalize the support for Source 2 games ( CS2, Deadlock ).
- [x] Finalize Windows support ( and its Workflow ).
- [ ] Add support for configuring convars and admins within the app.

For the full list, check out [the issue](https://github.com/tsuza/mannager-source/issues/1).

## Contributors
<a href="https://github.com/tsuza/mannager-source/graphs/contributors">
  <img src="https://contrib.rocks/image?repo=tsuza/mannager-source" alt="contrib.rocks image" />
</a>
