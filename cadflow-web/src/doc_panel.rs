//! Markdown document panel and editor for CADFlow.
//!
//! Refactored to reference the "Tool Sets" panel style:
//! - Top header: "文档" title and [编辑] [预览] sub-tabs styled matching Tool Sets
//!   (chrome and tab_active background, no blue underline/fill)
//! - No extra buttons in the panel (all buttons in toolbar)
//! - Only document content area uses pure white background (#ffffff)
//! - Typography and font styling reference StackEdit
//! - Markdown preview parsed & rendered natively using egui_commonmark

use egui::{pos2, vec2, Color32, Rect, Sense, Stroke, Ui};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum DocSubTab {
    #[default]
    Edit,
    Preview,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DocAction {
    Undo,
    Redo,
    Bold,
    Italic,
    Heading,
    Strikethrough,
    UnorderedList,
    OrderedList,
    CheckList,
    Quote,
    Code,
    Table,
    Link,
    Image,
}

#[derive(Default)]
pub struct DocState {
    pub title: String,
    pub content: String,
    pub sub_tab: DocSubTab,
    pub undo_stack: Vec<String>,
    pub redo_stack: Vec<String>,
    pub cache: egui_commonmark::CommonMarkCache,
}

pub const DEFAULT_DOC_TEMPLATE: &str = r#"# 数字化工程设计与技术说明书 (CADFlow Design Notes)

> **项目编号**：CF-2026-001  
> **设计阶段**：方案设计 / 协同建模  
> **主设计师**：CADFlow 协同设计组  
> **设计时间**：2026年10月  

---

## 1. 工程概况与设计依据
本项目采用 **CADCraft / CADFlow** 跨平台数字化建模系统进行全流程协同设计与图纸管理。

### 1.1 参照技术标准与规范
- 《房屋建筑制图统一标准》GB/T 50001
- 《总图制图标准》GB/T 50103
- 《建筑制图标准》GB/T 50104
- 《CAD工程制图规则》GB/T 18229

---

## 2. 图层与线型规范定义
| 图层名称 (Layer) | 颜色标识 (Color) | 默认线型 (Linetype) | 用途分类与图元说明 |
| :--- | :--- | :--- | :--- |
| **0** | 白色 (7/White) | Continuous | 块定义基准与默认绘图图层 |
| **WALL** | 红色 (1/Red) | Continuous | 承重墙体、核心筒及主要剪力墙结构 |
| **COLUMN** | 品红 (6/Magenta) | Continuous | 钢结构与混凝土承重框架柱 |
| **DOOR_WIN** | 绿色 (3/Green) | Continuous | 门窗构件、开启弧线及洞口定位 |
| **DIMENSION** | 青色 (4/Cyan) | Continuous | 线性尺寸、对齐标注与标高符号 |
| **AXIS** | 红色 (1/Red) | Center | 主辅定位轴线与网格编号系统 |
| **FURNITURE** | 灰色 (8/Gray) | Continuous | 室内家具、设备布置与活动隔断 |

---

## 3. 设计任务推进与复核清单
- [x] 完成建筑平面轴线基准网格建立
- [x] 绘制外轮廓边界与剪力墙体布置
- [ ] 门窗位置复核与构件编码标注
- [ ] 导出标准 DXF 交换格式并提交校审
- [ ] 编制各专业协同管线综合预留方案

---

## 4. 技术备忘与注意事项
> 📌 **重要提示**：
> 1. 本面板支持全功能 **Markdown** 语法编辑与实时渲染；
> 2. 点击上方子标签可切换【编辑】与【预览】；
> 3. 工具栏右侧提供完整的 Markdown 编辑工具组；
> 4. 点击【文档】按钮可平滑展开与折叠文档面板。
"#;

impl DocState {
    pub fn new() -> Self {
        Self {
            title: "设计说明.md".to_string(),
            content: DEFAULT_DOC_TEMPLATE.to_string(),
            sub_tab: DocSubTab::Edit,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            cache: egui_commonmark::CommonMarkCache::default(),
        }
    }

    /// 执行来自顶部工具栏的 Markdown 编辑操作
    pub fn apply_action(&mut self, action: DocAction) {
        match action {
            DocAction::Undo => {
                if let Some(prev) = self.undo_stack.pop() {
                    self.redo_stack.push(self.content.clone());
                    self.content = prev;
                }
            }
            DocAction::Redo => {
                if let Some(next) = self.redo_stack.pop() {
                    self.undo_stack.push(self.content.clone());
                    self.content = next;
                }
            }
            DocAction::Bold => self.record_and_insert("**粗体文字**"),
            DocAction::Italic => self.record_and_insert("*斜体文字*"),
            DocAction::Heading => self.record_and_insert("\n## 标题\n"),
            DocAction::Strikethrough => self.record_and_insert("~~删除线文字~~"),
            DocAction::UnorderedList => self.record_and_insert("\n- 列表项\n"),
            DocAction::OrderedList => self.record_and_insert("\n1. 序号项\n"),
            DocAction::CheckList => self.record_and_insert("\n- [ ] 待办项\n"),
            DocAction::Quote => self.record_and_insert("\n> 引用内容\n"),
            DocAction::Code => self.record_and_insert("\n```rust\n// 代码块\n```\n"),
            DocAction::Table => self.record_and_insert("\n| 标题1 | 标题2 |\n| :--- | :--- |\n| 单元格1 | 单元格2 |\n"),
            DocAction::Link => self.record_and_insert("[链接说明](https://example.com)"),
            DocAction::Image => self.record_and_insert("![图片描述](https://example.com/image.png)"),
        }
    }

    fn record_and_insert(&mut self, snippet: &str) {
        self.undo_stack.push(self.content.clone());
        self.redo_stack.clear();
        self.content.push_str(snippet);
    }

    /// 渲染重构后的文档面板 (Req 9 & Req 11)
    /// - 头部样式参考工具集 (Tool Sets) 面板：标签与子标签样式 (背景色与选中色与工具集一致)
    /// - 仅文档内容部分底色使用纯白色
    /// - Markdown 字符串自动解析渲染为原生 egui 元素 (egui_commonmark)
    pub fn ui(&mut self, ui: &mut Ui) {
        let t = cadcraft_ui_egui::theme::Tokens::get();
        let p = ui.painter().clone();

        // 1. 最上方“文档”主标签与“编辑/预览”子标签栏 (高度 28px，严格参考 CADCraft 工具集样式)
        let tab_h = 28.0;
        let full_w = ui.available_width();
        let (tab_bar_rect, _) = ui.allocate_exact_size(vec2(full_w, tab_h), Sense::hover());
        p.rect_filled(tab_bar_rect, 0.0, t.chrome);

        // “文档”主标题标签 (固定宽 76px，显示当前处于文档侧边栏)
        let doc_title_w = 76.0;
        let title_rect = Rect::from_min_size(tab_bar_rect.min, vec2(doc_title_w, tab_h));
        p.rect_filled(title_rect, 0.0, t.tab_active);
        p.text(
            title_rect.center(),
            egui::Align2::CENTER_CENTER,
            "文档",
            egui::FontId::proportional(13.5),
            t.text,
        );

        // “编辑”与“预览”子标签
        let subtabs = [(DocSubTab::Edit, "编辑"), (DocSubTab::Preview, "预览")];
        let sub_w = 64.0;
        let mut x = title_rect.right() + 4.0;
        let mut clicked_sub = None;

        for (variant, label) in subtabs {
            let tr = Rect::from_min_size(pos2(x, tab_bar_rect.top()), vec2(sub_w, tab_h));
            let active = self.sub_tab == variant;
            let resp = ui.interact(tr, ui.id().with(("doc_subtab", label)), Sense::click());

            p.rect_filled(tr, 0.0, if active { t.tab_active } else { t.chrome });
            p.text(
                tr.center(),
                egui::Align2::CENTER_CENTER,
                label,
                egui::FontId::proportional(13.0),
                if active { t.text } else { t.text_dim },
            );

            if resp.clicked() {
                clicked_sub = Some(variant);
            }
            x += sub_w;
        }

        if let Some(st) = clicked_sub {
            self.sub_tab = st;
        }

        // 分隔线
        p.hline(tab_bar_rect.x_range(), tab_bar_rect.bottom(), Stroke::new(1.0, t.border));

        // 2. 文档内容区域（纯白色背景，包含编辑框或原生 CommonMark 渲染器）
        let content_rect = Rect::from_min_max(pos2(tab_bar_rect.left(), tab_bar_rect.bottom()), ui.max_rect().max);
        ui.painter().rect_filled(content_rect, 0.0, Color32::WHITE);

        ui.scope_builder(
            egui::UiBuilder::new().max_rect(content_rect.shrink2(vec2(16.0, 14.0))),
            |ui| {
                // 在白色背景下设置优雅易读的深色文字排版风格 (参考 StackEdit)
                let text_color = Color32::from_rgb(0x24, 0x29, 0x2f);
                ui.visuals_mut().override_text_color = Some(text_color);
                ui.visuals_mut().widgets.noninteractive.fg_stroke = Stroke::new(1.0, text_color);
                ui.visuals_mut().widgets.inactive.fg_stroke = Stroke::new(1.0, text_color);

                match self.sub_tab {
                    DocSubTab::Edit => {
                        egui::ScrollArea::vertical()
                            .id_salt("cadflow_doc_edit_scroll")
                            .auto_shrink([false, false])
                            .show(ui, |ui| {
                                ui.add(
                                    egui::TextEdit::multiline(&mut self.content)
                                        .desired_width(f32::INFINITY)
                                        .frame(egui::Frame::NONE)
                                        .font(egui::TextStyle::Body),
                                );
                            });
                    }
                    DocSubTab::Preview => {
                        egui::ScrollArea::vertical()
                            .id_salt("cadflow_doc_preview_scroll")
                            .auto_shrink([false, false])
                            .show(ui, |ui| {
                                ui.spacing_mut().item_spacing = vec2(0.0, 8.0);
                                egui_commonmark::CommonMarkViewer::new().show(ui, &mut self.cache, &self.content);
                            });
                    }
                }
            },
        );
    }
}
