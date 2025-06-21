# basic greeter I made with egui

## Installation
* if you are nixos and using flakes see Nixos Installation
* install cargo
* install and configure greetd
```bash
cargo install --git https://github.com/astaugaard/egui-greeter.git
```

create config file at /etc/greetd/egui-greeter.json (example below)
```json
{
  "default_session_name": "Niri",
  "default_session_command": "niri-session",
  "user": "estaugaard"
}
```
run in cage from greetd. (use paths to where it is installed for you, or make sure it is on the path when running this command)
```
/bin/cage -s -- /home/<USERNAME>/.cargo/bin/egui-greeter
```
thats it!!!

## Nixos Installation
```nix
# flake input
egui-greeter = {
  url = "github:astaugaard/egui-greeter/main";
  inputs.nixpkgs.follows = "nixpkgs";
};

# include module
outputs = inputs@{ ... }:
  ...
  nixosConfigurations.<host name> = lib.nixosSystem {
    modules = [
      inputs.egui-greeter.nixosModules
      ...
    ];
    ...
  };
  ...

# enable and configure egui-greeter
...
programs.egui-greeter = {
  enable = true;

  default_session_name = "Niri";
  default_session_command = "niri-session";
  user = config.mysystem.user;
};
...
```

## screenshot
![image](https://github.com/user-attachments/assets/d3706938-7967-416f-8031-e6277eb2ddab)
there isn't currently any configuration of how it looks since I made it for myself, and my own use for fun. Though if you want the feature to be able to configure something let me know (I'm happy to help).


