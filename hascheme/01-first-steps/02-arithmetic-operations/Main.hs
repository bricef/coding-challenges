module Main where

import System.Environment
import Text.Format
 
main :: IO ()
main = do
    args <- getArgs
    let first = args !! 0
    let second = args !! 1
    let result = (read first :: Integer) + (read second)
    putStrLn (format "Sum is {0}" [show result])
