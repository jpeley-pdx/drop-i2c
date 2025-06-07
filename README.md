
# Rust Embedded CS510

## Homework 2: Drop

## Student John Eley

## Program Assignment

Write a Rust program for the MicroBit with the following specification.

While the board is not falling (IMU ≥ 0.5g) it should be silent, and the board should display a single dot on the center LED.

While the board is falling (IMU < 0.5g) it should "yell" with a 1KHz square-wave tone on the speaker and show an exclamation point on the display:

```
..*..
..*..
..*..
.....
..*..
```

### Results:

The program ran well. I decided to embelish a little bit. I used the PWM function to vary the frequency of the sound. The board will now make a descending sound if going down, and an ascending sound when going up. It's pretty cool. 

I didn't add code to detect tilt. For now, if the board is tilted, the program things it is falling and sounds the descending sound. 