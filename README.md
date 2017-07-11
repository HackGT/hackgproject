# hackgproject [![Build Status](https://travis-ci.org/HackGT/hackgproject.svg?branch=master)](https://travis-ci.org/HackGT/hackgproject)
A CLI to create and manage projects that work with our infra.

```bash
hackgproject 0.1.12
Michael Eden <themichaeleden@gmail.com>

USAGE:
    hackgproject [SUBCOMMAND]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    help    Prints this message or the help of the given subcommand(s)
    init    Update/Create HackGT boilerplate
    test    Test the travis-ci build and build your docker image.
```

## Install!

This package comes as a single binary with no dependencies for osx/linux
[here](https://github.com/HackGT/hackgproject/releases/latest).

Throw this binary in `/usr/bin/` or similar so you can run it from anywhere.

## Create a new project!

### Web Services

Let's say we want to make a badging system for HackGT, we'll call our
app `badger` so we can have a cute mascot in the future. Just run:

```bash
$ hackgproject init badger

"badger" does not exist, creating it.
Initialized empty Git repository in ~/hackgt/hackgproject/badger/.git/
Writing '.travis.d/build.sh'.
Writing '.travis.yml'.
Writing '.gitignore'.
Writing 'LICENSE'.
Writing 'README.md'.

┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓
┃ You're almost up and running! Just a few more steps: ┃
┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛

1. Create this repo on GitHub: https://github.com/HackGT/
2. Hit 'Restart Build' to get your project set up with all of
HackGT's infra: https://travis-ci.org/HackGT/travis-secrets-setter
```

This sets up all the stuff you'll need for a local HackGT repo.

The last step is hitting `Restart Build`
[here](https://travis-ci.org/HackGT/travis-secrets-setter).
This is a travis job that integrates your new repo into our infra,
you'll get automated testing and deployment under `badger.dev.hack.gt`!

Have fun and happy hacking!

### Static Websites

You can build static websites too!

```bash
$ hackgproject init --static badger

"badger" does not exist, creating it.
Initialized empty Git repository in /home/michaeleden/cde/hackgt/hackgproject/badger/.git/
Creating a static HTML project!
Writing '.travis.d/build.sh'.
Writing '.travis.yml'.
Writing '.gitignore'.
Writing 'LICENSE'.
Writing 'README.md'.
Writing 'CNAME'.
Writing 'index.html'.
Switched to a new branch 'gh-pages'

Just push and go to https://badger.static.hack.gt !
```

## Run your project!

If you want to test how your project will be tested and built when uploaded,
run:

```bash
hackgproject test
```

If this passes, so should your cloud build. Easy! Run your project after with:

```bash
docker run -it badger # or the name of your repo
```

