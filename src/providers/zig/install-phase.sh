if [ -f .gitmodules ]; then
    echo ".gitmodules found, updating submodules"
    git submodule update --init --recursive
    echo "done updating submodules"
fi

if [ -f gyro.zzz ]; then
    echo "gyro.zzz found, using Gyro..."

    echo "installing Gyro"
    app_wd=$(pwd)
    git clone https://github.com/mattnite/gyro.git /gyro
    cd /gyro
    zig build -Drelease-safe
    cd $app_wd

    echo "installing dependencies"
    /gyro/gyro fetch

    echo "done installing"
fi