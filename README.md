# lau-quest

> Quests that teach mathematics through gameplay. Build structures, explore biomes, train agents — and learn conservation laws, topology, and spectral theory without realizing it.

## What This Does

The quest/mission system for the **Lau (Layered Agent-UI)** gamified learning platform. Young users explore a voxel game world where every quest teaches a real mathematical concept. Build a bridge and learn about graph connectivity. Explore a biome and discover conservation laws. Train an agent and understand optimization.

Quests are defined as JSON — objectives, rewards, prerequisites, and progress tracking. The system handles dependencies, chaining, and completion detection.

## The Key Idea

The best learning happens when you don't know you're learning. Lau quests wrap serious math concepts in game mechanics:
- "Check if your world conserves energy" → conservation laws
- "Visit all PLATO rooms and collect readings" → data collection and analysis
- "Teach your agent to predict" → machine learning fundamentals
- "Build a structure with rotational symmetry" → group theory

Players see game objectives. They're actually doing mathematics.

## Install

```bash
cargo add lau-quest
```

## Quick Start

### Define a Quest

```rust
use lau_quest::{Quest, QuestDefinition, Objective, GameEvent, Reward};

let quest = QuestDefinition::builder()
    .id("first-conservation")
    .title("The Conservationist")
    .description("Check if your world's energy is conserved!")
    .objective(Objective::ConservationChallenge {
        target_error: 0.01,
        topology: "simple-graph".into(),
    })
    .objective(Objective::BuildStructure {
        name: "energy-meter".into(),
        min_blocks: 10,
    })
    .reward(Reward::Badge("conservation-novice"))
    .reward(Reward::UnlockBiome("CrystalCaves"))
    .build();
```

### Track Progress

```rust
use lau_quest::QuestTracker;

let mut tracker = QuestTracker::new();
tracker.register(quest);

// Game events advance quest progress
tracker.on_event(&GameEvent::ConservationChecked {
    topology: "simple-graph".into(),
    error: 0.005,
})?;

tracker.on_event(&GameEvent::StructureBuilt {
    name: "energy-meter".into(),
})?;

// Check completion
let status = tracker.status("first-conservation");
println!("Progress: {}/{} objectives", status.completed(), status.total());
if status.is_complete() {
    println!("Quest complete! Rewards: {:?}", status.rewards());
}
```

### Quest Chains

```rust
// Prerequisites create quest chains
let advanced = QuestDefinition::builder()
    .id("topologist")
    .title("The Topologist")
    .description("Discover the shape of your world")
    .prerequisite("first-conservation")  // must complete first
    .objective(Objective::ObserveRoom {
        room_id: "topology-lab".into(),
        min_readings: 5,
    })
    .build();
```

### Available Events

| GameEvent | Triggers Objective |
|-----------|-------------------|
| `BlockPlaced` | BuildStructure |
| `RoomVisited { room_id }` | ObserveRoom |
| `ConservationChecked { topology, error }` | ConservationChallenge |
| `AgentTrained { agent_id, accuracy }` | TeachAgent |
| `StructureBuilt { name }` | BuildStructure |
| `BiomeExplored { biome }` | ExploreBiome |
| `Collaborated { other_agent_id }` | CollaborateWith |

### Objective Types

| Objective | What the Player Does |
|-----------|---------------------|
| `BuildStructure { name, min_blocks }` | Build something in the voxel world |
| `ObserveRoom { room_id, min_readings }` | Visit a PLATO room and collect data |
| `ConservationChallenge { target_error, topology }` | Verify conservation laws |
| `TeachAgent { agent_id, target_accuracy }` | Train an ML agent |
| `ExploreBiome { count }` | Visit different biomes |
| `CollaborateWith { count }` | Work with other players/agents |

## API Reference

### QuestDefinition

| Method | Description |
|--------|-------------|
| `QuestDefinition::builder()` | Create builder |
| `.id(id)` | Unique quest identifier |
| `.title(title)` | Display name |
| `.description(desc)` | What the player sees |
| `.objective(obj)` | Add an objective |
| `.prerequisite(id)` | Require another quest first |
| `.reward(reward)` | Add a reward |
| `.build()` | Build the definition |

### QuestTracker

| Method | Description |
|--------|-------------|
| `QuestTracker::new()` | Create tracker |
| `tracker.register(quest)` | Register a quest |
| `tracker.on_event(event)` | Process a game event |
| `tracker.status(id)` | Check quest status |
| `tracker.available()` | Quests ready to start |
| `tracker.active()` | Currently in-progress |
| `tracker.completed()` | Finished quests |

### Reward

| Variant | Description |
|---------|-------------|
| `Badge(name)` | Achievement badge |
| `UnlockBiome(biome)` | Access new area |
| `UnlockAgent(ability)` | New agent capability |
| `Points(amount)` | Score points |

## Testing

27 tests covering: quest creation, objective matching, event processing, completion detection, prerequisites, chaining, rewards, serialization.

## Part of the Lau Platform

- **lau-git-world** — Git-native game worlds
- **lau-quest** — You are here
- **lau-biome** — 10 distinct ecological zones
- **lau-spatial** — Spatial indexing for game worlds
- **lau-audio** — Procedural audio from math
- **lau-scheduler** — Tick-based game loop
- **lau-memory-arena** — Custom allocator for game entities
- **lau-genealogy** — Lineage tracking
- **lau-recipe** — Crafting recipes

## License

MIT
