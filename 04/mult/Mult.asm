// Multiplies R0 and R1 and stores the result in R2.
// (R0, R1, R2 refer to RAM[0], RAM[1], and RAM[2], respectively.)
//
// This program only needs to handle arguments that satisfy
// R0 >= 0, R1 >= 0, and R0*R1 < 32768.

    // R2 = 0
    @R2
    M=0

(LOOP)
    // if (R1 == 0) goto END
    @R1
    D=M
    @END
    D;JEQ

    // R2 = R2 + R0
    @R0
    D=M
    @R2
    M=D+M

    // R1 = R1 - 1
    @R1
    M=M-1

    // goto LOOP
    @LOOP
    0;JMP
(END)
