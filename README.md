# Agricola AI

The Agricola AI is an AI agent, capable of playing the turn-based strategy (tbs) board game Agricola, by Uwe Rosenberg. Its objective is to find the best possible strategy to play the game, and in the process identify broken strategies and OP components (cards for example). The objective to create such an AI agent, in general for any game, is to identify certain broken strategies in the game, and someday allow the game designers to fine tune these strategies and reduce the effect of the OP components - thereby making the game more balanced.

The AI agent has different modes, and even the most simple 'Random AI' mode can play the game at a reasonable level. The strategy of finding the best move is done by simulating play from a given state all the way to the end of the game. The 'MCTS AI' uses the Monte Carlo Tree Search technique to optimally pick the most promising branches arising from a given state.

The project is written entirely in Rust and thus greatly benefits from Rust's memory safety and speed.

## System Design

Here is a brief explanation of the several entities in the game. 
- Game : this is the struct representing the game state.
- Player : the struct representing the Player state. Several of these structs are part of the game state. Some parts of this struct should be hidden from other Players.
- ActionSpaces : Spaces on the board where a Player can place one of its workers (i.e., family members) - Agricola is part of a large number of tbs games that categorize themselves as worker-placement games.

## AI techniques

There are several techniques to evaluate best moves in a tbs game. Minimax / Negamax is a well known technique to find best moves in a two player zero sum game, such as Chess. Alpha-Beta pruning is a technique used to prune the exponentially growing search tree in the Minimax algorithm. Though this is a very good technique, it often fails to explore certain branches which has a delayed reward - e.g., a sacrifice (bad initial move) that leads to checkmate (best final result) - and is very dependent on its evaluation function for a particular game state. 

MCTS, on the other hand, makes no such assumptions. It is a purely knowledge-agnostic technique, which means that it requires no prior knowledge of how a game state is evaluated. It picks up on the best strategies as it searches through the game tree and is theoretically capable of perfect play given infinite resources. Hence its performance scales up with the number of searches it performs.

Agricola is a complex game with several actions within actions - which are known as split actions. MCTS is one of the few techniques that works with split action games as well.

Agricola is also an 'imperfect information' game, where certain information about the future board state as well as player hands (Occupation and Minor Improvement cards) are hidden from other players. While forward simulating playouts for such games from the perspective of player A, care must be taken to sample hidden actions from the entire hidden state space for all other players apart from A, rather than only from their own possible hidden actions. This ensures that player A does not 'learn' about the strategies that follow after other players play a hidden card from their hand, way before they actually play it.

## How to Run

To run the release version (assuming you have Rust installed), from the parent directory run 

```
cargo run --release -- <num_players> <optional : human_player>
```

where you can specify the number of players and whether the first player is a human player.


## Results

Results from a game played by 4 MCTS AI agents. A total of 10000 simulations are performed for each move.

```
0.Player (2/2) SCORE 22 has [11 Wd][7 Cl][1 Rd][1 Gr][1 Vg][2 Room Clay House][Pastures [2 => 4 Sheep][2 => 1 Cow(s)][1 => 1 Pig(s)][1 => 2 Sheep]][CH4][JY][PY]
1.Player (3/3) SCORE 38 has [1 Children][1 Cl][5 Rd][3 Vg][2 Room Clay House][Pastures [2 => 4 Pig(s)][2 => 4 Sheep][1 => 1 Cow(s)][1 => 1 Pig(s)]][Fields [0][0][0][1V][0][2G]][FP3][BMW][S]
2.Player (2/2) SCORE 27 has [5 Gr][2 Vg][1 Pig][2 Room Wood House][Pastures [2 + S => 8 Sheep][2 => 4 Cow(s)]][Fields [1G][1V][0][2G][0][1V]][FP2][X]
3.Player (2/2) SCORE 26 has [6 Fd][1 Cl][1 Rd][4 Gr][3 Vg][1 Sheep][2 Room Clay House][Pastures [2 => 4 Sheep][2 => 1 Pig(s)][1]][Fields [0][1G][0][1G][0]][WL][CO]
Time elapsed: 432.691308125s
Scores [22, 38, 27, 26]
Fitness [-16, 11, -11, -12]
```

## TODO

- Best fencing arrangements instead of same structure.
- Best sowing strategy according to score.
- Implement OCCs.
- Implement Minors.

### MCTS Strategy

- Randomly sample `n` immediate actions for each action. This is to identify all possible split actions. For example, for a build Improvements action, one can build one of several possible improvements. Similarly paying for harvest can be done in one of many different possible ways. A branch should exist for each such choice that exists within an action. One option is to enumerate this tree in a deterministic way. Another is to sample `n` number of times until all choices are revealed. 

- From [this article](https://stackoverflow.com/questions/36664993/mcts-uct-with-a-scoring-system) the Upper Confidence Bound for choosing a move in the MCTS strategy can be modified to use scores rather than win-rate. 

The win-rate formula picks a move with the following probability : w_i / n_i + sqrt(2 * N_i / n_i). 

The score/fitness probability can be given as (x_i - a_i) / (b_i - a_i) + sqrt(2 * N_i / n_i) so that the win-rate is again normalized.

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
