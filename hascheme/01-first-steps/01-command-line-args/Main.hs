module Main where

import System.Environment
import Text.Format
 
main :: IO ()
main = do
    args <- getArgs
    let first = args !! 0
    let second = args !! 1
    putStrLn (format "Hello, {0} {1}" [first, second])
