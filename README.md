# fondoso

A CLI tool written in Rust to create *fondos* (wallpapers) and more 
procedurally generated images.

Special thanks to [**@udf**](https://github.com/udf) for his initial
Python script that happened to yield such cool images.

## showcase

Here are some cool things done with the program:

### Random colours and positions
`./fondoso --random --number 10`

(Equivalent to `./fondoso -rn10`)

<img src="https://user-images.githubusercontent.com/6297805/38497021-e9b13154-3bff-11e8-927d-911501d8e0ba.png" height="200px" />


### Custom positions and colours
`./fondoso --positions "255,255 : 0,0 : -1,0 : -1,-1 : 0,-1" --colours "255,255,255 : 50,50,50"`

(Equivalent to `./fondoso -p255,255:0,0:-1,0:-1,-1:0,-1 -c255,255,255:50,50,50`)

<img src="https://user-images.githubusercontent.com/6297805/38497095-3a34aca0-3c00-11e8-8ce5-df93c3810172.png" height="200px" />


### Different randomization kinds
`./fondoso --random --number 10 --kind 80`

(Equivalent to `./fondoso -rn10 -k80`)

<img src="https://user-images.githubusercontent.com/6297805/38517952-e63c6214-3c3b-11e8-980c-b8909a563832.png" height="200px" />


### Mixing different kinds and custom positions
`./fondoso --kind treerev --positions 250,250`

(Equivalent to `./fondoso -ktreerev -p250,250`)

<img src="https://user-images.githubusercontent.com/6297805/38579410-32ee9e30-3d07-11e8-9c41-0296aa3192a2.png" height="200px" />
