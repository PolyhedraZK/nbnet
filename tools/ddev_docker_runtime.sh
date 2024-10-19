#!/usr/bin/env bash

#################################################
#### Ensure we are in the right path. ###########
#################################################
if [[ 0 -eq $(echo $0 | grep -c '^/') ]]; then
    # relative path
    EXEC_PATH=$(dirname "`pwd`/$0")
else
    # absolute path
    EXEC_PATH=$(dirname "$0")
fi

EXEC_PATH=$(echo ${EXEC_PATH} | sed 's@/\./@/@g' | sed 's@/\.*$@@')
cd $EXEC_PATH || exit 1
#################################################

source utils.sh

#################################################

docker_file=$1
dir="/tmp/NB_DOCKER_RUNTIME_BUILD_DIR_${RANDOM}_$(date +%s)"

mkdir -p ${HOME}/__NB_DATA__/{usr_local_bin,tmp} || die

# Allow some unknown files
chmod -R 1777 ${HOME}/__NB_DATA__ 2>/dev/null # `|| die`

mkdir $dir || die
cd $dir || die
cp ~/.ssh/authorized_keys ./ || die
cp $docker_file ./Dockerfile || die

######################################################

which podman || alias docker='podman'

docker images --format=json | grep '"Tag":"\\u003cnone\\u003e"' | jq '.ID' | xargs docker image rm

docker_image_cnt=$(docker images --format json | jq '.Tag' | grep -c 'nbnet_24.04')
podman_image_cnt=$(podman images --format json | jq '.[].Names' | grep -c 'nbnet_24.04')

if [[ 0 -eq ${docker_image_cnt} && 0 -eq ${podman_image_cnt} ]]; then
    docker pull ubuntu:24.04 || die
    docker tag ubuntu:24.04 ubuntu:nbnet_24.04 || die
fi

which docker
docker build -t ubuntu:nbnet_runtime_v0 . || die

docker rm -f nbnet_runtime

docker run --restart always -d --network=host \
    -v ${HOME}/__NB_DATA__/tmp:/tmp \
    -v ${HOME}/__NB_DATA__/usr_local_bin:/usr/local/bin \
    --name nbnet_runtime \
    ubuntu:nbnet_runtime_v0 || die

docker ps

#################################################

rm -rf $dir
