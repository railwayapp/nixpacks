# Haskell Support through Stack

Haskell with Stack is detected if your project has a `package.yaml` file and any `.hs` source files.

**Install**:

```sh
sudo apt-get update && sudo apt-get install -y libgmp-dev gcc binutils make && stack setup
```

**Build**:

```sh
stack build
```

**Run**:

Assumes that `package.yaml` has a list of `executables`.

```sh
stack run $(head packageYaml.executables)
```