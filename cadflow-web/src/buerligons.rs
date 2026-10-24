//! Buerligons UI specifications for CADFlow (egui):
//! - Top toolbar height: 48px, tool button: 32x32px, corner radius: 7px, spacing: 2px
//! - Preserves CADCraft's native theme tokens (Tokens::get()), toolbar colors, and icons
//! - Leftmost collapsed multi-level dropdown menu with 18x18 hamburger icon and MENU label,
//!   containing all CADCraft menu trees, sub-items, and shortcuts dynamically
//! - Vertical tool dock: 34px width (28x28px button, 3px padding, 2px spacing)
//! - Sidebar: default width 288px, resizable range 264..=560px

use cadcraft_ui_egui::CadApp;
use cadcraft_ui_egui::icons::{self, Icon};
use cadcraft_ui_egui::menus::{self, Entry};
use cadcraft_ui_egui::theme::Tokens;
use egui::{pos2, vec2, Color32, CornerRadius, Margin, Rect, Sense, Stroke, Ui};

// ----------------------------------------------------------------------------
// Buerligons 核心尺寸参数
// ----------------------------------------------------------------------------

/// 顶部工具栏高度：固定为 48.0 (--rcad-bar-h: 48px)
pub const BAR_HEIGHT: f32 = 48.0;
/// 内部工具按钮尺寸：32.0 x 32.0 (--rcad-tool: 32px)
pub const TOOL_SIZE: f32 = 32.0;
/// 按钮圆角：7 (Buerligons 规范: 7px)
pub const TOOL_CORNER_RADIUS: u8 = 7;
/// 按钮间距：2.0
pub const TOOL_SPACING: f32 = 2.0;

/// 竖向绘图工具栏 (Dock) 宽度：34.0 (28x28 按钮 + 3px 内边距)
pub const DOCK_WIDTH: f32 = 34.0;
pub const DOCK_TOOL_SIZE: f32 = 28.0;
pub const DOCK_PADDING: f32 = 3.0;

/// 左侧属性/特征面板 (Sidebar) 默认宽度：288.0，范围 264.0..=560.0
pub const SIDEBAR_DEFAULT_WIDTH: f32 = 288.0;
pub const SIDEBAR_MIN_WIDTH: f32 = 264.0;
pub const SIDEBAR_MAX_WIDTH: f32 = 560.0;

/// 主工具栏图标大小：20.0 x 20.0 (线宽 1.5)
pub const MAIN_ICON_SIZE: f32 = 20.0;
/// 撤销/重做/菜单图标大小：18.0 x 18.0 (线宽 1.61)
pub const SUB_ICON_SIZE: f32 = 18.0;

// ----------------------------------------------------------------------------
// CADCraft 工具栏所有图标与命令组定义（对齐 CADCraft 原生规范）
// ----------------------------------------------------------------------------

pub const TOOLBAR_GROUPS: &[&[(Icon, &str, &str, Option<&str>)]] = &[
    // Group 1: 文件操作
    &[
        (Icon::New, "new", "New Drawing", Some("Cmd+N")),
        (Icon::Open, "open", "Open", Some("Cmd+O")),
        (Icon::Save, "qsave", "Save", Some("Cmd+S")),
        (Icon::SaveAs, "saveas", "Save As", None),
    ],
    // Group 2: 撤销 / 重做
    &[
        (Icon::Undo, "u", "Undo", Some("Cmd+Z")),
        (Icon::Redo, "redo", "Redo", Some("Cmd+Shift+Z")),
    ],
    // Group 3: 打印与发布
    &[
        (Icon::Plot, "plot", "Print", None),
        (Icon::Publish, "publish", "Batch Publish", None),
        (Icon::PageSetup, "pagesetup", "Page Setup Manager", None),
        (Icon::Preview, "preview", "Plot Preview", None),
    ],
    // Group 4: 导入导出与共享
    &[
        (Icon::Import, "import", "Import", None),
        (Icon::Export, "export", "Export", None),
        (Icon::Attach, "attach", "Attach", None),
        (Icon::Share, "share", "Share", None),
    ],
    // Group 5: 视图导航与平移
    &[
        (Icon::ZoomWindow, "zoom.window", "Zoom Window", None),
        (Icon::Pan, "pan", "Pan", None),
        (Icon::Orbit, "3dorbit", "Orbit", None),
    ],
    // Group 6: 视图与面板管理
    &[
        (Icon::ZoomExtents, "zoom.extents", "Zoom Extents", None),
        (Icon::Properties, "ui.toggle.palettes", "Properties", None),
        (Icon::Layers, "ui.dialog.layers", "Layer Properties Manager", None),
        (Icon::Blocks, "ui.dialog.blocks", "Blocks", None),
    ],
    // Group 7: 测量与查询
    &[
        (Icon::Measure, "dist", "Measure Distance", None),
        (Icon::List, "list", "List", None),
        (Icon::Area, "area", "Area", None),
    ],
    // Group 8: 帮助
    &[
        (Icon::Help, "ui.dialog.about", "Help", None),
    ],
];

/// 快捷键展示格式化：在 Windows/Linux 上显示 Ctrl，在 macOS 上显示 ⌘
pub fn format_shortcut(s: &str, mac: bool) -> String {
    if !mac {
        return s.replace("Cmd+", "Ctrl+");
    }
    let shift = if s.contains("Shift+") { "⇧" } else { "" };
    let cmd = if s.contains("Cmd+") { "⌘" } else { "" };
    format!("{shift}{cmd}{}", s.replace("Cmd+", "").replace("Shift+", ""))
}

/// 兼容方法
pub trait UiMenuExt {
    fn close_menu(&mut self);
}

impl UiMenuExt for Ui {
    fn close_menu(&mut self) {
        self.close();
    }
}

// ----------------------------------------------------------------------------
// 动态多级菜单渲染（遍历 CADCraft 原生菜单树）
// ----------------------------------------------------------------------------

fn render_menu_entry(ui: &mut Ui, e: &Entry, clicked: &mut Option<String>) {
    match e {
        Entry::Item {
            label,
            id,
            shortcut,
            enabled,
        } => {
            let mut b = egui::Button::new(cadcraft_ui_egui::i18n::t(label));
            if let Some(s) = shortcut {
                b = b.shortcut_text(format_shortcut(s, ui.ctx().os().is_mac()));
            }
            if ui.add_enabled(*enabled, b).clicked() {
                *clicked = Some(id.clone());
                ui.close();
            }
        }
        Entry::Sub { label, children } => {
            ui.menu_button(cadcraft_ui_egui::i18n::menu(label), |ui| {
                ui.set_min_width(148.0);
                for c in children {
                    render_menu_entry(ui, c, clicked);
                }
            });
        }
    }
}

/// 绘制 Buerligons 风格的折叠菜单按钮（18x18 汉堡图标 + MENU 字样 + 包含 CADCraft 全部菜单项与子项的下拉菜单）
pub fn draw_collapsed_menu(ui: &mut Ui, app: &mut CadApp) {
    let t = Tokens::get();

    // 1. 局部覆盖 egui 菜单样式，对齐 Buerligons 的 .ant-dropdown-menu 规范，同时保留 CADCraft 原生配色
    let mut style = (**ui.style()).clone();
    style.spacing.menu_margin = Margin::same(4);            // 菜单外框内边距 4px
    style.spacing.menu_width = 148.0;                       // 菜单最小宽度 148px
    style.spacing.interact_size = vec2(64.0, 28.0);         // 菜单项高度 28px
    style.spacing.button_padding = vec2(9.0, 5.0);          // 菜单项内边距 5px 9px
    style.spacing.item_spacing = vec2(4.0, 2.0);

    // 使用 CADCraft 原生主题颜色
    style.visuals.window_fill = t.chrome;
    style.visuals.window_corner_radius = CornerRadius::same(10); // 菜单圆角 10px
    style.visuals.menu_corner_radius = CornerRadius::same(10);
    style.visuals.window_stroke = Stroke::new(1.0, t.border);

    style.visuals.widgets.inactive.weak_bg_fill = Color32::TRANSPARENT;
    style.visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, t.text);
    style.visuals.widgets.hovered.weak_bg_fill = t.control_hover.gamma_multiply(0.6);
    style.visuals.widgets.hovered.bg_stroke = Stroke::NONE;
    style.visuals.widgets.hovered.corner_radius = CornerRadius::same(6); // 菜单项圆角 6px
    style.visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, t.text);
    style.visuals.widgets.active.weak_bg_fill = t.control;
    style.visuals.widgets.open.weak_bg_fill = t.control;
    ui.set_style(style);

    // 2. 设置顶层 MENU 按钮尺寸为 32px 高度与 7px 圆角，为左侧 18x18 图标预留空间
    ui.spacing_mut().interact_size = vec2(72.0, TOOL_SIZE);
    ui.spacing_mut().button_padding = vec2(10.0, 6.0);

    let mut clicked = None;
    let tree = menus::tree(app);

    let menu_resp = ui.menu_button("      MENU", |ui| {
        ui.set_min_width(148.0);
        for (name, entries) in &tree {
            ui.menu_button(cadcraft_ui_egui::i18n::menu(name), |ui| {
                ui.set_min_width(148.0);
                for e in entries {
                    render_menu_entry(ui, e, &mut clicked);
                }
            });
        }
    });

    // 3. 在 MENU 按钮左侧居中绘制 CADCraft 原生 18x18 汉堡图标 (Icon::Menu)
    let btn_rect = menu_resp.response.rect;
    let icon_rect = Rect::from_min_size(
        pos2(btn_rect.left() + 7.0, btn_rect.center().y - 9.0),
        vec2(SUB_ICON_SIZE, SUB_ICON_SIZE),
    );
    icons::paint(ui.painter(), icon_rect, Icon::Menu, false);

    // 执行菜单点击的命令
    if let Some(id) = clicked {
        menus::activate(app, &id);
    }
}

// ----------------------------------------------------------------------------
// 工具栏按钮与分隔符绘制
// ----------------------------------------------------------------------------

/// 通用工具栏按钮：尺寸 32x32，圆角 7px，使用 CADCraft 原生配色
pub fn draw_tool_icon_button(
    ui: &mut Ui,
    btn_size: f32,
    icon_size: f32,
    active: bool,
    tooltip: &str,
    draw_icon: impl FnOnce(&egui::Painter, Rect),
) -> egui::Response {
    let t = Tokens::get();
    let (rect, resp) = ui.allocate_exact_size(vec2(btn_size, btn_size), Sense::click());
    let resp = resp.on_hover_text(tooltip);

    if ui.is_rect_visible(rect) {
        // 背景颜色与 7px 圆角
        let bg = if active || resp.is_pointer_button_down_on() {
            t.control
        } else if resp.hovered() {
            t.control_hover.gamma_multiply(0.6)
        } else {
            Color32::TRANSPARENT
        };

        if bg != Color32::TRANSPARENT {
            ui.painter().rect_filled(rect, CornerRadius::same(TOOL_CORNER_RADIUS), bg);
        }

        // 居中计算图标绘制区域
        let icon_rect = Rect::from_center_size(rect.center(), vec2(icon_size, icon_size));
        draw_icon(ui.painter(), icon_rect);

        // 激活态底部指示条
        if active {
            let indicator = Rect::from_min_max(
                pos2(rect.left() + 6.0, rect.bottom() - 4.0),
                pos2(rect.right() - 6.0, rect.bottom() - 2.0),
            );
            ui.painter().rect_filled(indicator, CornerRadius::same(1), t.accent);
        }
    }
    resp
}

/// 工具栏竖向分割线：宽 1px，高 18px，左右对称边距
pub fn draw_toolbar_sep(ui: &mut Ui) {
    let t = Tokens::get();
    let (rect, _) = ui.allocate_exact_size(vec2(11.0, TOOL_SIZE), Sense::hover());
    let x = rect.center().x;
    let yc = rect.center().y;
    ui.painter().line_segment(
        [pos2(x, yc - 9.0), pos2(x, yc + 9.0)],
        Stroke::new(1.0, t.border),
    );
}

// ----------------------------------------------------------------------------
// 完整顶部工具栏装配
// ----------------------------------------------------------------------------

/// 绘制 Buerligons 规范的完整顶部工具栏：
/// - 高度 48px，保留 CADCraft 原生工具栏背景与控件颜色
/// - 最左侧折叠式 MENU 按钮（展开 CADCraft 全部菜单与子项）
/// - 包含 CADCraft 工具栏全部图标（New, Open, Save, Undo, Redo, Plot, Import, Export, Zoom, Layers, Measure, Help 等）
pub fn draw_top_toolbar(ui: &mut Ui, app: &mut CadApp) {
    let t = Tokens::get();
    let mac = ui.ctx().os().is_mac();

    egui::Panel::top("buerligons_top_toolbar")
        .exact_size(BAR_HEIGHT)
        .frame(
            egui::Frame::NONE
                .fill(t.chrome)
                .stroke(Stroke::new(1.0, t.border)),
        )
        .show(ui, |ui| {
            let r = ui.max_rect();
            let mut clicked = None;

            ui.scope_builder(
                egui::UiBuilder::new().max_rect(r.shrink2(vec2(8.0, 8.0))),
                |ui| {
                    egui::ScrollArea::horizontal()
                        .id_salt("cadcraft_toolbar_scroll")
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            ui.horizontal_centered(|ui| {
                                ui.spacing_mut().item_spacing.x = TOOL_SPACING;

                                // 1. [最左侧] 折叠式多级子菜单 (18x18 图标 + MENU 字样 + CADCraft 全部菜单树)
                                draw_collapsed_menu(ui, app);

                                // 2. 依次渲染 CADCraft 工具栏全部图标组
                                for group in TOOLBAR_GROUPS {
                                    // 组间分割线
                                    draw_toolbar_sep(ui);

                                    for (icon, cmd, label, shortcut) in group.iter() {
                                        let is_undo_redo = matches!(*icon, Icon::Undo | Icon::Redo);
                                        let icon_sz = if is_undo_redo { SUB_ICON_SIZE } else { MAIN_ICON_SIZE };

                                        let tip = match shortcut {
                                            Some(sc) => format!(
                                                "{} ({})",
                                                cadcraft_ui_egui::i18n::t(label),
                                                format_shortcut(sc, mac)
                                            ),
                                            None => cadcraft_ui_egui::i18n::t(label).to_string(),
                                        };

                                        let resp = draw_tool_icon_button(
                                            ui,
                                            TOOL_SIZE,
                                            icon_sz,
                                            false,
                                            &tip,
                                            |p, rect| {
                                                icons::paint(p, rect, *icon, false);
                                            },
                                        );
                                        icons::describe_control(ui, &resp, label, None, false);

                                        if resp.clicked() {
                                            clicked = Some(*cmd);
                                        }
                                    }
                                }
                            });
                        });
                },
            );

            // 处理工具栏图标点击
            if let Some(cmd) = clicked {
                app.start(cmd);
            }
        });
}

// ----------------------------------------------------------------------------
// 常用 Panel 辅助方法 (可直接用于外层 App 布局)
// ----------------------------------------------------------------------------

pub struct TopBottomPanel;
impl TopBottomPanel {
    pub fn top(id: &'static str) -> egui::Panel {
        egui::Panel::top(id)
    }
    pub fn bottom(id: &'static str) -> egui::Panel {
        egui::Panel::bottom(id)
    }
}

pub struct SidePanel;
impl SidePanel {
    pub fn left(id: &'static str) -> egui::Panel {
        egui::Panel::left(id)
    }
    pub fn right(id: &'static str) -> egui::Panel {
        egui::Panel::right(id)
    }
}

/// 顶部工具栏 Panel (固定高度 48px)
pub fn top_toolbar_panel(id: &'static str) -> egui::Panel {
    let t = Tokens::get();
    egui::Panel::top(id)
        .exact_size(BAR_HEIGHT)
        .frame(
            egui::Frame::NONE
                .fill(t.chrome)
                .inner_margin(Margin::symmetric(8, 8))
                .stroke(Stroke::new(1.0, t.border)),
        )
}

/// 竖向绘图工具栏 (Dock) Panel (固定宽度 34px)
pub fn side_dock(id: &'static str) -> egui::Panel {
    let t = Tokens::get();
    egui::Panel::left(id)
        .exact_size(DOCK_WIDTH)
        .resizable(false)
        .frame(
            egui::Frame::NONE
                .fill(t.chrome)
                .inner_margin(Margin::same(DOCK_PADDING as i8))
                .stroke(Stroke::new(1.0, t.border)),
        )
}

/// 左侧属性/特征面板 (Sidebar) Panel (默认宽度 288px，范围 264..=560px)
pub fn sidebar(id: &'static str) -> egui::Panel {
    egui::Panel::left(id)
        .default_size(SIDEBAR_DEFAULT_WIDTH)
        .min_size(SIDEBAR_MIN_WIDTH)
        .max_size(SIDEBAR_MAX_WIDTH)
}
