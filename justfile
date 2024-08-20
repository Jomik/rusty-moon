default_target := "aarch64-unknown-linux-gnu"
project := "rusty-moon"

build target=(default_target):
  cargo build --release --target={{target}}

deploy host target=(default_target): (build target)
  ssh {{host}} 'systemctl stop {{project}}'
  scp ./target/{{target}}/release/{{project}} {{host}}:/home/pi/{{project}}/
  ssh {{host}} 'systemctl start {{project}}'
