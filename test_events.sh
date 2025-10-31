#!/bin/bash
echo "Testing event listener - will copy 100 items with 2 second delay"
for i in {1..100}; do
    echo "Copying test item $i"
    wl-copy "Test Event $i - $(date +%T)"
    sleep 2
done
echo "Done"
