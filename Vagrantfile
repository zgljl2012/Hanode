# -*- mode: ruby -*-
# vi: set ft=ruby :

Vagrant.configure("2") do |config|
    config.vm.box = "ubuntu/focal64"
    config.vm.synced_folder '.', '/vagrant', disabled: false

    config.vm.provider "virtualbox" do |v|
      v.memory = 8192
      v.cpus = 4
    end

    # Dependencies
    config.vm.provision "shell", inline: <<-SHELL
      set -e -u -o pipefail
      sudo apt-get update -y
      sudo apt install -y git gcc docker.io
      sudo apt-get install -y pkg-config libsystemd-dev libdbus-glib-1-dev build-essential libelf-dev libseccomp-dev
      apt install -y libssl-dev protobuf-compiler
      service docker start
    SHELL

    # Rust
    config.vm.provision "shell", privileged: false, inline: <<-SHELL
      curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
      echo "export PATH=$PATH:$HOME/.cargo/bin" >> ~/.bashrc
    SHELL
  end
