
# Rust Embedded CS510

## Homework 1: Game of Life

## Student John Eley

## Program Assignment

Write a program that interactively plays Game of Life on your MB2.

### Specs:

- The program runs the game at 10 frames per second (updates once per 100ms).
- The program starts with a random board.
- While the A button is held, the board is re-randomized every frame.
- Ignore the B button if the A button is pressed
- If the B button is pressed, the board is "complemented": every "on" cell is turned "off" and every "off" cell is turned "on". The B button is then ignored for 5 frames (0.5s).
- If the program reaches a state where all cells on the board are off, the program waits 5 frames (0.5s). If it has not received a button press, it then starts with a new random board.

Otherwise, normal Life steps are taken.

### Results:

The program ran well. However, I initially tried to use Vec arrays to store the information for every cell. It was a complex way to do implement the game. I was influenced by more general methods that could expand to arbitrary world sizes. I eventually dumped this method and used Bart's life code. it was much more elegant. 

The button implementations went fine. The ignore logic was a bit convoluted.

I added a serial display in addtion to the LED display. In order to see it, connect the board with a terminal program set to 115200,8,n,1. The terminal should display a copy of what is displayed on the LED array. 

I also implemented the hardware RNG for the random functions. It was pretty straight forward. 

One odd thing... I used nested for loops to interate through the array. Clippy advised a different method using an iterator style. That worked ok, except for specific loops where it broke execution. FOr that reason I disabled the clippy suggestion for that. 

