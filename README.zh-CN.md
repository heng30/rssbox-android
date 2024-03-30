<div style="display: flex, margin: 8px">
    <img src="./screenshot/rssbox-cn.png" width="100"/>
    <img src="./screenshot/rssbox2-cn.png" width="100"/>
</div>

[English Documentation](./README.md)

#### 简介
这是一个RSS客户端。专门为安卓端写的，当然你也可以编译Linux, Windows (也许Macos)平台的程序。这是一个纯Rust项目，界面基于`Slint UI`。 遇到任何问题都可以给我反馈。

#### 功能
- [x] 生成钱包账号
- [x] 恢复钱包
- [x] 发送和接收BTC
- [x] 展示交易活动
- [x] 地址簿

##### 安卓平台编译信息
- `min-sdk-version = 23`
- `max-sdk-version = 34`
- `target-sdk-version = 32`

#### 如何构建?
- 安装 `Rust` 和 `Cargo`
- 安装 Android `sdk`, `ndk`, `jdk17`, 和设置对应的环境变量
- 运行 `make` 编译安卓平台程序
- 运行 `make debug` 编译桌面平台程序
- 参考 [Makefile](./Makefile) 了解更多信息

#### 参考
- [Slint Language Documentation](https://slint-ui.com/releases/1.0.0/docs/slint/)
- [github/slint-ui](https://github.com/slint-ui/slint)
- [Viewer for Slint](https://github.com/slint-ui/slint/tree/master/tools/viewer)
- [LSP (Language Server Protocol) Server for Slint](https://github.com/slint-ui/slint/tree/master/tools/lsp)
- [top-rss-list](https://github.com/weekend-project-space/top-rss-list)
- [rss-list](https://github.com/saveweb/rss-list)
- [developer.android.com](https://developer.android.com/guide)

