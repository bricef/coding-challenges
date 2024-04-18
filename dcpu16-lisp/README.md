DCPU16-Lisp
===========

A lisp compiler for the DCPU16 processor, with a DCPU-16 assembly assembler, interpreter, emulator and disassembler.

Usage:
------

Compiling beetle-lisp to DCPU-16 machine code:

    compile.py < program.bl \
        | assemble.py > program.obj


Running the machine code on the emulator:

    compile.py < program.bl \
        | assemble.py \
        | dcpu-emulate.py

Running the assembly on the DCPU-16 beetle assembly interpreter:

    compile.py < program.bl \
        | basm-interpret.py

Running a beetle-lisp program directly in the beetle interpreter (Beetle Lisp Interactive Console, 'BLIC'):

    blic.py < program.bl


### Future expansion ###

Accept other languages above the toolchain.
  
    < program.ext tokenize.py --ext \
        | astify.py --ext \
        | compile.py \
        | assemble.py > program.obj

Tools:
------

###tokenize.py
**[future]** Takes a program in an other language and tokenises it (token stream is long list of s-exprs) 

###astify.py
**[future]** Turns a token stream into a valid beetle-lisp program.

###compile.py
Compiles valid beetle-lisp program into DCPU16 assembly from stdin to stdout

###assemble.py
Assembles a valid DCPU (.dasm or .dasm16) program into binary from stdin to stdout 

###dcpu-emulate.py
Takes a valid binary DCPU16 instruction stream and interprets it. 

###basm-interpret.py
Takes a valid beetle assembly program on stdin and interprets it.
Equivalent to `assemble.py | dcpu-emulate.py`, but faster.

###blic.py
Beetle lisp interpreter. Takes a valid beetle lisp program on stdin 
and interprets in within the confines of a virtual DCPU-16.
Equivalent to `compile.py | assemble.py | dcpu-emulate.py`, but faster.

##Note on emulation/interpretation:
The specifications for the screen/keyboard/removable memory/etc... are work in progress.

Beetle-Lisp:
------------

    (defun my-add \[x y]
       (+ x y))
