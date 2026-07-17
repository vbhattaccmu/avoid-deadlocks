
# Avoid Deadlocks

A Rust-based coordination prototype for collision avoidance among agents following predefined paths.

A centralized monitor receives agent state over RabbitMQ, detects geometric intersections, and instructs agents to **stop** or **resume**. A REST API exposes the monitor's current view of each agent.

## Architecture

```text
Robot 1 ----\
Robot 2 -----\       RabbitMQ       +-------------------+
Robot 3 ------+--------------------->| Collision monitor |
Robot 4 -----/                       +---------+---------+
                                                |
                                                v
                                          REST state API
```

## Components

- `monitor` — ingests state updates, detects collisions, and publishes control actions
- `robot` — simulates an agent moving on a predefined route
- `execution_scripts` — Docker and RabbitMQ setup/cleanup scripts

## Assumptions

- Agents are represented as rectangles.
- Paths are predetermined.
- A collision is a geometric intersection between two agent rectangles.
- Supported control actions are `Stop` and `Resume`.

## Run

Prerequisites:

- Rust
- Docker
- Docker Compose v2

```bash
cd execution_scripts
chmod +x run.sh cleanup.sh
./run.sh
```

Query a robot:

```bash
curl http://localhost:9000/state/robot1
```

Clean up:

```bash
./cleanup.sh
```

## Tests

```bash
cd monitor
cargo test
```

## Current Limitations

- Central monitor is a single coordination point.
- Configuration is currently optimized for four agents.
- Agent generation/configuration should be automated for larger simulations.
- Network delays, stale state, monitor failure, and conflicting control messages require additional handling.
- This is a coordination experiment, not a certified robotics safety controller.

