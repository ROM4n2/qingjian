# 主题（草稿，2026-09-18）

## 目标

主题要能做到搜狗皮肤那个程度，而且更自由：二次元立绘探出窗口、九宫格图片背景、描边字、多层阴影、高亮条滑动、循环小动画。
编辑方式对标 Figma：画布上直接摆图层，所见即所得，导出一个主题包；mac 与 Windows 上由同一个渲染器画出同样的效果。

不做（至少第一版）：

- 可以交互的主题（主题里写按钮逻辑、脚本）。主题只管画，点击命中仍然只认候选、翻页与状态条的格子。
- 行为设置：候选排序、每页个数、快捷键，留在 `config.toml`。

## 核心模型：自由图层 + 数据插槽 + 组件

候选窗不是一张静态设计稿，内容每按一个键就变：候选几个、多长，有没有译文，拼音多长。所以不能让设计师画一张图、我们原样贴出来。
Figma 本身已经解决了「内容会变的设计稿」：自动布局（auto layout）、约束（constraints）、组件与变体（components / variants）。主题就照搬这三样。

| 概念 | 在主题里是什么 | 例子 |
|---|---|---|
| 图层 | 自由摆放的节点：框、矩形、椭圆、矢量路径、图片、文字 | 立绘、花边、角标、猫耳 |
| 插槽 | 绑定到输入法数据的节点，内容由引擎填 | 拼音行、候选列表、整句补全、页码 |
| 组件 | 插槽里每一项长什么样，带变体 | 候选项：普通 / 高亮 / 云端 / 生词 |
| 自动布局 | 框按内容撑开，子节点按方向、间距、对齐排列（等价 flexbox） | 候选列表横排或竖排、窗口随内容变宽 |
| 约束 | 窗口尺寸变化时，装饰图层贴哪边 | 立绘固定在右上角，底纹横向拉伸 |
| 效果 | 投影、内阴影、图层模糊、不透明度、混合模式、描边 | 发光字、柔和投影 |
| 动画 | 状态之间的过渡 + 循环动画 | 高亮条滑动、窗口弹出、星星闪烁 |

设计师画的是「一个候选项」和「候选项之间怎么排」，渲染器按实际数据把组件实例化 N 份，再由自动布局排好。
这与 Figma 里「用组件 + 自动布局做一个列表」完全同构，所以将来可以做 Figma 插件导出主题（见「编辑器」一节）。

### 数据契约

主题能绑定的数据就是引擎与主题之间的 API，要版本化、只增不改：

| 数据 | 字段 |
|---|---|
| `preedit` | 各段文字与样式（已输入 / 未确认 / 删除线）、光标位置 |
| `candidates[]` | `index`、`text`、`annotation[]`（每段带 `tone`：译文 / 生词 / 词性）、`cloud`、`highlighted` |
| `trailing` | 整句补全或临时状态的文字、是否来自云端 |
| `page` | 页码文字、有无上一页 / 下一页 |
| `mode` | 中 / 英、全角 / 半角、方案名（状态条用） |
| `layout` | 用户选的竖排 / 横排，主题可以按它切换两套布局，也可以声明只支持一种 |

编辑器预览时用一组样例数据（短词、长词、带译文、云端词、组句中、空列表）轮流看，保证各种情况下都不走样。

## 主题包

```
sakura/                   源形态：一个目录
  theme.json              图层树 + 样式 + 动画，必需
  images/…                PNG / APNG（带透明）
  fonts/…                 随主题分发的字体（只放许可允许分发的）
  preview.png             主题列表里的缩略图
sakura.qjtheme            分发形态：同样结构的 zip
```

- 描述文件用 **JSON**，不用 TOML：图层树嵌套深，TOML 写深层嵌套很难读；它主要由编辑器生成，配 JSON Schema 做校验和编辑器提示，手改是次要的。
- 用户主题放在配置目录 `themes/`，目录或 `.qjtheme` 都认；内置主题随包。
- 包内路径一律相对 `theme.json`，禁止 `..` 与绝对路径。

## theme.json 草图

下面是一个「樱花」主题的骨架：九宫格背景图，右上角探出窗口的立绘，候选项是组件，高亮在变体之间切换时带过渡，角落有一个循环闪烁的星星。

```jsonc
{
  "schema": 1,
  "meta": { "id": "sakura", "name": "樱花", "author": "…", "license": "CC-BY-4.0" },

  "variables": {
    "ink": { "light": "#3a2a3aff", "dark": "#f5e6f0ff" },
    "petal": { "light": "#ffb7d5ff", "dark": "#b0507aff" }
  },

  "fonts": [
    { "file": "fonts/LXGWWenKai.ttf", "family": "LXGW WenKai", "license": "OFL-1.1", "subset": "tongyong-8105" }
  ],
  "text": { "family": ["LXGW WenKai", "system"], "emoji": "system" },

  "components": {
    "candidate": {
      "type": "frame",
      "layout": { "direction": "row", "gap": 4, "padding": [4, 10], "align": "center" },
      "fill": null,
      "radius": 10,
      "children": [
        { "type": "text", "bind": "index", "style": { "size": 11, "color": "@ink", "opacity": 0.5 } },
        { "type": "icon", "name": "cloud", "visible": "cloud", "size": 13 },
        { "type": "text", "bind": "text", "style": { "size": 16, "color": "@ink",
            "stroke": { "width": 0, "color": "#ffffff" } } },
        { "type": "annotation", "bind": "annotation", "style": { "size": 12 } }
      ],
      "variants": {
        "highlighted": {
          "fill": { "linear": 90, "stops": ["@petal", "#ffd9e8ff"] },
          "effects": [{ "type": "drop-shadow", "blur": 6, "y": 2, "color": "#ff7fb040" }],
          "children.text.style.stroke.width": 2
        }
      },
      "transition": { "duration": 120, "easing": "ease-out" }
    }
  },

  "root": {
    "type": "frame",
    "name": "window",
    "layout": { "direction": "column", "gap": 2, "padding": [14, 16, 12, 16] },
    "size": { "width": "hug", "height": "hug", "min_width": 220 },
    "fill": { "image": "images/paper.png", "slice": [24, 24, 24, 24] },
    "effects": [{ "type": "drop-shadow", "blur": 12, "y": 6, "color": "#0000005a" }],
    "enter": { "from": { "opacity": 0, "y": 4 }, "duration": 90 },
    "children": [
      { "type": "slot", "bind": "preedit", "style": { "size": 12, "color": "@ink" } },
      { "type": "slot", "bind": "candidates", "component": "candidate",
        "layout": { "vertical": { "direction": "column" }, "horizontal": { "direction": "row", "gap": 6 } } },
      { "type": "slot", "bind": "page", "align": "end", "style": { "size": 11, "color": "@ink" } },

      { "type": "image", "src": "images/girl.png", "absolute": true,
        "constraints": { "x": "right", "y": "top" }, "offset": [18, -46], "size": [96, 120] },
      { "type": "image", "src": "images/star.apng", "absolute": true,
        "constraints": { "x": "left", "y": "bottom" }, "offset": [-8, 6], "loop": true }
    ]
  }
}
```

要点：

- **`absolute` + `constraints`**：不参与自动布局的装饰层，按约束贴在窗口某个角或边上，可以用 `offset` 伸出窗口外（立绘探头）。
- **变体覆盖**：变体只写和默认形态不同的属性，路径写法 `children.text.style.stroke.width`。编辑器里就是 Figma 的变体面板。
- **变量带深浅两值**：深浅色不再是两份主题，而是每个颜色变量有 `light` / `dark` 两个值，跟随系统切换。
- **`hug` / `fill` / 定值**：尺寸写法照搬 Figma（按内容撑开 / 撑满父框 / 固定）。

## 已实现的格式（schema 1，2026-09-18）

上面的草图是目标；下面是渲染器现在认得的写法，内置主题 `crates/qingjian-render/themes/qingjian/theme.json` 全部用它写成，与原先写死的排版逐像素一致（快照测试）。

内置主题三个：`qingjian` 青简绿（缺省）、`system-blue` 系统蓝、`wechat` 微信绿，后两个 `extends` 青简绿、只改颜色变量。
设置界面（mac 偏好设置、Windows 设置程序「候选窗口」页）按显示名列出，写回 `[general] theme` 的 id。

用户主题（2026-09-18）：`<用户数据目录>/themes/<id>/theme.json`，由 `ThemeLibrary` 与内置主题合成一个列表（内置在前、用户按 id 排）。
目录名必须等于 `meta.id`，不能与内置主题重名、不能是外观词，读不进来的跳过并记警告。热加载靠目录戳（各 `theme.json` 的修改时间与大小）：
mac 在激活期间每秒的配置检查里 `refresh()`，Windows Server 在热加载轮询里比戳、变了就把设置重发给 UI 线程重读；保存后约一秒生效。
`extends` 目前只能以内置主题为底；主题包（`.qjtheme`）、图片与字体随包还没做。

顶层：`extends`（可选，以某个内置主题为底：对象逐键合并、数组与标量整个替换）、`schema`、`meta`（id / name / author / license）、`variables`（颜色，`"#…"` 或 `{ "light", "dark" }`）、
`text`（`gamma` 可分深浅；`styles` 是命名的字号、行高、字重，节点用 `"font": "名字"` 引用）、`components`、`windows`（`vertical` / `horizontal` 两个根节点）、`status`（状态条）。

| 节点 `type` | 属性 |
|---|---|
| `frame` | `direction`（row / column）、`fill`、`border`、`radius`、`children`；带 `table: { row_height }` 时排成表格 |
| `text` | `bind` 或 `text`、`font`、`color` |
| `icon` | `icon`（cloud / gear）、`size`、`color`；盒子缺省与图标同大，图标垂直居中 |
| `preedit` | `font`、`typed` / `rest` / `struck` 三种拼音颜色、`caret: { width, color }` |
| `annotation` | `bind`、`font`、`gloss` / `fresh` / `faint` 三种深浅 |
| `use` | `component`：引用组件，这里写的盒子属性盖过组件根节点的 |
| `repeat` | `bind`（`candidates`）、`component`：每项候选实例化一份 |

所有节点都可写：`when`（显示条件）、`margin` / `padding`（一个数或 `[上, 右, 下, 左]`，外边距可写 `"auto"`）、`gap`、`width` / `height` / `min_width`、
`position: "absolute"` + `inset`、`align_self`（start / end / center / stretch）、`span: "row"`（表格里横跨整行）、`opacity`（整棵子树）、`effects`。长度单位是点。

- **填充**（第 2 阶段第 1 步，2026-09-18）：颜色写法（含条件颜色）；`{ "linear": 角度, "stops": [...] }`（CSS 角度）；
  `{ "radial": [cx, cy], "stops": [...] }`（圆心按比例、半径到最远角）；色标是颜色（均分）或 `[颜色, 位置]`；
  `{ "image": "images/x.png", "slice": [上, 右, 下, 左], "scale": 2 }`（路径相对 theme.json，slice 为图片像素的九宫格切边、不写就拉伸，scale 为一个点对几个图片像素）。
  `border: { width, color }` 画在内侧。图片随主题目录加载（`Theme::from_dir`），只认 PNG、边长 ≤ 4096、路径不能出主题目录。
- **效果**（第 2 阶段第 2 步）：`effects: [{ "type": "drop-shadow" | "inner-shadow", "x", "y", "blur", "spread", "color" }]`，按写的顺序画；
  形状取节点自己画出来的 alpha（圆角框、九宫格图片的透明边、文字），不含子节点；`spread` 只对框生效。投影垫在节点底下、半透明填充会透出来，
  内阴影压在填充上、子节点下。窗口阴影就是根节点的投影：渲染器按根节点投影伸出的距离在位图四周留边，壳按内容区对齐光标，
  mac 面板关掉系统阴影（系统绘制退路仍用系统阴影），两端同一份像素。状态条的阴影写在 `status.effects`。
- **字重与行内字体**：样式里 `weight` 写 100–900 或 `thin` / `extralight` / `light` / `regular` / `medium` / `semibold` / `bold` / `extrabold` / `black`；
  节点的 `font` 也可以写 `{ "base": "index", "size": 12, "weight": "semibold" }` 在某个样式上改几项，只改字号时行高等比缩放。
  mac 的 SF 是可变字体、苹方是多字重集合；Windows 另加载 Segoe UI 与雅黑 / 正黑的粗细体文件，缺的字重挑最近的一档。
- **锁定外观**：`meta.appearance: "light" | "dark"`，不再跟随外观设置切换；用浅色图片做底的主题要锁浅色。
- 能力展示主题在 `crates/qingjian-render/tests/themes/showcase/`（程序生成的图片），快照测试覆盖。

- **表格**：`repeat` 出来的每份组件是一行，组件根节点的子节点依次是各列，每列取各行最宽，行高固定；末尾自动补一列吃掉剩余宽度，
  `span: "row"` 的格子（高亮条）因此能横跨整个表格。列间距用各格的外边距写。竖排三列对齐就靠它。
- **条件**：`when` 是数据字段名，`!` 取反，`a|b` 任一成立。候选项里：`highlighted`、`cloud`、`annotation`、`first`、`last`；
  整帧：`preedit`、`trailing`、`trailing.cloud`、`page`、`candidates`、`annotations`（任一候选有译文）、`highlighted`、`highlighted.annotation`。
- **绑定**：文字 `index`、`text`（候选项）、`page`、`trailing.text`；译文 `annotation`（候选项）、`highlighted.annotation`；列表 `candidates`。
- **条件颜色**：节点上的颜色可以写 `{ "if": "highlighted", "then": "@hl_text", "else": "@text" }`，条件写法同 `when`。
  高亮候选换一套颜色（系统蓝、微信绿的白字）靠它；青简绿里 `hl_*` 变量与普通颜色同值。
- **容错**：加载时检查颜色变量、文字样式、组件引用，找不到的记警告；渲染时颜色退回透明、样式退回 16/19、组件不画。

还没做：变体覆盖（现在用 `when` 分支与条件颜色代替）、图层模糊、伸出窗口的装饰、文字描边、动画、主题包与字体随包。

## 渲染器要变成什么样

现在的渲染器是「代码里写死的排版 + 一个 Theme 结构」。主题要做到上面那样，渲染器要换成**场景图 + 布局 + 合成**三段：

```
theme.json + Frame 数据
  → 实例化：插槽展开成组件实例，挑变体，算出每个节点的属性（含动画当前值）
  → 布局：Taffy（Rust 的 flexbox / grid 引擎，Bevy、Dioxus、Zed 都在用）算出每个节点的矩形
  → 绘制：tiny-skia 按图层顺序画，效果（阴影、模糊）在离屏图上做，再按混合模式合成
  → 位图 + 各命中区域（候选 i、翻页、状态条格子）
```

| 能力 | 做法 | 难度 |
|---|---|---|
| 自动布局 / 约束 | Taffy；约束是布局后按父框尺寸再算一遍绝对层的位置 | 中 |
| 填充：纯色、线性 / 径向渐变、图片、九宫格 | tiny-skia 自带渐变；图片 `Pixmap::decode_png` + 九宫格切片 | 低 |
| 矢量形状、描边 | tiny-skia `Path` / `stroke_path`；形状用 SVG path 字符串存 | 低 |
| 投影、内阴影、图层模糊 | 现有盒式模糊推广到任意 alpha 遮罩 | 低 |
| 混合模式、不透明度、裁剪 | tiny-skia 自带混合模式与遮罩 | 低 |
| 文字描边、文字阴影、渐变字 | 现在字形走 swash 位图；描边与渐变要用 swash 取字形轮廓，按路径画 | 中 |
| APNG 帧动画 | `png` crate 解帧，按时间取帧 | 低 |
| 属性动画与过渡 | 每个节点的可动画属性（位置、尺寸、不透明度、颜色、旋转、缩放）按缓动曲线插值 | 中 |
| 窗口伸出装饰 | 位图比窗口「锚点矩形」大，壳按锚点矩形对齐光标；透明像素的点击要穿透 | 中，要真机验 |

**做不到的：背景毛玻璃**。自绘位图读不到窗口后面的内容。要毛玻璃只能由平台来做（mac `NSVisualEffectView` 垫在位图下，Win11 DWM 的 Acrylic / Mica），这属于平台能力：主题里声明 `"backdrop": "blur"`，平台不支持时退回半透明纯色。
在 Windows 上，分层窗口能不能叠 DWM 背景效果要先验。

## 动画

两类：

1. **状态过渡**：同一节点在两帧之间属性变了就插值，比如高亮从第 1 项移到第 2 项、窗口弹出与消失、候选换页。由 `transition` 或 `enter` / `exit` 声明。
2. **循环动画**：APNG 帧动画，以及节点上的关键帧（`keyframes`：飘动的花瓣、呼吸的光晕）。

对渲染管线的影响：

- 现在是「来一帧数据画一次」。有动画时，壳要按显示器刷新率驱动重画（mac `CVDisplayLink` / `CADisplayLink`，Windows 用定时器 + `UpdateLayeredWindow`），动画都结束了就停。
- **首帧不能等动画**：按键到候选出现的延迟是输入法的命门。过渡只影响之后的帧，第一帧永远立刻画出完整内容。
- 缓存：不动的图层画一次，存成离屏位图；每帧只重画会动的那几层再合成。2 倍屏、600×400 像素、每帧只做合成，一帧应在 1–2 ms 内。这个数要实测。
- 省电：窗口隐藏时停止所有动画；循环动画帧率上限 30；跟随系统的「减少动态效果」设置，开着时只保留瞬时切换。
- 这些都由渲染器提供（`Renderer::advance(dt)` 之类），壳只负责按时钟调用、贴图。两个平台的动画因此逐帧一致。

## 字体

原稿里的规则不变：

- 回退链写成 `family: ["…", "system"]`，`system` 表示平台界面字体 + 按 locale 选的中日字体。
- 开源许可（OFL、Apache-2.0 等白名单，且 OS/2 `fsType` 允许嵌入）的字体，导出时默认打进包，裁成《通用规范汉字表》8105 字 + ASCII + 常用标点 + 学习语言需要的字符。
- 系统自带或商业字体（苹方、微软雅黑、方正、汉仪……）只写字族名，导出时提示会退回下一个字体。
- 包里的字体按 SHA-256 去重，存进数据目录 `fonts/`。
- emoji 用系统的，或者随包带 Noto Color Emoji（OFL）/ Twemoji（CC-BY，要署名）。
- 用 `system` 字体时保留平台补偿参数（text_gamma、opsz），观感跟原生一致；用包内字体时两端取同样的值，逐像素一致。

## 内置主题怎么办

内置「青简」主题也用这个格式写。现在的布局（顶部拼音行，候选三列：序号 / 词 / 译文）用一个竖排 frame 套行组件就能表达。
译文列对齐需要「同一列宽度取各行最大值」：这是表格语义，flexbox 表达不了。有两个办法：Taffy 的 grid 布局，或者给列表插槽加一个 `columns: "aligned"` 选项。先用 grid 试。

迁移是这项工作的验收标准：用新格式写的内置主题，与现在的渲染器逐像素对比，没有差异才算完成。基准已由快照测试 `crates/qingjian-render/tests/snapshots.rs` 固定（2026-09-18，mac 21 张），重构期间每一步都要过它。

## 兼容与安全

- `schema` 是整数，只在不兼容改动时才加一；数据契约也跟着同一个版本号走。
- 不认识的节点类型与属性：记警告并跳过这一个节点或属性，不让整个主题失败。编辑器导出时保留原样。
- 引用错误（变量、组件、图片不存在）：这一处退回默认值，记警告。
- 资源上限：单张图片边长 ≤ 4096 像素，APNG 帧数 ≤ 240，整个包 ≤ 32 MB，节点数 ≤ 2000；动画与离屏缓存受内存预算约束，超了就关掉动画，只画静态首帧。
- 主题是纯数据，没有脚本；字体与图片都由纯 Rust 的解析器处理。

## 编辑器

硬要求：**预览必须由同一个渲染器来画**，否则所见即所得是假的。

两条路，建议都走，按先后顺序：

1. **Figma 插件导出**（先做）：设计师本来就在用 Figma，Figma 的节点树（frame、auto layout、constraints、components / variants、effects、variables）与本格式几乎一一对应。
   插件按命名约定识别插槽（图层名叫 `slot:candidates` 这类），导出 `theme.json` 加切图。这是最快让设计师上手的路，也能倒逼格式与 Figma 对齐。
   局限：Figma 里看到的是 Figma 自己画的效果，与我们渲染器会有细微差异，所以导出后仍要在我们的预览里确认。动画要靠 Figma 原型或插件面板补充。
2. **自己的网页编辑器**（后做）：渲染器编译成 WASM，画布 + 图层面板 + 属性面板 + 样例数据切换 + 动画时间轴，放在官网上，用户直接导出 `.qjtheme`。
   浏览器读不到系统字体文件，预览只用包内字体和编辑器提供的开源字体，选系统字体时要标明「预览用的是替身字体」。

## 通用层与输入法层

这个方向做完，渲染器实际上就是一个「场景图 + 布局 + 动画 + 位图输出」的通用 2D 引擎，能拆出去单独发布：

- **通用层**：画布、颜色、填充、效果、文字、字体、场景图、Taffy 布局、动画、主题文件格式的基础部分。不知道「候选」这个概念。
- **输入法层**：数据契约、插槽类型、内置主题、`Frame` 到插槽数据的转换。

现在就按这个边界组织目录，以后拆起来是改 Cargo.toml 的事。要单独定许可（主仓库是 GPL），或者保留商业授权的可能，得在第一个外部贡献进来之前定好 CLA 或宽松许可。

## 分阶段

| 阶段 | 内容 | 验收 |
|---|---|---|
| 1 | 场景图 + Taffy 布局 + 现有能力（纯色、圆角、阴影、文字）；内置主题改写成 JSON（2026-09-18 完成） | 与现渲染器逐像素一致，首帧耗时不劣化 |
| 2 | 图片 / 九宫格、渐变、描边、效果、混合模式、绝对层与约束、窗口伸出 | 做出一个二次元静态主题，mac / Windows 并排一致 |
| 3 | 动画：过渡 + APNG + 关键帧；壳侧时钟驱动 | 高亮滑动流畅，空闲时 CPU 为 0，首帧延迟不变 |
| 4 | Figma 插件导出 | 设计师从 Figma 出一个主题，装上即用 |
| 5 | 网页编辑器（WASM） | 不装 Figma 也能做主题 |

## 配置键（2026-09-18 定）

`[general] appearance` 是外观（system / light / dark），`[general] theme` 是主题 id（缺省 `qingjian`）。2026-09-18 之前外观写在 `theme` 里：
没写 `appearance` 且 `theme` 是这三个词之一时按外观读、主题用内置的，所以这三个词不能当主题 id。读取一律走 `GeneralConfig::appearance()` / `theme_id()`。
Server ↔ DLL 帧协议里的字段在 Rust 里改名为 `appearance`，线上仍叫 `theme`，旧 DLL 照常能解析，协议版本不变。

## 待定

- **透明区点击穿透**：Windows 分层窗口的全透明像素天然穿透；macOS 的 NSPanel 要实测。
- **毛玻璃**：各平台的背景效果能否与自绘位图叠加，要实测。
- **Linux**：原生 Wayland 下候选窗由 Fcitx5 / IBus 画，主题要生效得自己画面板，单独调研；在那之前只承诺 mac + Windows。
- **Lottie**：设计师常用它做动效。Rust 里现成的 Lottie 渲染（velato）绑定 vello / GPU，和我们的 CPU 管线不合。第一版不支持，看需求再说。
