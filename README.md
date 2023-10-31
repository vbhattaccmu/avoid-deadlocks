# Deadlock Avoidance Service (DAS)

DAS addresses the challenges of deadlock resolution and collision avoidance among agents or robots navigating in an environment. This is achieved by implementing a centralized monitoring hub that actively listens to and communicates with the agents, instructing them to either stop or resume their movements as needed.

RabbitMQ has been used for communication between the hub and the agents, and a REST API has been designed to report the states of the agents from the hub.

## Assumptions

1. All the agents are rectangular shaped and are moving on a predefined path.

2. Collision is defined as the geometrical intersection between two rectangles.

3. Following (1), since the paths are predefined, there are only two possible states/control actions of a robot: `Stop` or `Resume`.

# Demo
A demo is available [here](https://www.loom.com/share/8fa5027381eb4898ba3899ee1f99351d?sid=7f160c08-31ae-4960-91c4-0a1485c40439)

## Technical Goals

The goal of this project is to showcase how:

- Servers can communicate with each other over message queues in Rust.

- REST APIs can be used in addition to in-memory databases to monitor the states of agents participating in the system.

## Project Directory

DAS consists of the following crates:

- `monitor`: A centralized monitoring service (or the hub) that accumulates states from agents every 10 milliseconds and sends back states to the robot with an objective of collision avoidance/deadlock resolution over RabbitMQ. The monitor also supports REST APIs for reading the current state of all robots in the system.

- `robot`: A robot is an agent that sends/receives states to/from the hub through its own message queue and moves along its predefined route.

- `execution_scripts`: A set of scripts to set up RabbitMQ server, robot, and collision monitor service containers.

## Test Environment Configuration

- Users are required to install Docker and Docker Compose (v2) on their machines. See [Install Docker Engine on Ubuntu](https://docs.docker.com/engine/install/ubuntu/) and [Install the Compose plugin](https://docs.docker.com/compose/install/linux/).

- To install Rust, you will need the following commands:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
rustup default 1.66
```

## Setup and start the system

1. Navigate to execution_scripts directory.

2. Run the bash file to build and start docker containers
```bash
cd execution_scripts
sudo ./run.sh
```
This will set up all the agents in the network and the RabbitMQ cluster.

4. Clean up the docker containers
```bash
sudo ./cleanup.sh
```
In case there is an error while running docker compose due to an incompatible Ubuntu version, please replace the line
```
docker compose up
```
with 
```
docker-compose up
```

Once the system is set up, you can query the Collision Monitor REST API to verify if it's working:

```
curl -X GET 'http://localhost:9000/state/robot1'
```
For responses, please check the API documentation section below.

## Logs 

The crates have logs enabled. They can be inspected from their respective directories set in config.toml.

## Run unit tests 
The implementation defines unit tests covering different collission scenarios among agents.

To run it navigate to `monitor` crate and run 
```
cargo test
```

## API Documentation 
The monitoring service comes with a REST API endpoint to read current state of an agent to provide ease of access of the results in the system.
The REST API to read agent states is provided by the following endpoint. The listening port is conigurable.

GET /state/<device_id>

Response : JSON body of current state of the Robot designated by Robot1 

Example Call:

```
curl -X GET 'http://localhost:9000/state/robot1
```

A successful 200 Response:

```
{
    "x": 13.0,
    "y": 12.3,
    "theta": 1.57,
    "loaded": false,
    "timestamp": 1657453020000,
    "path": [
        {
            "x": 10.0,
            "y": 12.3,
            "theta": 1.57
        },
        {
            "x": 11.0,
            "y": 12.3,
            "theta": 1.57
        },
        {
            "x": 12.0,
            "y": 12.3,
            "theta": 1.57
        },
        {
            "x": 13.0,
            "y": 12.3,
            "theta": 1.57
        }
    ],
    "device_id": "robot1",
    "state": "Resume",
    "battery_level": 87.2
}

```

## Error Codes
The following are the error codes emitted by the hub API in case there are any errors in communication.

| Error Code   |    Error Type          | Description                               |
|:------------:|:----------------------:|-------------------------------------------|
|     2101     |  INCORRECT_INPUT       | Represents an incorrect input endpoint URL.      |
|     2102     |  INCORRECT_DB_RECORD   | Indicates an error occurred when querying a record. If the database is empty, the query will return this error code. |
|     2103     |  DESERIALIZATION_FAILURE | Indicates a failure in serde deserialization of a message in the hub during an endpoint call. |


## Notes

### Service configuration
The services written are configuraton heavy. They use .toml and their definitions can be found in config.rs files of individual crates.

### Number of Agents
Currently the number of agents used in this crate is limited to 4 and if the number is changed to 1000 or even more the config.toml file and init_states.json has to be generated by some program.

### OS
The system is known to work in Ubuntu 22.04.For other platforms, DAS hasn't been tested yet.


