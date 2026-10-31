//! Markdown document panel and editor for CADFlow.
//!
//! Refactored to mount native HTML <div> overlay powered by EasyMDE:
//! - Top header: "文档" title matching CADCraft Tool Sets style
//! - Sub header: File tabs ("未命名", editable on click, '+' to create "未命名"+number, switchable)
//! - Content area: mounts native HTML <div> overlay with EasyMDE for native Chinese IME, typography & performance
//! - Bottom footer: "预览" toggle button (pressed/raised states) and live character counter
//! - Sub-pixel synchronized sliding animation: overlay rectangle coordinates sync every frame with the panel

use std::sync::Arc;
use egui::{pos2, vec2, Color32, CornerRadius, Rect, Sense, Stroke, Ui};
use serde::{Deserialize, Serialize};
use wasm_bindgen::JsCast;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DocFile {
    pub title: String,
    pub content: String,
}

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
> 1. 本面板支持全功能 **Markdown** 原生 DOM 编辑与实时渲染；
> 2. 点击下方【预览】按钮可切换编辑与预览视图（有按下与抬起两种状态）；
> 3. 点击上方文件标签可切换文档或点击修改名称，点击右侧【+】号分配“未命名”+数字新建文档；
> 4. 顶部主工具栏提供 Markdown 快捷操作工具组；
> 5. 点击【文档】按钮可平滑展开与折叠文档侧边栏。
"#;

pub struct DocState {
    pub files: Vec<DocFile>,
    pub active_index: usize,
    pub next_file_number: usize,
    pub is_preview: bool,
    pub editing_tab: Option<usize>,
    pub just_started_editing: bool,
    pub title: String,
    pub content: String,
    pub sub_tab: DocSubTab,
    pub undo_stack: Vec<String>,
    pub redo_stack: Vec<String>,
    overlay_mounted: bool,
}

impl Default for DocState {
    fn default() -> Self {
        Self::new()
    }
}

impl DocState {
    pub fn new() -> Self {
        Self {
            files: Vec::new(),
            active_index: 0,
            next_file_number: 1,
            is_preview: false,
            editing_tab: None,
            just_started_editing: false,
            title: String::new(),
            content: String::new(),
            sub_tab: DocSubTab::Edit,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            overlay_mounted: false,
        }
    }

    /// 导出所有文档数据用于打包序列化
    pub fn export_payload(&mut self) -> crate::dxf_doc::CadflowDocsPayload {
        if !self.files.is_empty() {
            let current_text = self.get_content();
            if let Some(active_file) = self.files.get_mut(self.active_index) {
                active_file.content = current_text;
            }
        }
        crate::dxf_doc::CadflowDocsPayload {
            version: 1,
            files: self.files.iter().map(|f| crate::dxf_doc::DocFilePayload {
                title: f.title.clone(),
                content: f.content.clone(),
            }).collect(),
            active_index: self.active_index,
        }
    }

    /// 从反序列化的数据中加载文档列表并更新编辑器
    pub fn load_payload(&mut self, payload: crate::dxf_doc::CadflowDocsPayload) {
        self.files = payload.files.into_iter().map(|f| DocFile {
            title: f.title,
            content: f.content,
        }).collect();
        self.active_index = if self.files.is_empty() {
            0
        } else {
            payload.active_index.min(self.files.len() - 1)
        };
        self.editing_tab = None;
        self.next_file_number = self.files.len() + 1;
        let content = self.files.get(self.active_index).map(|f| f.content.as_str()).unwrap_or("");
        self.set_easymde_content(content);
    }

    /// 清空所有文档
    pub fn clear(&mut self) {
        self.files.clear();
        self.active_index = 0;
        self.next_file_number = 1;
        self.editing_tab = None;
        self.title.clear();
        self.content.clear();
        self.set_easymde_content("");
    }

    /// 执行来自顶部工具栏的 Markdown 编辑操作 (通过 EasyMDE 实例 API 实现)
    pub fn apply_action(&mut self, action: DocAction) {
        // 若当前处于预览状态，执行编辑操作前自动退出预览
        if self.is_preview {
            self.set_preview(false);
        }

        let js = match action {
            DocAction::Undo => "if (window.__cadflow_easymde) { window.__cadflow_easymde.codemirror.undo(); }",
            DocAction::Redo => "if (window.__cadflow_easymde) { window.__cadflow_easymde.codemirror.redo(); }",
            DocAction::Bold => "if (window.__cadflow_easymde) { EasyMDE.toggleBold(window.__cadflow_easymde); }",
            DocAction::Italic => "if (window.__cadflow_easymde) { EasyMDE.toggleItalic(window.__cadflow_easymde); }",
            DocAction::Heading => "if (window.__cadflow_easymde) { EasyMDE.toggleHeadingSmaller(window.__cadflow_easymde); }",
            DocAction::Strikethrough => "if (window.__cadflow_easymde) { EasyMDE.toggleStrikethrough(window.__cadflow_easymde); }",
            DocAction::UnorderedList => "if (window.__cadflow_easymde) { EasyMDE.toggleUnorderedList(window.__cadflow_easymde); }",
            DocAction::OrderedList => "if (window.__cadflow_easymde) { EasyMDE.toggleOrderedList(window.__cadflow_easymde); }",
            DocAction::CheckList => "if (window.__cadflow_easymde) { EasyMDE.toggleCheckList(window.__cadflow_easymde); }",
            DocAction::Quote => "if (window.__cadflow_easymde) { EasyMDE.toggleBlockquote(window.__cadflow_easymde); }",
            DocAction::Code => "if (window.__cadflow_easymde) { EasyMDE.toggleCodeBlock(window.__cadflow_easymde); }",
            DocAction::Table => "if (window.__cadflow_easymde) { EasyMDE.drawTable(window.__cadflow_easymde); }",
            DocAction::Link => "if (window.__cadflow_easymde) { EasyMDE.drawLink(window.__cadflow_easymde); }",
            DocAction::Image => "if (window.__cadflow_easymde) { EasyMDE.drawImage(window.__cadflow_easymde); }",
        };
        let _ = js_sys::eval(js);
    }

    /// 切换预览状态（按下/抬起）
    pub fn toggle_preview(&mut self) {
        self.set_preview(!self.is_preview);
    }

    /// 设置预览状态并同步 EasyMDE 的视图模式
    pub fn set_preview(&mut self, preview: bool) {
        self.is_preview = preview;
        self.sub_tab = if preview { DocSubTab::Preview } else { DocSubTab::Edit };

        if let Some(window) = web_sys::window() {
            let is_prev_val = wasm_bindgen::JsValue::from_bool(preview);
            let _ = js_sys::Reflect::set(&window, &"__cadflow_doc_is_preview".into(), &is_prev_val);
        }

        let script = if preview {
            r#"
            if (window.__cadflow_easymde) {
                const mde = window.__cadflow_easymde;
                if (mde.isSideBySideActive()) { EasyMDE.toggleSideBySide(mde); }
                if (!mde.isPreviewActive()) { EasyMDE.togglePreview(mde); }
            }
            "#
        } else {
            r#"
            if (window.__cadflow_easymde) {
                const mde = window.__cadflow_easymde;
                if (mde.isPreviewActive()) { EasyMDE.togglePreview(mde); }
                if (mde.isSideBySideActive()) { EasyMDE.toggleSideBySide(mde); }
                mde.codemirror.refresh();
            }
            "#
        };
        let _ = js_sys::eval(script);
    }

    /// 保持向后兼容：切换子标签模式
    pub fn set_subtab(&mut self, target: DocSubTab) {
        self.set_preview(target == DocSubTab::Preview);
    }

    /// 清空 EasyMDE 文本内容
    pub fn clear_easymde_content(&mut self) {
        if let Some(file) = self.files.get_mut(self.active_index) {
            file.content.clear();
        }
        let _ = js_sys::eval(
            r#"
            if (window.__cadflow_easymde) {
                window.__cadflow_easymde.value("");
                if (window.__cadflow_easymde.isPreviewActive()) {
                    EasyMDE.togglePreview(window.__cadflow_easymde);
                    EasyMDE.togglePreview(window.__cadflow_easymde);
                }
            }
            "#,
        );
    }

    /// 更新 EasyMDE 中的内容 (安全地通过 window 传递，避免转义问题)
    pub fn set_easymde_content(&self, content: &str) {
        if let Some(window) = web_sys::window() {
            let js_val = wasm_bindgen::JsValue::from_str(content);
            let _ = js_sys::Reflect::set(&window, &"__cadflow_temp_doc_content".into(), &js_val);
            let is_prev_val = wasm_bindgen::JsValue::from_bool(self.is_preview);
            let _ = js_sys::Reflect::set(&window, &"__cadflow_doc_is_preview".into(), &is_prev_val);
            let script = r#"
                if (window.__cadflow_easymde) {
                    const mde = window.__cadflow_easymde;
                    mde.value(window.__cadflow_temp_doc_content || "");
                    delete window.__cadflow_temp_doc_content;
                    if (window.__cadflow_doc_is_preview) {
                        if (mde.isSideBySideActive()) { EasyMDE.toggleSideBySide(mde); }
                        if (mde.isPreviewActive()) {
                            EasyMDE.togglePreview(mde);
                            EasyMDE.togglePreview(mde);
                        } else {
                            EasyMDE.togglePreview(mde);
                        }
                    } else {
                        if (mde.isPreviewActive()) {
                            EasyMDE.togglePreview(mde);
                        }
                        if (mde.isSideBySideActive()) { EasyMDE.toggleSideBySide(mde); }
                        mde.codemirror.refresh();
                    }
                    mde.codemirror.clearHistory();
                }
            "#;
            let _ = js_sys::eval(script);
        }
    }

    /// 新建文档（分配“未命名”+数字）
    pub fn create_new_doc(&mut self) {
        // 1. 保存当前激活标签页的内容
        let current_text = self.get_content();
        if let Some(active_file) = self.files.get_mut(self.active_index) {
            active_file.content = current_text;
        }

        // 2. 分配名称：“未命名”+数字
        let mut num = self.next_file_number;
        while self.files.iter().any(|f| f.title == format!("未命名{}", num)) {
            num += 1;
        }
        let title = format!("未命名{}", num);
        self.next_file_number = num + 1;

        // 3. 追加新文档
        self.files.push(DocFile {
            title,
            content: String::new(),
        });

        // 4. 切换到新建文档，退出预览状态并清空编辑器
        self.active_index = self.files.len() - 1;
        self.editing_tab = None;
        if self.is_preview {
            self.set_preview(false);
        }
        self.set_easymde_content("");
    }

    /// 切换到指定索引的文件标签页
    pub fn switch_to_tab(&mut self, new_index: usize) {
        if new_index >= self.files.len() || new_index == self.active_index {
            return;
        }
        // 1. 保存当前激活标签页的内容
        let current_text = self.get_content();
        if let Some(active_file) = self.files.get_mut(self.active_index) {
            active_file.content = current_text;
        }

        // 2. 更新激活索引
        self.active_index = new_index;
        self.editing_tab = None;

        // 3. 载入目标文档内容至 EasyMDE
        let target_content = self.files[new_index].content.clone();
        self.set_easymde_content(&target_content);
    }

    /// 关闭指定索引的文件标签页
    pub fn close_tab(&mut self, index: usize) {
        if index >= self.files.len() {
            return;
        }

        let is_closing_active = index == self.active_index;
        self.files.remove(index);

        if self.files.is_empty() {
            self.active_index = 0;
            self.editing_tab = None;
            self.title.clear();
            self.content.clear();
            self.set_easymde_content("");
            return;
        }

        let new_active = if index < self.active_index {
            self.active_index.saturating_sub(1)
        } else if index == self.active_index {
            if index >= self.files.len() {
                self.files.len().saturating_sub(1)
            } else {
                self.active_index
            }
        } else {
            self.active_index
        };

        self.active_index = new_active.min(self.files.len() - 1);
        self.editing_tab = None;

        if is_closing_active {
            let target_content = self.files[self.active_index].content.clone();
            self.set_easymde_content(&target_content);
        }
    }

    /// 获取当前 EasyMDE 中的字符数量（用于底部状态栏展示）
    pub fn get_char_count(&self) -> usize {
        js_sys::eval("window.__cadflow_easymde ? Array.from(window.__cadflow_easymde.value()).length : 0")
            .ok()
            .and_then(|v| v.as_f64())
            .map(|n| n as usize)
            .unwrap_or_else(|| {
                self.files.get(self.active_index).map(|f| f.content.chars().count()).unwrap_or(0)
            })
    }

    /// 获取 EasyMDE 当前文本
    pub fn get_content(&self) -> String {
        js_sys::eval("window.__cadflow_easymde ? window.__cadflow_easymde.value() : ''")
            .ok()
            .and_then(|v| v.as_string())
            .unwrap_or_else(|| {
                self.files.get(self.active_index).map(|f| f.content.clone()).unwrap_or_default()
            })
    }

    /// 通过 web-sys 在 body 下挂载原生 <div> 浮层并初始化 EasyMDE
    pub fn ensure_overlay_mounted(&mut self) {
        if self.overlay_mounted {
            return;
        }
        let Some(window) = web_sys::window() else { return };
        let Some(document) = window.document() else { return };
        let Some(body) = document.body() else { return };

        if document.get_element_by_id("doc-easymde-overlay").is_none() {
            // 1. 创建原生 <div> 浮层容器
            let Ok(overlay_div) = document.create_element("div") else { return };
            let Ok(overlay_div) = overlay_div.dyn_into::<web_sys::HtmlElement>() else { return };
            overlay_div.set_id("doc-easymde-overlay");

            // 2. 创建供 EasyMDE 绑定的 <textarea>
            let Ok(textarea) = document.create_element("textarea") else { return };
            let Ok(textarea) = textarea.dyn_into::<web_sys::HtmlTextAreaElement>() else { return };
            textarea.set_id("doc-easymde-textarea");
            let initial_content = self.files.get(self.active_index).map(|f| f.content.as_str()).unwrap_or("");
            textarea.set_value(initial_content);

            if overlay_div.append_child(&textarea).is_err() {
                return;
            }
            if body.append_child(&overlay_div).is_err() {
                return;
            }

            // 3. 调用全局 EasyMDE 初始化实例（彻底关闭内部工具栏，使用顶部主工具栏，Req 4）
            let init_js = r#"
                (function() {
                    const el = document.getElementById('doc-easymde-textarea');
                    if (el && window.EasyMDE && !window.__cadflow_easymde) {
                        window.__cadflow_easymde = new EasyMDE({
                            element: el,
                            spellChecker: false,
                            status: false,
                            autoDownloadFontAwesome: false,
                            minHeight: "100%",
                            toolbar: false
                        });
                    }
                })();
            "#;
            let _ = js_sys::eval(init_js);

            if self.is_preview {
                let _ = js_sys::eval("if (window.__cadflow_easymde && !window.__cadflow_easymde.isPreviewActive()) { EasyMDE.togglePreview(window.__cadflow_easymde); }");
            }
        }

        self.overlay_mounted = true;
    }

    /// 通过 web-sys 实时同步原生 <div> 浮层的矩形区域位置、尺寸与显隐状态
    pub fn sync_overlay_rect(&mut self, rect: Option<Rect>) {
        self.ensure_overlay_mounted();

        let Some(window) = web_sys::window() else { return };
        let Some(document) = window.document() else { return };
        let Some(el) = document.get_element_by_id("doc-easymde-overlay") else { return };
        let Ok(html_el) = el.dyn_into::<web_sys::HtmlElement>() else { return };
        let style = html_el.style();

        match rect {
            Some(r) if r.width() > 1.0 && r.height() > 1.0 => {
                let _ = style.set_property("display", "block");
                let _ = style.set_property("left", &format!("{:.2}px", r.min.x));
                let _ = style.set_property("top", &format!("{:.2}px", r.min.y));
                let _ = style.set_property("width", &format!("{:.2}px", r.width()));
                let _ = style.set_property("height", &format!("{:.2}px", r.height()));

                // 动画滑动或尺寸调整时刷新 CodeMirror 布局
                let _ = js_sys::eval(
                    "if (window.__cadflow_easymde) { window.__cadflow_easymde.codemirror.refresh(); }",
                );
            }
            _ => {
                let _ = style.set_property("display", "none");
            }
        }
    }

    /// 渲染文档侧边栏面板界面
    pub fn ui(&mut self, ui: &mut Ui) {
        let t = cadcraft_ui_egui::theme::Tokens::get();
        let p = ui.painter().clone();

        // 同步 title 和 sub_tab
        if let Some(f) = self.files.get(self.active_index) {
            self.title = f.title.clone();
        }
        self.sub_tab = if self.is_preview { DocSubTab::Preview } else { DocSubTab::Edit };

        // 消除标签行与内容行之间的默认垂直间距，确保主标签与次标签无缝贴合
        ui.spacing_mut().item_spacing = vec2(0.0, 0.0);

        // --------------------------------------------------------------------
        // 1. 第一行：“文档”主标签栏 (高度 28px，严格参考 CADCraft 工具集主标签样式)
        // --------------------------------------------------------------------
        let main_h = 28.0;
        let full_w = ui.available_width();
        let (main_bar_rect, _) = ui.allocate_exact_size(vec2(full_w, main_h), Sense::hover());
        p.rect_filled(main_bar_rect, 0.0, t.chrome);

        // “文档”主标题标签 (固定宽 76px，背景为 tab_active，文字为 text)
        let doc_title_w = 76.0;
        let title_rect = Rect::from_min_size(main_bar_rect.min, vec2(doc_title_w, main_h));
        p.rect_filled(title_rect, 0.0, t.tab_active);
        p.text(
            title_rect.center(),
            egui::Align2::CENTER_CENTER,
            "文档",
            egui::FontId::proportional(13.5),
            t.text,
        );

        // --------------------------------------------------------------------
        // 2. 第二行：文件标签栏 (高度 26px，无缝紧贴主标签，支持文件标签切换、重命名及新建)
        // --------------------------------------------------------------------
        let sub_h = 26.0;
        let (sub_bar_rect, _) = ui.allocate_exact_size(vec2(full_w, sub_h), Sense::hover());
        p.rect_filled(sub_bar_rect, 0.0, t.chrome_dark);
        p.hline(sub_bar_rect.x_range(), sub_bar_rect.bottom(), Stroke::new(1.0, t.border));

        let mut switch_to = None;
        let mut tab_to_close = None;
        let mut start_editing = None;
        let mut create_new = false;
        let mut finish_editing = false;

        ui.scope_builder(
            egui::UiBuilder::new().max_rect(sub_bar_rect),
            |tab_ui| {
                tab_ui.spacing_mut().item_spacing = vec2(0.0, 0.0);
                egui::ScrollArea::horizontal()
                    .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
                    .show(tab_ui, |tab_ui| {
                        tab_ui.horizontal(|tab_ui| {
                            tab_ui.spacing_mut().item_spacing = vec2(0.0, 0.0);
                            let font_id = egui::FontId::proportional(13.5);
                            let closable = !self.files.is_empty();

                            for (i, file) in self.files.iter_mut().enumerate() {
                                let active = self.active_index == i;
                                let is_editing = self.editing_tab == Some(i);

                                // 测量文字宽度
                                let text_w = tab_ui.painter().layout_no_wrap(file.title.clone(), font_id.clone(), t.text).size().x;
                                let close_reserve = if closable { 16.0 } else { 0.0 };
                                let tab_w = (text_w + 18.0 + close_reserve).clamp(64.0, 160.0);

                                if is_editing {
                                    let (tab_rect, _) = tab_ui.allocate_exact_size(vec2(tab_w, sub_h), Sense::hover());
                                    p.rect_filled(tab_rect, 0.0, t.tab_active);
                                    p.hline(tab_rect.x_range(), tab_rect.top(), Stroke::new(1.5, t.accent));
                                    p.vline(tab_rect.right(), tab_rect.y_range(), Stroke::new(1.0, t.border));

                                    tab_ui.scope_builder(
                                        egui::UiBuilder::new().max_rect(tab_rect.shrink2(vec2(4.0, 2.0))),
                                        |edit_ui| {
                                            let edit_resp = edit_ui.add(
                                                egui::TextEdit::singleline(&mut file.title)
                                                    .font(font_id.clone())
                                                    .desired_width(tab_w - 8.0),
                                            );
                                            if self.just_started_editing {
                                                edit_resp.request_focus();
                                                self.just_started_editing = false;
                                            }
                                            if edit_resp.lost_focus() || edit_ui.input(|inp| inp.key_pressed(egui::Key::Enter) || inp.key_pressed(egui::Key::Escape)) {
                                                finish_editing = true;
                                            }
                                        },
                                    );
                                } else {
                                    let (tab_rect, tab_resp) = tab_ui.allocate_exact_size(vec2(tab_w, sub_h), Sense::click());
                                    let hovered = tab_resp.hovered();

                                    let bg_col = if active {
                                        t.tab_active
                                    } else if hovered {
                                        t.chrome
                                    } else {
                                        t.chrome_dark
                                    };
                                    p.rect_filled(tab_rect, 0.0, bg_col);
                                    if active {
                                        p.hline(tab_rect.x_range(), tab_rect.top(), Stroke::new(1.5, t.accent));
                                    }
                                    p.vline(tab_rect.right(), tab_rect.y_range(), Stroke::new(1.0, t.border));

                                    // 关闭按钮 (多于一个标签时显示)
                                    let mut close_clicked = false;
                                    if closable {
                                        let close_rect = Rect::from_center_size(
                                            pos2(tab_rect.right() - 10.0, tab_rect.center().y),
                                            vec2(14.0, 14.0),
                                        );
                                        let close_resp = tab_ui.interact(close_rect, tab_ui.id().with(("tab_close", i)), Sense::click());
                                        if close_resp.hovered() {
                                            p.rect_filled(close_rect, 2.0, t.control_hover);
                                        }
                                        if active || hovered || close_resp.hovered() {
                                            let arm = 3.5;
                                            let c = close_rect.center();
                                            let x_col = if close_resp.hovered() { t.text } else { t.text_dim };
                                            p.line_segment([pos2(c.x - arm, c.y - arm), pos2(c.x + arm, c.y + arm)], Stroke::new(1.2, x_col));
                                            p.line_segment([pos2(c.x - arm, c.y + arm), pos2(c.x + arm, c.y - arm)], Stroke::new(1.2, x_col));
                                        }
                                        if close_resp.clicked() {
                                            close_clicked = true;
                                        }
                                    }

                                    // 标签文本 (超长省略)
                                    let text_color = if active { t.text } else { t.text_dim };
                                    let avail_text_w = (tab_w - 12.0 - close_reserve).max(0.0);
                                    let (galley, cut) = elided(&p, &file.title, font_id.clone(), text_color, avail_text_w);
                                    let text_pos = pos2(tab_rect.left() + 6.0, tab_rect.center().y - galley.size().y / 2.0);
                                    p.galley(text_pos, galley, text_color);

                                    let tip = if cut {
                                        format!("{}\n(点击修改名称)", file.title)
                                    } else if active {
                                        "点击修改名称".to_string()
                                    } else {
                                        "点击切换标签页".to_string()
                                    };
                                    let tab_resp = tab_resp.on_hover_text(tip);

                                    if close_clicked {
                                        tab_to_close = Some(i);
                                    } else if tab_resp.clicked() {
                                        if active {
                                            start_editing = Some(i);
                                        } else {
                                            switch_to = Some(i);
                                        }
                                    } else if tab_resp.double_clicked() {
                                        if !active {
                                            switch_to = Some(i);
                                        }
                                        start_editing = Some(i);
                                    }
                                }
                            }

                            // '+' 新建文档按钮
                            let (plus_rect, plus_resp) = tab_ui.allocate_exact_size(vec2(26.0, sub_h), Sense::click());
                            let plus_hovered = plus_resp.hovered();
                            if plus_hovered {
                                p.rect_filled(plus_rect, 0.0, t.chrome);
                            }
                            let pc = plus_rect.center();
                            let arm = 4.5;
                            let plus_col = if plus_hovered { t.text } else { t.text_dim };
                            p.line_segment([pos2(pc.x - arm, pc.y), pos2(pc.x + arm, pc.y)], Stroke::new(1.4, plus_col));
                            p.line_segment([pos2(pc.x, pc.y - arm), pos2(pc.x, pc.y + arm)], Stroke::new(1.4, plus_col));
                            p.vline(plus_rect.right(), plus_rect.y_range(), Stroke::new(1.0, t.border));

                            if plus_resp.on_hover_text("新建文档").clicked() {
                                create_new = true;
                            }
                        });
                    });
            },
        );

        if finish_editing {
            if let Some(i) = self.editing_tab {
                if let Some(file) = self.files.get_mut(i) {
                    let trimmed = file.title.trim().to_string();
                    file.title = if trimmed.is_empty() {
                        if i == 0 { "未命名".to_string() } else { format!("未命名{}", i) }
                    } else {
                        trimmed
                    };
                }
            }
            self.editing_tab = None;
        }

        if let Some(i) = tab_to_close {
            self.close_tab(i);
        } else if let Some(i) = switch_to {
            self.switch_to_tab(i);
            if let Some(edit_i) = start_editing {
                if edit_i == i {
                    self.editing_tab = Some(i);
                    self.just_started_editing = true;
                }
            }
        } else if let Some(i) = start_editing {
            self.editing_tab = Some(i);
            self.just_started_editing = true;
        }

        if create_new {
            self.create_new_doc();
        }

        // --------------------------------------------------------------------
        // 3. 底部状态与快捷操作栏（“预览”切换按钮 + 实时字符数统计）
        // --------------------------------------------------------------------
        let footer_h = 28.0;
        let footer_rect = Rect::from_min_size(
            pos2(main_bar_rect.left(), ui.max_rect().bottom() - footer_h),
            vec2(full_w, footer_h),
        );
        p.rect_filled(footer_rect, 0.0, t.chrome);

        ui.scope_builder(
            egui::UiBuilder::new().max_rect(footer_rect.shrink2(vec2(10.0, 4.0))),
            |ui| {
                ui.horizontal_centered(|ui| {
                    let preview_text = egui::RichText::new("预览")
                        .size(11.5)
                        .color(if self.is_preview { t.text } else { t.text_dim });
                    let preview_btn = egui::Button::new(preview_text)
                        .selected(self.is_preview)
                        .fill(if self.is_preview { t.tab_active } else { t.chrome_dark })
                        .stroke(if self.is_preview { Stroke::new(1.2, t.accent) } else { Stroke::new(1.0, t.border) })
                        .corner_radius(CornerRadius::same(3));

                    let preview_resp = ui.add_enabled(!self.files.is_empty(), preview_btn);
                    let tooltip = if self.files.is_empty() {
                        "暂无文档"
                    } else if self.is_preview {
                        "退出预览（当前为按下状态）"
                    } else {
                        "预览文档（当前为抬起状态）"
                    };

                    if preview_resp.on_hover_text(tooltip).clicked() {
                        self.toggle_preview();
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            egui::RichText::new(format!("{} 字符", self.get_char_count()))
                                .size(11.0)
                                .color(Color32::GRAY),
                        );
                    });
                });
            },
        );

        // --------------------------------------------------------------------
        // 4. 文档中央可编辑矩形区域：在 egui 画布中仅占位，上方挂载原生 EasyMDE 浮层
        // --------------------------------------------------------------------
        let content_rect = Rect::from_min_max(
            pos2(sub_bar_rect.left(), sub_bar_rect.bottom()),
            pos2(ui.max_rect().right(), footer_rect.top()),
        );
        ui.allocate_rect(content_rect, Sense::hover());

        if self.files.is_empty() {
            self.sync_overlay_rect(None);
            p.rect_filled(content_rect, 0.0, t.panel);
            p.text(
                content_rect.center(),
                egui::Align2::CENTER_CENTER,
                "暂无文档，点击上方 '+' 新建文档",
                egui::FontId::proportional(13.0),
                t.text_dim,
            );
        } else {
            self.sync_overlay_rect(Some(content_rect));
        }
    }
}

/// 单行文本根据宽度省略截断函数
fn elided(p: &egui::Painter, text: &str, font: egui::FontId, color: egui::Color32, max_width: f32) -> (Arc<egui::Galley>, bool) {
    let mut job = egui::text::LayoutJob::simple_singleline(text.to_string(), font, color);
    job.wrap = egui::text::TextWrapping {
        max_width: max_width.max(0.0),
        max_rows: 1,
        break_anywhere: true,
        overflow_character: Some('…'),
    };
    let galley = p.layout_job(job);
    let cut = galley.elided;
    (galley, cut)
}

