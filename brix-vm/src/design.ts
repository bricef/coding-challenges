type Source = string
type Bytes = Uint8Array

// VM implementation
interface Memory{}
interface Stack{}


interface Grammar {}
interface Terminal {}
interface NonTerminal {}

interface Instruction {
  Bytes(): Bytes
  Asm(): string
  Exec(memory:Memory, stack:Stack): Stack
}
interface Bytecode {
  Instructions(): Array<Instruction>
  AsBytes(): Bytes
  AsAsm(): string
}
interface Context {}
interface Token {}
type ASTNode = Terminal | NonTerminal

// Frontend
type Tokeniser = (source: Source) => Array<Token>
type TokenParser = (tokens: Array<Token>) => ASTNode
type Parser = (source: Source) => ASTNode
type ParserCompiler = (grammar: Grammar) => Parser

// Middle End
type Transform = (ast: ASTNode) => ASTNode

// Backend
type ASTCompiler = (ast: ASTNode) => Bytes
type BytecodeCompiler = (ast: ASTNode) => Bytecode

// Interpreters
type Interpreter = (source: Source, ctx: Context) => any
type ASTInterpreter = (ast: ASTNode, ctx: Context) => any
type BytecodeInterpreter = (bytecode: Bytecode, ctx: Context) => any

// Helpers
type Dissasembler = (bytes: Bytes) => ASTNode
type BytecodeDisassembler = (bytecode: Bytes) => Bytecode

// compositions
// Compiler(ParserCompiler(grammar)) -> String => String
