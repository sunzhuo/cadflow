//! Markdown document panel and editor for CADFlow.
//!
//! Refactored to mount native HTML <div> overlay powered by EasyMDE:
//! - Top header: "文档" title and [编辑] [预览] sub-tabs styled matching Tool Sets
//! - Content area: mounts native HTML <div> overlay with EasyMDE for native Chinese IME, typography & performance
//! - Bottom footer: "清空内容" quick action button and live character counter
//! - Sub-pixel synchronized sliding animation: overlay rectangle coordinates sync every frame with the panel

use egui::{pos2, vec2, Color32, Rect, Sense, Ui};
use serde::{Deserialize, Serialize};
use wasm_bindgen::JsCast;

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
> 2. 点击上方子标签可切换【编辑】与【预览】；
> 3. 顶部主工具栏提供 Markdown 快捷操作工具组；
> 4. 点击【文档】按钮可平滑展开与折叠文档侧边栏。
"#;

pub struct DocState {
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
            title: "设计说明.md".to_string(),
            content: DEFAULT_DOC_TEMPLATE.to_string(),
            sub_tab: DocSubTab::Edit,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            overlay_mounted: false,
        }
    }

    /// 执行来自顶部工具栏的 Markdown 编辑操作 (通过 EasyMDE 实例 API 实现)
    pub fn apply_action(&mut self, action: DocAction) {
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

    /// 切换子标签模式并同步 EasyMDE 的视图 (编辑 / 预览)
    pub fn set_subtab(&mut self, target: DocSubTab) {
        self.sub_tab = target;
        let script = match target {
            DocSubTab::Edit => {
                r#"
                if (window.__cadflow_easymde) {
                    const mde = window.__cadflow_easymde;
                    if (mde.isPreviewActive()) { EasyMDE.togglePreview(mde); }
                    if (mde.isSideBySideActive()) { EasyMDE.toggleSideBySide(mde); }
                    mde.codemirror.refresh();
                }
                "#
            }
            DocSubTab::Preview => {
                r#"
                if (window.__cadflow_easymde) {
                    const mde = window.__cadflow_easymde;
                    if (mde.isSideBySideActive()) { EasyMDE.toggleSideBySide(mde); }
                    if (!mde.isPreviewActive()) { EasyMDE.togglePreview(mde); }
                }
                "#
            }
        };
        let _ = js_sys::eval(script);
    }

    /// 清空 EasyMDE 文本内容
    pub fn clear_easymde_content(&self) {
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

    /// 获取当前 EasyMDE 中的字符数量（用于底部状态栏展示）
    pub fn get_char_count(&self) -> usize {
        js_sys::eval("window.__cadflow_easymde ? Array.from(window.__cadflow_easymde.value()).length : 0")
            .ok()
            .and_then(|v| v.as_f64())
            .map(|n| n as usize)
            .unwrap_or(0)
    }

    /// 获取 EasyMDE 当前文本
    pub fn get_content(&self) -> String {
        js_sys::eval("window.__cadflow_easymde ? window.__cadflow_easymde.value() : ''")
            .ok()
            .and_then(|v| v.as_string())
            .unwrap_or_else(|| self.content.clone())
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
            textarea.set_value(DEFAULT_DOC_TEMPLATE);

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

    /// 渲染文档侧边栏面板界面 (Req 2, Req 3)
    pub fn ui(&mut self, ui: &mut Ui) {
        let t = cadcraft_ui_egui::theme::Tokens::get();
        let p = ui.painter().clone();

        // 消除标签行与内容行之间的默认垂直间距，确保主标签与次标签无缝贴合 (Req 2)
        ui.spacing_mut().item_spacing = vec2(0.0, 0.0);

        // --------------------------------------------------------------------
        // 1. 第一行：“文档”主标签栏 (高度 28px，严格参考 CADCraft 工具集主标签样式，Req 7, 9)
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
            egui::FontId::proportional(13.0),
            t.text,
        );

        // --------------------------------------------------------------------
        // 2. 第二行：“编辑”与“预览”次标签栏 (高度 26px，无缝紧贴主标签，0 空隙，Req 2, Req 3)
        // --------------------------------------------------------------------
        let sub_h = 26.0;
        let (sub_bar_rect, _) = ui.allocate_exact_size(vec2(full_w, sub_h), Sense::hover());
        p.rect_filled(sub_bar_rect, 0.0, t.chrome);

        let subtabs = [
            (DocSubTab::Edit, "编辑"),
            (DocSubTab::Preview, "预览"),
        ];
        let sub_w = 64.0;
        let mut sub_x = sub_bar_rect.left();
        let mut clicked_sub = None;

        for (variant, label) in subtabs {
            let tr = Rect::from_min_size(pos2(sub_x, sub_bar_rect.top() + 1.0), vec2(sub_w, sub_h - 2.0));
            let active = self.sub_tab == variant;
            let resp = ui.interact(tr, ui.id().with(("doc_subtab", label)), Sense::click());

            p.rect_filled(tr, 0.0, if active { t.tab_active } else { t.chrome });
            p.text(
                tr.center(),
                egui::Align2::CENTER_CENTER,
                label,
                egui::FontId::proportional(12.5),
                if active { t.text } else { t.text_dim },
            );

            if resp.clicked() {
                clicked_sub = Some(variant);
            }
            sub_x += sub_w + 4.0;
        }

        if let Some(st) = clicked_sub {
            self.set_subtab(st);
        }

        // --------------------------------------------------------------------
        // 3. 底部状态与快捷操作栏（清空内容 + 实时字符数统计）
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
                    if ui.small_button("清空内容").clicked() {
                        self.clear_easymde_content();
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

        self.sync_overlay_rect(Some(content_rect));
    }
}
