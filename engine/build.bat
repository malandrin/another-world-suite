echo off
call wasm-pack build --target no-modules
if %errorlevel% == 1 goto end
copy pkg\awlib_bg.wasm ..\frontend\src\assets\awlib.bin
:end