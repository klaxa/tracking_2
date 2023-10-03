#!/bin/sh

DEFAULT_TRACKING_IDLE_FILE=/tmp/tracking-idle

if [ -z $TRACKING_IDLE_FILE ]
then
    TRACKING_IDLE_FILE=$DEFAULT_TRACKING_IDLE_FILE
fi

if [ ! -z "$1" ]
then
    if [ -e $TRACKING_IDLE_FILE ]
    then
        rm $TRACKING_IDLE_FILE
    else
        touch $TRACKING_IDLE_FILE
    fi
fi


if [ -e $TRACKING_IDLE_FILE ]
then
    echo "⏸"
    echo "⏸"
    echo "#EEEEEE"

else
    echo "⏺"
    echo "⏺"
    now=`date +%s`
    if [ `expr $now % 2` -eq 0 ]
    then
        echo "#FF0000"
    else
        echo "#880000"
    fi
fi
