#!/usr/bin/env python

import fileinput
import StringIO
import re
import sys
import array

linepat = re.compile("(\:[a-zA-Z0-9-_]*)?\s*([a-zA-Z]*)\s+(.*?),(.*)(;.*)?")
datpat = re.compile("(\:[a-zA-Z0-9-_]*)?\s*([a-zA-Z]*)\s+(.*)")

class ParseError(Exception):
  def __init__(self, msg, linenum):
    self.msg = msg
    self.line=linenum

def datparse(line, linenum):
  """returns data in raw format"""
  label, op, rest = re.match(datpat, line).groups() 

  datbuffer = array.array('H')
  
  START = 1
  STRING = 2
  OUT = 3
  NUMBER = 4
  END = 5
  ERROR = -1
 
  tbuf = []

  def instart(c):
    if c == '"':
      return STRING
    elif re.match("\s", c):
      return OUT
    elif c == ",":
      raise ParseError("Bad syntax. Comma found straight after opcode.", linum)
    elif c == ";":
      return END
    else:
      tbuf.append(c)
      return NUMBER
  
  def instring(c):
    if c == '"':
      if datbuffer[-1]== "\\":
        datbuffer[-1]=c
        return STRING
      else:
        return OUT
    else:
      datbuffer.append(ord(c))
      return STRING 

  def inout(c):
    if c == '"':
      return STRING
    elif re.match("\s", c):
      return OUT
    elif c == ",":
      return OUT
    elif c == ";":
      return END
    else:
      tbuf.append(c)
      return NUMBER

  def innumber(c):
    if re.match("\s", c) or (c in ",;"):
      num = "".join(tbuf)
      c_num = int(num, 0)
      if c_num > 0xffff:
       raise ValueError("Integer %s is too big. Max size is 0xffff (65535)")
      else:
        datbuffer.append(c_num)
        del tbuf[:]
        return OUT
    else:
      tbuf.append(c)
      return NUMBER
  
  states={
    START: instart,
    STRING:instring,
    OUT:inout,
    NUMBER:innumber
  }
  
  state = START
  for char in rest:
    state = states[state](char)
    if state == ERROR:
      raise ParseError("Parse error.", linenum)
    if state == END:
      break
  datbuffer.byteswap()
  return datbuffer


line_i = 0
for line in fileinput.input():
  line_i +=1
  if line:
    label, op, a, b, comment = re.match(linepat, line).groups()
    if op == "dat": # we treat dat specially, since we have to do string parsing
      sys.stdout.write(datparse(line, line_i).tostring())
     


