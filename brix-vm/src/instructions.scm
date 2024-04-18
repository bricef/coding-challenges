#|

# Brix VM instruction set

## Summary

The Brix VM is a stack-based virtual machine.

* 64+8 bit wide operand stack (64 bits data, 8 bits type).
* 0-operand 16 bit instructions. (2 bytes)
* 32 bit total virtual address space
* 8bit (byte) addressable memory (Implies 2^32 adrresses ~ 4GiB per threadlet)

It supports multiple stack programs (threadlet) running concurrently with completely independent runspaces and will task-switch between program.

Cross-threadlet communication is via message passing, addressed by threadlet index, starting at 0.

It supports up to 2^32 concurrent threadlets.

Uniform data and code space. A program's code is loaded at the end of its memory space. This implies a maximum program size of 2^32 words.


|#


(deftypes
    ; Types are 8-bit wide uints.
    (float "64 bit wide IEE floating point")
    (uint "64 bit wide unsigned integer")
    (int "64 bit signed integer in two's complement")
    (raw "64 bit raw")
    (address "Location in memory")
    (slice "Segment of memory")
    (bool "True or false")
    (empty "Represents existence but no content")
    (nothing "Represents lack of existence")
    (opcode "A machine instruction")
    (block "A slice of memory that contains instructions")
    (type "A bare Brix type")
)

(defexceptions
    ; Represented as uint constants
    (INSTRUCTION_FAULT "Invalid instruction.")
    (BAD_SLICE "Reference invalid.")
    (STACK_EXHAUSTED "No operand available on the stack for operation")
    (STACK_FULL "No more space on the operand stack")
    (UNREACHABLE "Executed unreachable instruction")
    (NOT_IMPLEMENTED "Instruction is valid, but not implemented.")
    (WRITE_PROTECTED "Attempted to write to a segment of protected memory")
)

(defops 
    (section operations
        (noop 
            "Does nothing"
            [] -> []
            #x0000)
        
        (push-raw
            "Push a raw value to the operand stack"
            [] -> [raw]
            #xFEED)
        
        (push-address
            "Push an address to the operand stack"
            [] -> [address]
            #xFADD)

        (push-(T)
            "Push value with type"
            [] -> [T]
            (concat #xFF T))

        (if
            "Choose a value according to boolean"
            [A, B, bool] -> [A if bool else B])

        ; Numerical operations
        (add-(T:float|int|uint|address)
            "Add two values together. Must be of the same type."
            [T, T] -> [T])

        (sub-(T:float|int|uint|address)
            "Substract a value from another."
            [T, T] -> [T])

        (mul-(T:float|int|uint)
            "Multiply two numbers"
            [T, T] -> [T])

        (div-(T:float|int|uint)
            "Divide two numbers"
            [T, T] -> [T])
        
        (rem-(T:float|int|uint)
            "Find the remainder from division"
            [T, T] -> [T])

        ; Logical operations
        (not
            "Bitwise inversion"
            [T:bool|uint|int|raw] -> [T])
        
        (or 
            "Bitwise or"
            [T:bool|uint|int|raw] -> [T])
        
        (xor
            "Bitwise exclusive or"
            [T:bool|int|uint|raw] -> [T])
        
        (and
            "Bitwise and"
            [T:bool|int|uint|raw] -> [T])

        ; bitwise operations
        (rotate-(direction)
            "Bitwise rotate in *direction (right|left)"
            [T] -> [T])
        
        
        (shift-(direction)-(bitfill)
            "Bitwise shift in *direction (right|left) fill with *bitfill (one|zero)"
            [T] -> [T])
        
        (popcount
            "Number of '1' bits"
            [T] -> [uint])
        
        ; Types
        (coerce-(T)
            "Coerce the type of stack's topmost operand"
            [X] -> [T]
            (concat #xAB T))
        
        (type-of 
            "Get type of value"
            [T] -> [type])
        
        ((from)-to-(to)
            "Where *from and *to are types. Potentially lossy."
            (supported:
                (float int)
                (float uint)
                (float bool)
                (uint float)
                (uint int)
                (uint bool)
                (uint address)
                (address uint)
                (uint opcode)
                (int float)
                (int uint)
                (int address)
                (int bool)
                (address int)
                (slice block)
                (block slice))
            [A] -> [B]
        )
        
        ; slices

        (start-of
            "The start of a slice"
            [slice] -> [address])
        
        (end-of
            "End of slice (last byte of slice)"
            [slice] -> [address])
        
        (size-of
            "The length of a slice in bytes"
            [slice] -> [uint])
        
        (build-slice
            "Build a slice from addresses"
            [address, address] -> [slice])
    )


    (section stack
        (clear
            "Clear the stack"
            [*] -> [])

        (dup 
            "Duplicate the topmost operand in the stack"
            [T] -> [T, T])
        
        (dup-many
            "Duplicate many operands at once"
            [vec(*), uint] -> [vec(*), vec(*)])

        (drop
            "Remove topmost operand"
            [T] -> [])

        (drop-many
            "Remove many operands from the stack"
            [vec(*), uint] -> [])

        (save-stack
            "Saves a section of the current stack to a memory slice"
            "uint operand is number of operand to save"
            [vec(T), uint, address] -> [])
    )

    (section memory

        ; Load operations
        (read-raw
            "Reads a raw value from memory"
            [address] -> [raw] )

        (read-as-(T)
            "Read a typed value from memory"
            [address] -> [T])

        (read-vec-raw 
            "Read a slice onto the stack, word by word."
            [slice] -> [vec(raw)])

        (read-vec-as-(T)
            "Read a slice onto the stack, word by word with a given type"
            [slice] -> [vec(T)])
        
        (read-byte
            "Read the first byte at address"
            [address] -> [uint])
        
        (load-slice-(T)
            "Load a slice of memory into the operand stack"
            "Will interpret the memory as vector of 64 bit operand."
            "Type for operand will be *type"
            [slice] -> [T]
            (concat #x0A T))
        
        ; Store operations
        (write
            "Write some data to memory"
            [T, address] -> [])
        
        (write-byte
            "Write a single byte to memory"
            [uint, address] -> [])

        ; Direct memory manipulation
        (reverse-bytes
            "reverse a slice of memory in place, by bytes"
            [slice] -> [])
        
        (copy
            "Copy a segment of memory to another"
            [slice, slice] -> [])
        
        (zero
            "Zero out a segment of memory"
            [slice] -> [])

    )

    (section thread
        (yield
            "Cooperatively yield execution to another threadled"
            [] -> [])

        (get-current-thread
            "Get the current thread name, as a uint"
            [] -> [uint])
        
        (get-number-executing-thread
            "Get the current number of active threads"
            [] -> [uint])
        
        (get-thread-ops
            "Get the current number of operations executed by this thread."
            [] -> [uint])
        
        (spawn
            "Spawn a new threadlet with block as its initial code"
            "Returns the new thread's name as a uint"
            [block] -> [uint])
        
        (kill 
            "End the execution of a threadlet by name"
            "Cannot kill threadlet with a name lower than self"
            "Killing thread 0 shuts down the machine"
            [uint] -> [])

        (message
            "Message named thread with contents of slice"
            "blocks until received."
            [slice, uint] -> [])
        
        (has-message
            "Does the current thread have a message?"
            "If it does, uint is size of message in bytes otherwise 0"
            [] -> [uint, bool])
        
        (receive
            "Copy the awaiting message into the slice"
            "Message is truncated to fit slice."
            [slice] -> [])
    )

    (section program
        (get-code
            "Get the current thread's initial code slice"
            [] -> [slice])

        (register-handler
            "Register code to execute if trap value uint is raised"
            [uint, block] -> [])
        
        (jump
            "Move execution pointer to address"
            [address] -> [])
        
        (current-address
            "Add the current execution pointer to the operand stack"
            [] -> [address])
        
        (unreachable
            "Instruction will trigger an unreachable exception"
            [] -> [])
    )
        
)