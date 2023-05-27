#!/bin/sh

sudo rc-service shapeless-blog stop

sudo cp ~/shapeless-blog/shapeless-blog /usr/local/bin/shapeless-blog

sudo cp ~/shapeless-blog/shapeless-blog.init /etc/init.d/shapeless-blog

sudo chmod +x /etc/init.d/shapeless-blog

sudo rc-service shapeless-blog start
