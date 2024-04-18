
// Futamura Projections
type Program = (ctx:Context) => any
type Compiler = (source: Source) => Bytes

type FutamuraOne = (interpreter: Interpreter, source:string) => Bytes
type FutamuraTwo = (specialiser: FutamuraOne, interpreter: Interpreter) => Compiler;
type FutamuraThree = (interpreter: Interpreter) => Compiler
