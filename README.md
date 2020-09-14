[![LXD](https://linuxcontainers.org/static/img/containers.png)](https://linuxcontainers.org/lxd)
# rlxc
Simple side-project to implement a Rust binary using
[LXC](https://github.com/lxc/lxc) to run containers.
Currently covers:

- `lxc-attach` -> `rlxc exec <name> <command>...`
* `lxc-start` -> `rlxc start <name>`
- `lxc-execute -> rlxc start <name> [command]`
- `lxc-stop` -> `rlxc stop`
- `lxc-ls` -> `rlxc list`
- `lxc-console` -> `rlxc login <name>`

as well as:

- `rlxc help`
- `rlxc version`

# LXC
For information about LXC see [here](https://github.com/lxc/lxc).
