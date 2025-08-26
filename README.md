# 🎮 Deemak - Text Adventure Game Engine 🎲

<div align="center">
  <img src="assets/deemak_logo.png" alt="Deemak logo" width="50%">
</div>

> [!NOTE]
> Deemak is still under development. It is yet not in a packageable state. You can try instructions below to run it locally and test it out.
> For any issues, please create an issue. We would love your feedback and contributions.

Deemak is a Text Based Adventure Game Engine written in Rust. It is inspired by MIT's [Terminus](https://web.mit.edu/mprat/Public/web/Terminus/Web/main.html). It is designed to be a simple, flexible, and extensible engine for creating text-based adventure games and enjoying them in Deemak GUI or Web.

<details>
<summary>Why the name "Deemak"? 🪲</summary>

Well, we wanted to name it something related to `Term` for terminal, so we though of `Termite`. During the initial brainstorming, we thought Termite will be one game and its Game Engine will be called `Deemak` (which means Termite in Hindi). But then we got confused what `Termite` would even be? So we decided to keep the name `Deemak` for the Game Engine itself! Pretty fancy, right?

</details>

Deemak uses a `Sekai` (which means World in Japanese) file, of format `.deemak`(with proper metadata set) to load the game. Use deemak CLI to create and play your own games!

---

## ✨ Features

### 🎯 For Players

For the users who want to play games created using Deemak, you can:

- 🎮 Play text-based adventure games in Deemak GUI or Web.
- 🚶‍♀️ Walk around different locations, interact with objects, and solve puzzles.
- 🔓 Unlock new levels, open chests and find hidden secrets within sekai.

### 🛠️ For Developers

For Developers hoping to create their own games using Deemak, our game engine:

- 🔄 Takes your file directory and automatically converts it into a playable game.
- 🔐 Provides a simple single command to lock levels and chests with your passwords within the game.
- 📦 One step command line instruction to export to a playable `.deemak` for the users, with your set password protecting users from accessing the files directly.
- 🛡️ Provides strong security for protection of hidden passwords for locked chests and levels, to prevent hacking and cheating.

---

## 📚 How to use Deemak (For Developers)

> ⚠️ Since Deemak is still under development, Only Developing instructions are provided.

1. Clone the repository

```bash
git clone https://github.com/databasedIISc/deemak.git
cd deemak
```

> 💡 **Tip:** For any information, you can try `deemak --help` or `deemak <command> --help` for specific command.

2. Run the following command to start terminal version -

Note that we need to pass the world directory as the first argument.

```bash
cargo run sekai.deemak dev play
cargo run sekai.deemak dev play --web # to run in web
```

- 🌐 For web, open your browser and navigate to: http://localhost:8000
- ⚙️ To change the port, you go to .env file and change the `BACKEND_PORT` value (default BACKEND_PORT=8001).

Dev Mode automatically runs in Debug mode providing you with detailed logs and functionality to create and test your game.

3. To create a new game, run the following command:

```bash
cargo run sekai dev create # sekai is a directory you created
```

Check out more functionality using `--help` flag.

---

## 🧩 How Deemak Works

Here is a brief overview of how Deemak works:

### 1. `.deemak` File Format 📄

This file is created after special encryption and compression of the game directory. The header of the binary should contain `dbdeemak` for verification. Also a Developer set password is required to initially lock the game. It is subjected to change(for now) since the password is not stored within the file.

### 2. Game Working 🎮

The game calls an file or directory as `object`. Each object contains its metadata. All the folders must contain a `.dirinfo` folder containing all hidden metadata like `info.json` and passwords. This is automatically created and You need not manually create it(Unless you want to edit the description of the directory level).

`info.json` is the main file contains the description of the directory, location, what all objects are present in the directory with their permission bits and other metadata.

When you run the game, it loads the `sekai` directory and starts from the root directory. You can navigate through the directories, interact with objects, and unlock new levels or chests using the commands provided, all handling of the `.dirinfo` is automatically handled by Deemak.

### 3. Security and Unlocking 🔒

You can lock certain directories or files using a password. This is done using strong encryption algorithms to ensure that the passwords are not easily guessable or hackable.

> 🚧 The unlocking feature is still under development.

We implement Two separate locking mechanisms - Level Locking and Chest Locking. Level locking is higher security and it meant for directories, while chest locking is meant for both files and directories. The intuition of which is use when is as the name suggests, but you can think about it as **"You are entering a locked door vs You are opening a sub task of opening a locked treasure Chest"**.

### 4. During Execution of the Game ⚡

The game is run in a temporary location hidden from the User(with no read access to a third user). The deemak file in opened and extracted in the temporary location and executed with all functionalities there. When you exit, your progress must be saved by the User, otherwise it will replay from the last saved location(or START if no save is present).

If you are a developer(in Dev Mode), too avoid pain going to the temporary location, the temporary location is set to your CWD. You can provide both a directory or a `.deemak` file in Dev Mode. You will also get detailed logs of what is happening in the background for debugging purposes. Since you are the developer, you know the password, so use `--dev` flag to provide the password and enjoy the dev mode functionalities. Any 3rd User playing the game will not have access to these functionalities because of your private password.

---

## 📝 TODO's

- [ ] Debug input folder/deemak format file for developers, with appropriate location of run.
- [ ] Fix unlocking feature.
- [ ] Release a stable version.
- [ ] Create User Info and data for saving game progress.

> 🐛 For any issues and suggestions, please create an issue in our GitHub repository.

---

## 👏 Acknowledgements

This project was developed under Delta-25 initiative by UnderGraduate [Databased](https://databased.iisc.ac.in) Club at IISc, Bangalore, India. This was done as project of Rust Learning Group. The software is developed by:

- [Anirudh Gupta](https://github.com/AnirudhG07) 👨‍💻
- [Palak Raisinghani](https://github.com/Pal-R-S) 👩‍💻
- [Pritesh Jogirdhar](https://github.com/Pritesh299) 👨‍💻
- [Nikhil Jamuda](https://github.com/nimx89) 👨‍💻
- [Aditya Thakkar](https://github.com/Aditya-A-Thakkar) 👨‍💻
- Thanks to [Aditya Arsh](https://github.com/chocabloc) for logo and brainstorming. 🎨
