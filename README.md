# chromogene
Generates perceptually optimized color schemes for diagrams, terminals or code editors
by optimizing over the color’s perceived distances with genetic algorithms.

# Usage
Pipe a description the fitness function into the program and it will try to generate a colorscheme with maximum fitness. A sample fitness-function would be:
```
maximize min fixeddist
approximate 40 min freedist
minimize stddev luminance 1 2
minimize stddev chroma 1 2
```
(I'm working on a new file format right now, this is just a proof of concept.)

Then start it using:

```bash
cat solarized | cargo run --release
```
(you probably need Rust beta or nightly)

![demo](https://github.com/fdietze/chromogene/raw/master/demo.gif)
