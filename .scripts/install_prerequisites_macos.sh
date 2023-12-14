#!/bin/bash

# check if rust is installed
if ! command -v rustc &> /dev/null
then
    echo "Rust is not installed. Installing now..."

    # Install Rust
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

    # Add Rust to the PATH
    source $HOME/.cargo/env

    echo "Rust installation complete."
else
    echo "Rust is already installed. Version:"
    rustc --version
fi

# check if homebrew is installed
if ! command -v brew &> /dev/null
then
    echo "Homebrew not found. Installing now..."

    # Install Homebrew
    /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

    echo "Homebrew installation complete."
else
    echo "Homebrew is already installed."
fi

# install both java and kafka
brew install java
brew install kafka