//! Markdown document panel and editor for CADFlow.
//!
//! Refactored to reference the "Tool Sets" panel style:
//! - Top header: "文档" text only
//! - Sub-tabs: "编辑" (Edit) and "预览" (Preview)
//! - No other functional buttons inside the panel (all buttons are located in the top toolbar)
//! - Pure white background (#ffffff) except for borders
//! - Typography and font styling reference StackEdit

use egui::{pos2, vec2, Color32, CornerRadius, Margin, Rect, RichText, Sense, Stroke, Ui, Vec2};
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

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct DocState {
    pub title: String,
    pub content: String,
    pub sub_tab: DocSubTab,
    #[serde(skip)]
    pub undo_stack: Vec<String>,
    #[serde(skip)]
    pub redo_stack: Vec<String>,
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

impl Default for DocState {
    fn default() -> Self {
        Self {
            title: "设计说明.md".to_string(),
            content: DEFAULT_DOC_TEMPLATE.to_string(),
            sub_tab: DocSubTab::Edit,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }
}

impl DocState {
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

    /// 渲染重构后的文档面板（纯白底色，参考工具集面板样式）
    pub fn ui(&mut self, ui: &mut Ui) {
        let t = cadcraft_ui_egui::theme::Tokens::get();

        // 1. 最上方“文档”字样标题栏 (高度 32px，参考工具集顶部样式)
        let header_h = 32.0;
        let (hr, _) = ui.allocate_exact_size(vec2(ui.available_width(), header_h), Sense::hover());
        ui.painter().rect_filled(hr, 0.0, Color32::WHITE);
        ui.painter().text(
            pos2(hr.left() + 14.0, hr.center().y),
            egui::Align2::LEFT_CENTER,
            "文档",
            egui::FontId::proportional(15.0),
            Color32::from_rgb(0x2c, 0x3e, 0x50),
        );
        ui.painter().hline(hr.x_range(), hr.bottom(), Stroke::new(1.0, t.border));

        // 2. 下面是“编辑”和“预览”两个子标签页切换栏 (高度 30px)
        let tab_h = 30.0;
        let (tr, _) = ui.allocate_exact_size(vec2(ui.available_width(), tab_h), Sense::hover());
        ui.painter().rect_filled(tr, 0.0, Color32::from_rgb(0xf8, 0xf9, 0xfa));

        let tab_w = 72.0;
        let tabs = [(DocSubTab::Edit, "编辑"), (DocSubTab::Preview, "预览")];
        let mut clicked_tab = None;

        for (idx, (variant, label)) in tabs.iter().enumerate() {
            let tab_rect = Rect::from_min_size(pos2(tr.left() + 10.0 + idx as f32 * (tab_w + 4.0), tr.top()), vec2(tab_w, tab_h));
            let is_active = self.sub_tab == *variant;
            let resp = ui.interact(tab_rect, ui.id().with(("doc_tab", idx)), Sense::click());

            if is_active {
                ui.painter().rect_filled(tab_rect, CornerRadius::same(0), Color32::WHITE);
                let indicator = Rect::from_min_max(pos2(tab_rect.left(), tab_rect.bottom() - 2.0), tab_rect.max);
                ui.painter().rect_filled(indicator, CornerRadius::same(1), Color32::from_rgb(0x3d, 0x8b, 0xfd));
            }

            ui.painter().text(
                tab_rect.center(),
                egui::Align2::CENTER_CENTER,
                *label,
                egui::FontId::proportional(13.0),
                if is_active { Color32::from_rgb(0x2c, 0x3e, 0x50) } else { Color32::from_rgb(0x7f, 0x8c, 0x8d) },
            );

            if resp.clicked() {
                clicked_tab = Some(*variant);
            }
        }

        if let Some(tab) = clicked_tab {
            self.sub_tab = tab;
        }

        ui.painter().hline(tr.x_range(), tr.bottom(), Stroke::new(1.0, t.border));

        // 3. 面板主体内容区域（纯白底色，StackEdit 字体与样式）
        egui::Frame::NONE
            .fill(Color32::WHITE)
            .inner_margin(Margin::same(12))
            .show(ui, |ui| {
                let avail_h = ui.available_height() - 10.0;
                match self.sub_tab {
                    DocSubTab::Edit => {
                        egui::ScrollArea::vertical()
                            .id_salt("stackedit_editor_scroll")
                            .auto_shrink([false, false])
                            .max_height(avail_h)
                            .show(ui, |ui| {
                                let font_id = egui::FontId::new(16.0, egui::FontFamily::Proportional);
                                ui.add(
                                    egui::TextEdit::multiline(&mut self.content)
                                        .desired_width(ui.available_width())
                                        .desired_rows(((avail_h / 24.0) as usize).max(10))
                                        .font(font_id)
                                        .text_color(Color32::from_rgb(0x2c, 0x3e, 0x50))
                                        .frame(egui::Frame::NONE),
                                );
                            });
                    }
                    DocSubTab::Preview => {
                        egui::ScrollArea::vertical()
                            .id_salt("stackedit_preview_scroll")
                            .auto_shrink([false, false])
                            .max_height(avail_h)
                            .show(ui, |ui| {
                                render_stackedit_markdown(ui, &self.content);
                            });
                    }
                }
            });
    }
}

/// 按照 StackEdit 风格渲染 Markdown 文档内容 (白底色，清爽优雅排版)
pub fn render_stackedit_markdown(ui: &mut Ui, markdown: &str) {
    let text_color = Color32::from_rgb(0x2c, 0x3e, 0x50);       // #2c3e50 (StackEdit 默认深蓝灰字色)
    let heading_color = Color32::from_rgb(0x1a, 0x25, 0x2f);    // #1a252f
    let accent_color = Color32::from_rgb(0x3d, 0x8b, 0xfd);     // #3d8bfd
    let code_bg = Color32::from_rgb(0xf6, 0xf8, 0xfa);          // #f6f8fa
    let border_color = Color32::from_rgb(0xe1, 0xe4, 0xe8);     // #e1e4e8

    let mut in_code_block = false;
    let mut code_buffer = String::new();
    let mut table_rows: Vec<Vec<String>> = Vec::new();

    let lines: Vec<&str> = markdown.lines().collect();

    for line in lines {
        let trimmed = line.trim();

        // 1. 代码块处理
        if trimmed.starts_with("```") {
            if in_code_block {
                egui::Frame::new()
                    .fill(code_bg)
                    .stroke(Stroke::new(1.0, border_color))
                    .corner_radius(4.0)
                    .inner_margin(Margin::same(10))
                    .show(ui, |ui| {
                        ui.label(
                            RichText::new(&code_buffer)
                                .font(egui::FontId::new(14.0, egui::FontFamily::Monospace))
                                .color(Color32::from_rgb(0x24, 0x29, 0x2e)),
                        );
                    });
                code_buffer.clear();
                in_code_block = false;
            } else {
                in_code_block = true;
            }
            continue;
        }

        if in_code_block {
            code_buffer.push_str(line);
            code_buffer.push('\n');
            continue;
        }

        // 2. 表格收集与渲染
        if trimmed.starts_with('|') && trimmed.ends_with('|') {
            if trimmed.contains("---") {
                continue;
            }
            let cells: Vec<String> = trimmed
                .split('|')
                .filter(|s| !s.is_empty())
                .map(|s| s.trim().to_string())
                .collect();
            if !cells.is_empty() {
                table_rows.push(cells);
            }
            continue;
        } else if !table_rows.is_empty() {
            render_stackedit_table(ui, &table_rows, code_bg, border_color, text_color);
            table_rows.clear();
        }

        // 3. 空行
        if trimmed.is_empty() {
            ui.add_space(6.0);
            continue;
        }

        // 4. 水平分割线
        if trimmed == "---" || trimmed == "***" || trimmed == "___" {
            ui.add_space(6.0);
            let (sep_r, _) = ui.allocate_exact_size(vec2(ui.available_width(), 1.0), Sense::hover());
            ui.painter().hline(sep_r.x_range(), sep_r.center().y, Stroke::new(1.0, border_color));
            ui.add_space(6.0);
            continue;
        }

        // 5. 标题处理 (StackEdit 风格字号与底边线)
        if let Some(h1) = trimmed.strip_prefix("# ") {
            ui.add_space(10.0);
            ui.label(RichText::new(h1).size(22.0).strong().color(heading_color));
            let (sep_r, _) = ui.allocate_exact_size(vec2(ui.available_width(), 1.0), Sense::hover());
            ui.painter().hline(sep_r.x_range(), sep_r.center().y, Stroke::new(1.0, border_color));
            ui.add_space(4.0);
            continue;
        }
        if let Some(h2) = trimmed.strip_prefix("## ") {
            ui.add_space(8.0);
            ui.label(RichText::new(h2).size(18.5).strong().color(heading_color));
            let (sep_r, _) = ui.allocate_exact_size(vec2(ui.available_width(), 1.0), Sense::hover());
            ui.painter().hline(sep_r.x_range(), sep_r.center().y, Stroke::new(1.0, border_color));
            ui.add_space(3.0);
            continue;
        }
        if let Some(h3) = trimmed.strip_prefix("### ") {
            ui.add_space(6.0);
            ui.label(RichText::new(h3).size(16.0).strong().color(heading_color));
            continue;
        }
        if let Some(h4) = trimmed.strip_prefix("#### ") {
            ui.add_space(4.0);
            ui.label(RichText::new(h4).size(14.5).strong().color(Color32::from_rgb(0x4a, 0x55, 0x68)));
            continue;
        }

        // 6. 引用块 (StackEdit 风格绿/蓝条左边框)
        if let Some(quote) = trimmed.strip_prefix("> ") {
            egui::Frame::new()
                .fill(code_bg)
                .stroke(Stroke::NONE)
                .corner_radius(CornerRadius::same(2))
                .inner_margin(Margin::symmetric(10, 6))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        let vr = ui.min_rect();
                        ui.painter().vline(vr.left() + 1.0, vr.y_range(), Stroke::new(3.5, Color32::from_rgb(0x42, 0xb9, 0x83)));
                        ui.add_space(6.0);
                        render_formatted_text(ui, quote, text_color, code_bg, accent_color);
                    });
                });
            continue;
        }

        // 7. 任务列表
        if let Some(task) = trimmed.strip_prefix("- [ ] ") {
            ui.horizontal(|ui| {
                let mut checked = false;
                ui.checkbox(&mut checked, "");
                render_formatted_text(ui, task, text_color, code_bg, accent_color);
            });
            continue;
        }
        if let Some(task) = trimmed.strip_prefix("- [x] ").or_else(|| trimmed.strip_prefix("- [X] ")) {
            ui.horizontal(|ui| {
                let mut checked = true;
                ui.checkbox(&mut checked, "");
                render_formatted_text(ui, task, text_color, code_bg, accent_color);
            });
            continue;
        }

        // 8. 无序列表
        if let Some(item) = trimmed.strip_prefix("- ").or_else(|| trimmed.strip_prefix("* ")) {
            ui.horizontal(|ui| {
                ui.label(RichText::new("  •").size(15.0).color(accent_color).strong());
                render_formatted_text(ui, item, text_color, code_bg, accent_color);
            });
            continue;
        }

        // 9. 有序列表
        if let Some((num, rest)) = parse_ordered_list(trimmed) {
            ui.horizontal(|ui| {
                ui.label(RichText::new(format!("  {num}.")).size(14.5).color(accent_color));
                render_formatted_text(ui, rest, text_color, code_bg, accent_color);
            });
            continue;
        }

        // 10. 普通段落
        ui.horizontal_wrapped(|ui| {
            render_formatted_text(ui, trimmed, text_color, code_bg, accent_color);
        });
    }

    if !table_rows.is_empty() {
        render_stackedit_table(ui, &table_rows, code_bg, border_color, text_color);
    }
}

fn parse_ordered_list(s: &str) -> Option<(usize, &str)> {
    let dot_idx = s.find(". ")?;
    let num_str = &s[..dot_idx];
    let num = num_str.parse::<usize>().ok()?;
    Some((num, &s[dot_idx + 2..]))
}

fn render_stackedit_table(ui: &mut Ui, rows: &[Vec<String>], bg: Color32, border: Color32, text: Color32) {
    if rows.is_empty() {
        return;
    }
    ui.add_space(4.0);
    egui::Frame::new()
        .fill(Color32::WHITE)
        .stroke(Stroke::new(1.0, border))
        .corner_radius(CornerRadius::same(3))
        .inner_margin(Margin::same(8))
        .show(ui, |ui| {
            egui::Grid::new("stackedit_table_grid")
                .striped(true)
                .spacing(Vec2::new(14.0, 6.0))
                .show(ui, |ui| {
                    for (row_idx, row) in rows.iter().enumerate() {
                        for cell in row {
                            if row_idx == 0 {
                                ui.label(RichText::new(clean_markdown_syntax(cell)).strong().size(13.5).color(text));
                            } else {
                                render_formatted_text(ui, cell, text, bg, Color32::from_rgb(0x3d, 0x8b, 0xfd));
                            }
                        }
                        ui.end_row();
                    }
                });
        });
    ui.add_space(4.0);
}

fn clean_markdown_syntax(s: &str) -> String {
    s.replace("**", "").replace("~~", "").replace(['*', '`'], "")
}

/// 内联格式化渲染（粗体、斜体、代码、普通文字）
fn render_formatted_text(ui: &mut Ui, text: &str, text_color: Color32, code_bg: Color32, accent: Color32) {
    let mut i = 0;
    let chars: Vec<char> = text.chars().collect();
    let len = chars.len();
    let mut buf = String::new();

    while i < len {
        // **粗体**
        if i + 1 < len && chars[i] == '*' && chars[i + 1] == '*' {
            if !buf.is_empty() {
                ui.label(RichText::new(&buf).size(15.0).color(text_color));
                buf.clear();
            }
            i += 2;
            let start = i;
            while i + 1 < len && !(chars[i] == '*' && chars[i + 1] == '*') {
                i += 1;
            }
            let bold_text: String = chars[start..i].iter().collect();
            ui.label(RichText::new(bold_text).size(15.0).strong().color(text_color));
            if i + 1 < len {
                i += 2;
            }
            continue;
        }

        // ~~删除线~~
        if i + 1 < len && chars[i] == '~' && chars[i + 1] == '~' {
            if !buf.is_empty() {
                ui.label(RichText::new(&buf).size(15.0).color(text_color));
                buf.clear();
            }
            i += 2;
            let start = i;
            while i + 1 < len && !(chars[i] == '~' && chars[i + 1] == '~') {
                i += 1;
            }
            let strike_text: String = chars[start..i].iter().collect();
            ui.label(RichText::new(strike_text).size(15.0).strikethrough().color(Color32::from_rgb(0x7f, 0x8c, 0x8d)));
            if i + 1 < len {
                i += 2;
            }
            continue;
        }

        // *斜体*
        if chars[i] == '*' {
            if !buf.is_empty() {
                ui.label(RichText::new(&buf).size(15.0).color(text_color));
                buf.clear();
            }
            i += 1;
            let start = i;
            while i < len && chars[i] != '*' {
                i += 1;
            }
            let italic_text: String = chars[start..i].iter().collect();
            ui.label(RichText::new(italic_text).size(15.0).italics().color(text_color));
            if i < len {
                i += 1;
            }
            continue;
        }

        // `代码`
        if chars[i] == '`' {
            if !buf.is_empty() {
                ui.label(RichText::new(&buf).size(15.0).color(text_color));
                buf.clear();
            }
            i += 1;
            let start = i;
            while i < len && chars[i] != '`' {
                i += 1;
            }
            let code_text: String = chars[start..i].iter().collect();
            ui.label(
                RichText::new(code_text)
                    .font(egui::FontId::new(13.5, egui::FontFamily::Monospace))
                    .background_color(code_bg)
                    .color(accent),
            );
            if i < len {
                i += 1;
            }
            continue;
        }

        buf.push(chars[i]);
        i += 1;
    }

    if !buf.is_empty() {
        ui.label(RichText::new(buf).size(15.0).color(text_color));
    }
}
