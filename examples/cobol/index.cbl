       IDENTIFICATION DIVISION.
       PROGRAM-ID. HELLO-WORLD.

       DATA DIVISION.
       WORKING-STORAGE SECTION.

       01  STRINGS.
           03 HELLO                PIC X(11) VALUE
             'Hello from'.
           03 WORLD                PIC X(6) VALUE
             'cobol!'.

       PROCEDURE DIVISION.
       000-MAINLINE SECTION.
       000-START.
           DISPLAY STRINGS.
       000-EXIT.
       EXIT-PROGRAM.



