# countdown_numbers
A Countdown numbers game solver written in Rust.

## The Countdown numbers game
[Countdown](https://en.wikipedia.org/wiki/Countdown_(game_show)) is a British Channel 4 game show which has been running since November 1982. It was based on the French TV show [Des Chiffres Et Des Lettres](https://en.wikipedia.org/wiki/Des_chiffres_et_des_lettres).

In the numbers game on the show 6 cards are selected from a set of 24 cards. 4 of the cards are 'big numbers' - 100, 75, 50 and 25, and the remaining 20 cards are 2 each of 1-10.

A random target number is then selected in the range 100 to 999. Players must try and reach the target number using the selected numbers using addition, subtraction, multiplcation and division. The calculation must always remain positive and no fractions are allowed. [Rules](https://en.wikipedia.org/wiki/Countdown_(game_show)#Numbers_round)

## Game solver
To solve a numbers game the `solve` binary is run:
```
cargo run --release --bin solve -- 756 25 75 3 9 6 10
```
or use the covenience script (unix):
```
./solve.sh 756 25 75 3 9 6 10
```
The output looks like this:

![solve](https://user-images.githubusercontent.com/4271248/190327456-307aecb4-02f0-42f5-8f71-377bc96e52e8.png)

The solutions are sorted by shortest number of steps to reach the target. Each solution shows the equation in [reverse Polish notation](https://en.wikipedia.org/wiki/Reverse_Polish_notation), a single line equation and individual steps.

## Statistical analysis
The `solve_all` binary will produce a file for each combination of cards possible containing details of all of the possible targets for the chosen cards.
To run:
```
cargo run --release --bin solve_all
```
Or use the convenience scripts:
```
./solve_all.sh
```
The `stats` binary can then be run to post-process the output directory and produce overall statistics.
```
cargo run --release --bin stats solutions-NC-100-75-50-25-10-10-9-9-8-8-7-7-6-6-5-5-4-4-3-3-2-2-1-1
```
Output from this is included in the repostitory.
