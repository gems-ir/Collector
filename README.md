# Collector

This tool can collect different artifact on running system.

Like a kape but faster, more secure and open source in rust 🦀.

## 🏗️ Build Poject

You need to install [rust](https://www.rust-lang.org/fr/tools/install) on you computer.

You can use this following command to run the project for test:

```bash
cargo run --bin collector_cli -- -h
```

Or build in production mode:

```bash
cargo build --release --bin collector_cli
```

### Build packer

If you want to create a binary with pre-configuration, fill with your settings inside the "collector_packer_config.json" file.
Ater that, run this following command:

```bash
cargo build --bin collector_packer --release
```

### Build under Linux

It's able to build rust project under linux to windows.
To do that execute the following command 
```bash
apt-get install gcc-mingw-w64-x86-64 -y
apt-get install gcc -y
apt-get install build-essential -y
rustup target add x86_64-pc-windows-gnu
```
after that you can build the project for example:
```bash
cargo build --target x86_64-pc-windows-gnu --bin collector_cli --release
```

## Run collector

The project is build to easy to run.
You can just start the binary and the process go to start.

## 🆘 Help command

```bash
This tool was an artifact collector fast and secure. It can collect low level files.

Usage: collector_cli.exe [OPTIONS] [COMMAND]

Commands:
  ressources  Ressource list options
  help        Print this message or the help of the given subcommand(s)

Options:
  -s, --source <SOURCE>
          The source of collecting artifact [default: C:\]
  -d, --destination <DESTINATION>
          The destination of collecting artifact [default: .\out\]
  -r, --ressources <RESSOURCES>
          Ressources selection. You can list with "ressources" command. Exemple: MFT,Prefetch,EVTX [default: All]
  -p, --path-ressources <PATH_RESSOURCES>
          Path to artifact resources [default: .\Ressources\]
      --zip
          Zip the output directory
      --pass <PASS>
          Set zip password
      --vss
          Collect from vss. (longer)
      --log
          Print log output in terminal. (longer)
  -v, --verbose
          Verbose log
  -h, --help
          Print help
  -V, --version
          Print version
```

## 👨‍💻 Features

- [ ] Collect file in low level
- [X] VSS (Collect from volume shadow copy)
- [X] Add ZIP password
- [X] Emebed config file and ressources to execute in click and lauch binary.
- [ ] Adaptive collect
- [ ] GUI