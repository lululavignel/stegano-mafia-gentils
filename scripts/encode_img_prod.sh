#!/bin/sh

[ ! -d ~/Images/steg/s256-c-p-0.1 ] && mkdir ~/Images/steg/s256-c-p-0.1
[ ! -d ~/Images/steg/s256-c-p-0.2 ] && mkdir ~/Images/steg/s256-c-p-0.2
[ ! -d ~/Images/steg/s256-c-p-0.4 ] && mkdir ~/Images/steg/s256-c-p-0.4
[ ! -d ~/Images/steg/s256-c-p-0.5 ] && mkdir ~/Images/steg/s256-c-p-0.5
[ ! -d ~/Images/steg/s256-c-p-0.8 ] && mkdir ~/Images/steg/s256-c-p-0.8
[ ! -d ~/Images/steg/s256-c-p-0.99 ] && mkdir ~/Images/steg/s256-c-p-0.99


[ ! -d ~/Images/steg/base_img/logs ] && mkdir ~/Images/steg/base_img/logs
[ ! -d ~/Images/steg/s256-c-p-0.1/logs ] && mkdir ~/Images/steg/s256-c-p-0.1/logs
[ ! -d ~/Images/steg/s256-c-p-0.2/logs ] && mkdir ~/Images/steg/s256-c-p-0.2/logs
[ ! -d ~/Images/steg/s256-c-p-0.4/logs ] && mkdir ~/Images/steg/s256-c-p-0.4/logs
[ ! -d ~/Images/steg/s256-c-p-0.5/logs ] && mkdir ~/Images/steg/s256-c-p-0.5/logs
[ ! -d ~/Images/steg/s256-c-p-0.8/logs ] && mkdir ~/Images/steg/s256-c-p-0.8/logs
[ ! -d ~/Images/steg/s256-c-p-0.99/logs ] && mkdir ~/Images/steg/s256-c-p-0.99/logs

for prob in "0.1" "0.2" "0.4" "0.5" "0.8" "0.99" 
do
    echo $prob
    for file in ~/Images/steg/base_img/*.png
    do
        echo "${file%.*}.png"
        base_name=$(basename ${file})
        echo ~/Images/steg/s256-c-p-${prob}/${base_name}
        ./target/release/steg -w -l -i "$file" ~/Images/steg/s256-c-p-${prob}/${base_name} -t ./veryverylong.txt -g sha256 -c aes_key -p "${prob}"

    done
done
