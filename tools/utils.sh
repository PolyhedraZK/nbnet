
# for 'perl'
export LC_ALL=en_US.UTF-8

export OSMAKE=make
if [[ "FreeBSD"  == `uname -s` ]]; then
    export OSMAKE=gmake
fi

function dbg() {
    msg=$*
    echo -e "\033[01m[** Debug **]\033[00m $msg"
}

function log() {
    msg="$*"
    echo -e "\033[01m[##  Log  ##]\033[00m $msg"
}

function die() {
    log "$*"
    echo -e "\033[31;01m[!! Panic !!]\033[00m $msg"
    exit 1
}

which jq 2>/dev/null 1>&2
if [[ 0 -ne $? ]]; then
    go install github.com/itchyny/gojq/cmd/gojq@latest || exit 1
    binpath=$(go env GOPATH)/bin
    cp ${binpath}/gojq ${binpath}/jq || exit 1
    export PATH=${binpath}:${PATH}
fi
