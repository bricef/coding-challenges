package main

import (
	"flag"
	"fmt"
	"io"
	"math"
	"os"
	"strconv"

	"github.com/eapache/channels"
	"github.com/tebeka/deque"
)

func exitOnError(err error) {
	if err != nil {
		panic(err)
	}
}

// lineIndexes will stream line start indexes for file file.
// Buffers reads. Suitable for very large lines.
func lineIndexes(file *os.File) <-chan int64 {

	out := make(chan int64)

	go func() {
		bufSize := 1024
		buffer := make([]byte, bufSize)
		out <- 0 // First line

		for {
			currentFileIndex, err := file.Seek(0, os.SEEK_CUR)
			exitOnError(err)

			bytesRead, err := file.Read(buffer)
			if err != nil {
				if err == io.EOF {
					break
				}
				exitOnError(err)
			}

			for bufferIndex := 0; bufferIndex < bytesRead; bufferIndex++ {
				if buffer[bufferIndex] == '\n' {
					filePos := currentFileIndex + int64(bufferIndex) + 1 // next line's start index.
					out <- filePos
				}
			}
		}
		close(out)
	}()
	return out
}

func slidingWindow(in <-chan int64, windowsize int) <-chan []int64 {
	out := make(chan []int64)
	go func() {
		dq := deque.New()
		for v := range in {
			dq.Append(v)
			if dq.Len() == windowsize {
				slice := make([]int64, windowsize)
				for i := 0; i < windowsize; i++ {
					v, err := dq.Get(i)
					slice[i] = v.(int64)
					exitOnError(err)
				}
				out <- slice
				dq.PopLeft()
			}
		}
		// check deque for content
		if dq.Len() >= windowsize {
			slice := make([]int64, windowsize)
			for i := 0; i < windowsize; {
				v, err := dq.Get(i)
				slice[i] = v.(int64)
				exitOnError(err)
			}
			out <- slice
		}
		close(out)
	}()
	return out
}

// Enumerated is simply a way of keeping the index of items in a channel
type Enumerated struct {
	position int64
	lines    []int64
}

func enumerate(in <-chan []int64, startAt int64) <-chan Enumerated {
	out := make(chan Enumerated)
	go func() {
		// We need the index of the streams so we can put them back together
		index := startAt

		for slice := range in {
			out <- Enumerated{position: index, lines: slice}
			index++
		}
		close(out)
	}()
	return out
}

// FirstTo will send the first element from the output channel
// and send it to the channel given as arg, then close that channel
func FirstTo(out channels.SimpleInChannel) channels.SimpleInChannel {
	in := channels.NewInfiniteChannel()
	go func() {
		v := <-in.Out()
		out.In() <- v
		out.Close()
	}()
	return in
}

// LastTo will send the last elemnt of the output's channel to the
// out argument channel
func LastTo(out channels.SimpleInChannel) channels.SimpleInChannel {
	in := channels.NewInfiniteChannel()
	go func() {
		var last interface{}
		for {
			v, ok := <-in.Out()
			if !ok {
				out.In() <- last
				out.Close()
				break
			}
			last = v
		}
	}()
	return in
}

// average will calculate the average of the values passed in
// it will ignore NaN values and if all values passed in are NaN,
// it will return 0.0
func average(vals ...float64) float64 {
	var acc float64
	var nums float64
	for _, v := range vals {
		if !math.IsNaN(v) {
			acc += v
			nums++
		}
	}
	if nums == 0 {
		return 0.0 // We can't figure out a value. default to 0.0
	}
	return acc / nums
}

func writeCSVRow(filename string, values chan string) {
	go func() {
		outfile, err := os.Create(filename)
		exitOnError(err)
		defer outfile.Close()

		for v := range values {
			outfile.WriteString(v)
			outfile.WriteString(",")
		}
	}()
}

type BufferedLineReader io.Reader

type BufferedLineReaderData struct {
	file      io.ReaderAt
	lineStart int64
	lineEnd   int64
	buffer    []byte
	fileIndex int64
	bufIndex  int
	eofIndex  int
	EOF       bool
}

func NewBufferedLineReader(file io.ReaderAt, lineStart int64, lineEnd int64, bufferSize int) BufferedLineReader {
	return &BufferedLineReaderData{
		file:      file,
		lineStart: lineStart,
		lineEnd:   lineEnd,
		fileIndex: lineStart,
		buffer:    make([]byte, bufferSize),
		EOF:       false,
	}
}

func (br *BufferedLineReaderData) fillBuffer() (int, error) {
	if br.EOF {
		return 0, io.EOF
	}

	readSize := br.bufIndex - 1 // how much stale data is in the buffer
	//Q: will this take us over the line limit?
	if 

	// Step 1: move unread data to the beginning of the buffer
	copy(br.buffer, br.buffer[br.bufIndex:cap(br.buffer)])
	fileWriteIndex := cap(br.buffer) - br.bufIndex // Where to write new data from
	br.bufIndex = 0

	// Step 2: read data from file into rest of buffer

	fileBytes := make([]byte, readSize)
	bytesRead, err := br.file.ReadAt(fileBytes, br.fileIndex)
	if err != nil && err != io.EOF {
		panic(err)
	}
	br.fileIndex += bytesRead
	copy(br.buffer[fileWriteIndex:], fileBytes)

}

func (br *BufferedLineReaderData) Read(outBuf []byte) (int, error) {
	if br.EOF {
		return 0, io.EOF
	}

	if br.fileIndex == br.lineStart {
		// we've never read before. Let's start

	}

	oldBufIndex := br.bufIndex

	if cap(outBuf)+br.bufIndex > br.eofIndex {
		// we're trying to read past EOF
		outBuf = br.buffer[br.bufIndex:br.eofIndex]
		len := br.eofIndex - br.bufIndex
		//close this reader
		br.bufIndex = br.eofIndex
		br.EOF = true
		return len, io.EOF
	} else if cap(outBuf)+br.bufIndex > len(br.buffer) {
		// We're trying to read past the end of the buffer

	} else {
		br.bufIndex += cap(outBuf)
		outBuf = br.buffer[oldBufIndex:br.bufIndex]
		return br.bufIndex - oldBufIndex, nil
	}

}

func (br *BufferedLineReaderData) ReadByte() (byte, error) {
	return 0, nil
}

func emitCSVRowDatums(file *os.File, rowStart int64, rowEnd int64) <-chan string {
	out := make(chan string)
	go func() {
		// fmt.Println("Begin row emitter", rowStart, rowEnd)
		readOffset := rowStart
		bufSize := 10
		buffer := make([]byte, bufSize)
		var previousBufferRemaining []byte

		for {
			// read to buffer
			bytesRead, err := file.ReadAt(buffer, readOffset)
			if err != nil && err != io.EOF {
				exitOnError(err)
			}

			// process buffer
			tokenStart := 0
			bufferIndex := 0
			// fmt.Println("reading buffer")
			for ; bufferIndex < bytesRead; bufferIndex++ {

				if buffer[bufferIndex] == ',' || buffer[bufferIndex] == '\n' {
					tokenSlice := []byte{}
					thisBuffer := buffer[tokenStart:bufferIndex]
					if len(previousBufferRemaining) > 0 {
						tokenSlice = append(tokenSlice, previousBufferRemaining...)
					}
					tokenSlice = append(tokenSlice, thisBuffer...)
					out <- string(tokenSlice)
					tokenStart = bufferIndex + 1
					// fmt.Printf("previous: '%s', thisBuffer: '%s', tokenSlice: '%s', \n", string(previousBufferRemaining), string(thisBuffer), string(tokenSlice))
				}
			}
			if tokenStart < bufferIndex {
				previousBufferRemaining = buffer[tokenStart:len(buffer)]
			}

			readOffset += int64(bytesRead)

			// if EOF, we're done after this buffer.
			if err == io.EOF {
				break
			}
		}
		if len(previousBufferRemaining) > 0 {
			// we have a final token
			out <- string(previousBufferRemaining)
		}

		close(out)
	}()
	return out
}

func xxx(tokenstreams ...<-chan string) {
	//n := len(tokenstreams)
	// n is odd? no-bail
	// set n x n matrix
	//push (n-1)/2 into matrix
	// while streams
	// push column into matrix
	// calculate output
	// until (n-1)/2 left
	// pop column out of matrix
	// calculate output

}

// StringCh2FloatCh will convert strings in the input channel to float64s.
// A parse error leads to a NaN being output instead.
func StringCh2FloatCh(in <-chan string) <-chan float64 {
	out := make(chan float64)
	go func() {
		v, err := strconv.ParseFloat(<-in, 64)
		if err != nil {
			if err == strconv.ErrRange || err == strconv.ErrSyntax {
				out <- math.NaN()
				return
			}
			exitOnError(err)
		}
		out <- v
	}()
	return out
}

// func teedSlidingWindow()

func processFile(infile *os.File, outfile *os.File) {

	// lastCh := channels.NewInfiniteChannel()
	// firstCh := channels.NewInfiniteChannel()
	// displayCh := channels.NewInfiniteChannel()

	// STRATEGY - Each line should be a channel that emits tokens.

	indexes := lineIndexes(infile)

	lineOffsets := slidingWindow(indexes, 2)

	for lineOffset := range lineOffsets {
		fmt.Println(lineOffset)
		outchan := emitCSVRowDatums(infile, lineOffset[0], lineOffset[1])
		go func() {
			for str := range outchan {
				fmt.Println(str)
			}
		}()
	}

	// windows := slidingWindow(indexes, 3)
	// numberedWindows := enumerate(windows, 1)

	// channels.Tee(
	// 	channels.Wrap(numberedWindows),
	// 	FirstTo(firstCh),
	// 	LastTo(lastCh),
	// 	displayCh)

	// // This will work fine for all lines except the first and last.
	// display := make(chan Enumerated)
	// channels.Unwrap(displayCh, display)

	// for enumerated := range display {
	// 	fmt.Println(enumerated.position)
	// 	fmt.Println(enumerated.lines)
	// }

	// fmt.Println(<-firstCh.Out())
	// fmt.Println(<-lastCh.Out())
}

func processLines(infile *os.File, lineOffsets []int64) chan string {
	out := make(chan string)
	go func() {

	}()
	return out
}

func main() {
	var in = flag.String("in", "", "Input file")
	var out = flag.String("out", "", "Output file")

	flag.Parse()

	// Open input file
	infile, err := os.Open(*in)
	exitOnError(err)
	defer infile.Close()

	// Open output file
	outfile, err := os.Create(*out)
	exitOnError(err)
	defer outfile.Close()

	processFile(infile, outfile)

	// outfile.Seek(100, whence)
	// var byteSlice = []byte("Bytes!\n")
	// _, err = outfile.Write(byteSlice)
	// exitOnError(err)

	// b := infile.At(0)
	// fmt.Printf("%c\n", b)
}
