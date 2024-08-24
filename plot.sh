#!/bin/bash

FILE=$1
FILE_PATH=results/$FILE
NAME=$2

mkdir $FILE_PATH/$NAME
mv $FILE_PATH/$FILE $FILE_PATH/$NAME
mv $FILE_PATH/${FILE}_sizes.csv $FILE_PATH/$NAME
python3 plot.py $FILE_PATH/$NAME/${FILE}_sizes.csv --output $FILE_PATH/$NAME/plot
