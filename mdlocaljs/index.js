#!/usr/bin/env node
import remarkParse from 'remark-parse'
import remarkStringify from 'remark-stringify'
import remarkFrontmatter from 'remark-frontmatter'
import remarkGfm from 'remark-gfm'
import {unified} from 'unified'
import {visit} from 'unist-util-visit'
import fs from "fs-extra";
import { program } from "commander";

async function downloadImage(url, location) {
    let filename = url.split('/').pop();
    fs.ensureDir(location);
    let relurl = `${location}/${filename}`;
    
    let data = await fetch(url);
    let buffer = await data.arrayBuffer();
    await fs.writeFile(relurl, new Uint8Array(buffer));
    return relurl;
}

function remarkImageLocaliser({directory}) {
  /**
   * @param {import('mdast').Root} tree
   */
  return async function (tree) {
    // See https://github.com/syntax-tree/unist-util-visit-parents/issues/8#issuecomment-1413405543
    // for how to do async transformations.
    let images = [];

    visit(tree, "image", function (node) {
      images.push(node);
    });

    for(const img of images){
        let url = img.url;
        let localurl = await downloadImage(url, directory);
        img.url = localurl;
    }

    return tree;

  }
}


(async function main(){
  
  program
    .name('mdlocaljs')
    .description('A utility to process markdown files and download images locally.')
    .version('0.0.1')
    .option('-i, --input <filepath>', 'File path to process', 'stdin')
    .option('-d, --directory <dirpath>', 'Directory path to download images to', './images')
    .option('-o, --output <outputpath>', 'Output path', 'stdout')
    .action(async (options) => {
      console.log(options);
      let input;
      let output;
      if (options.input === 'stdin') {
        input = process.stdin;
      }else{
        input = await fs.readFile(options.input);
      }

      if (options.output === 'stdout') {
        output = process.stdout;
      } else {
        output = await fs.createWriteStream(options.output);
      }

      let t = await unified()
        .use(remarkParse)
        .use(remarkFrontmatter)
        .use(remarkGfm)
        .use(remarkImageLocaliser, {directory: options.directory})
        .use(remarkStringify);

      let result = await t.process(input);

      output.write(String(result));
    });
  
    program.parse(process.argv);
})();