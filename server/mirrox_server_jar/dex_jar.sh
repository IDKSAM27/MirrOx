#!/bin/bash
# Below command tells shell to exit the script immediately if any command returns a non zero exit status
set -e

# CONFIG
D8_BAT="C:\Users\Sampreet\AppData\Local\Android\Sdk\build-tools\35.0.0\d8.bat"

echo "
[*] Cleaning Gradle..."
./gradlew clean

echo "
[*] Gradle Building..."
./gradlew buildMirroxJar 

echo "
[*] mkdir build/dex"
mkdir build/dex

echo "
[*] Dexxing the Jar..."
"$D8_BAT" --output=build/dex build/libs/mirrox_server-1.0.jar

echo "
[*] Going to build/dex..."
cd build/dex

echo "
[*] Jarring the classes.dex..."
jar cf mirrox_server.dex.jar classes.dex

cd ..
cd ..

echo "
[✅] mirrox_server.dex.jar is ready !!!"