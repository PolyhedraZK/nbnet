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

docker_file=$1
scd_bin=$2
expander_bin=$3

dir="/tmp/NB_DOCKER_RUNTIME_BUILD_DIR_${RANDOM}_$(date +%s)"

mkdir -p ${HOME}/__NB_DATA__/{usr_local_bin,tmp} || exit 1

# Allow some unknown files
chmod -R 1777 ${HOME}/__NB_DATA__ 2>/dev/null # `|| exit 1`

mkdir $dir || exit 1
cd $dir || exit 1
cp ~/.ssh/authorized_keys ./ || exit 1
cp ${EXEC_PATH}/entrypoint.sh ./ || exit 1
cp $scd_bin ./scd || exit 1
cp $expander_bin ./expander-exec || exit 1
cp $docker_file ./Dockerfile || exit 1

######################################################

which podman || alias docker='podman'

docker images --format=json | grep '"Tag":"\\u003cnone\\u003e"' | jq '.ID' | xargs docker image rm

docker_image_cnt=$(docker images --format json | jq '.Tag' | grep -c 'nbnet_24.04')
podman_image_cnt=$(podman images --format json | jq '.[].Names' | grep -c 'nbnet_24.04')

if [[ 0 -eq ${docker_image_cnt} && 0 -eq ${podman_image_cnt} ]]; then
    docker pull ubuntu:24.04 || exit 1
    docker tag ubuntu:24.04 ubuntu:nbnet_24.04 || exit 1
fi

which docker
docker build --build-arg UID=$(id -u) -t ubuntu:nbnet_runtime_v0 . || exit 1

docker rm -f nbnet_runtime

docker run -d \
    --user $(id -u) \
    --network=host \
    -v ${HOME}/__NB_DATA__/tmp:/tmp \
    -v ${HOME}/__NB_DATA__/usr_local_bin:/usr/local/bin \
    --name nbnet_runtime \
    ubuntu:nbnet_runtime_v0 || exit 1

docker ps

#################################################

rm -rf $dir
