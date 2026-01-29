# Usage: version_check.sh MIN_VERSION CURRENT_VERSION

ver=$(echo -e "$1\n$2" | sort -V | head -n1)

if [ "$ver" != "$1" ];then
    echo -e '\033[1;31m'
    echo Unsupported QEMU Version: $2
    echo Minimum Supported QEMU Version is: $1.
    echo -e '\0'
    exit 1
fi