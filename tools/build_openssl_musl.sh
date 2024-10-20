#!/bin/bash

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

ln -sf /usr/include/x86_64-linux-gnu/asm /usr/include/x86_64-linux-musl/asm || die

ln -sf /usr/include/asm-generic /usr/include/x86_64-linux-musl/asm-generic || die

ln -sf /usr/include/linux /usr/include/x86_64-linux-musl/linux || die

mkdir -p /usr/local/musl || die

cd /tmp || die

if [ !-f /tmp/OpenSSL_1_1_1f.tar.gz ]; then
    wget https://github.com/openssl/openssl/archive/OpenSSL_1_1_1f.tar.gz || die
fi

tar -xpf OpenSSL_1_1_1f.tar.gz || die

cd openssl-OpenSSL_1_1_1f/ || die

CC="musl-gcc -fPIE -pie" ./Configure no-shared no-async --prefix=/usr/local/musl --openssldir=/usr/local/musl/ssl linux-x86_64 || die

make depend || die

make -j$(nproc) || die

make install || die
