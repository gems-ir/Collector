# Collector

This tool can collect different artifact on running system.

Like a Kape but faster, more secure and open source in rust 🦀.

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

### Resources

You can download the collection resources here: [https://github.com/gems-ir/Resources.git](https://github.com/gems-ir/Resources.git) 

### Build packer

If you want to create a binary with pre-configuration, fill with your settings inside the "collector_packer_config.json" file.
Then, run the following command:

```bash
cd Collector
git clone https://github.com/gems-ir/Resources.git
cargo build --bin collector_packer --release
```

### Build under Linux

It is possible to build the rust project under Linux for Windows OS.
To do this, run the following command
```bash
apt-get install gcc-mingw-w64-x86-64 -y
apt-get install gcc -y
apt-get install build-essential -y
rustup target add x86_64-pc-windows-gnu
```
after whcih you can build the project for example:
```bash
cargo build --target x86_64-pc-windows-gnu --bin collector_cli --release
```

## Run collector

The project is designed to be easy to run.
You can simply launch the binary and the process will start.

## 🆘 Help command

```bash
This tool was an artefact collector fast and secure. It can collect low level files.

Usage: collector_cli.exe [OPTIONS] [COMMAND]

Commands:
  resources  Resource list options
  help       Print this message or the help of the given subcommand(s)

Options:
  -s, --source <SOURCE>
          The source path of collecting artifact [default: C:\]
  -d, --destination <DESTINATION>
          The destination path of collecting artifact [default: .\out\]
  -r, --resources <RESOURCES>
          Resources selection. You can list with "resources" command. Exemple: MFT,Prefetch,EVTX [default: All]
  -p, --path-resources <PATH_RESOURCES>
          Path to artifact resources [default: .\Resources\]
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

- [X] Low-level file collection
- [X] VSS (Collect from volume shadow copy)
- [X] Add ZIP password
- [X] Emebed config file and resources to execute in click and lauch binary.
- [ ] Adaptive collect
- [ ] GUI