# DeepFinder

DeepFinder is a powerful Rust tool that helps you to find duplicate files in a directory and its subdirectories.<br>
It can also find files with the same content with different names in different subdirectories.

This software has been developed to be used especially for a CLI use.
However, DeepFinder has the ability to save the output in different formats like JSON, CSV or XML that can be used in other software


[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://img.shields.io/badge/License-MIT-blue.svg)
[![unsafe forbidden](https://img.shields.io/badge/unsafe-forbidden-success.svg)](https://github.com/rust-secure-code/safety-dance/)
![GitHub release](https://img.shields.io/github/v/release/XenorInspire/DeepFinder)


## Features

- [ ] Find duplicate files in a directory and its subdirectories with the same name or not
- [ ] Generate a report in JSON, CSV or XML format
- [ ] Display the progress of the search
- [ ] Can be used in a script or integrated into another software


## Installation


### Install the packaged version

Download the latest version of DeepFinder from [the release page](https://github.com/XenorInspire/DeepFinder/releases) corresponding to your operating system and your CPU architecture.

#### For Debian-based systems :

```
sudo apt install ./deepfinder_<version>_<arch>.deb
```

#### For RedHat-based systems :

```
sudo dnf install ./deepfinder_<version>_<arch>.rpm
```

#### For Windows :

Just download the .exe file and execute it. You can also add the path to the environment variables to use it in the command line directly without specifying the full path.

### Install from the source code

First, if you don't have rustup installed, you can install it by following the instructions on the official website : [rustup.rs](https://rustup.rs/)

#### Choose a directory and clone the repository :  
```
git clone https://github.com/XenorInspire/DeepFinder.git
```
Move in the directory :  
```
cd DeepFinder/
```
#### Compile the project :

```
cargo build --release
```

The binary will be in the 'target/release/' directory.

## Use DeepFinder

```
deepfinder <path> [options]
```

Use the '-h' or '--help' argument to display the help menu :

```
$ deepfinder --help

Usage: deepfinder <path> [options]
Options:
  -n, --name                            Find the duplicates by their name.
                                        Selected by default if both -n and -a arguments are not specified.
  -a, --hash-algorithm                  Find the duplicates from the hash.
                                        It can be used to compare the content of the files.
                                        You can choose between: md5, sha1, sha224, sha256, sha384, sha512,
                                        sha3-224, sha3-256, sha3-384, sha3-512, blake2b-512, blake2s-256 and whirlpool.
  -f, --hidden-files                    Enable search for hidden files.
  -c <path>, --csv-display              Export the results to stdin in a CSV format.
  -C <path>, --csv-output <path>        Export the results in a CSV file.
  -j <path>, --json-display             Export the results to stdin in a JSON format.
  -J <path>, --json-output <path>       Export the results in a JSON file.
  -x <path>, --xml-display              Export the results to stdin in a XML format.
  -X <path>, --xml-output <path>        Export the results in a XML file.
  -v, --version                         Display the version of DeepFinder.
  -h, --help                            Display this help message.

```


# Licence

This application is licensed under the MIT License. See the [LICENSE](LICENSE) file for more information.
