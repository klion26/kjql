# kjql

> A JSON Query Language CLI tool
> the repo is a manually copy from yamafaktory/jql used to learn rust

##  Installation 

```shell
cargo install kjql
```

## Usage
Want to get the version of a NodeJS package.json file?
```shell
kjql package.json version
```

You can chain selectors with `.` and numbers to access children and indexes in arrays.
```shell
kjql package.json devDependencies.react
kjql package.json keywords.3
```

Given the following JSON file:
```shell
{
  ".valid": 1337
}
```
You can access the `.valid` key as follow:
```shell
kjql package.json '".valid"'
```
And get some (limited) help for now.
```shell
kjql --help
```