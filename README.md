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
or use the convenience script (unix):
```
./solve.sh 756 25 75 3 9 6 10
```
The output looks like this:

![solve](https://user-images.githubusercontent.com/4271248/190327456-307aecb4-02f0-42f5-8f71-377bc96e52e8.png)

The solutions are sorted by shortest number of steps to reach the target. Each solution shows the equation in [reverse Polish notation](https://en.wikipedia.org/wiki/Reverse_Polish_notation), a single line equation and individual steps.

An example with only 5 solutions:

```
$ ./solve.sh -i 917 100 25 5 3 3 1
...
5 solutions found
25 - 5 + (((3 × 100) - 1) × 3) = 917
((((5 + 100) × 3) - 1) × 3) - 25 = 917
(((3 × 100) - 1) × 3) + 25 - 5 = 917
(((3 × 100) - 1) × 3) + 25 - 5 = 917
(((3 × 100) - 1) × 3) - 5 + 25 = 917
```

## Statistical Analysis
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

### Card Combinations

|               | Combinations |
|--------------:|-------------:|
| Overall       |       13,243 |
| 0 big numbers |        2,850 |
| 1 big numbers |        5,808 |
| 2 big numbers |        3,690 |
| 3 big numbers |          840 |
| 4 big numbers |           55 |

### Success Rate

|               | Success Rate |
|--------------:|-------------:|
| Overall       |          91% |
| 0 big numbers |          77% |
| 1 big number  |          95% |
| 2 big numbers |          96% |
| 3 big numbers |          92% |
| 4 big numbers |          88% |

### Hardest Target

|               | Target | Success Rate |
|---------------|-------:|-------------:|
| Overall       |    947 |          68% |
| 0 big numbers |    831 |          29% |
| 1 big number  |    941 |          73% |
| 2 big numbers |    967 |          81% |
| 3 big numbers |    863 |          68% |
| 4 big numbers |    839 |          40% |

### Worst Cards

|               | Card Set         | Success Rate |
|---------------|:----------------:|-------------:|
| 0 big numbers | 3 3 2 2 1 1      |          0%  |
| 1 big number  | 25 3 2 2 1 1     |          27% |
| 2 big numbers | 100 50 2 2 1 1   |          31% |
| 3 big numbers | 100 75 50 2 1 1  |          25% |
| 4 big numbers | 100 75 50 25 1 1 |          36% |

### Best Cards

|               | Success Rate | Card Sets      |
|---------------|-------------:|:--------------:|
| Overall       |         100% | 1,226 / 13,243 |
| 0 big numbers |         100% | 5 / 2,850 <br/> 10 9 8 8 7 6 <br/> 10 9 8 7 5 2 <br/> 10 9 8 7 6 5 <br/> 10 10 9 8 7 6 <br/> 10 9 8 7 4 3 |
| 1 big number  |         100% | 614 / 5,808    |
| 2 big numbers |         100% | 603 / 3,960    |
| 3 big numbers |         100% | 4 / 840 <br/> 100 75 25 9 8 6 <br/> 100 50 25 10 7 6 <br/> 100 75 50 9 8 2 <br/> 100 75 25 9 8 2 |
| 4 big numbers |          98% | 1 / 55 <br/> 100 75 50 25 9 8 |
