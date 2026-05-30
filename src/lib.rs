//! # lau-quest
//!
//! Quest/mission system for the **Lau** (Layered Agent-UI) gamified learning platform.
//! Younger users learn PLATO/Grand Pattern concepts through quests in a voxel game world.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// GameEvent
// ---------------------------------------------------------------------------

/// Simplified game events that can advance quest progress.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum GameEvent {
    BlockPlaced,
    RoomVisited { room_id: String },
    ConservationChecked { topology: String, error: f64 },
    AgentTrained { agent_id: String, accuracy: f64 },
    StructureBuilt { name: String },
    BiomeExplored { biome: String },
    Collaborated { other_agent_id: String },
}

// ---------------------------------------------------------------------------
// Objective
// ---------------------------------------------------------------------------

/// A single objective within a quest.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Objective {
    /// Build something in the voxel world.
    BuildStructure { name: String, min_blocks: usize },
    /// Visit a PLATO room and collect data.
    ObserveRoom { room_id: String, min_readings: usize },
    /// Achieve conservation within tolerance.
    ConservationChallenge { target_error: f64, topology: String },
    /// Help an agent learn to predict.
    TeachAgent { agent_id: String, target_accuracy: f64 },
    /// Visit N different room types.
    ExploreBiome { biomes: usize },
    /// Work with N other agents.
    Collaborate { other_agents: usize },
}

impl Objective {
    fn initial_state(&self) -> ObjectiveState {
        match self {
            Self::BuildStructure { name, .. } => ObjectiveState::BuildStructure {
                blocks_placed: 0,
                structure_built: false,
                target_name: name.clone(),
            },
            Self::ObserveRoom { room_id, .. } => ObjectiveState::ObserveRoom {
                readings: 0,
                target_room: room_id.clone(),
            },
            Self::ConservationChallenge { topology, .. } => ObjectiveState::Conservation {
                best_error: f64::MAX,
                topology: topology.clone(),
            },
            Self::TeachAgent { agent_id, .. } => ObjectiveState::TeachAgent {
                best_accuracy: 0.0,
                target_agent: agent_id.clone(),
            },
            Self::ExploreBiome { .. } => ObjectiveState::ExploreBiome {
                visited: HashMap::new(),
            },
            Self::Collaborate { .. } => ObjectiveState::Collaborate {
                partners: HashMap::new(),
            },
        }
    }

    fn is_satisfied(&self, state: &ObjectiveState) -> bool {
        match (self, state) {
            (
                Self::BuildStructure { name, min_blocks },
                ObjectiveState::BuildStructure {
                    blocks_placed,
                    structure_built,
                    target_name,
                },
            ) => target_name == name && *blocks_placed >= *min_blocks && *structure_built,
            (
                Self::ObserveRoom { room_id, min_readings },
                ObjectiveState::ObserveRoom {
                    readings,
                    target_room,
                },
            ) => target_room == room_id && *readings >= *min_readings,
            (
                Self::ConservationChallenge {
                    target_error,
                    topology,
                },
                ObjectiveState::Conservation {
                    best_error,
                    topology: t,
                },
            ) => topology == t && *best_error <= *target_error,
            (
                Self::TeachAgent {
                    agent_id,
                    target_accuracy,
                },
                ObjectiveState::TeachAgent {
                    best_accuracy,
                    target_agent,
                },
            ) => target_agent == agent_id && *best_accuracy >= *target_accuracy,
            (
                Self::ExploreBiome { biomes },
                ObjectiveState::ExploreBiome { visited },
            ) => visited.len() >= *biomes,
            (
                Self::Collaborate { other_agents },
                ObjectiveState::Collaborate { partners },
            ) => partners.len() >= *other_agents,
            _ => false,
        }
    }
}

// ---------------------------------------------------------------------------
// ObjectiveState (internal tracking)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
enum ObjectiveState {
    BuildStructure {
        blocks_placed: usize,
        structure_built: bool,
        target_name: String,
    },
    ObserveRoom {
        readings: usize,
        target_room: String,
    },
    Conservation {
        best_error: f64,
        topology: String,
    },
    TeachAgent {
        best_accuracy: f64,
        target_agent: String,
    },
    ExploreBiome {
        visited: HashMap<String, usize>,
    },
    Collaborate {
        partners: HashMap<String, usize>,
    },
}

impl ObjectiveState {
    fn process(&mut self, event: &GameEvent) {
        match (self, event) {
            (
                ObjectiveState::BuildStructure {
                    blocks_placed,
                    ..
                },
                GameEvent::BlockPlaced,
            ) => {
                *blocks_placed += 1;
            }
            (
                ObjectiveState::BuildStructure {
                    structure_built,
                    target_name,
                    ..
                },
                GameEvent::StructureBuilt { name },
            ) if name == target_name => {
                *structure_built = true;
            }
            (
                ObjectiveState::ObserveRoom {
                    readings,
                    target_room,
                },
                GameEvent::RoomVisited { room_id },
            ) if room_id == target_room => {
                *readings += 1;
            }
            (
                ObjectiveState::Conservation {
                    best_error,
                    topology,
                },
                GameEvent::ConservationChecked {
                    topology: t,
                    error,
                },
            ) if t == topology && *error < *best_error => {
                *best_error = *error;
            }
            (
                ObjectiveState::TeachAgent {
                    best_accuracy,
                    target_agent,
                },
                GameEvent::AgentTrained {
                    agent_id,
                    accuracy,
                },
            ) if agent_id == target_agent && *accuracy > *best_accuracy => {
                *best_accuracy = *accuracy;
            }
            (
                ObjectiveState::ExploreBiome { visited },
                GameEvent::BiomeExplored { biome },
            ) => {
                *visited.entry(biome.clone()).or_insert(0) += 1;
            }
            (
                ObjectiveState::Collaborate { partners },
                GameEvent::Collaborated { other_agent_id },
            ) => {
                *partners.entry(other_agent_id.clone()).or_insert(0) += 1;
            }
            _ => {}
        }
    }
}

// ---------------------------------------------------------------------------
// Reward
// ---------------------------------------------------------------------------

/// Rewards earned from completing quests.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Reward {
    Badge(String),
    UnlockRecipe(String),
    VoxelPalette(String),
    AgentSkin(String),
    Title(String),
}

// ---------------------------------------------------------------------------
// Quest
// ---------------------------------------------------------------------------

/// A quest with objectives and rewards.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Quest {
    pub id: String,
    pub title: String,
    pub description: String,
    pub objectives: Vec<Objective>,
    pub rewards: Vec<Reward>,
    pub difficulty: u8,
}

impl Quest {
    /// Create a new quest, clamping difficulty to 1–5.
    pub fn new(
        id: impl Into<String>,
        title: impl Into<String>,
        description: impl Into<String>,
        objectives: Vec<Objective>,
        rewards: Vec<Reward>,
        difficulty: u8,
    ) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            description: description.into(),
            objectives,
            rewards,
            difficulty: difficulty.clamp(1, 5),
        }
    }

    /// Start tracking progress on this quest.
    pub fn start(&self) -> QuestProgress {
        QuestProgress::new(self)
    }
}

// ---------------------------------------------------------------------------
// QuestProgress
// ---------------------------------------------------------------------------

/// Tracks completion of each objective within a quest.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestProgress {
    quest_id: String,
    states: Vec<ObjectiveState>,
    completed: Vec<bool>,
    objective_count: usize,
}

impl QuestProgress {
    fn new(quest: &Quest) -> Self {
        Self {
            quest_id: quest.id.clone(),
            states: quest.objectives.iter().map(|o| o.initial_state()).collect(),
            completed: vec![false; quest.objectives.len()],
            objective_count: quest.objectives.len(),
        }
    }

    /// Overall completion as 0.0 – 1.0.
    pub fn progress(&self) -> f64 {
        if self.objective_count == 0 {
            return 1.0;
        }
        let done = self.completed.iter().filter(|&&c| c).count();
        done as f64 / self.objective_count as f64
    }

    /// Whether all objectives are complete.
    pub fn is_complete(&self) -> bool {
        self.completed.iter().all(|&c| c)
    }

    /// Advance progress based on a game event. Needs the quest reference
    /// to match objectives against tracked state.
    pub fn update(&mut self, event: &GameEvent, quest: &Quest) {
        for (i, state) in self.states.iter_mut().enumerate() {
            state.process(event);
            if !self.completed[i] && quest.objectives[i].is_satisfied(state) {
                self.completed[i] = true;
            }
        }
    }
}

// ---------------------------------------------------------------------------
// QuestGenerator
// ---------------------------------------------------------------------------

/// Generates age-appropriate quests.
pub struct QuestGenerator;

impl QuestGenerator {
    /// Simple exploration and building quests for beginners.
    pub fn for_beginner() -> Vec<Quest> {
        vec![
            Quest::new(
                "beginner-01",
                "First Steps",
                "Place your first block in the world.",
                vec![Objective::BuildStructure {
                    name: "starter".into(),
                    min_blocks: 1,
                }],
                vec![Reward::Badge("First Block".into())],
                1,
            ),
            Quest::new(
                "beginner-02",
                "Room Explorer",
                "Visit the PLATO lobby and collect 3 readings.",
                vec![Objective::ObserveRoom {
                    room_id: "lobby".into(),
                    min_readings: 3,
                }],
                vec![Reward::VoxelPalette("neon".into())],
                1,
            ),
            Quest::new(
                "beginner-03",
                "Biome Wanderer",
                "Explore 2 different biomes.",
                vec![Objective::ExploreBiome { biomes: 2 }],
                vec![Reward::Title("Wanderer".into())],
                1,
            ),
            Quest::new(
                "beginner-04",
                "Tiny Tower",
                "Build a tower with at least 10 blocks.",
                vec![Objective::BuildStructure {
                    name: "tower".into(),
                    min_blocks: 10,
                }],
                vec![Reward::Badge("Tower Builder".into())],
                2,
            ),
            Quest::new(
                "beginner-05",
                "Friendly Face",
                "Collaborate with 1 other agent.",
                vec![Objective::Collaborate { other_agents: 1 }],
                vec![Reward::AgentSkin("friendly".into())],
                2,
            ),
        ]
    }

    /// Conservation challenges and agent training for intermediate learners.
    pub fn for_intermediate() -> Vec<Quest> {
        vec![
            Quest::new(
                "inter-01",
                "Conservation Scout",
                "Achieve conservation error below 0.1 in the simple corridor topology.",
                vec![Objective::ConservationChallenge {
                    target_error: 0.1,
                    topology: "corridor".into(),
                }],
                vec![Reward::Badge("Conservation Scout".into())],
                3,
            ),
            Quest::new(
                "inter-02",
                "Agent Tutor",
                "Train agent alice to 75% accuracy.",
                vec![Objective::TeachAgent {
                    agent_id: "alice".into(),
                    target_accuracy: 0.75,
                }],
                vec![Reward::UnlockRecipe("agent-boost".into())],
                3,
            ),
            Quest::new(
                "inter-03",
                "Multi-Room Observer",
                "Visit both the lobby and garden rooms, collecting 5 readings each.",
                vec![
                    Objective::ObserveRoom {
                        room_id: "lobby".into(),
                        min_readings: 5,
                    },
                    Objective::ObserveRoom {
                        room_id: "garden".into(),
                        min_readings: 5,
                    },
                ],
                vec![Reward::VoxelPalette("garden".into())],
                3,
            ),
            Quest::new(
                "inter-04",
                "Conservation Expert",
                "Achieve conservation error below 0.01 in the branching topology.",
                vec![Objective::ConservationChallenge {
                    target_error: 0.01,
                    topology: "branching".into(),
                }],
                vec![Reward::Title("Conservation Expert".into())],
                4,
            ),
        ]
    }

    /// Multi-room orchestration and system design for advanced learners.
    pub fn for_advanced() -> Vec<Quest> {
        vec![
            Quest::new(
                "advanced-01",
                "System Architect",
                "Build a complex structure and achieve conservation in branching topology.",
                vec![
                    Objective::BuildStructure {
                        name: "complex".into(),
                        min_blocks: 50,
                    },
                    Objective::ConservationChallenge {
                        target_error: 0.005,
                        topology: "branching".into(),
                    },
                ],
                vec![
                    Reward::Badge("System Architect".into()),
                    Reward::UnlockRecipe("advanced-blocks".into()),
                ],
                4,
            ),
            Quest::new(
                "advanced-02",
                "Master Teacher",
                "Train agents alice and bob to 90% accuracy each.",
                vec![
                    Objective::TeachAgent {
                        agent_id: "alice".into(),
                        target_accuracy: 0.9,
                    },
                    Objective::TeachAgent {
                        agent_id: "bob".into(),
                        target_accuracy: 0.9,
                    },
                ],
                vec![
                    Reward::Title("Master Teacher".into()),
                    Reward::AgentSkin("professor".into()),
                ],
                5,
            ),
            Quest::new(
                "advanced-03",
                "World Builder",
                "Explore 5 biomes, collaborate with 3 agents, and build a mega structure.",
                vec![
                    Objective::ExploreBiome { biomes: 5 },
                    Objective::Collaborate { other_agents: 3 },
                    Objective::BuildStructure {
                        name: "mega".into(),
                        min_blocks: 100,
                    },
                ],
                vec![
                    Reward::Badge("World Builder".into()),
                    Reward::VoxelPalette("rainbow".into()),
                    Reward::Title("World Builder".into()),
                ],
                5,
            ),
        ]
    }
}

// ---------------------------------------------------------------------------
// LearningPath
// ---------------------------------------------------------------------------

/// Ordered sequence of quests teaching Grand Pattern concepts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningPath {
    quests: Vec<Quest>,
    current_index: usize,
    progress_per_quest: Vec<f64>,
}

impl LearningPath {
    /// Build the canonical 5-lesson learning path.
    pub fn new() -> Self {
        let quests = vec![
            Quest::new(
                "lesson-1",
                "Vibe is a Feeling",
                "Learn that rooms have mono-dimensional vibes. Visit 3 rooms and observe their vibes.",
                vec![Objective::ObserveRoom {
                    room_id: "vibe-room".into(),
                    min_readings: 3,
                }],
                vec![Reward::Badge("Vibe Observer".into())],
                1,
            ),
            Quest::new(
                "lesson-2",
                "Conservation Detective",
                "Verify conservation laws hold. Achieve error below 0.05.",
                vec![Objective::ConservationChallenge {
                    target_error: 0.05,
                    topology: "simple".into(),
                }],
                vec![Reward::Badge("Conservation Detective".into())],
                2,
            ),
            Quest::new(
                "lesson-3",
                "Room Whisperer",
                "Train an agent by feeding it readings. Get it to 80% accuracy.",
                vec![Objective::TeachAgent {
                    agent_id: "whisperer".into(),
                    target_accuracy: 0.8,
                }],
                vec![Reward::Title("Room Whisperer".into())],
                3,
            ),
            Quest::new(
                "lesson-4",
                "The Dissolving Room",
                "See a perfectly adapted room dissolve. Explore 4 biomes to find dissolving rooms.",
                vec![Objective::ExploreBiome { biomes: 4 }],
                vec![Reward::VoxelPalette("dissolve".into())],
                4,
            ),
            Quest::new(
                "lesson-5",
                "Build Your World",
                "Create a room structure in voxels with at least 30 blocks.",
                vec![Objective::BuildStructure {
                    name: "my-room".into(),
                    min_blocks: 30,
                }],
                vec![
                    Reward::Badge("World Creator".into()),
                    Reward::UnlockRecipe("room-parts".into()),
                ],
                5,
            ),
        ];
        Self {
            progress_per_quest: vec![0.0; quests.len()],
            quests,
            current_index: 0,
        }
    }

    /// The currently active quest.
    pub fn current_quest(&self) -> Option<&Quest> {
        self.quests.get(self.current_index)
    }

    /// Move to the next quest after completion.
    pub fn advance(&mut self) {
        if self.current_index < self.quests.len() {
            self.progress_per_quest[self.current_index] = 1.0;
            self.current_index += 1;
        }
    }

    /// Overall learning path progress (0.0 – 1.0).
    pub fn progress(&self) -> f64 {
        if self.quests.is_empty() {
            return 1.0;
        }
        self.progress_per_quest.iter().sum::<f64>() / self.quests.len() as f64
    }

    /// Total number of quests.
    pub fn len(&self) -> usize {
        self.quests.len()
    }

    /// Whether the path is empty.
    pub fn is_empty(&self) -> bool {
        self.quests.is_empty()
    }
}

impl Default for LearningPath {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quest_difficulty_clamped_low() {
        let q = Quest::new("t", "t", "t", vec![], vec![], 0);
        assert_eq!(q.difficulty, 1);
    }

    #[test]
    fn quest_difficulty_clamped_high() {
        let q = Quest::new("t", "t", "t", vec![], vec![], 9);
        assert_eq!(q.difficulty, 5);
    }

    #[test]
    fn quest_start_creates_progress() {
        let q = Quest::new(
            "q1",
            "Test",
            "desc",
            vec![Objective::BuildStructure {
                name: "x".into(),
                min_blocks: 5,
            }],
            vec![],
            1,
        );
        let p = q.start();
        assert_eq!(p.quest_id, "q1");
        assert!(!p.is_complete());
    }

    #[test]
    fn build_structure_completes() {
        let q = Quest::new(
            "q",
            "Build",
            "desc",
            vec![Objective::BuildStructure {
                name: "tower".into(),
                min_blocks: 3,
            }],
            vec![Reward::Badge("b".into())],
            1,
        );
        let mut p = q.start();
        for _ in 0..3 {
            p.update(&GameEvent::BlockPlaced, &q);
        }
        assert!(!p.is_complete()); // still needs StructureBuilt
        p.update(&GameEvent::StructureBuilt { name: "tower".into() }, &q);
        assert!(p.is_complete());
        assert!((p.progress() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn build_structure_wrong_name_ignored() {
        let q = Quest::new(
            "q",
            "Build",
            "desc",
            vec![Objective::BuildStructure {
                name: "tower".into(),
                min_blocks: 1,
            }],
            vec![],
            1,
        );
        let mut p = q.start();
        p.update(&GameEvent::BlockPlaced, &q);
        p.update(&GameEvent::StructureBuilt { name: "wall".into() }, &q);
        assert!(!p.is_complete());
    }

    #[test]
    fn observe_room_completes() {
        let q = Quest::new(
            "q",
            "Observe",
            "desc",
            vec![Objective::ObserveRoom {
                room_id: "lab".into(),
                min_readings: 2,
            }],
            vec![],
            2,
        );
        let mut p = q.start();
        p.update(&GameEvent::RoomVisited { room_id: "lab".into() }, &q);
        assert!(!p.is_complete());
        p.update(&GameEvent::RoomVisited { room_id: "lab".into() }, &q);
        assert!(p.is_complete());
    }

    #[test]
    fn observe_room_wrong_room_ignored() {
        let q = Quest::new(
            "q",
            "Observe",
            "desc",
            vec![Objective::ObserveRoom {
                room_id: "lab".into(),
                min_readings: 1,
            }],
            vec![],
            1,
        );
        let mut p = q.start();
        p.update(
            &GameEvent::RoomVisited {
                room_id: "other".into(),
            },
            &q,
        );
        assert!(!p.is_complete());
    }

    #[test]
    fn conservation_challenge_completes() {
        let q = Quest::new(
            "q",
            "Conservation",
            "desc",
            vec![Objective::ConservationChallenge {
                target_error: 0.1,
                topology: "corridor".into(),
            }],
            vec![],
            3,
        );
        let mut p = q.start();
        p.update(
            &GameEvent::ConservationChecked {
                topology: "corridor".into(),
                error: 0.05,
            },
            &q,
        );
        assert!(p.is_complete());
    }

    #[test]
    fn conservation_high_error_not_complete() {
        let q = Quest::new(
            "q",
            "Conservation",
            "desc",
            vec![Objective::ConservationChallenge {
                target_error: 0.01,
                topology: "corridor".into(),
            }],
            vec![],
            3,
        );
        let mut p = q.start();
        p.update(
            &GameEvent::ConservationChecked {
                topology: "corridor".into(),
                error: 0.5,
            },
            &q,
        );
        assert!(!p.is_complete());
    }

    #[test]
    fn conservation_wrong_topology_ignored() {
        let q = Quest::new(
            "q",
            "Conservation",
            "desc",
            vec![Objective::ConservationChallenge {
                target_error: 0.1,
                topology: "corridor".into(),
            }],
            vec![],
            3,
        );
        let mut p = q.start();
        p.update(
            &GameEvent::ConservationChecked {
                topology: "other".into(),
                error: 0.001,
            },
            &q,
        );
        assert!(!p.is_complete());
    }

    #[test]
    fn teach_agent_completes() {
        let q = Quest::new(
            "q",
            "Teach",
            "desc",
            vec![Objective::TeachAgent {
                agent_id: "alice".into(),
                target_accuracy: 0.75,
            }],
            vec![],
            3,
        );
        let mut p = q.start();
        p.update(
            &GameEvent::AgentTrained {
                agent_id: "alice".into(),
                accuracy: 0.8,
            },
            &q,
        );
        assert!(p.is_complete());
    }

    #[test]
    fn teach_agent_wrong_agent_ignored() {
        let q = Quest::new(
            "q",
            "Teach",
            "desc",
            vec![Objective::TeachAgent {
                agent_id: "alice".into(),
                target_accuracy: 0.5,
            }],
            vec![],
            3,
        );
        let mut p = q.start();
        p.update(
            &GameEvent::AgentTrained {
                agent_id: "bob".into(),
                accuracy: 0.9,
            },
            &q,
        );
        assert!(!p.is_complete());
    }

    #[test]
    fn explore_biome_completes() {
        let q = Quest::new(
            "q",
            "Explore",
            "desc",
            vec![Objective::ExploreBiome { biomes: 3 }],
            vec![],
            2,
        );
        let mut p = q.start();
        p.update(&GameEvent::BiomeExplored { biome: "forest".into() }, &q);
        p.update(&GameEvent::BiomeExplored { biome: "desert".into() }, &q);
        assert!(!p.is_complete());
        p.update(&GameEvent::BiomeExplored { biome: "ocean".into() }, &q);
        assert!(p.is_complete());
    }

    #[test]
    fn collaborate_completes() {
        let q = Quest::new(
            "q",
            "Collab",
            "desc",
            vec![Objective::Collaborate { other_agents: 2 }],
            vec![],
            2,
        );
        let mut p = q.start();
        p.update(
            &GameEvent::Collaborated {
                other_agent_id: "a1".into(),
            },
            &q,
        );
        p.update(
            &GameEvent::Collaborated {
                other_agent_id: "a2".into(),
            },
            &q,
        );
        assert!(p.is_complete());
    }

    #[test]
    fn collaborate_same_agent_counts_once() {
        let q = Quest::new(
            "q",
            "Collab",
            "desc",
            vec![Objective::Collaborate { other_agents: 2 }],
            vec![],
            2,
        );
        let mut p = q.start();
        for _ in 0..5 {
            p.update(
                &GameEvent::Collaborated {
                    other_agent_id: "a1".into(),
                },
                &q,
            );
        }
        assert!(!p.is_complete());
    }

    #[test]
    fn multi_objective_partial_progress() {
        let q = Quest::new(
            "q",
            "Multi",
            "desc",
            vec![
                Objective::ObserveRoom {
                    room_id: "r1".into(),
                    min_readings: 1,
                },
                Objective::ExploreBiome { biomes: 2 },
            ],
            vec![],
            2,
        );
        let mut p = q.start();
        p.update(&GameEvent::RoomVisited { room_id: "r1".into() }, &q);
        assert!(!p.is_complete());
        assert!((p.progress() - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn reward_equality() {
        assert_eq!(Reward::Badge("x".into()), Reward::Badge("x".into()));
        assert_ne!(Reward::Badge("x".into()), Reward::Badge("y".into()));
    }

    #[test]
    fn quest_generator_beginner() {
        let quests = QuestGenerator::for_beginner();
        assert_eq!(quests.len(), 5);
        for q in &quests {
            assert!(q.difficulty <= 2);
        }
    }

    #[test]
    fn quest_generator_intermediate() {
        let quests = QuestGenerator::for_intermediate();
        assert_eq!(quests.len(), 4);
    }

    #[test]
    fn quest_generator_advanced() {
        let quests = QuestGenerator::for_advanced();
        assert_eq!(quests.len(), 3);
    }

    #[test]
    fn learning_path_current_quest() {
        let lp = LearningPath::new();
        assert_eq!(lp.current_quest().unwrap().id, "lesson-1");
    }

    #[test]
    fn learning_path_advance() {
        let mut lp = LearningPath::new();
        assert_eq!(lp.current_quest().unwrap().id, "lesson-1");
        lp.advance();
        assert_eq!(lp.current_quest().unwrap().id, "lesson-2");
    }

    #[test]
    fn learning_path_full_progress() {
        let mut lp = LearningPath::new();
        assert!((lp.progress() - 0.0).abs() < f64::EPSILON);
        lp.advance();
        assert!((lp.progress() - 0.2).abs() < f64::EPSILON);
        lp.advance();
        lp.advance();
        lp.advance();
        lp.advance();
        assert!((lp.progress() - 1.0).abs() < f64::EPSILON);
        assert!(lp.current_quest().is_none());
    }

    #[test]
    fn learning_path_len() {
        let lp = LearningPath::new();
        assert_eq!(lp.len(), 5);
        assert!(!lp.is_empty());
    }

    #[test]
    fn quest_serialization_roundtrip() {
        let q = Quest::new(
            "s-test",
            "Serialize",
            "test desc",
            vec![Objective::BuildStructure {
                name: "x".into(),
                min_blocks: 5,
            }],
            vec![Reward::Badge("test".into())],
            3,
        );
        let json = serde_json::to_string(&q).unwrap();
        let q2: Quest = serde_json::from_str(&json).unwrap();
        assert_eq!(q, q2);
    }

    #[test]
    fn game_event_serialization_roundtrip() {
        let e = GameEvent::ConservationChecked {
            topology: "branching".into(),
            error: 0.01,
        };
        let json = serde_json::to_string(&e).unwrap();
        let e2: GameEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(e, e2);
    }

    #[test]
    fn progress_with_no_objectives() {
        let q = Quest::new("empty", "Empty", "no objectives", vec![], vec![], 1);
        let p = q.start();
        assert!(p.is_complete());
        assert!((p.progress() - 1.0).abs() < f64::EPSILON);
    }
}
