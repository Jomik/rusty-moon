target := "aarch64-unknown-linux-gnu"
project := "rusty-moon"
host := "tv06.local"

build-for-pi:
  cargo build --release --target={{target}}

deploy-pi: build-for-pi
  ssh {{host}} 'systemctl stop {{project}}'
  scp ./target/{{target}}/release/{{project}} {{host}}:/home/pi/{{project}}/
  ssh {{host}} 'systemctl start {{project}}'
