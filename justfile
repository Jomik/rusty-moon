target := "aarch64-unknown-linux-gnu"
project := "rusty-moon"
host := "tv06.local"

build-for-pi:
  cargo build --release --target={{target}}

deploy-pi: build-for-pi
  scp ./target/{{target}}/release/{{project}} {{host}}:/home/pi/{{project}}/
