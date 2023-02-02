# Quests

The Quests System is an important feature that facilitates users to explore the world, unlock achievements and potentially receive rewards. A quest is a series of steps or tasks that a user has to complete. Each step or task has an acceptance criteria to consider it as done. A quest designer has to define the steps and the order or the path to the end, so the quest is finished when those steps are completed.

## Architecture
![Quests](docs/architecture.svg)

## Use the project

### Requirements
- You should have `docker-compose` installed on your PC.

### Quests Server
In order to run the server API, you should use the next `make` command from the project's root:
```shell
    make run-server
```

In order to run the server API in dev mode (adds the watch mode), you should use the below command:
```shell
    make run-devserver
```

### Testing
In order to run all project's tests, you should run the next command:
```shell
    make tests
```

To run specfic test, there are some useful commands:
- Server:
```shell
    make test-server
```
- Quest Database:
```shell
    make test-db
```
- Quest Definitions:
```shell
    make test-definitions
```