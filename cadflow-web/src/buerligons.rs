//! Buerligons UI specifications for CADFlow:
//! - Top toolbar height: 48px, tool button: 32x32px, corner radius: 7px, spacing: 2px
//! - "CADFlow" menu button (18x18 hamburger icon + CADFlow text + full CADCraft menu tree translated to Chinese)
//! - "CAD侧边栏" toggle button with minimum spacing from Redo (separated by standard TOOL_SPACING)
//! - Sidebar boundary extends continuously across the toolbar in both open and closed states
//! - Zoom tools: Zoom Extents is placed BEFORE Zoom Window, in the same group: [ZoomExtents, ZoomWindow, Pan]
//! - "文档侧边栏" toggle button with sliding animation
//! - StackEdit-styled markdown edit buttons group to the right of document button (without Undo/Redo)
//! - Help dialog with title "CADFlow" and tabs "About" & "Command"

use cadcraft_ui_egui::CadApp;
use cadcraft_ui_egui::icons::{self, Icon};
use cadcraft_ui_egui::menus::{self, Entry};
use cadcraft_ui_egui::theme::Tokens;
use egui::{pos2, vec2, Color32, CornerRadius, Margin, Rect, RichText, Sense, Stroke, Ui};

// ----------------------------------------------------------------------------
// 核心尺寸参数
// ----------------------------------------------------------------------------

pub const BAR_HEIGHT: f32 = 48.0;
pub const TOOL_SIZE: f32 = 32.0;
pub const TOOL_CORNER_RADIUS: u8 = 7;
pub const TOOL_SPACING: f32 = 2.0;

// 侧边栏按钮宽度与最小宽度限制 (保证收起与滑动时与恢复按钮间距不小于 TOOL_SPACING)
pub const SIDEBAR_DEFAULT_WIDTH: f32 = 288.0;
pub const SIDEBAR_MIN_WIDTH: f32 = 226.0;
pub const SIDEBAR_MAX_WIDTH: f32 = 560.0;

// 文档侧边栏宽度限制 (确保容纳全部工具栏编辑按钮，Req 10)
pub const DOC_DEFAULT_WIDTH: f32 = 520.0;
pub const DOC_MIN_WIDTH: f32 = 450.0;
pub const DOC_MAX_WIDTH: f32 = 860.0;

pub const MAIN_ICON_SIZE: f32 = 20.0;
pub const SUB_ICON_SIZE: f32 = 18.0;

// ----------------------------------------------------------------------------
// CAD 工具栏保留的图标与命令组 (Req 5: 范围缩放放在窗口缩放前，并跟它一组)
// ----------------------------------------------------------------------------

pub type CadTool = (Icon, &'static str, &'static str, Option<&'static str>);

pub const CAD_TOOL_GROUPS: &[&[CadTool]] = &[
    // Group 1: 视图导航 (范围缩放，窗口缩放，平移)
    &[
        (Icon::ZoomExtents, "zoom.extents", "范围缩放", None),
        (Icon::ZoomWindow, "zoom.window", "窗口缩放", None),
        (Icon::Pan, "pan", "平移", None),
    ],
    // Group 2: 测量与查询
    &[
        (Icon::Measure, "dist", "测量距离", None),
    ],
    // Group 3: 帮助
    &[
        (Icon::Help, "cadflow.help", "帮助", None),
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

// ----------------------------------------------------------------------------
// 动态多级菜单渲染（遍历 CADCraft 原生菜单树并采用中文语言包）
// ----------------------------------------------------------------------------

fn render_menu_entry(ui: &mut Ui, e: &Entry, clicked: &mut Option<String>) {
    match e {
        Entry::Item {
            label,
            id,
            shortcut,
            enabled,
        } => {
            let label_tr = crate::sidebar::tr_ui(label);
            let mut b = egui::Button::new(label_tr);
            if let Some(s) = shortcut {
                b = b.shortcut_text(format_shortcut(s, ui.ctx().os().is_mac()));
            }
            if ui.add_enabled(*enabled, b).clicked() {
                *clicked = Some(id.clone());
                ui.close();
            }
        }
        Entry::Sub { label, children } => {
            let label_tr = crate::sidebar::tr_ui(label);
            ui.menu_button(label_tr, |ui| {
                ui.set_min_width(148.0);
                for c in children {
                    render_menu_entry(ui, c, clicked);
                }
            });
        }
    }
}

/// 绘制 Buerligons 风格的折叠菜单按钮（18x18 汉堡图标 + CADFlow 字样 + 包含全部菜单项与子项的中文下拉菜单）
pub fn draw_collapsed_menu(
    ui: &mut Ui,
    app: &mut CadApp,
    show_help_dialog: &mut bool,
) {
    let t = Tokens::get();

    let mut style = (**ui.style()).clone();
    style.spacing.menu_margin = Margin::same(4);
    style.spacing.menu_width = 148.0;
    style.spacing.interact_size = vec2(64.0, 28.0);
    style.spacing.button_padding = vec2(9.0, 5.0);
    style.spacing.item_spacing = vec2(4.0, 2.0);
    // 保持 CADFlow 菜单按钮字体为 12.5px 并禁止折行
    style.text_styles.insert(egui::TextStyle::Button, egui::FontId::proportional(12.5));
    style.wrap_mode = Some(egui::TextWrapMode::Extend);

    style.visuals.window_fill = t.chrome;
    style.visuals.window_corner_radius = CornerRadius::same(10);
    style.visuals.menu_corner_radius = CornerRadius::same(10);
    style.visuals.window_stroke = Stroke::new(1.0, t.border);

    style.visuals.widgets.inactive.weak_bg_fill = Color32::TRANSPARENT;
    style.visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, t.text);
    style.visuals.widgets.hovered.weak_bg_fill = t.control_hover.gamma_multiply(0.6);
    style.visuals.widgets.hovered.bg_stroke = Stroke::NONE;
    style.visuals.widgets.hovered.corner_radius = CornerRadius::same(6);
    style.visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, t.text);
    style.visuals.widgets.active.weak_bg_fill = t.control;
    style.visuals.widgets.open.weak_bg_fill = t.control;
    ui.set_style(style);

    ui.spacing_mut().interact_size = vec2(96.0, TOOL_SIZE);
    ui.spacing_mut().button_padding = vec2(10.0, 6.0);

    let mut clicked = None;
    let tree = menus::tree(app);

    let menu_resp = ui.menu_button("      CADFlow", |ui| {
        ui.set_min_width(148.0);
        for (name, entries) in &tree {
            let name_tr = crate::sidebar::tr_ui(name);
            if *name == "Help" {
                // Req 14 / Req 9: 去掉菜单中的帮助（Help）的子菜单，点击后直接弹出 CADFlow 帮助对话框
                if ui.button(name_tr).clicked() {
                    clicked = Some("cadflow_help".into());
                    ui.close();
                }
            } else {
                ui.menu_button(name_tr, |ui| {
                    ui.set_min_width(148.0);
                    for e in entries {
                        render_menu_entry(ui, e, &mut clicked);
                    }
                });
            }
        }
    });

    let btn_rect = menu_resp.response.rect;
    let icon_rect = Rect::from_min_size(
        pos2(btn_rect.left() + 7.0, btn_rect.center().y - 9.0),
        vec2(SUB_ICON_SIZE, SUB_ICON_SIZE),
    );
    icons::paint(ui.painter(), icon_rect, Icon::Menu, false);

    if let Some(id) = clicked {
        if id == "cadflow_help" {
            *show_help_dialog = true;
        } else {
            menus::activate(app, &id);
        }
    }
}

// ----------------------------------------------------------------------------
// Buerligons 风格图标绘制
// ----------------------------------------------------------------------------

pub fn paint_sidebar_toggle(p: &egui::Painter, r: Rect, open: bool, fg: Color32) {
    let box_rect = Rect::from_center_size(r.center(), vec2(16.0, 14.0));
    p.rect_stroke(box_rect, CornerRadius::same(2), Stroke::new(1.5, fg), egui::StrokeKind::Inside);
    let split_x = box_rect.left() + 5.0;
    p.vline(split_x, box_rect.y_range(), Stroke::new(1.5, fg));
    if open {
        let fill_rect = Rect::from_min_max(box_rect.min, pos2(split_x, box_rect.bottom()));
        p.rect_filled(fill_rect, CornerRadius { nw: 2, sw: 2, ne: 0, se: 0 }, fg);
    }
}

pub fn paint_doc_toggle(p: &egui::Painter, r: Rect, open: bool, fg: Color32) {
    let box_rect = Rect::from_center_size(r.center(), vec2(16.0, 14.0));
    p.rect_stroke(box_rect, CornerRadius::same(2), Stroke::new(1.5, fg), egui::StrokeKind::Inside);
    let split_x = box_rect.right() - 5.0;
    p.vline(split_x, box_rect.y_range(), Stroke::new(1.5, fg));
    if open {
        let fill_rect = Rect::from_min_max(pos2(split_x, box_rect.top()), box_rect.max);
        p.rect_filled(fill_rect, CornerRadius { nw: 0, sw: 0, ne: 2, se: 2 }, fg);
    }
}

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

        let icon_rect = Rect::from_center_size(rect.center(), vec2(icon_size, icon_size));
        draw_icon(ui.painter(), icon_rect);

        if active {
            let indicator = Rect::from_min_max(
                pos2(rect.left() + 6.0, rect.bottom() - 3.5),
                pos2(rect.right() - 6.0, rect.bottom() - 1.5),
            );
            ui.painter().rect_filled(indicator, CornerRadius::same(1), t.accent);
        }
    }
    resp
}

pub fn draw_doc_tool_btn(ui: &mut Ui, icon: crate::doc_icons::DocIcon, tooltip: &str) -> egui::Response {
    let t = Tokens::get();
    let btn_sz = 28.0;
    let (rect, resp) = ui.allocate_exact_size(vec2(btn_sz, btn_sz), Sense::click());
    let resp = resp.on_hover_text(tooltip);

    if ui.is_rect_visible(rect) {
        let bg = if resp.is_pointer_button_down_on() {
            t.control
        } else if resp.hovered() {
            t.control_hover.gamma_multiply(0.5)
        } else {
            Color32::TRANSPARENT
        };

        if bg != Color32::TRANSPARENT {
            ui.painter().rect_filled(rect, CornerRadius::same(5), bg);
        }

        let icon_sz = 16.0;
        let icon_rect = Rect::from_center_size(rect.center(), vec2(icon_sz, icon_sz));
        crate::doc_icons::paint_doc_icon(ui.painter(), icon_rect, icon, t.text);
    }
    resp
}

#[allow(dead_code)]
pub fn draw_stackedit_btn(ui: &mut Ui, label: &str, tooltip: &str) -> egui::Response {
    let t = Tokens::get();
    let btn_sz = 28.0;
    let (rect, resp) = ui.allocate_exact_size(vec2(btn_sz, btn_sz), Sense::click());
    let resp = resp.on_hover_text(tooltip);

    if ui.is_rect_visible(rect) {
        let bg = if resp.is_pointer_button_down_on() {
            t.control
        } else if resp.hovered() {
            t.control_hover.gamma_multiply(0.5)
        } else {
            Color32::TRANSPARENT
        };

        if bg != Color32::TRANSPARENT {
            ui.painter().rect_filled(rect, CornerRadius::same(5), bg);
        }

        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            label,
            egui::FontId::proportional(13.0),
            t.text,
        );
    }
    resp
}

// ----------------------------------------------------------------------------
// 播放控制器状态与速度滑块
// ----------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlaybackState {
    pub is_playing: bool,
    pub speed_progress: f32, // 0.0 .. 1.0 (0.35 对应初始 5x)
    pub show_settings: bool,
}

impl Default for PlaybackState {
    fn default() -> Self {
        Self {
            is_playing: false,
            speed_progress: 0.35,
            show_settings: false,
        }
    }
}

pub fn progress_to_speed_text(p: f32) -> String {
    if p >= 0.98 {
        "MAX".to_string()
    } else if p <= 0.01 {
        "0.1x".to_string()
    } else if p < 0.35 {
        // 0.01..0.35 映射到 0.1x .. 5.0x
        let t = (p - 0.01) / 0.34;
        let val = 0.1 + (5.0 - 0.1) * t;
        if val < 1.0 {
            format!("{:.1}x", val)
        } else {
            format!("{:.0}x", val.round())
        }
    } else {
        // 0.35..0.98 映射到 5.0x .. 50.0x
        let t = (p - 0.35) / 0.63;
        if t < 0.03 {
            "5x".to_string()
        } else {
            let val = 5.0 + (50.0 - 5.0) * t;
            format!("{:.0}x", val.round())
        }
    }
}

pub fn draw_speed_slider(ui: &mut Ui, progress: &mut f32) -> egui::Response {
    let track_w = 84.0;
    let track_h = 7.0;
    let thumb_r = 6.0;
    let (rect, resp) = ui.allocate_exact_size(vec2(track_w, TOOL_SIZE), Sense::click_and_drag());
    let resp = resp.on_hover_text(format!("调节速度 (当前: {})", progress_to_speed_text(*progress)));

    if (resp.clicked() || resp.dragged())
        && let Some(pos) = resp.interact_pointer_pos()
    {
        let left = rect.left() + thumb_r;
        let right = rect.right() - thumb_r;
        let new_p = if right > left {
            ((pos.x - left) / (right - left)).clamp(0.0, 1.0)
        } else {
            0.5
        };
        *progress = new_p;
    }

    if ui.is_rect_visible(rect) {
        let p = ui.painter();
        let center_y = rect.center().y;
        let left_x = rect.left() + thumb_r;
        let right_x = rect.right() - thumb_r;
        let thumb_x = left_x + (right_x - left_x) * (*progress).clamp(0.0, 1.0);

        // 1. 底槽胶囊条 (暗色背景，对应截图)
        let track_rect = Rect::from_center_size(
            pos2(rect.center().x, center_y),
            vec2(rect.width(), track_h),
        );
        p.rect_filled(
            track_rect,
            CornerRadius::same((track_h / 2.0).round() as u8),
            Color32::from_rgb(46, 50, 56),
        );
        p.rect_stroke(
            track_rect,
            CornerRadius::same((track_h / 2.0).round() as u8),
            Stroke::new(1.0, Color32::from_rgb(62, 66, 74)),
            egui::StrokeKind::Inside,
        );

        // 2. 激活段填充 (浅亮灰胶囊左侧部分)
        if thumb_x > rect.left() + 2.0 {
            let active_rect = Rect::from_min_max(
                track_rect.min,
                pos2(thumb_x.min(track_rect.right()), track_rect.max.y),
            );
            p.rect_filled(
                active_rect,
                CornerRadius {
                    nw: (track_h / 2.0).round() as u8,
                    sw: (track_h / 2.0).round() as u8,
                    ne: 0,
                    se: 0,
                },
                Color32::from_rgb(220, 224, 230),
            );
        }

        // 3. 滑动手柄圆形圆钮 (白色圆形小手柄)
        let thumb_center = pos2(thumb_x, center_y);
        p.circle_filled(thumb_center, thumb_r + 1.0, Color32::from_black_alpha(45));
        p.circle_filled(thumb_center, thumb_r, Color32::from_rgb(240, 242, 246));
        p.circle_stroke(
            thumb_center,
            thumb_r,
            Stroke::new(0.8, Color32::from_rgb(205, 209, 215)),
        );
    }

    resp
}

pub fn draw_toolbar_sep(ui: &mut Ui) {
    let t = Tokens::get();
    let (rect, _) = ui.allocate_exact_size(vec2(10.0, TOOL_SIZE), Sense::hover());
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

pub fn draw_top_toolbar(
    ui: &mut Ui,
    app: &mut CadApp,
    sidebar_open: &mut bool,
    sidebar_width: &mut f32,
    doc_open: &mut bool,
    doc_width: &mut f32,
    doc_state: &mut crate::doc_panel::DocState,
    show_help_dialog: &mut bool,
) {
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
            let total_rect = ui.max_rect();
            let mut clicked_cad_cmd = None;
            let mut doc_action = None;

            let slide_progress = ui.ctx().animate_bool(egui::Id::new("cadflow_doc_slide"), *doc_open);

            // ================================================================
            // 1. 左侧区域 (Leading Section)：
            //    CADFlow 菜单 -> 分割线 -> 撤销 -> 恢复 -> CAD侧边栏按钮 -> 侧边栏垂直边界线
            //    Req 3, Req 4: 边界线与侧边栏边界完全对齐并贯穿工具栏
            // ================================================================
            let is_sb_open = *sidebar_open;
            let current_sidebar_w = (*sidebar_width).clamp(SIDEBAR_MIN_WIDTH, SIDEBAR_MAX_WIDTH);

            let mut cursor_x = total_rect.left() + 8.0;

            // [1] CADFlow 菜单 (宽 96px)
            ui.scope_builder(
                egui::UiBuilder::new().max_rect(Rect::from_min_size(
                    pos2(cursor_x, total_rect.center().y - TOOL_SIZE / 2.0),
                    vec2(96.0, TOOL_SIZE),
                )),
                |ui| {
                    draw_collapsed_menu(ui, app, show_help_dialog);
                },
            );
            cursor_x += 96.0 + TOOL_SPACING;

            // [2] 分割线
            ui.scope_builder(
                egui::UiBuilder::new().max_rect(Rect::from_min_size(
                    pos2(cursor_x, total_rect.center().y - TOOL_SIZE / 2.0),
                    vec2(10.0, TOOL_SIZE),
                )),
                |ui| {
                    draw_toolbar_sep(ui);
                },
            );
            cursor_x += 10.0 + TOOL_SPACING;

            // [3] 撤销 (Undo)
            let undo_tip = format!("{} ({})", crate::sidebar::tr_ui("Undo"), format_shortcut("Cmd+Z", mac));
            ui.scope_builder(
                egui::UiBuilder::new().max_rect(Rect::from_min_size(
                    pos2(cursor_x, total_rect.center().y - TOOL_SIZE / 2.0),
                    vec2(TOOL_SIZE, TOOL_SIZE),
                )),
                |ui| {
                    let undo_resp = draw_tool_icon_button(ui, TOOL_SIZE, SUB_ICON_SIZE, false, &undo_tip, |p, r| {
                        icons::paint(p, r, Icon::Undo, false);
                    });
                    if undo_resp.clicked() {
                        clicked_cad_cmd = Some("u");
                    }
                },
            );
            cursor_x += TOOL_SIZE + TOOL_SPACING;

            // [4] 恢复 (Redo)
            let redo_tip = format!("{} ({})", crate::sidebar::tr_ui("Redo"), format_shortcut("Cmd+Shift+Z", mac));
            ui.scope_builder(
                egui::UiBuilder::new().max_rect(Rect::from_min_size(
                    pos2(cursor_x, total_rect.center().y - TOOL_SIZE / 2.0),
                    vec2(TOOL_SIZE, TOOL_SIZE),
                )),
                |ui| {
                    let redo_resp = draw_tool_icon_button(ui, TOOL_SIZE, SUB_ICON_SIZE, false, &redo_tip, |p, r| {
                        icons::paint(p, r, Icon::Redo, false);
                    });
                    if redo_resp.clicked() {
                        clicked_cad_cmd = Some("redo");
                    }
                },
            );

            // [5] CAD 侧边栏按钮与垂直边界线位置计算 (Req 3, Req 4)
            let min_sb_btn_x = total_rect.left() + SIDEBAR_MIN_WIDTH - TOOL_SIZE - 4.0;
            let (sb_btn_x, boundary_x) = if is_sb_open {
                let bx = total_rect.left() + current_sidebar_w;
                ((bx - TOOL_SIZE - 4.0).max(min_sb_btn_x), bx)
            } else {
                (min_sb_btn_x, min_sb_btn_x + TOOL_SIZE + 4.0)
            };

            ui.scope_builder(
                egui::UiBuilder::new().max_rect(Rect::from_min_size(
                    pos2(sb_btn_x, total_rect.center().y - TOOL_SIZE / 2.0),
                    vec2(TOOL_SIZE, TOOL_SIZE),
                )),
                |ui| {
                    let sb_resp = draw_tool_icon_button(ui, TOOL_SIZE, SUB_ICON_SIZE, false, "CAD侧边栏", |p, r| {
                        paint_sidebar_toggle(p, r, is_sb_open, t.text);
                    });
                    if sb_resp.clicked() {
                        *sidebar_open = !*sidebar_open;
                    }
                },
            );

            // Req 3, Req 4: 侧边栏右边界竖线贯穿整个工具栏，与侧边栏边界完全对齐
            ui.painter().vline(boundary_x, total_rect.y_range(), Stroke::new(1.0, t.border));

            // ================================================================
            // 2. 右侧区域 (Trailing Section)：
            //    包含“文档侧边栏”按钮与 StackEdit 风格的文档编辑按钮组
            //    边界与文档面板对齐，滑动动画效果展示/隐藏 (Req 8, Req 10)
            // ================================================================
            let full_doc_w = (*doc_width).clamp(DOC_MIN_WIDTH, DOC_MAX_WIDTH);
            let effective_doc_w = full_doc_w * slide_progress;

            let doc_area_rect = if slide_progress > 0.001 {
                Rect::from_min_max(
                    pos2((total_rect.right() - effective_doc_w).max(boundary_x + 180.0), total_rect.top()),
                    total_rect.max,
                )
            } else {
                Rect::from_min_max(
                    pos2(total_rect.right() - (TOOL_SIZE + 16.0), total_rect.top()),
                    total_rect.max,
                )
            };

            // 若文档展开且滑动进度大于 0，绘制贯通工具栏的边界线
            if slide_progress > 0.001 {
                ui.painter().vline(doc_area_rect.left(), total_rect.y_range(), Stroke::new(1.0, t.border));
            }

            ui.scope_builder(
                egui::UiBuilder::new().max_rect(doc_area_rect.shrink2(vec2(6.0, 8.0))),
                |ui| {
                    ui.horizontal_centered(|ui| {
                        ui.spacing_mut().item_spacing.x = 2.0;

                        // [1] “文档侧边栏”按钮 (Req 8)
                        let is_doc_open = *doc_open;
                        let doc_resp = draw_tool_icon_button(ui, TOOL_SIZE, SUB_ICON_SIZE, false, "文档侧边栏", |p, r| {
                            paint_doc_toggle(p, r, is_doc_open, t.text);
                        });
                        if doc_resp.clicked() {
                            *doc_open = !*doc_open;
                        }

                        // [2] 展开状态下渲染 StackEdit Markdown 编辑工具组 (已去除撤销/重做编辑，Req 8)
                        if slide_progress > 0.5 {
                            draw_toolbar_sep(ui);

                            if draw_doc_tool_btn(ui, crate::doc_icons::DocIcon::Bold, &format!("加粗 ({})", format_shortcut("Cmd+Shift+B", mac))).clicked() {
                                doc_action = Some(crate::doc_panel::DocAction::Bold);
                            }
                            if draw_doc_tool_btn(ui, crate::doc_icons::DocIcon::Italic, &format!("斜体 ({})", format_shortcut("Cmd+Shift+I", mac))).clicked() {
                                doc_action = Some(crate::doc_panel::DocAction::Italic);
                            }
                            if draw_doc_tool_btn(ui, crate::doc_icons::DocIcon::Heading, &format!("标题 ({})", format_shortcut("Cmd+Shift+H", mac))).clicked() {
                                doc_action = Some(crate::doc_panel::DocAction::Heading);
                            }
                            if draw_doc_tool_btn(ui, crate::doc_icons::DocIcon::Strikethrough, &format!("删除线 ({})", format_shortcut("Cmd+Shift+S", mac))).clicked() {
                                doc_action = Some(crate::doc_panel::DocAction::Strikethrough);
                            }

                            draw_toolbar_sep(ui);

                            if draw_doc_tool_btn(ui, crate::doc_icons::DocIcon::UnorderedList, &format!("无序列表 ({})", format_shortcut("Cmd+Shift+U", mac))).clicked() {
                                doc_action = Some(crate::doc_panel::DocAction::UnorderedList);
                            }
                            if draw_doc_tool_btn(ui, crate::doc_icons::DocIcon::OrderedList, &format!("有序列表 ({})", format_shortcut("Cmd+Shift+O", mac))).clicked() {
                                doc_action = Some(crate::doc_panel::DocAction::OrderedList);
                            }
                            if draw_doc_tool_btn(ui, crate::doc_icons::DocIcon::CheckList, &format!("待办清单 ({})", format_shortcut("Cmd+Shift+C", mac))).clicked() {
                                doc_action = Some(crate::doc_panel::DocAction::CheckList);
                            }

                            draw_toolbar_sep(ui);

                            if draw_doc_tool_btn(ui, crate::doc_icons::DocIcon::Blockquote, &format!("引用块 ({})", format_shortcut("Cmd+Shift+Q", mac))).clicked() {
                                doc_action = Some(crate::doc_panel::DocAction::Quote);
                            }
                            if draw_doc_tool_btn(ui, crate::doc_icons::DocIcon::Code, &format!("代码块 ({})", format_shortcut("Cmd+Shift+K", mac))).clicked() {
                                doc_action = Some(crate::doc_panel::DocAction::Code);
                            }
                            if draw_doc_tool_btn(ui, crate::doc_icons::DocIcon::Table, &format!("表格 ({})", format_shortcut("Cmd+Shift+T", mac))).clicked() {
                                doc_action = Some(crate::doc_panel::DocAction::Table);
                            }
                            if draw_doc_tool_btn(ui, crate::doc_icons::DocIcon::Link, &format!("超链接 ({})", format_shortcut("Cmd+Shift+L", mac))).clicked() {
                                doc_action = Some(crate::doc_panel::DocAction::Link);
                            }
                            if draw_doc_tool_btn(ui, crate::doc_icons::DocIcon::Image, &format!("图片 ({})", format_shortcut("Cmd+Shift+G", mac))).clicked() {
                                doc_action = Some(crate::doc_panel::DocAction::Image);
                            }
                        }
                    });
                },
            );

            // ================================================================
            // 3. 中间区域 (Center Section)：
            //    CAD 工具（范围缩放、窗口缩放、平移、测量、帮助）
            //    Req 4: 前面不再绘制单独分隔线，直接以侧边栏边界线为分割
            // ================================================================
            let center_rect = Rect::from_min_max(
                pos2(boundary_x + 4.0, total_rect.top()),
                pos2(doc_area_rect.left() - 4.0, total_rect.bottom()),
            );

            if center_rect.width() > 50.0 {
                ui.scope_builder(
                    egui::UiBuilder::new().max_rect(center_rect.shrink2(vec2(4.0, 8.0))),
                    |ui| {
                        egui::ScrollArea::horizontal()
                            .id_salt("cad_center_toolbar_scroll")
                            .auto_shrink([false, false])
                            .show(ui, |ui| {
                                ui.horizontal_centered(|ui| {
                                    ui.spacing_mut().item_spacing.x = TOOL_SPACING;

                                    // --------------------------------------------------------
                                    // 播放控制组件组 (播放/暂停、停止、“速度”、进度条、速度文本、“设置”)
                                    // --------------------------------------------------------
                                    let playback_id = egui::Id::new("cadflow_playback_toolbar_state");
                                    let mut playback_state = ui.data_mut(|d| d.get_temp::<PlaybackState>(playback_id)).unwrap_or_default();

                                    // [1] 播放 / 暂停按钮 (点击切换状态)
                                    let play_tip = if playback_state.is_playing { "暂停" } else { "播放" };
                                    let play_resp = draw_tool_icon_button(
                                        ui,
                                        TOOL_SIZE,
                                        MAIN_ICON_SIZE,
                                        playback_state.is_playing,
                                        play_tip,
                                        |p, rect| {
                                            let icon = if playback_state.is_playing {
                                                crate::doc_icons::DocIcon::Pause
                                            } else {
                                                crate::doc_icons::DocIcon::Play
                                            };
                                            crate::doc_icons::paint_doc_icon(p, rect, icon, t.text);
                                        },
                                    );
                                    if play_resp.clicked() {
                                        playback_state.is_playing = !playback_state.is_playing;
                                    }

                                    // [2] 停止按钮
                                    let stop_resp = draw_tool_icon_button(
                                        ui,
                                        TOOL_SIZE,
                                        MAIN_ICON_SIZE,
                                        false,
                                        "停止",
                                        |p, rect| {
                                            crate::doc_icons::paint_doc_icon(p, rect, crate::doc_icons::DocIcon::Stop, t.text);
                                        },
                                    );
                                    if stop_resp.clicked() {
                                        playback_state.is_playing = false;
                                    }

                                    // [3] “速度”标签
                                    ui.add_space(4.0);
                                    ui.label(RichText::new("速度").size(13.0).color(t.text));
                                    ui.add_space(2.0);

                                    // [4] 进度条 / 速度滑块
                                    draw_speed_slider(ui, &mut playback_state.speed_progress);
                                    ui.add_space(2.0);

                                    // [5] 速度文本展示 (初始 5x，最小 0.1x，最大 MAX)
                                    let speed_text = progress_to_speed_text(playback_state.speed_progress);
                                    ui.allocate_ui(vec2(32.0, TOOL_SIZE), |ui| {
                                        ui.centered_and_justified(|ui| {
                                            ui.label(RichText::new(speed_text).size(13.0).color(t.text));
                                        });
                                    });
                                    ui.add_space(2.0);

                                    // [6] 设置按钮
                                    let settings_resp = draw_tool_icon_button(
                                        ui,
                                        TOOL_SIZE,
                                        MAIN_ICON_SIZE,
                                        playback_state.show_settings,
                                        "设置",
                                        |p, rect| {
                                            crate::doc_icons::paint_doc_icon(p, rect, crate::doc_icons::DocIcon::Settings, t.text);
                                        },
                                    );
                                    if settings_resp.clicked() {
                                        playback_state.show_settings = !playback_state.show_settings;
                                    }

                                    if playback_state.show_settings {
                                        egui::Window::new("播放设置")
                                            .collapsible(false)
                                            .resizable(false)
                                            .open(&mut playback_state.show_settings)
                                            .anchor(egui::Align2::LEFT_TOP, [center_rect.left() + 40.0, BAR_HEIGHT + 4.0])
                                            .show(ui.ctx(), |ui| {
                                                ui.label(RichText::new("速度快捷切换:").strong());
                                                ui.horizontal(|ui| {
                                                    for (preset_p, label) in [
                                                        (0.01, "0.1x"),
                                                        (0.08, "1x"),
                                                        (0.15, "2x"),
                                                        (0.35, "5x"),
                                                        (0.42, "10x"),
                                                        (1.0, "MAX"),
                                                    ] {
                                                        if ui.button(label).clicked() {
                                                            playback_state.speed_progress = preset_p;
                                                        }
                                                    }
                                                });
                                            });
                                    }

                                    ui.data_mut(|d| d.insert_temp(playback_id, playback_state));

                                    // 分隔线：播放控制组与 CAD 绘图视图工具组(从“范围缩放”开始)之间
                                    draw_toolbar_sep(ui);

                                    for (g_idx, group) in CAD_TOOL_GROUPS.iter().enumerate() {
                                        // Req 4: 第一个工具组前不画分隔线 (以侧边栏边界线代替)
                                        if g_idx > 0 {
                                            draw_toolbar_sep(ui);
                                        }

                                        for (icon, cmd, label, shortcut) in group.iter() {
                                            let label_tr = crate::sidebar::tr_ui(label);
                                            let tip = match shortcut {
                                                Some(sc) => format!("{} ({})", label_tr, format_shortcut(sc, mac)),
                                                None => label_tr,
                                            };

                                            let resp = draw_tool_icon_button(
                                                ui,
                                                TOOL_SIZE,
                                                MAIN_ICON_SIZE,
                                                false,
                                                &tip,
                                                |p, rect| {
                                                    icons::paint(p, rect, *icon, false);
                                                },
                                            );

                                            if resp.clicked() {
                                                clicked_cad_cmd = Some(*cmd);
                                            }
                                        }
                                    }
                                });
                            });
                    },
                );
            }

            // 处理 CAD 命令点击
            if let Some(cmd) = clicked_cad_cmd {
                if cmd == "cadflow.help" {
                    *show_help_dialog = true;
                } else {
                    app.start(cmd);
                }
            }

            // 处理 Markdown 编辑操作
            if let Some(act) = doc_action {
                doc_state.apply_action(act);
            }
        });
}

/// 渲染 CADFlow 专用帮助对话框 (Req 14)
/// - 对话框标题：“CADFlow”
/// - 标签页：About (清空) 与 Command (沿用 CADCraft Help 命令参考)
pub fn show_cadflow_help_dialog(app: &mut CadApp, ctx: &egui::Context, open: &mut bool) {
    let t = Tokens::get();
    let mut start_cmd = None;

    egui::Window::new("CADFlow")
        .id(egui::Id::new("cadflow_help_window"))
        .open(open)
        .default_size(vec2(640.0, 480.0))
        .pivot(egui::Align2::CENTER_CENTER)
        .default_pos(ctx.content_rect().center())
        .show(ctx, |ui| {
            let id = ui.id().with("help_tab");
            let mut active_tab = ui.data_mut(|d| d.get_temp::<usize>(id)).unwrap_or(0);

            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 8.0;
                for (idx, name) in ["About", "Command"].iter().enumerate() {
                    let is_active = active_tab == idx;
                    if ui.selectable_label(is_active, *name).clicked() {
                        active_tab = idx;
                        ui.data_mut(|d| d.insert_temp(id, active_tab));
                    }
                }
            });
            ui.separator();

            if active_tab == 0 {
                // About 标签页内容清空 (Req 14)
                ui.vertical_centered(|ui| {
                    ui.add_space(60.0);
                    ui.heading("CADFlow");
                    ui.add_space(8.0);
                    ui.label(RichText::new("Web CAD & Markdown Workspace").small().color(t.text_dim));
                });
            } else {
                // Command 标签页沿用原 CADCraft Help 命令参考对话框内容
                let filter_id = ui.id().with("cmdfilter");
                let mut filter = ui.data_mut(|d| d.get_temp::<String>(filter_id)).unwrap_or_default();
                ui.horizontal(|ui| {
                    ui.label(cadcraft_ui_egui::i18n::t("Filter"));
                    ui.text_edit_singleline(&mut filter);
                });
                ui.data_mut(|d| d.insert_temp(filter_id, filter.clone()));
                let f = filter.to_lowercase();

                egui::ScrollArea::vertical()
                    .id_salt("cadflow_cmd_scroll")
                    .show(ui, |ui| {
                        egui::Grid::new("cadflow_cmdref").striped(true).num_columns(3).show(ui, |ui| {
                            for c in cadcraft_engine::command_specs() {
                                if !f.is_empty()
                                    && !c.id.contains(&f)
                                    && !c.label.to_ascii_lowercase().contains(&f)
                                    && !cadcraft_ui_egui::i18n::t(c.label).to_lowercase().contains(&f)
                                {
                                    continue;
                                }
                                if ui.link(c.id.to_ascii_uppercase()).clicked() {
                                    start_cmd = Some(c.id);
                                }
                                ui.label(cadcraft_ui_egui::i18n::t(c.label));
                                ui.label(
                                    RichText::new(if c.aliases.is_empty() {
                                        String::new()
                                    } else {
                                        c.aliases.join(", ").to_ascii_uppercase()
                                    })
                                    .small(),
                                );
                                ui.end_row();
                            }
                        });
                    });
            }
        });

    if let Some(cmd) = start_cmd {
        *open = false;
        app.ui.dialog = None;
        app.start(cmd);
    }
}




