
enum constants {
    // Traps (uint)
    INSTRUCTION_FAULT,
    BAD_REF,
    STACK_EXHAUSTED,
    STACK_FULL,
    CODE_EXHAUST,
    UNREACHABLE,
    NOT_IMPLEMENTED,
    IMMUTABLE,

    // Bools (bool)
    FALSE,
    TRUE,

    // Nil (nil)
    NIL,

    // Types (type)
    T_float,
    T_int,
    T_uint,
    T_bool,
    T_block,
    T_raw,
    T_iraw,
    T_iblock,
    T_nil,
    T_address,
    T_type,
    T_opcode,
}

const instructions = {
    stack: [
        "noop",
        "push",
        "dup",
        "drop",
        
        // Numeric operations 
        "add_float",
        "sub_float",
        "mul_float",
        "div_float",

        "add_int",
        "sub_int",
        "mul_int",
        "div_int",
                
        "add_uint",
        "sub_uint",
        "mul_uint",
        "div_uint",
        
        //bitwise operations
        "not",
        "rotr",
        "rotl",
        "lshift",
        "rshift",
        "xor",
        "or",
        "and",
        "popcount",

        // type operations
        "typeof",

        // Type convertions
        "conv_float_uint",
        "conv_float_int",
        "conv_float_bool",
        "conv_uint_float",
        "conv_uint_int",
        "conv_uint_address",
        "conv_uint_bool",
        "conv_uint_opcode",
        "conv_int_float",
        "conv_int_uint",
        "conv_int_address",
        "conv_int_bool",
        "conv_address_uint",
        "conv_address_int",
        "coerce",

        // Functions on ranges (raw, iraw, block, iblock)
        "start_address",
        "end_address",
        "sizeof",
        "build_raw",
        "build_iraw",
        "build_block",
        "build_iblock",

        // conditional
        "if",
    ],
    memory: [
        'read_raw', // [address] -> [raw]
        'read_as', // [type<T>,address] -> [T], which is equivalent to read_raw, followed by coerce
        'read_vec_as', // [type<T>, range] -> [vec(T)]
        "reverse",
        "get",
        "put",
        "load",
        "store",
        "copy",
        "zero",
        "write_ops",
        "read_ops",
    ],
    thread: [
        "yield",
        "push_current_thread",
        "push_total_threads",
        "msg",
        "kill",
        "spawn",
        "push_thread_clock",
        "has_message",
        "load_message",
    ],
    program: [
        "register_interrupt",
        "push_current_block",
        "jump",
    ],
    machine: [
        "push_max_concurrency",
        // "register_microcode",
    ]
}