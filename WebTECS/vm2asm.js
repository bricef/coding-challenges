#!/usr/bin/env node


var _ = require('lodash');
var common = require('./common');
var vmparser = require('./parsers/vm_parser')

var Initialisation = [
	"// StackPointer initialisation",
    "@SP",
    "@256",
    "D=A",
    "@SP",
    "M=D"
];


var EndOfProgram = [
	"// End of program",
	"(ENDPROG)",
	"@ENDPROG",
	"0;JMP"
];

var POPD = [
	"@SP",
	"A=M",
	"D=M",
	"@SP",
	"M=M-1"
]

var operators = {
	add:[
		"// ADD",
		POPD,
		"@SP",
		"A=M-1",
		"M=D+M"
	],
	sub:[
		"// SUB",
		POPD,
		"@SP",
		"A=M-1",
		"M=D-M"
	]	
	/*
	"neg":,
	"eq":,
	"gt":,
	"lt":,
	"and":,
	"or":,
	"not":,
	*/
};


var segment = {
	argument:["@ARG", "A=M"],
	local:["@LCL", "A=M"],
	static:[],
	contant:function(index){
		if(index > 0xffff){
			FATAL("Cannot access constant  segment > "+0xffff);
		}
		return "@"+index
	},
	this:["@THIS", "A=M"],
	that:["@THAT", "A=M"],
	pointer:["@3"],
	temp:["@5"]

}

var _handlers = {
	operator: function(AST, processor){
		var op = processor.utils.stringify(AST);
		if(!(op in operators)){
			processor.utils.FATAL("Operator Not handled: "+op);
		}
		return operators[op];
	},
	function: function(AST, processor){
		var name = processor.utils.stringify(AST.children[0]);
		var nLocals = parseInt(processor.utils.stringify(AST.children[1]));
		return [
			"// Function "+name+" "+nLocals
		];
	},

	push: function(AST, processor){
		var segment = processor.utils.stringify(AST.children[0]);
		var index = parseInt(processor.utils.stringify(AST.children[1].children[0]));
		return [
			"// Push "+segment+" "+index,
			"//..."
		];
	},
	pop: function(AST, processor){
		var segment = processor.utils.stringify(AST.children[0]);
		var index = parseInt(processor.utils.stringify(AST.children[1].children[0]));
		return [
			"// Pop "+segment+" "+index
		];
	},
	return: function(AST, processor){
		return [
			"// return "
		];
	},
	program: function(AST, processor){
		 return _.flatten([Initialisation, AST.children.map(function(node){ return processor.visit(node); }), EndOfProgram]).join('\n')+"\n";
	}
};

var parser = new vmparser.VmParser();
var processor = new common.Processor(parser, _handlers);

common.make_program(
	processor,
	{
		input_format: "Hack VM bytcode",
		output_format: "HACK assembly"
	}
);
