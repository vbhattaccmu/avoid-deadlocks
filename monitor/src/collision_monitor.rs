use serde_derive::{Deserialize, Serialize};
use std::{collections::HashSet, f64};

use crate::config::CollisionMonitorConfig;

/// [CollisionMonitor] defines the struct for the collision monitoring system.
#[derive(Debug)]
pub(crate) struct CollisionMonitor {
    // current Collision Monitor configuration
    pub config: CollisionMonitorConfig,
}

impl CollisionMonitor {
    /// `new` creates a new instance of `CollisionMonitor`.
    pub(crate) fn new(config: CollisionMonitorConfig) -> Self {
        CollisionMonitor { config }
    }

    /// `trigger_collision_monitor` triggeres the collision detection and deadock detection methods
    /// once all the agents are done
    pub(crate) fn trigger_collision_monitor(
        &self,
        mut robots: Vec<Robot>,
    ) -> Result<Vec<Robot>, String> {
        if robots.len() != self.config.num_agents {
            return Err("Not yet received all agent records".to_string());
        }

        self.update_robot_state(&mut robots);

        Ok(robots)
    }

    /// `update_robot_state` updates states of robots after detecting conflicts and deadlocks.
    pub(crate) fn update_robot_state(&self, robots: &mut [Robot]) {
        let mut conflicts = self.detect_collisions(robots);
        let mut deadlock = !conflicts.is_empty();

        // if conflicts are empty simply update next state and move
        // robot to mext coordinate
        if conflicts.is_empty() {
            for robot in robots.iter_mut() {
                self.update_motion_coordinates(robot);
            }
        }

        while !conflicts.is_empty() && !deadlock {
            // Define the conflict resolution order
            let conflict_order: Vec<usize> = conflicts.iter().map(|&(i, _)| i).collect();

            for &idx in &conflict_order {
                let (first_conflict_idx, second_conflict_idx) = conflicts[idx];

                if robots[first_conflict_idx].state == MotionState::Pause.to_string()
                    || robots[second_conflict_idx].state == MotionState::Pause.to_string()
                {
                    continue;
                }

                let (new_state_i, new_state_j) = self.resolve_collision();

                if new_state_i == MotionState::Pause && new_state_j == MotionState::Pause {
                    deadlock = true;
                    break;
                }

                if new_state_i == MotionState::Resume {
                    self.update_motion_coordinates(&mut robots[first_conflict_idx]);
                }

                if new_state_j == MotionState::Resume {
                    self.update_motion_coordinates(&mut robots[second_conflict_idx]);
                }

                robots[first_conflict_idx].state = new_state_i.to_string();
                robots[second_conflict_idx].state = new_state_j.to_string();
            }

            conflicts = self.detect_collisions(robots);

            if !conflicts.is_empty() {
                self.resolve_deadlock(robots, &conflicts);
            }
        }

        if deadlock {
            for robot in robots {
                robot.state = MotionState::Pause.to_string();
            }
        }
    }

    /// `detect_collisions` detects collission between all robots at current timestamp.
    fn detect_collisions(&self, robots: &[Robot]) -> Vec<(usize, usize)> {
        let mut conflicts: Vec<(usize, usize)> = Vec::new();

        for idx in 0..robots.len() {
            for jdx in (idx + 1)..robots.len() {
                if self.will_collision_occur(&robots[idx], &robots[jdx]) {
                    conflicts.push((idx, jdx));
                }
            }
        }

        conflicts
    }

    /// `resolve_collision` resolves the collision between two robots we assume both agents stop (Pause) to avoid collision
    fn resolve_collision(&self) -> (MotionState, MotionState) {
        (MotionState::Pause, MotionState::Pause)
    }

    /// `resolve_deadlock` resolves deadlocks in case conflicts occur
    fn resolve_deadlock(&self, robots: &mut [Robot], conflicts: &[(usize, usize)]) {
        let mut handled_conflicts: HashSet<(usize, usize)> = HashSet::new();

        for &(first_conflict_idx, second_conflict_idx) in conflicts {
            if handled_conflicts.contains(&(first_conflict_idx, second_conflict_idx)) {
                continue;
            }

            let robot_a = &robots[first_conflict_idx];
            let robot_b = &robots[second_conflict_idx];

            let (new_state_i, new_state_j) = if robot_a.state == MotionState::Pause.to_string() {
                self.update_motion_coordinates(&mut robots[second_conflict_idx]);

                (MotionState::Pause, MotionState::Resume)
            } else if robot_b.state == MotionState::Pause.to_string() {
                self.update_motion_coordinates(&mut robots[first_conflict_idx]);

                (MotionState::Resume, MotionState::Pause)
            } else {
                self.resolve_collision()
            };

            robots[first_conflict_idx].state = new_state_i.to_string();
            robots[second_conflict_idx].state = new_state_j.to_string();

            handled_conflicts.insert((first_conflict_idx, second_conflict_idx));
        }
    }

    /// `update_motion_coordinates` updates the current position if the current state of the robot is set to `Resume`.
    fn update_motion_coordinates(&self, robot: &mut Robot) {
        if robot.state == MotionState::Resume.to_string() {
            if let Some(current_index) = robot
                .path
                .iter()
                .position(|point| point.x == robot.x && point.y == robot.y)
            {
                if let Some(next_point) = robot.path.get(current_index + 1) {
                    robot.x = next_point.x;
                    robot.y = next_point.y;
                }
            }
        }
    }

    /// `will_collision_occur` checks if current robot will collide with others.
    fn will_collision_occur(&self, robot_a: &Robot, robot_b: &Robot) -> bool {
        if robot_a.device_id == robot_b.device_id {
            return false;
        }
        if self.collision_check_helper(robot_a, robot_b) {
            return true;
        }

        false
    }

    /// `collision_check_helper` checks collision between two robots based on their dimension and
    /// respective position in the grid.
    fn collision_check_helper(&self, robot: &Robot, other_robot: &Robot) -> bool {
        let robot_x_min = robot.x - self.config.width / 2.0;
        let robot_x_max = robot.x + self.config.width / 2.0;
        let robot_y_min = robot.y - self.config.height / 2.0;
        let robot_y_max = robot.y + self.config.height / 2.0;

        let other_robot_x_min = other_robot.x - self.config.width / 2.0;
        let other_robot_x_max = other_robot.x + self.config.width / 2.0;
        let other_robot_y_min = other_robot.y - self.config.height / 2.0;
        let other_robot_y_max = other_robot.y + self.config.height / 2.0;

        // adjust the bounding box coordinates based on the robot's rotation
        let (robot_x_min, robot_y_min) =
            self.rotate_bounding_box(robot_x_min, robot_y_min, robot.theta, robot.x, robot.y);
        let (robot_x_max, robot_y_max) =
            self.rotate_bounding_box(robot_x_max, robot_y_max, robot.theta, robot.x, robot.y);

        let (other_robot_x_min, other_robot_y_min) = self.rotate_bounding_box(
            other_robot_x_min,
            other_robot_y_min,
            other_robot.theta,
            other_robot.x,
            other_robot.y,
        );
        let (other_robot_x_max, other_robot_y_max) = self.rotate_bounding_box(
            other_robot_x_max,
            other_robot_y_max,
            other_robot.theta,
            other_robot.x,
            other_robot.y,
        );

        // check if the rotated bounding boxes of the robots intersect
        if robot_x_max < other_robot_x_min || robot_x_min > other_robot_x_max {
            return false;
        }

        if robot_y_max < other_robot_y_min || robot_y_min > other_robot_y_max {
            return false;
        }

        true
    }

    /// `rotate_bounding_box` corrects the point (x, y) around the origin (origin_x, origin_y) by angle `theta`
    fn rotate_bounding_box(
        &self,
        x: f64,
        y: f64,
        theta: f64,
        origin_x: f64,
        origin_y: f64,
    ) -> (f64, f64) {
        let translated_x = x - origin_x;
        let translated_y = y - origin_y;
        let rotated_x = translated_x * theta.cos() - translated_y * theta.sin();
        let rotated_y = translated_x * theta.sin() + translated_y * theta.cos();
        let final_x = rotated_x + origin_x;
        let final_y = rotated_y + origin_y;

        (final_x, final_y)
    }
}

/// [Robot] defines attributes which define the
/// current state of each robot.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Robot {
    /// x-coordinate of the robot
    pub x: f64,
    /// y-coordinate of the robot
    pub y: f64,
    /// angle of inclination to y-axis in radians
    pub theta: f64,
    /// loading status of the robot: true | false
    pub loaded: bool,
    /// current timestamp of the robot
    pub timestamp: i64,
    /// path of the robot
    pub path: Vec<Path>,
    /// device id of the robot
    pub device_id: String,
    /// state of the robot: resume | pending
    pub state: String,
    /// current battery level of the robot
    pub battery_level: f64,
}

/// [Path] defines attributes which define a
/// location of the robot.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Path {
    /// x-coordinate of the robot
    pub x: f64,
    /// y-coordinate of the robot
    pub y: f64,
    /// angle of inclination to y-axis in radians
    pub theta: f64,
}

/// [MotionState] defines current state of
/// motion of the robot.
#[derive(Debug, PartialEq)]
enum MotionState {
    Pause,
    Resume,
}

// impl for converting enums to string
impl MotionState {
    fn to_string(&self) -> String {
        match self {
            MotionState::Pause => "Pause".to_string(),
            MotionState::Resume => "Resume".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collision_monitor_update_robot_state() {
        let robot1 = Robot {
            x: 0.0,
            y: 0.0,
            theta: 0.0,
            loaded: false,
            timestamp: 0,
            path: vec![
                Path {
                    x: 0.0,
                    y: 0.0,
                    theta: 0.0,
                },
                Path {
                    x: 1.0,
                    y: 1.0,
                    theta: 0.0,
                },
                Path {
                    x: 2.0,
                    y: 2.0,
                    theta: 0.0,
                },
            ],
            device_id: "robot1".to_string(),
            state: MotionState::Resume.to_string(),
            battery_level: 100.0,
        };

        let robot2 = Robot {
            x: 10.0,
            y: 10.0,
            theta: 0.0,
            loaded: false,
            timestamp: 0,
            path: vec![
                Path {
                    x: 10.0,
                    y: 10.0,
                    theta: 0.0,
                },
                Path {
                    x: 20.0,
                    y: 20.0,
                    theta: 0.0,
                },
                Path {
                    x: 30.0,
                    y: 30.0,
                    theta: 0.0,
                },
            ],
            device_id: "robot2".to_string(),
            state: MotionState::Resume.to_string(),
            battery_level: 100.0,
        };

        let robot3 = Robot {
            x: 50.0,
            y: 50.0,
            theta: 0.0,
            loaded: false,
            timestamp: 0,
            path: vec![
                Path {
                    x: 50.0,
                    y: 50.0,
                    theta: 0.0,
                },
                Path {
                    x: 60.0,
                    y: 60.0,
                    theta: 0.0,
                },
                Path {
                    x: 70.0,
                    y: 70.0,
                    theta: 0.0,
                },
            ],
            device_id: "robot3".to_string(),
            state: MotionState::Resume.to_string(),
            battery_level: 100.0,
        };

        let robot4 = Robot {
            x: 3.0,
            y: 3.0,
            theta: 0.0,
            loaded: false,
            timestamp: 0,
            path: vec![
                Path {
                    x: 3.0,
                    y: 3.0,
                    theta: 0.0,
                },
                Path {
                    x: 4.0,
                    y: 4.0,
                    theta: 0.0,
                },
            ],
            device_id: "robot4".to_string(),
            state: MotionState::Resume.to_string(),
            battery_level: 100.0,
        };

        let robots = vec![
            robot1.clone(),
            robot2.clone(),
            robot3.clone(),
            robot4.clone(),
        ];
        let config = CollisionMonitorConfig {
            width: 1.0,
            height: 1.0,
            queue_hub_pw: String::new(),
            queue_hub_user: String::new(),
            hostname: String::new(),
            hub_listening_port: 5672,
            num_agents: 3,
            logs_dir: String::new(),
            listening_port: 9877,
            db_path: String::new(),
        };

        let collision_monitor = CollisionMonitor::new(config);

        let mut updated_robots = robots.clone();
        collision_monitor.update_robot_state(&mut updated_robots);

        assert_eq!(updated_robots[0].state, MotionState::Resume.to_string());
        assert_eq!(updated_robots[0].x, 1.0);
        assert_eq!(updated_robots[0].y, 1.0);

        assert_eq!(updated_robots[1].state, MotionState::Resume.to_string());
        assert_eq!(updated_robots[1].x, 20.0);
        assert_eq!(updated_robots[1].y, 20.0);

        assert_eq!(updated_robots[2].state, MotionState::Resume.to_string());
        assert_eq!(updated_robots[2].x, 60.0);
        assert_eq!(updated_robots[2].y, 60.0);

        assert_eq!(updated_robots[3].state, MotionState::Resume.to_string());
        assert_eq!(updated_robots[3].x, 4.0);
        assert_eq!(updated_robots[3].y, 4.0);
    }

    #[test]
    fn test_collision_monitor_detect_collisions() {
        // Create 3 robots for testing
        let robot1 = Robot {
            x: 0.0,
            y: 0.0,
            theta: 0.0,
            loaded: false,
            timestamp: 0,
            path: vec![
                Path {
                    x: 0.0,
                    y: 0.0,
                    theta: 0.0,
                },
                Path {
                    x: 1.0,
                    y: 1.0,
                    theta: 0.0,
                },
            ],
            device_id: "robot1".to_string(),
            state: MotionState::Resume.to_string(),
            battery_level: 100.0,
        };

        let robot2 = Robot {
            x: 1.0,
            y: 1.0,
            theta: 0.0,
            loaded: false,
            timestamp: 0,
            path: vec![
                Path {
                    x: 1.0,
                    y: 1.0,
                    theta: 0.0,
                },
                Path {
                    x: 2.0,
                    y: 2.0,
                    theta: 0.0,
                },
            ],
            device_id: "robot2".to_string(),
            state: MotionState::Resume.to_string(),
            battery_level: 100.0,
        };

        let robot3 = Robot {
            x: 2.0,
            y: 2.0,
            theta: 0.0,
            loaded: false,
            timestamp: 0,
            path: vec![
                Path {
                    x: 2.0,
                    y: 2.0,
                    theta: 0.0,
                },
                Path {
                    x: 3.0,
                    y: 3.0,
                    theta: 0.0,
                },
            ],
            device_id: "robot3".to_string(),
            state: MotionState::Resume.to_string(),
            battery_level: 100.0,
        };

        let robots = vec![robot1.clone(), robot2.clone(), robot3.clone()];
        let config = CollisionMonitorConfig {
            width: 1.0,
            height: 1.0,
            queue_hub_pw: String::new(),
            queue_hub_user: String::new(),
            hostname: String::new(),
            hub_listening_port: 5672,
            num_agents: 3,
            logs_dir: String::new(),
            listening_port: 9877,
            db_path: String::new(),
        };
        let collision_monitor = CollisionMonitor::new(config);

        let conflicts = collision_monitor.detect_collisions(&robots);

        assert_eq!(conflicts.len(), 2);
        assert_eq!(conflicts[0], (0, 1));
        assert_eq!(conflicts[1], (1, 2));
    }

    #[test]
    fn test_collision_monitor_resolve_deadlock() {
        let robot1 = Robot {
            x: 0.0,
            y: 0.0,
            theta: 0.0,
            loaded: false,
            timestamp: 0,
            path: vec![
                Path {
                    x: 0.0,
                    y: 0.0,
                    theta: 0.0,
                },
                Path {
                    x: 1.0,
                    y: 1.0,
                    theta: 0.0,
                },
            ],
            device_id: "robot1".to_string(),
            state: MotionState::Resume.to_string(),
            battery_level: 100.0,
        };

        let robot2 = Robot {
            x: 1.0,
            y: 1.0,
            theta: 0.0,
            loaded: false,
            timestamp: 0,
            path: vec![
                Path {
                    x: 1.0,
                    y: 1.0,
                    theta: 0.0,
                },
                Path {
                    x: 0.0,
                    y: 0.0,
                    theta: 0.0,
                },
            ],
            device_id: "robot2".to_string(),
            state: MotionState::Resume.to_string(),
            battery_level: 100.0,
        };

        let robots = vec![robot1.clone(), robot2.clone()];
        let config = CollisionMonitorConfig {
            width: 1.0,
            height: 1.0,
            queue_hub_pw: String::new(),
            queue_hub_user: String::new(),
            hostname: String::new(),
            hub_listening_port: 5672,
            num_agents: 2,
            logs_dir: String::new(),
            listening_port: 9877,
            db_path: String::new(),
        };

        let collision_monitor = CollisionMonitor::new(config);

        let conflicts = vec![(0, 1)];
        collision_monitor.resolve_deadlock(&mut robots.clone(), &conflicts);

        assert_eq!(robots[0].state, MotionState::Resume.to_string());
        assert_eq!(robots[1].state, MotionState::Resume.to_string());
    }

    #[test]
    fn test_collision_monitor_will_collision_occur() {
        // Create 2 robots for testing
        let robot1 = Robot {
            x: 0.0,
            y: 0.0,
            theta: 0.0,
            loaded: false,
            timestamp: 0,
            path: vec![
                Path {
                    x: 0.0,
                    y: 0.0,
                    theta: 0.0,
                },
                Path {
                    x: 1.0,
                    y: 1.0,
                    theta: 0.0,
                },
            ],
            device_id: "robot1".to_string(),
            state: MotionState::Resume.to_string(),
            battery_level: 100.0,
        };

        let robot2 = Robot {
            x: 1.0,
            y: 1.0,
            theta: 0.0,
            loaded: false,
            timestamp: 0,
            path: vec![
                Path {
                    x: 1.0,
                    y: 1.0,
                    theta: 0.0,
                },
                Path {
                    x: 2.0,
                    y: 2.0,
                    theta: 0.0,
                },
            ],
            device_id: "robot2".to_string(),
            state: MotionState::Resume.to_string(),
            battery_level: 100.0,
        };

        let config = CollisionMonitorConfig {
            width: 1.0,
            height: 1.0,
            queue_hub_pw: String::new(),
            queue_hub_user: String::new(),
            hostname: String::new(),
            hub_listening_port: 5672,
            num_agents: 2,
            logs_dir: String::new(),
            listening_port: 9877,
            db_path: String::new(),
        };

        let collision_monitor = CollisionMonitor::new(config);

        let collision_occurs = collision_monitor.will_collision_occur(&robot1, &robot2);

        assert_eq!(collision_occurs, true);
    }
}
