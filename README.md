# Game Simulator

The game simulator is a library that can be used to quickly implement any turn based game and simulate playing it. Its objective is to find the best possible strategy to play the game, and in the process identify broken strategies and OP components etc. This enables the game developers to fine tune these strategies and reduce the effect of OP components - thereby making the game more balanced.

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

# Agricola

## Fence arrangements

| Spaces | Pastures (Wood) |
|---|---|
|1|1 (4)|
|2|1 (6), 2 (7)|
|3|1 (8), 2 (9), 3 (10)|
|4|1 (8), 2(10), 3(11), 4(12)|
|5|1 (10), 2(11), 3(13), 4(14), 5(15)|
|6|1 (10), 2(12), 3(13, 14), 4(15)|
|7|1 (12), 2(13), 3(15)|
|8|1 (12), 2(14), 3(15)|
|9|1 (12), 2(15), 3(15)|
|10|1 (14), 2(15)|
|11|1 (14)|
|12|1 (14)|





