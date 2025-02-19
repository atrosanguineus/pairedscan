#!/bin/bash

root_folder="/home/ottavio/Work/glioblastoma_methyl_data/data/"
mapfile -t full_array < <( target/release/pairedscan $root_folder -igr )

for ((i = 0; i < ${#full_array[@]}; i=i+2)); do
    echo ${full_array[i]}
    echo ${full_array[i+1]}
done