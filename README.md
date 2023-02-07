# Game Simulator (Agricola)

The game simulator is a library that can be used to quickly implement any turn based game and simulate playing it. Its objective is to find the best possible strategy to play the game, and in the process identify broken strategies and OP components etc. This enables the game developers to fine tune these strategies and reduce the effect of OP components - thereby making the game more balanced.

The current version is a simulator of the popular strategy board game Agricola.

## System Design

Here is a brief explanation of how different entities in the game should interoperate.

### Game State (Data)

The game state is the overall representation of the game at any point in time. Some parts of the game state could be visible only to certain agents (see next).

### Agents (Entity)

Agents (Players) are entities that change the state of the game. The game itself could be represented as one of the agents (sets up next phase, rewards/punishes other agents etc).

### Phases (Entity)

A game phase is a period of time in the game, where a fixed set of actions (see later) need to be performed. This could be a part of the Game State.

### Actions (Function)

Actions are a set of functions that an Agent performs which change the game state.

## Objective

The objective of the simulator is to find the best possible action for agent A (the current agent in a turn based manner), given a set of possible actions, so that the outcome of the game is most favourable to A.

The simulator can determine the best possible strategy in one of the following ways

### Using a state evaluation function and simulating 't' actions in the future

This is how a chess engine works for example. Given a good static evaluation function, minimax + alpha-beta pruning can lead to very good results. Usually board games have a running point system to determine which player leads, the evaluation function in this case, can just be the point score + some probabilistic score for the resources/actions in hand.

Algorithm

```
1. Input : Game State S and current Agent A. Output best action f for A.
2. Determine list of actions C available to agent A
3. Simulate GameState S' <= f(S) for each action f \in C and traverse to depth d. Using minimax and alpha beta pruning, prune list of actions at each turn. Find action f that leads to the highest possible score for A using a static evaluation function at the terminal states. Return f.
```

### Monte Carlo simulation of a large number of games

In a Monte Carlo simulation, a large number of games are played and based on the results from these games, certain states are learnt to be better states than some others.

### Reinforcement learning

Playing out an entire game is often expensive especially with a large branching factor (lot of agents and lot of actions), in this case, a game is also played till a certain horizon in the future and the reward at that point is used to weigh the valuation of actions at the current time.

### TODO

- Improvements
- Animal re-org
- Cooking and Baking and converting other resources to food using majors
- Getting resources from future action spaces (e.g Well)
- Harvest : Food deduction, Food conversion, Grains and Veg from fields, Animal breeding (discard animals if cannot fit in pastures)
- Simulation!! Rewrite functions such that `f(state) = Vec<actions>`, and `action(state_1) = state_2` can be called
- Implement human player
- Implement AI player that chooses the best action by doing `n` random playouts from each immediate action and averaging the score for every action.
- Implement pure MCTS

### Results

Results from simplest algorithm - average of scores from 2000 random playouts from each action.

```
0.Player (4/4) SCORE 29 has [1 Fd][6 Wd][8 Cl][1 St][7 Gr][2 Vg][2 Room Clay House][Pastures [2 => 4 Sheep][2][1]][Fields [0][1G][0][1V][0][1V]][FP2][CH5][S]
1.Player (2/2) SCORE 23 has [10 Rd][4 Gr][3 Vg][1 Sheep][1 Cow][2 Room Wood House][Pastures [1 + S => 4 Pig(s)]][Fields [0][2G][1V][2G][0]][1 UF Stables][BMW][X]
2.Player (2/2) SCORE 26 has [1 Children][1 St][4 Gr][1 Vg][2 Room Clay House][Pastures [2 => 4 Sheep][2 => 3 Pig(s)][1 => 1 Cow(s)][1]][Fields [1G][0][1G]][WL]
3.Player (2/2) SCORE 19 has [2 Fd][2 Cl][1 St][1 Gr][2 Vg][2 Room Stone House][Pastures [2 + S => 4 Sheep][1 + S => 3 Cow(s)][1 + S => 3 Pig(s)]][Fields [0]][CH4][CO]
Time elapsed: 89.421770583s
Scores [29, 23, 26, 19]
```

## Misc commands

Show public functions and modules

```
cargo install cargo-modules
cargo modules generate tree --with-fns
```

LOC

```
git ls-files | grep '\.rs' | xargs wc -l
```

