alias l := list
alias ls := list

# List the available recipes
[default]
list:
    @just --list


# Check workspace or provided package
check target="workspace":
    @[[ "{{target}}" == "workspace" ]] && cargo check --workspace || cargo check -p "{{target}}"

# Check workspace
test target="workspace":
    @[[ "{{target}}" == "workspace" ]] && cargo test --workspace || cargo test -p "{{target}}"
