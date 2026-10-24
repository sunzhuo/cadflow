//! Buerligons UI specifications for CADFlow (egui):
//! - Top toolbar height: 48px, tool button: 32x32px, corner radius: 7px, spacing: 2px
//! - Vertical tool dock: 34px width (28x28px button, 3px padding, 2px spacing)
//! - Sidebar: default width 288px, resizable range 264..=560px
//! - Icons: main toolbar 20x20px (stroke 1.5), collapsed menu / undo / redo / dock 18x18px (stroke 1.61)
//! - Leftmost collapsed multi-level dropdown menu with 18x18 hamburger icon and MENU label

use egui::{pos2, vec2, Color32, CornerRadius, Margin, Rect, Sense, Stroke, Ui};

/// Buerligons design tokens and constants.
pub const BAR_HEIGHT: f32 = 48.0;
pub const TOOL_SIZE: f32 = 32.0;
pub const TOOL_CORNER_RADIUS: f32 = 7.0;
pub const TOOL_SPACING: f32 = 2.0;

pub const DOCK_WIDTH: f32 = 34.0;
pub const DOCK_TOOL_SIZE: f32 = 28.0;
pub const DOCK_PADDING: f32 = 3.0;

pub const SIDEBAR_DEFAULT_WIDTH: f32 = 288.0;
pub const SIDEBAR_MIN_WIDTH: f32 = 264.0;
pub const SIDEBAR_MAX_WIDTH: f32 = 560.0;

pub const MAIN_ICON_SIZE: f32 = 20.0;
pub const MAIN_ICON_STROKE: f32 = 1.5;
pub const SUB_ICON_SIZE: f32 = 18.0;
pub const SUB_ICON_STROKE: f32 = 1.61;

// Colors
pub const COLOR_INK: Color32 = Color32::from_rgb(0x0f, 0x13, 0x20);      // #0f1320
pub const COLOR_INK_2: Color32 = Color32::from_rgb(0x2a, 0x2f, 0x3a);    // #2a2f3a
pub const COLOR_MUTE: Color32 = Color32::from_rgb(0x5d, 0x64, 0x74);     // #5d6474
pub const COLOR_BORDER: Color32 = Color32::from_rgb(0xe3, 0xe5, 0xea);   // #e3e5ea
pub const COLOR_HOVER: Color32 = Color32::from_rgba_premultiplied(15, 19, 32, 12);
pub const COLOR_ACTIVE: Color32 = Color32::from_rgba_premultiplied(15, 19, 32, 20);

const MENU_CMD_ID: &str = "buerligons_menu_cmd";

/// Compatibility trait for `ui.close_menu()`.
pub trait UiMenuExt {
    fn close_menu(&mut self);
}

impl UiMenuExt for Ui {
    fn close_menu(&mut self) {
        self.close();
    }
}

/// Retrieve and clear any command triggered by `draw_collapsed_menu`.
pub fn take_menu_command(ctx: &egui::Context) -> Option<String> {
    ctx.data_mut(|d| d.remove_temp::<String>(egui::Id::new(MENU_CMD_ID)))
}

/// Record a command to be executed by the host application.
pub fn set_menu_command(ui: &mut Ui, cmd: &str) {
    ui.ctx().data_mut(|d| d.insert_temp(egui::Id::new(MENU_CMD_ID), cmd.to_string()));
}

/// 绘制 Buerligons 风格的折叠菜单按钮（18x18 汉堡图标 + MENU 字样 + 下拉多级子菜单）
pub fn draw_collapsed_menu(ui: &mut Ui) {
    // 1. 局部覆盖 egui 菜单样式，对齐 Buerligons 的 .ant-dropdown-menu 规范
    let mut style = (**ui.style()).clone();
    style.spacing.menu_margin = Margin::same(4);            // 菜单外框内边距 4px
    style.spacing.menu_width = 148.0;                       // 菜单最小宽度 148px
    style.spacing.interact_size = vec2(64.0, 28.0);         // 菜单项高度 28px
    style.spacing.button_padding = vec2(9.0, 5.0);          // 菜单项内边距 5px 9px
    style.spacing.item_spacing = vec2(4.0, 2.0);
    style.visuals.window_fill = Color32::WHITE;             // 菜单白底 #ffffff
    style.visuals.window_corner_radius = CornerRadius::same(10); // 菜单圆角 10px
    style.visuals.menu_corner_radius = CornerRadius::same(10);
    style.visuals.window_stroke = Stroke::new(1.0, Color32::from_rgba_unmultiplied(15, 19, 32, 20));
    style.visuals.widgets.inactive.weak_bg_fill = Color32::TRANSPARENT;
    style.visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, COLOR_INK_2); // #2a2f3a
    style.visuals.widgets.hovered.weak_bg_fill = Color32::from_rgba_unmultiplied(15, 19, 32, 12);    // 悬停背景
    style.visuals.widgets.hovered.bg_stroke = Stroke::NONE;
    style.visuals.widgets.hovered.corner_radius = CornerRadius::same(6);                             // 菜单项圆角 6px
    style.visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, COLOR_INK); // #0f1320
    ui.set_style(style);

    // 2. 临时调整顶层 MENU 按钮的高度(32px)与内边距，为左侧 18x18 图标留出空间
    ui.spacing_mut().interact_size = vec2(68.0, 32.0); // --rcad-tool: 32px
    ui.spacing_mut().button_padding = vec2(10.0, 6.0);

    // 使用空格预留左侧 18x18 图标 + 7px 间距的位置，点击后在正下方展开菜单
    let menu_resp = ui.menu_button("     MENU", |ui| {
        ui.set_min_width(148.0);

        // 一级子菜单：文件
        ui.menu_button("文件 (File)", |ui| {
            ui.set_min_width(148.0);
            if ui.button("新建 (New)        Ctrl+N").clicked() {
                set_menu_command(ui, "new");
                ui.close();
            }
            if ui.button("打开 (Open)...    Ctrl+O").clicked() {
                set_menu_command(ui, "open");
                ui.close();
            }
            ui.separator();
            if ui.button("保存 (Save)       Ctrl+S").clicked() {
                set_menu_command(ui, "qsave");
                ui.close();
            }
            if ui.button("导出 DXF...").clicked() {
                set_menu_command(ui, "export");
                ui.close();
            }
        });

        // 一级子菜单：编辑
        ui.menu_button("编辑 (Edit)", |ui| {
            ui.set_min_width(148.0);
            if ui.button("撤销 (Undo)       Ctrl+Z").clicked() {
                set_menu_command(ui, "u");
                ui.close();
            }
            if ui.button("重做 (Redo)       Ctrl+Y").clicked() {
                set_menu_command(ui, "redo");
                ui.close();
            }
            ui.separator();
            if ui.button("删除 (Delete)        Del").clicked() {
                set_menu_command(ui, "erase");
                ui.close();
            }
        });

        // 一级子菜单：视图
        ui.menu_button("视图 (View)", |ui| {
            ui.set_min_width(148.0);
            if ui.button("缩放至适应 (Zoom Extents)").clicked() {
                set_menu_command(ui, "zoom.extents");
                ui.close();
            }
            if ui.button("显示网格 (Grid)").clicked() {
                set_menu_command(ui, "grid");
                ui.close();
            }
        });

        ui.separator();

        // 一级子菜单：绘图（可继续嵌套二级子菜单）
        ui.menu_button("绘图 (Draw)", |ui| {
            ui.set_min_width(148.0);
            if ui.button("直线 (Line)").clicked() {
                set_menu_command(ui, "line");
                ui.close();
            }
            if ui.button("圆弧 (Arc)").clicked() {
                set_menu_command(ui, "arc");
                ui.close();
            }
            ui.menu_button("多边形 (Polygon)", |ui| {
                if ui.button("矩形 (Rectangle)").clicked() {
                    set_menu_command(ui, "rectang");
                    ui.close();
                }
                if ui.button("正多边形 (Regular)").clicked() {
                    set_menu_command(ui, "polygon");
                    ui.close();
                }
            });
        });
    });

    // 3. 在 MENU 按钮左侧绘制 Buerligons 的 18x18 三横线图标 (viewBox 0 0 20 20: M3.5 5.5h13 M3.5 10h13 M3.5 14.5h13)
    let btn_rect = menu_resp.response.rect;
    let icon_rect = Rect::from_min_size(
        pos2(btn_rect.left() + 8.0, btn_rect.center().y - 9.0),
        vec2(18.0, 18.0),
    );
    let icon_color = if menu_resp.response.hovered() || menu_resp.response.has_focus() {
        COLOR_INK // --rcad-ink
    } else {
        COLOR_MUTE // --rcad-mute
    };
    let stroke = Stroke::new(1.6, icon_color);
    let scale = 18.0 / 20.0; // 将 20x20 坐标映射到 18x18 像素
    for y_svg in [5.5, 10.0, 14.5] {
        let y = icon_rect.top() + y_svg * scale;
        let x0 = icon_rect.left() + 3.5 * scale;
        let x1 = icon_rect.left() + 16.5 * scale;
        ui.painter().line_segment([pos2(x0, y), pos2(x1, y)], stroke);
    }
}

/// 通用工具栏按钮：支持指定按钮外框大小 (如 32x32 或 28x28) 与内部图标大小 (如 20x20 或 18x18)
pub fn draw_tool_icon_button(
    ui: &mut Ui,
    btn_size: f32,
    icon_size: f32,
    active: bool,
    tooltip: &str,
    draw_icon: impl FnOnce(&egui::Painter, Rect, Color32),
) -> egui::Response {
    let (rect, resp) = ui.allocate_exact_size(vec2(btn_size, btn_size), Sense::click());
    let resp = resp.on_hover_text(tooltip);

    if ui.is_rect_visible(rect) {
        // 按钮背景与 7px 圆角
        let bg = if active || resp.is_pointer_button_down_on() {
            Color32::from_rgba_unmultiplied(15, 19, 32, 20) // --rcad-active
        } else if resp.hovered() {
            Color32::from_rgba_unmultiplied(15, 19, 32, 12) // --rcad-hover
        } else {
            Color32::TRANSPARENT
        };
        ui.painter().rect_filled(rect, CornerRadius::same(7), bg);

        // 居中计算图标绘制区域 (20x20 或 18x18)
        let icon_rect = Rect::from_center_size(rect.center(), vec2(icon_size, icon_size));
        let fg = if active || resp.hovered() {
            COLOR_INK // --rcad-ink
        } else {
            COLOR_INK_2 // --rcad-ink-2
        };
        draw_icon(ui.painter(), icon_rect, fg);

        // 激活态底部 2px 黑色指示横条 (.rcad-tool[aria-pressed="true"]::after)
        if active {
            let indicator = Rect::from_min_max(
                pos2(rect.left() + 8.0, rect.bottom() - 5.0),
                pos2(rect.right() - 8.0, rect.bottom() - 3.0),
            );
            ui.painter().rect_filled(indicator, CornerRadius::same(1), COLOR_INK);
        }
    }
    resp
}

/// 工具栏竖向分割线：宽 1px，高 18px，左右边距 7px (.rcad-tools-sep)
pub fn draw_toolbar_sep(ui: &mut Ui) {
    let (rect, _) = ui.allocate_exact_size(vec2(15.0, 32.0), Sense::hover());
    let x = rect.center().x;
    let yc = rect.center().y;
    ui.painter().line_segment(
        [pos2(x, yc - 9.0), pos2(x, yc + 9.0)],
        Stroke::new(1.0, COLOR_BORDER),
    );
}

// ----------------------------------------------------------------------------
// 矢量图标绘制函数 (映射 20x20 视口，保证精确线宽与视觉对齐)
// ----------------------------------------------------------------------------

/// 18x18 撤销图标 (线宽 1.61)
pub fn paint_undo(p: &egui::Painter, rect: Rect, color: Color32) {
    let s = rect.width() / 20.0;
    let at = |x: f32, y: f32| pos2(rect.min.x + x * s, rect.min.y + y * s);
    let st = Stroke::new(SUB_ICON_STROKE, color);
    let a0 = 140.0_f32.to_radians();
    let a1 = (-30.0_f32).to_radians();
    let pts: Vec<_> = (0..=12)
        .map(|i| {
            let a = a0 + (a1 - a0) * (i as f32 / 12.0);
            at(10.0 + 5.5 * a.cos(), 11.0 - 5.5 * a.sin())
        })
        .collect();
    p.line(pts, st);
    p.line_segment([at(4.5, 7.5), at(4.0, 10.5)], st);
    p.line_segment([at(4.5, 7.5), at(7.5, 8.5)], st);
}

/// 18x18 重做图标 (线宽 1.61)
pub fn paint_redo(p: &egui::Painter, rect: Rect, color: Color32) {
    let s = rect.width() / 20.0;
    let at = |x: f32, y: f32| pos2(rect.min.x + x * s, rect.min.y + y * s);
    let st = Stroke::new(SUB_ICON_STROKE, color);
    let a0 = 40.0_f32.to_radians();
    let a1 = 210.0_f32.to_radians();
    let pts: Vec<_> = (0..=12)
        .map(|i| {
            let a = a0 + (a1 - a0) * (i as f32 / 12.0);
            at(10.0 + 5.5 * a.cos(), 11.0 - 5.5 * a.sin())
        })
        .collect();
    p.line(pts, st);
    p.line_segment([at(15.5, 7.5), at(16.0, 10.5)], st);
    p.line_segment([at(15.5, 7.5), at(12.5, 8.5)], st);
}

/// 20x20 直线图标 (线宽 1.5)
pub fn paint_line(p: &egui::Painter, rect: Rect, color: Color32) {
    let s = rect.width() / 20.0;
    let at = |x: f32, y: f32| pos2(rect.min.x + x * s, rect.min.y + y * s);
    let st = Stroke::new(MAIN_ICON_STROKE, color);
    p.line_segment([at(4.0, 16.0), at(16.0, 4.0)], st);
    p.circle_filled(at(4.0, 16.0), 1.8 * s, color);
    p.circle_filled(at(16.0, 4.0), 1.8 * s, color);
}

/// 20x20 圆弧图标 (线宽 1.5)
pub fn paint_arc(p: &egui::Painter, rect: Rect, color: Color32) {
    let s = rect.width() / 20.0;
    let at = |x: f32, y: f32| pos2(rect.min.x + x * s, rect.min.y + y * s);
    let st = Stroke::new(MAIN_ICON_STROKE, color);
    let a0 = 200.0_f32.to_radians();
    let a1 = 340.0_f32.to_radians();
    let pts: Vec<_> = (0..=14)
        .map(|i| {
            let a = a0 + (a1 - a0) * (i as f32 / 14.0);
            at(10.0 + 7.5 * a.cos(), 7.0 - 7.5 * a.sin())
        })
        .collect();
    p.line(pts, st);
    p.circle_filled(at(10.0 + 7.5 * a0.cos(), 7.0 - 7.5 * a0.sin()), 1.6 * s, color);
    p.circle_filled(at(10.0 + 7.5 * a1.cos(), 7.0 - 7.5 * a1.sin()), 1.6 * s, color);
}

/// 20x20 矩形图标 (线宽 1.5)
pub fn paint_rect(p: &egui::Painter, rect: Rect, color: Color32) {
    let s = rect.width() / 20.0;
    let r = Rect::from_min_max(
        pos2(rect.min.x + 3.5 * s, rect.min.y + 4.5 * s),
        pos2(rect.min.x + 16.5 * s, rect.min.y + 15.5 * s),
    );
    p.rect_stroke(r, CornerRadius::same(1), Stroke::new(MAIN_ICON_STROKE, color), egui::StrokeKind::Inside);
}

/// 20x20 正多边形 (六边形) 图标 (线宽 1.5)
pub fn paint_polygon(p: &egui::Painter, rect: Rect, color: Color32) {
    let s = rect.width() / 20.0;
    let center = rect.center();
    let radius = 6.8 * s;
    let mut pts = Vec::with_capacity(7);
    for i in 0..6 {
        let angle = std::f32::consts::PI / 6.0 + i as f32 * std::f32::consts::PI / 3.0;
        pts.push(center + vec2(radius * angle.cos(), radius * angle.sin()));
    }
    pts.push(pts[0]);
    p.line(pts, Stroke::new(MAIN_ICON_STROKE, color));
}

/// 20x20 圆形图标 (线宽 1.5)
pub fn paint_circle(p: &egui::Painter, rect: Rect, color: Color32) {
    let s = rect.width() / 20.0;
    p.circle_stroke(rect.center(), 6.5 * s, Stroke::new(MAIN_ICON_STROKE, color));
}

// ----------------------------------------------------------------------------
// 顶层工具栏整体渲染 (48px 高，含 MENU 折叠菜单与各项工具)
// ----------------------------------------------------------------------------

pub fn draw_top_toolbar(ui: &mut Ui, app: &mut cadcraft_ui_egui::CadApp) {
    let mut clicked: Option<&'static str> = None;

    egui::Panel::top("rcad_top_bar")
        .exact_size(BAR_HEIGHT)
        .frame(
            egui::Frame::NONE
                .fill(Color32::WHITE)
                .stroke(Stroke::new(1.0, COLOR_BORDER)),
        )
        .show(ui, |ui| {
            let r = ui.max_rect();
            ui.scope_builder(
                egui::UiBuilder::new().max_rect(r.shrink2(vec2(8.0, 8.0))),
                |ui| {
                    egui::ScrollArea::horizontal()
                        .id_salt("rcad_toolbar_scroll")
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            ui.horizontal_centered(|ui| {
                                ui.spacing_mut().item_spacing.x = TOOL_SPACING;

                                // 1. [最左侧] 折叠式多级子菜单 (18x18 图标 + MENU 字样)
                                draw_collapsed_menu(ui);

                                // 分割线
                                draw_toolbar_sep(ui);

                                // 2. [引导区工具] 撤销 / 重做：按钮 32x32，图标 18x18，线宽 1.61
                                let undo_resp = draw_tool_icon_button(ui, TOOL_SIZE, SUB_ICON_SIZE, false, "撤销 Undo (Ctrl+Z)", |p, rect, color| {
                                    paint_undo(p, rect, color);
                                });
                                if undo_resp.clicked() {
                                    clicked = Some("u");
                                }

                                let redo_resp = draw_tool_icon_button(ui, TOOL_SIZE, SUB_ICON_SIZE, false, "重做 Redo (Ctrl+Y)", |p, rect, color| {
                                    paint_redo(p, rect, color);
                                });
                                if redo_resp.clicked() {
                                    clicked = Some("redo");
                                }

                                draw_toolbar_sep(ui);

                                // 3. [主建模/绘图工具区] 按钮 32x32，图标 20x20，线宽 1.5
                                let draw_tools: &[(&str, &str, fn(&egui::Painter, Rect, Color32))] = &[
                                    ("line", "直线 Line (L)", paint_line),
                                    ("arc", "圆弧 Arc (A)", paint_arc),
                                    ("circle", "圆 Circle (C)", paint_circle),
                                    ("rectang", "矩形 Rectangle (REC)", paint_rect),
                                    ("polygon", "正多边形 Polygon (POL)", paint_polygon),
                                ];

                                for (cmd, tip, draw_fn) in draw_tools {
                                    let active = false;
                                    let resp = draw_tool_icon_button(ui, TOOL_SIZE, MAIN_ICON_SIZE, active, tip, |p, rect, color| {
                                        draw_fn(p, rect, color);
                                    });
                                    if resp.clicked() {
                                        clicked = Some(cmd);
                                    }
                                }

                                draw_toolbar_sep(ui);

                                // 4. [视图与辅助工具区]
                                let view_tools: &[(&str, &str, cadcraft_ui_egui::icons::Icon)] = &[
                                    ("zoom.extents", "缩放至适应 Zoom Extents (Z E)", cadcraft_ui_egui::icons::Icon::ZoomExtents),
                                    ("pan", "平移 Pan (P)", cadcraft_ui_egui::icons::Icon::Pan),
                                    ("dist", "测量距离 Measure Distance (DI)", cadcraft_ui_egui::icons::Icon::Measure),
                                    ("ui.dialog.layers", "图层管理器 Layer Manager (LA)", cadcraft_ui_egui::icons::Icon::Layers),
                                    ("ui.toggle.palettes", "特性面板 Properties (PR)", cadcraft_ui_egui::icons::Icon::Properties),
                                ];

                                for (cmd, tip, icon) in view_tools {
                                    let resp = draw_tool_icon_button(ui, TOOL_SIZE, MAIN_ICON_SIZE, false, tip, |p, rect, _color| {
                                        cadcraft_ui_egui::icons::paint(p, rect, *icon, false);
                                    });
                                    if resp.clicked() {
                                        clicked = Some(cmd);
                                    }
                                }
                            });
                        });
                },
            );

            // 处理折叠菜单内部触发的命令
            if let Some(cmd) = take_menu_command(ui.ctx()) {
                app.start(&cmd);
            }
            // 处理工具栏图标按钮点击命令
            if let Some(c) = clicked {
                app.start(c);
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
    egui::Panel::top(id)
        .exact_size(BAR_HEIGHT)
        .frame(
            egui::Frame::NONE
                .fill(Color32::WHITE)
                .inner_margin(Margin::symmetric(8, 8))
                .stroke(Stroke::new(1.0, COLOR_BORDER)),
        )
}

/// 竖向绘图工具栏 (Dock) Panel (固定宽度 34px)
pub fn side_dock(id: &'static str) -> egui::Panel {
    egui::Panel::left(id)
        .exact_size(DOCK_WIDTH)
        .resizable(false)
        .frame(
            egui::Frame::NONE
                .fill(Color32::WHITE)
                .inner_margin(Margin::same(DOCK_PADDING as i8))
                .stroke(Stroke::new(1.0, COLOR_BORDER)),
        )
}

/// 左侧属性/特征面板 (Sidebar) Panel (默认宽度 288px，范围 264..=560px)
pub fn sidebar(id: &'static str) -> egui::Panel {
    egui::Panel::left(id)
        .default_size(SIDEBAR_DEFAULT_WIDTH)
        .min_size(SIDEBAR_MIN_WIDTH)
        .max_size(SIDEBAR_MAX_WIDTH)
}

