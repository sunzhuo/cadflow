//! Markdown document panel and editor for CADFlow.
//!
//! Provides a full-featured Markdown document editing and real-time preview panel
//! for CAD project specifications, layer descriptions, change logs, and design notes.

use egui::{Color32, RichText, Stroke, Ui, Vec2};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum DocMode {
    #[default]
    Split,
    Edit,
    Preview,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct DocState {
    pub title: String,
    pub content: String,
    pub mode: DocMode,
    #[serde(skip)]
    pub status_notice: Option<String>,
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
> 2. 点击上方工具栏可随时切换【编辑】、【预览】或【双栏】模式；
> 3. 支持快捷格式插入（标题、加粗、斜体、代码、列表、任务待办、表格）；
> 4. 点击【导出】按钮可将文档下载保存为本地 `.md` 文件；
> 5. 本文档面板无尺寸限制，可通过左边缘拖动条自由调整宽度。
"#;

impl Default for DocState {
    fn default() -> Self {
        Self {
            title: "设计说明.md".to_string(),
            content: DEFAULT_DOC_TEMPLATE.to_string(),
            mode: DocMode::Split,
            status_notice: None,
        }
    }
}

impl DocState {
    pub fn ui(&mut self, ui: &mut Ui, is_visible: &mut bool) {
        let accent_color = Color32::from_rgb(0x3d, 0x8b, 0xfd);

        ui.vertical(|ui| {
            // --- 顶部工具栏 ---
            ui.horizontal(|ui| {
                ui.heading(RichText::new("📄 设计文档").size(14.5).strong());
                ui.add_space(6.0);

                // 模式切换按钮
                let is_split = self.mode == DocMode::Split;
                let is_edit = self.mode == DocMode::Edit;
                let is_preview = self.mode == DocMode::Preview;

                if ui.selectable_label(is_split, "📑 双栏").clicked() {
                    self.mode = DocMode::Split;
                }
                if ui.selectable_label(is_edit, "✏️ 编辑").clicked() {
                    self.mode = DocMode::Edit;
                }
                if ui.selectable_label(is_preview, "👁️ 预览").clicked() {
                    self.mode = DocMode::Preview;
                }

                ui.add_space(6.0);
                ui.separator();
                ui.add_space(4.0);

                // 操作按钮
                if ui.button("💾 导出").on_hover_text("导出下载为本地 .md 文件").clicked() {
                    let filename = if self.title.trim().is_empty() {
                        "设计说明.md"
                    } else {
                        &self.title
                    };
                    #[cfg(target_arch = "wasm32")]
                    {
                        let _ = crate::download::download(filename, "text/markdown;charset=utf-8", self.content.as_bytes());
                        self.status_notice = Some("已触发导出下载".into());
                    }
                    #[cfg(not(target_arch = "wasm32"))]
                    {
                        let _ = std::fs::write(filename, &self.content);
                        self.status_notice = Some(format!("已保存至 {filename}"));
                    }
                }

                if ui.button("📋 复制").on_hover_text("复制全部 Markdown 文本到剪贴板").clicked() {
                    ui.ctx().copy_text(self.content.clone());
                    self.status_notice = Some("已复制到剪贴板".into());
                }

                if ui.button("🔄 模板").on_hover_text("重置为工程设计说明模板").clicked() {
                    self.content = DEFAULT_DOC_TEMPLATE.to_string();
                    self.status_notice = Some("已恢复默认模板".into());
                }

                if ui.button("🗑️ 清空").on_hover_text("清空文档内容").clicked() {
                    self.content.clear();
                    self.status_notice = Some("文档已清空".into());
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("✕").on_hover_text("隐藏文档面板").clicked() {
                        *is_visible = false;
                    }
                });
            });

            // 状态通知小提示
            if let Some(notice) = &self.status_notice {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(format!("✓ {notice}")).size(11.5).color(accent_color));
                });
            }

            ui.separator();

            // 快捷插入格式工具条（在编辑和双栏模式显示）
            if self.mode != DocMode::Preview {
                ui.horizontal_wrapped(|ui| {
                    ui.spacing_mut().item_spacing = Vec2::new(3.0, 2.0);
                    if ui.small_button("H1").on_hover_text("一级标题").clicked() {
                        insert_snippet(&mut self.content, "\n# 标题名称\n");
                    }
                    if ui.small_button("H2").on_hover_text("二级标题").clicked() {
                        insert_snippet(&mut self.content, "\n## 标题名称\n");
                    }
                    if ui.small_button("H3").on_hover_text("三级标题").clicked() {
                        insert_snippet(&mut self.content, "\n### 标题名称\n");
                    }
                    if ui.small_button("B 粗体").on_hover_text("加粗").clicked() {
                        insert_snippet(&mut self.content, "**粗体文字**");
                    }
                    if ui.small_button("I 斜体").on_hover_text("斜体").clicked() {
                        insert_snippet(&mut self.content, "*斜体文字*");
                    }
                    if ui.small_button("> 引用").on_hover_text("引用块").clicked() {
                        insert_snippet(&mut self.content, "\n> 引用内容\n");
                    }
                    if ui.small_button("`代码`").on_hover_text("行内代码").clicked() {
                        insert_snippet(&mut self.content, "`code`");
                    }
                    if ui.small_button("```代码块").on_hover_text("多行代码块").clicked() {
                        insert_snippet(&mut self.content, "\n```rust\n// 代码块\n```\n");
                    }
                    if ui.small_button("• 列表").on_hover_text("无序列表").clicked() {
                        insert_snippet(&mut self.content, "\n- 列表项\n");
                    }
                    if ui.small_button("1. 序号").on_hover_text("有序列表").clicked() {
                        insert_snippet(&mut self.content, "\n1. 序号项\n");
                    }
                    if ui.small_button("☑ 待办").on_hover_text("待办任务列表").clicked() {
                        insert_snippet(&mut self.content, "\n- [ ] 待办任务项\n");
                    }
                    if ui.small_button("📊 表格").on_hover_text("插入数据表格").clicked() {
                        insert_snippet(&mut self.content, "\n| 列1 | 列2 | 列3 |\n| :--- | :--- | :--- |\n| 单元格 | 单元格 | 单元格 |\n");
                    }
                    if ui.small_button("--- 分割线").on_hover_text("水平分割线").clicked() {
                        insert_snippet(&mut self.content, "\n---\n");
                    }
                });
                ui.separator();
            }

            // --- 主体区域 ---
            let available_height = ui.available_height() - 24.0;
            let mode = self.mode;

            match mode {
                DocMode::Edit => {
                    ui.allocate_ui_with_layout(
                        Vec2::new(ui.available_width(), available_height),
                        egui::Layout::top_down(egui::Align::Min),
                        |ui| {
                            render_editor(ui, &mut self.content, available_height);
                        },
                    );
                }
                DocMode::Preview => {
                    egui::ScrollArea::vertical()
                        .id_salt("doc_preview_scroll")
                        .auto_shrink([false, false])
                        .max_height(available_height)
                        .show(ui, |ui| {
                            render_markdown(ui, &self.content);
                        });
                }
                DocMode::Split => {
                    let total_width = ui.available_width();
                    let half_width = (total_width - 8.0) * 0.5;

                    ui.horizontal(|ui| {
                        // 左半部分：源码编辑
                        ui.allocate_ui_with_layout(
                            Vec2::new(half_width, available_height),
                            egui::Layout::top_down(egui::Align::Min),
                            |ui| {
                                ui.label(RichText::new("源码编辑").size(11.0).color(Color32::from_gray(160)));
                                render_editor(ui, &mut self.content, available_height - 18.0);
                            },
                        );

                        ui.separator();

                        // 右半部分：实时渲染
                        ui.allocate_ui_with_layout(
                            Vec2::new(ui.available_width(), available_height),
                            egui::Layout::top_down(egui::Align::Min),
                            |ui| {
                                ui.label(RichText::new("实时预览").size(11.0).color(Color32::from_gray(160)));
                                egui::ScrollArea::vertical()
                                    .id_salt("doc_split_preview_scroll")
                                    .auto_shrink([false, false])
                                    .max_height(available_height - 18.0)
                                    .show(ui, |ui| {
                                        render_markdown(ui, &self.content);
                                    });
                            },
                        );
                    });
                }
            }

            // --- 底部统计信息 ---
            ui.separator();
            ui.horizontal(|ui| {
                let line_count = self.content.lines().count();
                let char_count = self.content.chars().count();
                let word_count = self.content.split_whitespace().count();

                ui.label(
                    RichText::new(format!("行数: {line_count}   字数: {char_count}   词数: {word_count}"))
                        .size(11.0)
                        .color(Color32::from_gray(140)),
                );
            });
        });
    }
}

fn insert_snippet(content: &mut String, snippet: &str) {
    content.push_str(snippet);
}

fn render_editor(ui: &mut Ui, content: &mut String, height: f32) {
    egui::ScrollArea::vertical()
        .id_salt("doc_editor_scroll")
        .auto_shrink([false, false])
        .max_height(height)
        .show(ui, |ui| {
            ui.add(
                egui::TextEdit::multiline(content)
                    .desired_width(ui.available_width())
                    .desired_rows(((height / 18.0) as usize).max(8))
                    .font(egui::FontId::new(12.0, egui::FontFamily::Monospace)),
            );
        });
}

/// 解析并渲染 Markdown 内容
pub fn render_markdown(ui: &mut Ui, markdown: &str) {
    let text_color = Color32::from_rgb(0xd7, 0xdb, 0xe0);
    let accent_color = Color32::from_rgb(0x3d, 0x8b, 0xfd);
    let code_bg = Color32::from_rgb(0x28, 0x2e, 0x38);
    let border_color = Color32::from_rgb(0x38, 0x40, 0x4c);

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
                    .inner_margin(8.0)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(&code_buffer)
                                    .font(egui::FontId::new(12.0, egui::FontFamily::Monospace))
                                    .color(text_color),
                            );
                        });
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
            render_table(ui, &table_rows, code_bg, border_color, text_color);
            table_rows.clear();
        }

        // 3. 空行
        if trimmed.is_empty() {
            ui.add_space(4.0);
            continue;
        }

        // 4. 水平分割线
        if trimmed == "---" || trimmed == "***" || trimmed == "___" {
            ui.add_space(4.0);
            ui.separator();
            ui.add_space(4.0);
            continue;
        }

        // 5. 标题处理
        if let Some(h1) = trimmed.strip_prefix("# ") {
            ui.add_space(8.0);
            ui.label(RichText::new(h1).size(18.0).strong().color(text_color));
            ui.separator();
            ui.add_space(4.0);
            continue;
        }
        if let Some(h2) = trimmed.strip_prefix("## ") {
            ui.add_space(6.0);
            ui.label(RichText::new(h2).size(15.5).strong().color(text_color));
            ui.add_space(2.0);
            continue;
        }
        if let Some(h3) = trimmed.strip_prefix("### ") {
            ui.add_space(4.0);
            ui.label(RichText::new(h3).size(13.5).strong().color(text_color));
            continue;
        }
        if let Some(h4) = trimmed.strip_prefix("#### ") {
            ui.label(RichText::new(h4).size(12.5).strong().color(Color32::from_gray(160)));
            continue;
        }

        // 6. 引用块
        if let Some(quote) = trimmed.strip_prefix("> ") {
            egui::Frame::new()
                .fill(code_bg)
                .stroke(Stroke::new(1.0, border_color))
                .corner_radius(2.0)
                .inner_margin(6.0)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.painter().vline(ui.min_rect().left() + 2.0, ui.min_rect().y_range(), Stroke::new(3.0, accent_color));
                        ui.add_space(4.0);
                        render_formatted_inline(ui, quote, text_color, code_bg, accent_color);
                    });
                });
            continue;
        }

        // 7. 任务列表
        if let Some(task) = trimmed.strip_prefix("- [ ] ") {
            ui.horizontal(|ui| {
                let mut checked = false;
                ui.checkbox(&mut checked, "");
                render_formatted_inline(ui, task, text_color, code_bg, accent_color);
            });
            continue;
        }
        if let Some(task) = trimmed.strip_prefix("- [x] ").or_else(|| trimmed.strip_prefix("- [X] ")) {
            ui.horizontal(|ui| {
                let mut checked = true;
                ui.checkbox(&mut checked, "");
                render_formatted_inline(ui, task, text_color, code_bg, accent_color);
            });
            continue;
        }

        // 8. 无序列表
        if let Some(item) = trimmed.strip_prefix("- ").or_else(|| trimmed.strip_prefix("* ")) {
            ui.horizontal(|ui| {
                ui.label(RichText::new("  •").color(accent_color).strong());
                render_formatted_inline(ui, item, text_color, code_bg, accent_color);
            });
            continue;
        }

        // 9. 有序列表
        if let Some((num, rest)) = parse_ordered_list(trimmed) {
            ui.horizontal(|ui| {
                ui.label(RichText::new(format!("  {num}.")).color(accent_color));
                render_formatted_inline(ui, rest, text_color, code_bg, accent_color);
            });
            continue;
        }

        // 10. 普通文本段落
        ui.horizontal_wrapped(|ui| {
            render_formatted_inline(ui, trimmed, text_color, code_bg, accent_color);
        });
    }

    if !table_rows.is_empty() {
        render_table(ui, &table_rows, code_bg, border_color, text_color);
    }
}

fn parse_ordered_list(s: &str) -> Option<(usize, &str)> {
    let dot_idx = s.find(". ")?;
    let num_str = &s[..dot_idx];
    let num = num_str.parse::<usize>().ok()?;
    Some((num, &s[dot_idx + 2..]))
}

fn render_table(ui: &mut Ui, rows: &[Vec<String>], bg: Color32, border: Color32, text: Color32) {
    if rows.is_empty() {
        return;
    }
    ui.add_space(4.0);
    egui::Frame::new()
        .fill(bg)
        .stroke(Stroke::new(1.0, border))
        .corner_radius(3.0)
        .inner_margin(6.0)
        .show(ui, |ui| {
            egui::Grid::new("md_table_grid")
                .striped(true)
                .spacing(Vec2::new(12.0, 4.0))
                .show(ui, |ui| {
                    for (row_idx, row) in rows.iter().enumerate() {
                        for cell in row {
                            if row_idx == 0 {
                                ui.label(RichText::new(clean_markdown_syntax(cell)).strong().color(text));
                            } else {
                                render_formatted_inline(ui, cell, text, bg, Color32::from_rgb(0x3d, 0x8b, 0xfd));
                            }
                        }
                        ui.end_row();
                    }
                });
        });
    ui.add_space(4.0);
}

fn clean_markdown_syntax(s: &str) -> String {
    s.replace("**", "").replace('*', "").replace('`', "")
}

/// 内联格式化渲染（粗体、斜体、代码、普通文字）
fn render_formatted_inline(ui: &mut Ui, text: &str, text_color: Color32, code_bg: Color32, accent: Color32) {
    let mut i = 0;
    let chars: Vec<char> = text.chars().collect();
    let len = chars.len();

    let mut buf = String::new();

    while i < len {
        // **粗体**
        if i + 1 < len && chars[i] == '*' && chars[i + 1] == '*' {
            if !buf.is_empty() {
                ui.label(RichText::new(&buf).color(text_color));
                buf.clear();
            }
            i += 2;
            let start = i;
            while i + 1 < len && !(chars[i] == '*' && chars[i + 1] == '*') {
                i += 1;
            }
            let bold_text: String = chars[start..i].iter().collect();
            ui.label(RichText::new(bold_text).strong().color(text_color));
            if i + 1 < len {
                i += 2;
            }
            continue;
        }

        // *斜体*
        if chars[i] == '*' {
            if !buf.is_empty() {
                ui.label(RichText::new(&buf).color(text_color));
                buf.clear();
            }
            i += 1;
            let start = i;
            while i < len && chars[i] != '*' {
                i += 1;
            }
            let italic_text: String = chars[start..i].iter().collect();
            ui.label(RichText::new(italic_text).italics().color(text_color));
            if i < len {
                i += 1;
            }
            continue;
        }

        // `代码`
        if chars[i] == '`' {
            if !buf.is_empty() {
                ui.label(RichText::new(&buf).color(text_color));
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
                    .font(egui::FontId::new(12.0, egui::FontFamily::Monospace))
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
        ui.label(RichText::new(buf).color(text_color));
    }
}
