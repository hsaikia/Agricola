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

## How to Run

To run the release version (assuming you have Rust installed), from the parent directory run 

```
cargo run --release -- <num_players> <optional : human_player>
```

where you can specify the number of players and whether the first player is a human player.


## Results

Results from a game played by 4 Random AI agents. The Random AI calculates a winrate from 5000 random playouts from each action, and picks the action with the best winrate.

```
0.Player (3/3) SCORE 29 has [4 Wd][5 Gr][2 Room Wood House][Pastures [2 + S => 4 Cow(s)][2 + S => 3 Pig(s)][1 + S => 1 Sheep]][Fields [1G][1G][0][0][1G]][WL]
1.Player (3/3) SCORE 26 has [1 Children][7 Cl][1 St][1 Rd][5 Gr][3 Vg][1 Sheep][2 Room Stone House][Pastures [2 + S => 3 Pig(s)]][Fields [0][0][0]][1 UF Stables][CH4][PY]
2.Player (2/2) SCORE 18 has [7 Wd][2 Rd][1 Gr][3 Vg][2 Room Wood House][Pastures [2 => 4 Sheep][2 => 1 Cow(s)][1][1]][Fields [0][0]][FP2][JY][X]
3.Player (2/2) SCORE 33 has [1 Cl][10 Rd][2 Gr][2 Vg][2 Room Clay House][Pastures [2 => 4 Pig(s)][2 => 1 Cow(s)][1 => 1 Sheep][1 => 1 Pig(s)]][Fields [0][1G][0][1G][2G][1V]][CO][BMW][S]
Time elapsed: 222.746135375s
Scores [29, 26, 18, 33]
Fitness [-4, -7, -15, 4]
```

From only 100 random playouts

```
0.Player (3/3) SCORE 29 has [1 Children][1 St][1 Rd][9 Gr][4 Vg][1 Sheep][2 Room Wood House][Pastures [2 => 4 Sheep][2 => 3 Pig(s)][1]][Fields [1G][0][0][1G][0][0]][FP2][S]
1.Player (2/2) SCORE 26 has [8 Wd][1 Gr][2 Vg][2 Room Clay House][Pastures [2 + S => 3 Pig(s)][2 + S][1 + S][1 + S]][Fields [1V][0][1G][2G]][JY][X]
2.Player (2/2) SCORE 19 has [7 Cl][1 St][2 Gr][4 Vg][1 Sheep][2 Room Stone House][Pastures [1 => 2 Cow(s)]][Fields [0][0][0]][CH4][CO][PY]
3.Player (2/2) SCORE 22 has [1 Fd][1 St][5 Rd][1 Gr][2 Vg][2 Sheep][2 Room Clay House][Pastures [2 + S => 3 Cow(s)][2 + S => 3 Pig(s)]][Fields [1V][2G]][1 UF Stables][FP3][BMW]
Time elapsed: 4.477189375s
Scores [29, 26, 19, 22]
Fitness [3, -3, -10, -7]
```

## TODO

- Currently a lot of decisions within an action, like major improvement build is random. Implement 'within action decisions' such as best major to build, best resource conversion during harvest or best fencing arrangements using the same MCTS strategy.
- Implement OCCs.
- Implement Minors.
- Implement pure MCTS.
- Implement generic actions for fencing, farm expansion etc.

MCTS Strategy

- Randomly sample `n` immediate actions for each action. This is to identify all possible split actions. For example, for a build Improvements action, one can build one of several possible improvements. Similarly paying for harvest can be done in one of many different possible ways. A branch should exist for each such choice that exists within an action. One option is to enumerate this tree in a deterministic way. Another is to sample `n` number of times until all choices are revealed. 

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

