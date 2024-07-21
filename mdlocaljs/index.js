#!/usr/bin/env node
import util from 'util'
import path from 'path'
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
    let image_file_location = `${location}/${filename}`;
    let data = await fetch(url);
    let buffer = await data.arrayBuffer();
    await fs.writeFile(image_file_location, new Uint8Array(buffer));
    return image_file_location;

}

function is_remote_url (url) {
  return url.startsWith('http://') || url.startsWith('https://');
}

function remarkImageLocaliser({filepath, download_directory, }) {
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
        if(is_remote_url(url)){
          try{
            let image_file_location = await downloadImage(url, download_directory);
            let localurl = path.relative(path.dirname(filepath), image_file_location);
            img.url = localurl;
          }catch(e){
            console.error(`Failed to download image ${url} on line ${util.inspect(img.position.start.line)} ${e}`);
          }
        }
        
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
    .option('-r', '--replace', 'Modify the markdown file in place')
    .action(async (options) => {
      // console.log(options);
      let input;
      let output;
      let root_path;
      if (options.input === 'stdin') {
        input = process.stdin;
        root_path = process.cwd();
      }else{
        input = await fs.readFile(options.input);
        root_path = options.input;
      }

      let t = await unified()
        .use(remarkParse)
        .use(remarkFrontmatter)
        .use(remarkGfm)
        .use(remarkImageLocaliser, {filepath: root_path, download_directory: options.directory})
        .use(remarkStringify);

      if(options.r){
        console.log(`Modifying ${options.input} in place.`)
        output = await fs.createWriteStream(options.input)
      }else if(options.output == 'stdout') {
        output = process.stdout;
      }else{
        output = await fs.createWriteStream(options.output);
      }

      fs.ensureDir(options.directory);

      let result = await t.process(input);

      output.write(String(result));
    });
  
    program.parse(process.argv);
})();