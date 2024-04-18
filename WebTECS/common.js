var waxeye = require('waxeye');
var _ = require('lodash');

var PROG_SUFFIX = (new Date()).getTime().toString();

function Processor(parser, handlers){
	
	var self = {
		utils: {
			stringify: function (AST){
				return AST.children.join('');
			},
			logAST: function (AST){
				console.log(AST.type.toUpperCase()+"["+utils.stringify(AST)+"]");
			},
			getSuffix: function(){
				return PROG_SUFFIX;
			},
			FATAL: function (msg, obj) {
				console.error("FATAL EXCEPTION: "+msg+"\n"+obj.toString());
				process.exit(1);
			}
		},
		handlers: handlers
	};



	function visit (node){
		if(node.type in self.handlers){
			return self.handlers[node.type](node, self);
		}else if(node instanceof waxeye.AST){
			return node.children.map(function(node){ return visit(node); });
		}else{
			utils.FATAL("Could not deal with node: ", node);
		}

	};

	function process (data){
		var result = parser.parse(data);
		
		if (result instanceof waxeye.AST) {
			// We could indent based on nesting in the result...
			//return _.flatten([/*Initialisation,*/ visit(result, handlers)/* ,EndOfProgram*/]).join('\n')+"\n";
			return visit(result);
		}else {
			if (result instanceof waxeye.ParseError) {
				self.utils.FATAL("Parse error occured: ", result);
			}
			else {
				self.utils.FATAL("Null or empty file");
			}
		}
		return ""
	}
	
	self.visit = visit;
	self.process = process;
	
	return self;
}

function make_program(processor, config){
	var fs = require("fs");
	var util = require("util");
	var program = require('commander');

	if(!config){
		config = {};
	}

	program
	  .version('0.0.1')
	  .option('-i, --input [infile]', (config.input_format)?("Input file ("+config.input_format+")"):"Input file")
	  .option('-o, --output [outfile]', (config.output_format)?("Output file ("+config.output_format+")"):"Output file")
	  .parse(process.argv);

	
	var input,output;

	if(program.input){
		input = fs.createReadStream(program.input);
	}else{
		input = process.stdin;
	}


	if(program.output){
		output = fs.createWriteStream(program.output);
	}else{
		output = process.stdout;
	}

	input.resume();
	input.setEncoding('utf8');

	var data = '';
	input.on('data', function(buf){
		data += buf;
	});
	input.on('end', function(){
		output.write(processor.process(data));
	});
}

exports.Processor = Processor;
exports.make_program = make_program;