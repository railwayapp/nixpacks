export CONTAINER_ID=$(cat /etc/hostname)
export CONTAINER_PUBLIC_PORT=$(docker port $CONTAINER_ID 8080/tcp | cut -d':' -f2 | tr -d $'\n')