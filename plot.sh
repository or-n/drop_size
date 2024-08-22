#!/bin/bash

FILE=IMG_2880.mov
FILE_PATH=results/$FILE
NAME=hue05sl5

mkdir $FILE_PATH/$NAME
mv $FILE_PATH/$FILE $FILE_PATH/$NAME
mv $FILE_PATH/${FILE}_sizes.csv $FILE_PATH/$NAME
python3 plot.py $FILE_PATH/$NAME/${FILE}_sizes.csv --output $FILE_PATH/$NAME/plot
