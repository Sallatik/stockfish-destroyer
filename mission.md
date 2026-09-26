♟️ The Chess Challenge – Beat Stockfish! 🤖



🎯 The Mission
Your task is to build a chess application with your own computerized chess player that will attempt to defeat Stockfish.
How you build your player is up to you. You may use AI, machine learning, classical chess algorithms, heuristics, or a combination of different techniques.
The goal is simple:
🏆 Build the strongest chess player and defeat Stockfish at the highest possible Elo rating!

Use the Stockfish API/engine to play the matches.

📊 How Is Playing Strength Measured?
A chess player’s strength is usually measured using an Elo rating.
Here is an approximate reference:


Level
Elo
Approximate playing strength
Novice
1320
New player
Beginner
1400
Experienced beginner
Intermediate
1600
Lower club level
Advanced
1800
Good club player
Expert
2000
Strong club player
Master
2200
Very strong player
Grandmaster
2500
Grandmaster level
Stockfish 🤖
Unlimited
Extremely strong

For comparison, some of the world’s best players have approximately these ratings:
♟️ Magnus Carlsen: 2823
♟️ Hikaru Nakamura: 2792
♟️ Fabiano Caruana: 2785.9
♟️ Javokhir Sindarov: 2777.9
♟️ Wesley So: 2772.7

📜 Rules
For your competition entry to qualify, the following requirements must be met:
💻 Any programming language may be used.
🔄 You must use one or more of the following during development: /goal, /loop, or a dynamic workflow.
⏱️ Thinking time is limited to a maximum of 5 seconds per move per player. For Stockfish, for example: engine.play(board, chess.engine.Limit(time=5.0))
🤝 A draw does not count as a win. To beat a particular Elo level, your player must actually win the game.
💾 Every game must be saved, together with the Elo rating used by Stockfish.
▶️ Games must be available for replay afterward. The replay must display a graphical chessboard so the game can be followed from the first move to the last.
🔍 After each game, a skill must be used to analyze the match.
🧠 Each match must also be analyzed by two different sub-agents:
An agent acting as a world-class chess expert, using someone with Magnus Carlsen’s playing strength as a model.
An agent specializing in chess engines and chess engine development.
The agents must analyze the match and provide concrete suggestions for improvements that can be used to develop your chess engine further.
⚙️ Setting Stockfish’s Elo Rating
Stockfish’s playing strength must be limited using UCI_LimitStrength and UCI_Elo.
Example for Elo 1600:
engine.configure({
    "UCI_LimitStrength": True,
    "UCI_Elo": 1600
})
Once you have won, you can raise the Elo rating and try again. 🚀

🏆 Who Wins?
The winner is the team that can prove a victory against Stockfish at the highest configured Elo rating.
Example:
Team A defeats Stockfish at Elo 1800.
Team B defeats Stockfish at Elo 2000.
Team C defeats Stockfish at Elo 2200.
➡️ Team C wins! 🏆
The victory must be verifiable through the saved game, which must be replayable in the application.
🎁 Prize
Each member of the winning team will receive:
🏆 An exclusive chess set! ♟️
✨ Bonus Prize – Best Playing Experience
It is not just about building the strongest chess engine.
A bonus prize will also be awarded to the team that creates the most visually appealing and best overall user experience. 🎨♟️
Judging criteria include:
Chessboard design
Animations and move visualization
How easy it is to follow and replay a game
Presentation of results and Elo ratings
The overall “wow factor” 🤩
🚀 Good Luck!
Build. Play. Analyze. Improve. Raise your Elo.
And above all:
♟️ BEAT STOCKFISH! 🤖🔥
---

## 📣 Addendum (2026-09-26): what happens at the top

Stockfish's maximum `UCI_Elo` is **3190**, and some teams have already reached it. To decide a winner at that level:

- Once you beat Stockfish at Elo 3190, keep playing at 3190 and measure the **win ratio**: `wins / total games played at 3190` (draws and losses count as games, not wins).
- Also measure **how many moves it takes on average** to finish a game at 3190 (full moves; shorter wins are better).
- Both numbers must be **visible in the GUI**, next to the replayable games.
