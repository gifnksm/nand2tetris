// Runs an infinite loop that listens to the keyboard input.
// When a key is pressed (any key), the program blackens the screen,
// i.e. writes "black" in every pixel;
// the screen should remain fully black as long as the key is pressed.
// When no key is pressed, the program clears the screen, i.e. writes
// "white" in every pixel;
// the screen should remain fully clear as long as no key is pressed.

(LOOP)
    // D = M[KBD]
    @KBD
    D = M

    // if (D == 0) goto OFF
    @OFF
    D;JEQ

    // R0 = 0
    @R0
    M = -1 // (!0)

    // goto FILL
    @FILL
    0;JMP

(OFF)
    // R0 = 0
    @R0
    M = 0

    // goto FILL
    @FILL
    0;JMP

(FILL)
    // R1 = SCREEN
    @SCREEN
    D = A
    @R1
    M = D

(FILL_LOOP)
    // if (R1 > KBD) goto LOOP
    @R1
    D = M
    @KBD
    D = D - A // D = R1 - KBD
    @LOOP
    D;JGT

    // M[R1] = R0
    @R0
    D = M
    @R1
    A = M
    M = D

    // R1 = R1 + 1
    @R1
    M = M + 1

    // goto FILL_LOOP
    @FILL_LOOP
    0;JMP
