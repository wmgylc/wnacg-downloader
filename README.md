<p align="center">
    <img src="https://github.com/user-attachments/assets/0e266cd6-10db-4470-96ce-68d548363ae4" style="align-self: center"/>
</p>

# 📚 绅士漫画下载器

一个用于 wnacg.com 绅士漫画 的多线程下载器，带图形界面，带收藏夹，下载速度飞快。图形界面基于[Tauri](https://v2.tauri.app/start/)

🔽 在[Release页面](https://github.com/lanyeeee/wnacg-downloader/releases)可以直接下载

**如果本项目对你有帮助，欢迎点个 Star⭐ 支持！你的支持是我持续更新维护的动力🙏**

# 🖥️ 图形界面
![](https://github.com/user-attachments/assets/5745a2e7-67e9-4c0d-a776-498a8094a4e9)




# 📖 使用方法

## Docker / HTTP API

仓库里额外提供了一套单容器 Docker 配置。容器会常驻运行 `wnacg-api`，同时对外提供网页和 GET API。

构建并启动：

```bash
docker compose -f docker-compose.cli.yml up -d --build
```

页面入口：

```bash
http://127.0.0.1:3001
```

HTTP API 示例：

```bash
curl "http://127.0.0.1:3001/api/health"
curl "http://127.0.0.1:3001/api/search/keyword?q=fate&page=1"
curl "http://127.0.0.1:3001/api/search/tag?tag=%E4%BA%BA%E5%A6%BB&page=2"
curl "http://127.0.0.1:3001/api/comic?target=https://www.wnacg.com/photos-view-id-27566986.html"
curl "http://127.0.0.1:3001/api/download?target=https://www.wnacg.com/photos-index-aid-123456.html"
curl "http://127.0.0.1:3001/api/tasks"
```

当前可用接口：

- `GET /api/health`
- `GET /api/search/keyword?q=<关键词>&page=1`
- `GET /api/search/tag?tag=<标签>&page=1`
- `GET /api/comic?target=<漫画ID或URL>`
- `GET /api/download?target=<漫画ID或URL>`
- `GET /api/download/start?target=<漫画ID或URL>`
- `GET /api/tasks`
- `GET /api/tasks/<id>`

接口支持透传这些可选参数：

- `api_domain`
- `proxy`
- `download_dir`
- `config`
- `format` 仅 `download`
- `img_concurrency` 仅 `download`
- `img_interval_sec` 仅 `download`
- `use_original_filename=true` 仅 `download`

说明：

- 宿主机目录 `/vol2/1000/download` 会挂载到容器内的 `/data`
- 宿主机目录 `./docker-config` 会挂载到容器内的 `/config`
- 配置文件默认是 [`docker-config/wnacg-cli.json`](/Users/wmgylc/code/wnacg-downloader/docker-config/wnacg-cli.json)
- 页面和 API 共用同一个容器，对外端口是 `3001`
- 任务历史默认持久化到 `/data/wnacg-tasks.sqlite`，首次启动时如果发现旧的 `/data/wnacg-tasks.json`，会自动导入到 SQLite

下载完成后会自动生成同名 `.zip` 文件，并删除原始漫画目录，只保留压缩包。
下载过程中带两层失败恢复：单张图片默认重试 `2` 次，整本任务默认重试 `1` 次；如果最终仍然失败，会保留临时目录，便于后续续传。

当前默认值：

- 默认下载目录来自 [`docker-config/wnacg-cli.json`](/Users/wmgylc/code/wnacg-downloader/docker-config/wnacg-cli.json)，默认是 `/data`
- 默认图片并发数是 `5`
- 默认图片下载间隔是 `1` 秒
- 默认单张图片重试次数是 `2`
- 默认整本任务重试次数是 `1`

Webhook 配置示例：

```json
{
  "webhook_url": "https://example.com/webhook",
  "bark_url": "https://bark.wmgylc.top:10000/CzS6dEcWSikbSJomnfYgZT",
  "default_download_dir": "/data",
  "default_img_concurrency": 5,
  "default_img_interval_sec": 1,
  "default_img_retry_count": 2,
  "default_task_retry_count": 1
}
```

配置项说明：

- `webhook_url`：下载结束时回调的地址，成功和失败都会调用
- `bark_url`：按 Bark 推送方式发送通知，成功和失败都会调用，默认已经指向你的 Bark 地址
- `default_download_dir`：默认下载目的地，CLI 没传 `--download-dir` 时使用
- `default_img_concurrency`：默认图片并发数，CLI 没传 `--img-concurrency` 时使用。CLI/Docker 默认值当前为 `5`
- `default_img_interval_sec`：默认图片下载间隔秒数，CLI 没传 `--img-interval-sec` 时使用
- `default_img_retry_count`：单张图片下载失败时的默认重试次数
- `default_task_retry_count`：整本漫画下载失败时的默认重试次数

如果 `webhook_url` 非空，CLI 会在下载结束时发送 `POST` 请求；如果 `bark_url` 非空，CLI 还会按 Bark 风格发起 GET 推送。成功和失败都会发送，失败时会附带失败原因、已完成张数和总张数。`webhook_url` 的请求体示例：

```json
{
  "event": "download_finished",
  "status": "success",
  "comic_id": 123456,
  "title": "漫画标题",
  "download_dir": null,
  "zip_path": "/data/downloads/漫画标题.zip",
  "image_count": 42,
  "completed_images": 42,
  "total_images": 42,
  "reason": null
}
```

#### 🚀 不使用书架

1. **不需要登录**，直接使用`漫画搜索`
2. 直接点击卡片上的`一键下载` 或者 点击封面或标题进入`漫画详情`，里面也有`一键下载`
3. 下载完成后点击`打开目录`按钮查看结果

#### ⭐ 使用书架

1. 点击`账号登录`按钮完成登录
2. 使用`我的书架`，直接点击卡片上的`一键下载` 或者 点击封面或标题进入`漫画详情`，里面也有`一键下载`
3. 下载完成后点击`打开目录`按钮查看结果

**顺带一提，你可以在`本地库存`导出为pdf/cbz(zip)**

📹 下面的视频是完整使用流程，**没有H内容，请放心观看**

https://github.com/user-attachments/assets/cadbfa53-2f1f-4d55-8d0e-7e253624c09c


# ⚠️ 关于被杀毒软件误判为病毒

对于个人开发的项目来说，这个问题几乎是无解的(~~需要购买数字证书给软件签名，甚至给杀毒软件交保护费~~)  
我能想到的解决办法只有：

1. 根据下面的**如何构建(build)**，自行编译
2. 希望你相信我的承诺，我承诺你在[Release页面](https://github.com/lanyeeee/wnacg-downloader/releases)下载到的所有东西都是安全的。切勿轻信他人分享的文件，请**仅**在[Release页面](https://github.com/lanyeeee/wnacg-downloader/releases)下载。任何不是从该页面下载的版本均可能**已被篡改**并**真的包含病毒**(而非误报)，包括但不限于`网盘`、`通过邮箱或社交软件分享`、`issue或discussion里的文件`、`其他fork(仓库)`、`其他网站`

# 🛠️ 如何构建(build)

构建非常简单，一共就3条命令  
~~前提是你已经安装了Rust、Node、pnpm~~

#### 📋 前提

- [Rust](https://www.rust-lang.org/tools/install)
- [Node](https://nodejs.org/en)
- [pnpm](https://pnpm.io/installation)

#### 📝 步骤

#### 1. 克隆本仓库

```
git clone https://github.com/lanyeeee/wnacg-downloader.git
```

#### 2.安装依赖

```
cd wnacg-downloader
pnpm install
```

#### 3.构建(build)

```
pnpm tauri build
```

# 🤝 提交PR

**PR请提交至`develop`分支**

**如果想新加一个功能，请先开个`issue`或`discussion`讨论一下，避免无效工作**

其他情况的PR欢迎直接提交，比如：

1. 🔧 对原有功能的改进
2. 🐛 修复BUG
3. ⚡ 使用更轻量的库实现原有功能
4. 📝 修订文档
5. ⬆️ 升级、更新依赖的PR也会被接受

# ⚠️ 免责声明

- 本工具仅作学习、研究、交流使用，使用本工具的用户应自行承担风险
- 作者不对使用本工具导致的任何损失、法律纠纷或其他后果负责
- 作者不对用户使用本工具的行为负责，包括但不限于用户违反法律或任何第三方权益的行为

# 💬 其他

任何使用中遇到的问题、任何希望添加的功能，都欢迎提交issue或开discussion交流，我会尽力解决  
