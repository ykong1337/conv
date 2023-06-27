# Conv

使用FFmpeg合并音频，图片和字幕生成视频的工具

支持Whisper语音识别

### 使用
下载[FFmpeg](https://github.com/BtbN/FFmpeg-Builds/releases/latest)并设置环境变量($PATH:ffmpeg/bin)

Scoop:
```
scoop install ffmpeg
```

运行conv.exe(请把字幕文件和conv.exe放在同一目录下)

### 构建
安装
[CMake](https://cmake.org/download/)
[Rustup](https://rustup.rs/)
[LLVM](https://releases.llvm.org/download.html)
[C++ 构建工具](https://visualstudio.microsoft.com/visual-cpp-build-tools/)

Scoop:
```
scoop install cmake rustup llvm
```