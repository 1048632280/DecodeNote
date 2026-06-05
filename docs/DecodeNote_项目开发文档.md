# DecodeNote 项目开发文档

版本：0.1  
日期：2026-06-05  
定位：Windows 单端、现代极简工具风的轻量 TXT 编码识别与编辑器

## 1. 项目概述

DecodeNote 是一个面向 Windows 的轻量 TXT 编辑器。它看起来接近记事本：主体是一个可编辑的大文本框，支持打开、拖入、编辑、保存和另存为。但它的核心不是复杂编辑功能，而是帮助用户快速判断同一个 TXT 文件在不同编码视角下的文本呈现。

软件打开文件后，Rust 后端保留文件原始字节。用户点击底部编码按钮时，软件会用选中的编码对同一份原始字节整篇重新解码，并即时刷新编辑区内容。保存是独立行为：保存时把当前编辑区文本统一编码为指定目标编码写出，不尝试保留文件内部混合编码结构。

## 2. 产品目标

- 像记事本一样直接编辑 TXT 文件。
- 支持拖入 TXT 文件直接加载。
- 支持打开文件后根据窗口宽度自动换行。
- 支持底部常用中文编码按钮，一键整篇重新解码显示。
- 支持右下角更多编码菜单，覆盖冷门旧编码。
- 支持保存和另存为当前指定编码文档。
- 支持文件大小、当前编码、自动检测结果、解码错误等状态信息展示。
- 保持轻量、流畅、低复杂度，10MB 以内文本应无明显卡顿。

## 3. 非目标

- 不做多标签页。
- 不做富文本、Markdown 预览、语法高亮、代码补全。
- 不做行号、迷你地图、复杂搜索替换。
- 不做局部字节段编码修复。MVP 只支持整篇按某编码重新解码。
- 不保证 GB 级日志文件体验。目标文件大小为 10MB 以内。
- 不尝试保存“文件内多段混合编码”的原始结构。

## 4. 核心用户流程

### 4.1 打开文件

1. 用户点击打开按钮，或把 TXT 文件拖入窗口。
2. 前端拿到文件路径后调用 Rust 命令读取文件 bytes。
3. Rust 后端保存原始 bytes 到当前文档会话。
4. 后端按检测策略推测编码并解码为文本。
5. 前端把文本填入 CodeMirror 编辑器。
6. 底部状态栏显示文件大小、推测编码、当前解码编码和错误信息。

### 4.2 切换编码显示

1. 用户点击底部编码按钮，例如 `GBK`。
2. 如果当前文本有未保存修改，弹窗提示：重新解码会丢弃未保存修改，是否继续。
3. 用户确认后，前端调用 Rust 命令。
4. Rust 用 `GBK` 对当前文件原始 bytes 整篇重新解码。
5. 前端用解码结果替换编辑器全文。
6. 当前文档变为未修改状态，因为内容来自原始文件 bytes 的重新解释。

### 4.3 编辑与保存

1. 用户编辑当前文本。
2. 编辑器标记文档为已修改。
3. 用户点击保存。
4. Rust 将当前编辑区文本按目标编码编码为 bytes。
5. 覆盖原文件或写入另存为路径。
6. 保存成功后，后端用新 bytes 更新当前文档原始 bytes，文档变为未修改状态。

## 5. 技术选型

### 5.1 总体栈

- 桌面框架：Tauri
- 后端：Rust
- 前端：React + Vite + TypeScript
- 编辑器核心：CodeMirror 6
- 编码解码：Rust `encoding_rs`
- 编码检测：Rust `chardetng` + 自定义 BOM/UTF-16/UTF-8 启发式
- 文件对话框：Tauri Dialog Plugin
- 拖放文件：Tauri Webview `onDragDropEvent`

### 5.2 选型理由

Tauri 使用系统 WebView 承载前端界面，应用包体和运行时负担更小，Rust 后端适合处理文件 bytes、编码解码和保存。React/Vite 适合快速构建现代前端，CodeMirror 6 提供成熟的文本编辑模型、撤销重做、选择、输入法支持和可扩展视图层。10MB 内纯文本不需要 Monaco 这类更重的代码编辑器。

CodeMirror 6 在本项目中只启用纯文本编辑、自动换行和基础撤销重做，不启用代码语言包、行号、lint、补全等扩展，以维持记事本风格和低开销。

## 6. UI 设计方向

视觉风格采用“极简工具风”，不采用 Windows 11 毛玻璃、Fluent 拟态或强系统化圆角风格。

### 6.1 整体布局

```text
+--------------------------------------------------------------+
| 顶部工具栏：DecodeNote | 打开 | 保存 | 另存为 | 文件路径       |
+--------------------------------------------------------------+
|                                                              |
|                   主编辑区 CodeMirror                        |
|                 记事本风格、无行号、自动换行                 |
|                                                              |
+--------------------------------------------------------------+
| 状态信息 | UTF-8 | UTF-8 BOM | GBK | GB18030 | BIG5 | ... 更多 |
+--------------------------------------------------------------+
```

### 6.2 顶部工具栏

- 左侧显示应用名 `DecodeNote`。
- 提供 `打开`、`保存`、`另存为`。
- 显示当前文件名或完整路径的压缩展示。
- 有未保存修改时在文件名旁显示 `*` 或轻量提示点。

### 6.3 主编辑区

- 使用 CodeMirror 6。
- 无行号、无语法高亮、无迷你地图。
- 始终启用自动换行。
- 字体使用清晰的等宽字体栈：`Cascadia Mono`, `Consolas`, `Microsoft YaHei UI`, monospace。
- 编辑区占据窗口绝大部分空间。
- 拖入文件时显示轻量覆盖层：`释放以打开文本文件`。

### 6.4 底部状态栏

状态栏分为三段：

- 左侧：文件状态  
  `10.2 KB`、`已修改`、`行 12，列 8`

- 中间：编码信息  
  `当前：GBK`、`推测：GB18030`、`替换字符：3`

- 右侧：编码按钮和更多菜单  
  `UTF-8`、`UTF-8 BOM`、`GBK`、`GB18030`、`BIG5`、`UTF-16 LE`、`UTF-16 BE`、`更多`

当前解码编码按钮高亮。自动检测结果只作为提示，不等同于确定结论。

## 7. 编码策略

### 7.1 常用编码按钮

底部固定展示：

- `UTF-8`
- `UTF-8 BOM`
- `GBK`
- `GB18030`
- `BIG5`
- `UTF-16 LE`
- `UTF-16 BE`

说明：

- `UTF-8` 与 `UTF-8 BOM` 在显示解码上都按 UTF-8 处理，差异主要体现在保存时是否写入 BOM。
- `GBK` 更贴近常见简体中文旧文件。
- `GB18030` 覆盖范围更大，可作为中文遗留文本的强兜底。
- `BIG5` 面向繁体中文旧文件。
- `UTF-16 LE/BE` 需要配合 BOM 和空字节分布启发式检测。

### 7.2 更多编码菜单

右下角 `更多` 菜单提供：

- `GB2312`，作为 GBK 兼容别名处理
- `Shift_JIS`
- `EUC-JP`
- `ISO-2022-JP`
- `EUC-KR`
- `windows-1252`
- `windows-1250`
- `windows-1251`
- `windows-1256`
- `ISO-8859-2`
- `ISO-8859-5`
- `KOI8-R`
- `KOI8-U`
- `macintosh`

更多菜单只放冷门项，避免底部按钮拥挤。

### 7.3 自动检测策略

打开文件时按以下顺序检测：

1. 检查 BOM：
   - `EF BB BF` -> UTF-8 BOM
   - `FF FE` -> UTF-16 LE
   - `FE FF` -> UTF-16 BE
2. 检查是否为严格 UTF-8：
   - 若全文件可按 UTF-8 无错误解码，则推测为 UTF-8。
3. 检查 UTF-16 启发式：
   - 根据奇偶字节位的 `0x00` 分布推测 UTF-16 LE/BE。
4. 使用 `chardetng` 推测旧编码。
5. 如果检测不明确，中文场景兜底优先尝试 `GB18030`，同时在状态栏标注为低置信度推测。

### 7.4 解码错误展示

每次解码返回：

- `hadErrors`：是否出现解码错误。
- `replacementCount`：文本中的 `U+FFFD` 替换字符数量。
- `bom`：是否检测到 BOM。
- `encoding`：当前解码编码。
- `detectedEncoding`：打开文件时的推测编码。

状态栏显示 `替换字符：N`，帮助用户比较不同编码下的乱码程度。

## 8. 保存策略

### 8.1 保存

保存会把当前编辑区文本编码为目标编码并覆盖当前文件。

默认目标编码：

- 初次打开后：使用自动检测编码。
- 用户点击编码按钮重新解码后：使用当前按钮对应编码作为默认保存编码。
- 用户在另存为对话中选择了编码后：使用用户选择的编码。

### 8.2 另存为

另存为流程：

1. 用户点击 `另存为`。
2. 弹出保存路径选择。
3. 弹出或展示编码选择，默认值为当前解码编码。
4. Rust 将当前文本编码并写入目标路径。
5. 保存成功后，当前文档路径切换为新路径。

### 8.3 BOM 策略

- `UTF-8`：保存时不写 BOM。
- `UTF-8 BOM`：保存时写入 `EF BB BF`。
- `UTF-16 LE`：保存时写入 `FF FE`，随后写入 LE 字节序内容。
- `UTF-16 BE`：保存时写入 `FE FF`，随后写入 BE 字节序内容。
- `GBK / GB18030 / BIG5`：不写 BOM。

### 8.4 UTF-16 编码实现注意

`encoding_rs` 不提供 UTF-16LE/BE 编码器。保存 UTF-16 LE/BE 时由 Rust 自行实现：

```rust
let units = text.encode_utf16();
for unit in units {
    // LE: unit.to_le_bytes()
    // BE: unit.to_be_bytes()
}
```

解码 UTF-16 LE/BE 可使用 `encoding_rs` 对应解码能力，或在明确 BOM/用户选择时用 Rust 手动按字节序解析。

## 9. 架构设计

### 9.1 模块划分

```text
DecodeNote
├─ src/                         # React 前端
│  ├─ app/
│  │  ├─ App.tsx
│  │  ├─ editor/                # CodeMirror 封装
│  │  ├─ components/            # Toolbar、StatusBar、EncodingBar
│  │  ├─ hooks/                 # Tauri invoke、拖放、快捷键
│  │  └─ styles/
│  └─ main.tsx
├─ src-tauri/
│  ├─ src/
│  │  ├─ lib.rs
│  │  ├─ commands.rs            # Tauri 命令入口
│  │  ├─ document.rs            # 文档会话状态
│  │  ├─ encoding.rs            # 编码列表、解码、编码
│  │  ├─ detect.rs              # 编码检测
│  │  └─ file_io.rs             # 文件读取写入
│  └─ tauri.conf.json
└─ docs/
```

### 9.2 前后端职责

前端负责：

- 界面布局。
- CodeMirror 编辑器实例。
- 编码按钮、更多菜单、状态栏。
- 文件拖放事件监听。
- 未保存修改提示。
- 调用 Tauri 命令。

后端负责：

- 文件读取与写入。
- 原始 bytes 持有。
- 编码检测、解码、编码。
- 文件大小限制与错误返回。
- 保存后更新文档会话状态。

## 10. Rust 数据模型

```rust
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum EncodingId {
    Utf8,
    Utf8Bom,
    Gbk,
    Gb18030,
    Big5,
    Utf16Le,
    Utf16Be,
    ShiftJis,
    EucJp,
    Iso2022Jp,
    EucKr,
    Windows1252,
    // 更多冷门编码按需扩展
}

#[derive(Clone, Debug)]
pub struct DocumentSession {
    pub path: PathBuf,
    pub original_bytes: Vec<u8>,
    pub active_encoding: EncodingId,
    pub detected_encoding: Option<EncodingId>,
    pub save_encoding: EncodingId,
    pub revision: u64,
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct DecodeResult {
    pub text: String,
    pub encoding: EncodingId,
    pub detected_encoding: Option<EncodingId>,
    pub file_size: u64,
    pub had_errors: bool,
    pub replacement_count: usize,
    pub bom: Option<String>,
    pub revision: u64,
}
```

文档是否已修改由前端根据 CodeMirror 编辑事件维护。Rust 保存成功后返回新 revision，前端清除已修改标记。

## 11. Tauri 命令设计

```rust
#[tauri::command]
async fn open_file(path: String) -> Result<DecodeResult, AppError>;

#[tauri::command]
async fn decode_current_as(encoding: EncodingId) -> Result<DecodeResult, AppError>;

#[tauri::command]
async fn save_current(
    text: String,
    encoding: EncodingId,
) -> Result<SaveResult, AppError>;

#[tauri::command]
async fn save_as(
    path: String,
    text: String,
    encoding: EncodingId,
) -> Result<SaveResult, AppError>;

#[tauri::command]
async fn get_supported_encodings() -> Result<Vec<EncodingOption>, AppError>;
```

命令建议使用异步形式。解码和编码可以放入 `spawn_blocking`，避免大文件处理时阻塞 Tauri 运行时。

## 12. 前端实现要点

### 12.1 CodeMirror 使用方式

- 直接使用 CodeMirror 6 官方包封装 React 组件。
- 不把全文内容放入 React state 中随输入实时更新。
- CodeMirror 内部维护文档内容。
- React state 只维护：
  - 当前路径
  - 当前编码
  - 推测编码
  - 文件大小
  - 是否已修改
  - 状态栏统计
- 保存时再通过 `editorView.state.doc.toString()` 取全文。

这样可以减少 React 反复渲染大文本导致的卡顿。

### 12.2 自动换行

CodeMirror 启用 `EditorView.lineWrapping`，并通过 CSS 让编辑区高度填满窗口。用户不需要切换自动换行，始终开启。

### 12.3 拖放加载

前端使用 Tauri Webview 的拖放事件：

- `over`：显示覆盖层。
- `drop`：读取 `paths`，只接受第一个文件。
- `cancel`：隐藏覆盖层。

拖入时如果当前有未保存修改，先弹窗确认。

### 12.4 编码按钮响应

编码按钮点击流程：

1. 检查 dirty。
2. dirty 时弹出确认框。
3. 禁用编码按钮并显示轻量加载状态。
4. 调用 `decode_current_as`。
5. 用返回文本替换 CodeMirror 内容。
6. 更新状态栏。
7. 清除 dirty。

如果用户快速连续点击多个编码，前端使用请求序号，只接受最后一次请求结果。

## 13. 性能要求

目标文件：10MB 以内 TXT。

验收指标：

- 打开 10MB 文件：主观无明显卡顿，建议 1 秒内完成首屏展示。
- 编码切换：10MB 文件建议 1 秒内完成刷新。
- 编辑输入：普通输入无卡顿。
- 保存 10MB 文件：可接受短暂进度状态。

实现策略：

- Rust 端保留原始 bytes，避免每次从磁盘重读。
- 编码切换只做 bytes -> String，不做额外复杂分析。
- CodeMirror 不启用高成本扩展。
- React 不镜像编辑器全文。
- 解码/编码命令异步执行。
- 超过 10MB 的文件提示“超出推荐大小，可能变慢”。

## 14. 错误与边界处理

### 14.1 文件读取错误

- 文件不存在：提示文件不存在。
- 权限不足：提示无权限读取。
- 非文件路径：提示只支持文件。
- 空文件：打开为空文档，编码默认为 UTF-8。

### 14.2 解码错误

- 解码出现替换字符时不阻止显示。
- 状态栏显示替换字符数量。
- 用户可继续尝试其他编码。

### 14.3 保存错误

- 权限不足：提示保存失败。
- 目标路径被占用：提示保存失败，可另存为。
- 某些字符无法编码到目标编码：
  - 默认使用替换策略，并在保存前提示。
  - 后续可提供严格模式，但不进入 MVP。

### 14.4 未保存修改

以下操作需要提示：

- 打开新文件。
- 拖入新文件。
- 切换编码重新解码。
- 关闭窗口。

提示文案：

```text
当前内容有未保存修改。继续操作会丢弃这些修改，是否继续？
```

## 15. 安全与权限

- 只读取用户通过文件对话框或拖放提供的文件路径。
- 只写入用户保存或另存为选择的路径。
- Tauri 权限最小化，只启用必要的 dialog、文件访问和拖放相关能力。
- 不启用远程内容加载。
- 不需要网络权限。

## 16. 开发阶段计划

### 阶段 1：项目骨架与基础窗口

目标：

- 创建 Tauri + React + Vite + TypeScript 项目。
- 完成基础窗口、顶部工具栏、主编辑区、底部状态栏布局。
- 集成 CodeMirror 6，启用纯文本和自动换行。

交付：

- 可启动的 Windows 桌面应用。
- 空白编辑器可输入文本。

### 阶段 2：文件打开、拖放与自动检测

目标：

- 实现打开文件。
- 实现拖放加载 TXT。
- Rust 后端读取原始 bytes。
- 实现 BOM、UTF-8、UTF-16、chardetng 检测。
- 打开后显示文本和状态信息。

交付：

- 可打开 UTF-8、GBK、GB18030、BIG5、UTF-16 文件。
- 状态栏显示推测编码、文件大小、错误数量。

### 阶段 3：编码按钮与整篇重新解码

目标：

- 实现底部常用编码按钮。
- 实现更多编码菜单。
- 点击编码按钮整篇重新解码原始 bytes。
- 未保存修改时弹窗确认。
- 快速点击时只应用最后一次结果。

交付：

- 能快速比较不同编码下的显示效果。
- 10MB 内文件切换流畅。

### 阶段 4：保存与另存为

目标：

- 实现保存当前文件。
- 实现另存为指定编码。
- 实现 UTF-8 BOM、UTF-16 LE/BE BOM 写入策略。
- 保存后更新原始 bytes 和文档状态。

交付：

- 可把当前编辑文本保存为 UTF-8、GBK、GB18030、BIG5、UTF-16 等编码。
- 未保存状态准确。

### 阶段 5： polish 与打包

目标：

- 完成极简工具风主题。
- 完善浅色/深色主题。
- 完成错误提示、关闭确认。
- Windows 打包发布。

交付：

- 可安装或可执行的 Windows 版本。
- 基础用户说明。

## 17. 测试计划

### 17.1 编码样本

准备以下测试文件：

- UTF-8 无 BOM 中文文本
- UTF-8 BOM 中文文本
- GBK 简体中文文本
- GB18030 生僻字文本
- BIG5 繁体中文文本
- UTF-16 LE 文本
- UTF-16 BE 文本
- 含无效字节的损坏文本
- 10MB 中文文本

### 17.2 功能测试

- 打开文件后内容正确显示。
- 拖入文件可加载。
- 编码按钮可整篇重新解码。
- 未保存修改时切换编码会提示。
- 保存可覆盖原文件。
- 另存为可选择目标编码。
- 状态栏信息准确更新。

### 17.3 性能测试

- 1MB、5MB、10MB 文件打开时间。
- 1MB、5MB、10MB 文件编码切换时间。
- 大文本中连续输入是否卡顿。
- 保存耗时和 UI 响应。

### 17.4 回归测试

- 空文件。
- 只有英文 ASCII 的文件。
- 只有换行符的文件。
- CRLF/LF 混合文件。
- 文件路径包含中文和空格。
- 保存路径无权限。

## 18. 验收标准

MVP 完成时应满足：

- Windows 上可运行。
- 主体是记事本风格文本编辑区。
- 支持打开、拖入、编辑、保存、另存为。
- 打开 TXT 后始终自动换行。
- 底部常用编码按钮可即时整篇重新解码。
- 右下角更多菜单可选择冷门编码。
- 有未保存修改时，重新解码会提示。
- 状态栏显示文件大小、当前编码、推测编码、替换字符数量。
- 10MB 内文本打开、切换编码、编辑体验流畅。

## 19. 推荐依赖

Rust：

```toml
[dependencies]
tauri = { version = "2", features = [] }
tauri-plugin-dialog = "2"
encoding_rs = "0.8"
chardetng = "1"
serde = { version = "1", features = ["derive"] }
thiserror = "2"
```

前端：

```json
{
  "dependencies": {
    "@codemirror/state": "latest",
    "@codemirror/view": "latest",
    "@codemirror/commands": "latest",
    "@tauri-apps/api": "latest",
    "@tauri-apps/plugin-dialog": "latest",
    "react": "latest",
    "react-dom": "latest"
  },
  "devDependencies": {
    "@vitejs/plugin-react": "latest",
    "typescript": "latest",
    "vite": "latest"
  }
}
```

实际项目创建后应提交 lockfile，生产构建使用锁定版本，不在 CI 中漂移 `latest`。

## 20. 初始化命令建议

```powershell
npm create tauri-app@latest DecodeNote
```

交互选择建议：

- Package manager：按团队习惯选择 `pnpm` 或 `npm`
- Frontend：React
- Language：TypeScript
- UI template：Vite

创建后增加依赖：

```powershell
cd DecodeNote
npm install @codemirror/state @codemirror/view @codemirror/commands
npm install @tauri-apps/plugin-dialog
cd src-tauri
cargo add encoding_rs chardetng thiserror
cargo add tauri-plugin-dialog
```

## 21. 参考资料

- Tauri 官方文档：<https://tauri.app/start/>
- Tauri Dialog Plugin：<https://v2.tauri.app/plugin/dialog/>
- Tauri Webview Drag Drop API：<https://tauri.app/reference/javascript/api/namespacewebviewwindow/>
- Vite 官方指南：<https://vite.dev/guide/>
- CodeMirror 6 Reference：<https://codemirror.net/docs/ref/>
- encoding_rs 文档：<https://docs.rs/encoding_rs/latest/encoding_rs/>
- chardetng 文档：<https://docs.rs/chardetng/latest/chardetng/>
