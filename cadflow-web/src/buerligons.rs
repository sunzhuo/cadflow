//! Buerligons UI specifications for CADFlow:
//! - Top toolbar height: 48px, tool button: 32x32px, corner radius: 7px, spacing: 2px
//! - "CADFlow" menu button (18x18 hamburger icon + CADFlow text + full CADCraft menu tree translated to Chinese)
//! - Sidebar toggle button aligned with sidebar boundary (boundary extends continuously to toolbar)
//! - Dragging sidebar boundary moves the sidebar button within min/max bounds
//! - Removed: New, Open, Save, SaveAs, Plot, Publish, PageSetup, Preview, Import, Export,
//!   Attach, Share, Orbit, Properties, Layers, Blocks, List, Area
//! - Kept CAD tools: Zoom Window, Pan, Zoom Extents, Measure Distance, Help
//! - Document toggle button aligned with document panel boundary, moving with doc boundary
//! - Sliding effect when toggling document panel and markdown edit buttons
//! - StackEdit-styled markdown edit buttons group to the right of document button

use cadcraft_ui_egui::CadApp;
use cadcraft_ui_egui::icons::{self, Icon};
use cadcraft_ui_egui::menus::{self, Entry};
use cadcraft_ui_egui::theme::Tokens;
use egui::{pos2, vec2, Color32, CornerRadius, Margin, Rect, Sense, Stroke, Ui};

// ----------------------------------------------------------------------------
// 核心尺寸参数
// ----------------------------------------------------------------------------

pub const BAR_HEIGHT: f32 = 48.0;
pub const TOOL_SIZE: f32 = 32.0;
pub const TOOL_CORNER_RADIUS: u8 = 7;
pub const TOOL_SPACING: f32 = 2.0;

pub const SIDEBAR_DEFAULT_WIDTH: f32 = 288.0;
pub const SIDEBAR_MIN_WIDTH: f32 = 220.0;
pub const SIDEBAR_MAX_WIDTH: f32 = 560.0;

pub const DOC_DEFAULT_WIDTH: f32 = 460.0;
pub const DOC_MIN_WIDTH: f32 = 280.0;
pub const DOC_MAX_WIDTH: f32 = 800.0;

pub const MAIN_ICON_SIZE: f32 = 20.0;
pub const SUB_ICON_SIZE: f32 = 18.0;

// ----------------------------------------------------------------------------
// CAD 工具栏保留的图标与命令组（依需求移除不需要的按钮，保留视图、测量与帮助）
// ----------------------------------------------------------------------------

pub type CadTool = (Icon, &'static str, &'static str, Option<&'static str>);

pub const CAD_TOOL_GROUPS: &[&[CadTool]] = &[
    // Group 1: 视图导航与平移 (已去除三维动态观察 3dorbit)
    &[
        (Icon::ZoomWindow, "zoom.window", "窗口缩放", None),
        (Icon::Pan, "pan", "平移", None),
    ],
    // Group 2: 视图管理 (已去除特性、图层特性管理器、块)
    &[
        (Icon::ZoomExtents, "zoom.extents", "范围缩放", None),
    ],
    // Group 3: 测量与查询 (已去除列表、面积)
    &[
        (Icon::Measure, "dist", "测量距离", None),
    ],
    // Group 4: 帮助
    &[
        (Icon::Help, "ui.dialog.about", "帮助", None),
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

/// 绘制 Buerligons 风格的折叠菜单按钮（18x18 汉堡图标 + CADFlow 字样 + 包含全部菜单项与子项的中文下拉菜单）
pub fn draw_collapsed_menu(ui: &mut Ui, app: &mut CadApp) {
    let t = Tokens::get();

    let mut style = (**ui.style()).clone();
    style.spacing.menu_margin = Margin::same(4);
    style.spacing.menu_width = 148.0;
    style.spacing.interact_size = vec2(64.0, 28.0);
    style.spacing.button_padding = vec2(9.0, 5.0);
    style.spacing.item_spacing = vec2(4.0, 2.0);

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

    // 设置顶层 CADFlow 按钮尺寸为 32px 高度与 7px 圆角，为左侧 18x18 图标预留空间
    ui.spacing_mut().interact_size = vec2(96.0, TOOL_SIZE);
    ui.spacing_mut().button_padding = vec2(10.0, 6.0);

    let mut clicked = None;
    let tree = menus::tree(app);

    // 将原 MENU 改为 CADFlow
    let menu_resp = ui.menu_button("      CADFlow", |ui| {
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

    let btn_rect = menu_resp.response.rect;
    let icon_rect = Rect::from_min_size(
        pos2(btn_rect.left() + 7.0, btn_rect.center().y - 9.0),
        vec2(SUB_ICON_SIZE, SUB_ICON_SIZE),
    );
    icons::paint(ui.painter(), icon_rect, Icon::Menu, false);

    if let Some(id) = clicked {
        menus::activate(app, &id);
    }
}

// ----------------------------------------------------------------------------
// Buerligons 规范的侧边栏与文档切换按钮图标绘制
// ----------------------------------------------------------------------------

/// 绘制 Buerligons 风格侧边栏开关图标（外框矩形 + 左侧填满/空心）
pub fn paint_sidebar_toggle(p: &egui::Painter, rect: Rect, open: bool, fg: Color32) {
    let box_rect = Rect::from_center_size(rect.center(), vec2(15.0, 13.0));
    let cr = CornerRadius::same(2);
    p.rect_stroke(box_rect, cr, Stroke::new(1.4, fg), egui::StrokeKind::Inside);
    let split_x = box_rect.left() + 5.0;
    p.line_segment([pos2(split_x, box_rect.top()), pos2(split_x, box_rect.bottom())], Stroke::new(1.4, fg));
    if open {
        let fill_rect = Rect::from_min_max(box_rect.min, pos2(split_x, box_rect.bottom()));
        p.rect_filled(fill_rect, CornerRadius { nw: 2, sw: 2, ne: 0, se: 0 }, fg);
    }
}

/// 绘制 Buerligons 风格文档面板开关图标（外框矩形 + 右侧填满/空心）
pub fn paint_doc_toggle(p: &egui::Painter, rect: Rect, open: bool, fg: Color32) {
    let box_rect = Rect::from_center_size(rect.center(), vec2(15.0, 13.0));
    let cr = CornerRadius::same(2);
    p.rect_stroke(box_rect, cr, Stroke::new(1.4, fg), egui::StrokeKind::Inside);
    let split_x = box_rect.right() - 5.0;
    p.line_segment([pos2(split_x, box_rect.top()), pos2(split_x, box_rect.bottom())], Stroke::new(1.4, fg));
    if open {
        let fill_rect = Rect::from_min_max(pos2(split_x, box_rect.top()), box_rect.max);
        p.rect_filled(fill_rect, CornerRadius { nw: 0, sw: 0, ne: 2, se: 2 }, fg);
    }
}

/// 通用工具栏图标按钮
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

/// StackEdit 风格的文档小工具按钮
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

/// 工具栏竖向分割线：宽 1px，高 18px
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

            // 动画平滑进度：0.0 (收起) -> 1.0 (完全展开)
            let slide_progress = ui.ctx().animate_bool(egui::Id::new("cadflow_doc_slide"), *doc_open);

            // ================================================================
            // 1. 左侧区域 (Leading Section)：
            //    包含 CADFlow 菜单、撤销、恢复，并在最右侧放置“侧边栏”按钮
            //    边界与侧边栏对齐，并竖向贯穿至工具栏顶部
            // ================================================================
            let lead_width = if *sidebar_open {
                (*sidebar_width).clamp(SIDEBAR_MIN_WIDTH, SIDEBAR_MAX_WIDTH)
            } else {
                214.0 // 闭合时的紧凑宽度
            };

            let lead_rect = Rect::from_min_max(
                total_rect.min,
                pos2(total_rect.left() + lead_width, total_rect.bottom()),
            );

            // 渲染左侧区域内容
            ui.scope_builder(
                egui::UiBuilder::new().max_rect(lead_rect.shrink2(vec2(8.0, 8.0))),
                |ui| {
                    ui.horizontal_centered(|ui| {
                        ui.spacing_mut().item_spacing.x = TOOL_SPACING;

                        // [1] CADFlow 多级菜单
                        draw_collapsed_menu(ui, app);

                        // [2] 分割线
                        draw_toolbar_sep(ui);

                        // [3] 撤销 (Undo)
                        let undo_tip = format!("{} ({})", "撤销", format_shortcut("Cmd+Z", mac));
                        let undo_resp = draw_tool_icon_button(ui, TOOL_SIZE, SUB_ICON_SIZE, false, &undo_tip, |p, r| {
                            icons::paint(p, r, Icon::Undo, false);
                        });
                        if undo_resp.clicked() {
                            clicked_cad_cmd = Some("u");
                        }

                        // [4] 恢复 (Redo)
                        let redo_tip = format!("{} ({})", "恢复", format_shortcut("Cmd+Shift+Z", mac));
                        let redo_resp = draw_tool_icon_button(ui, TOOL_SIZE, SUB_ICON_SIZE, false, &redo_tip, |p, r| {
                            icons::paint(p, r, Icon::Redo, false);
                        });
                        if redo_resp.clicked() {
                            clicked_cad_cmd = Some("redo");
                        }

                        // [5] 侧边栏按钮：紧贴在左侧区最右边界
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let sb_tip = if *sidebar_open { "隐藏侧边栏" } else { "显示侧边栏" };
                            let is_open = *sidebar_open;
                            let sb_resp = draw_tool_icon_button(ui, TOOL_SIZE, SUB_ICON_SIZE, false, sb_tip, |p, r| {
                                paint_sidebar_toggle(p, r, is_open, t.text);
                            });
                            if sb_resp.clicked() {
                                *sidebar_open = !*sidebar_open;
                            }
                        });
                    });
                },
            );

            // 若侧边栏打开，在 lead_rect 右边缘绘制贯穿工具栏的边界线 (边界一直显示到工具栏)
            if *sidebar_open {
                ui.painter().vline(lead_rect.right(), total_rect.y_range(), Stroke::new(1.0, t.border));
            }

            // ================================================================
            // 2. 右侧区域 (Trailing Section)：
            //    包含“文档”按钮与 StackEdit 风格的文档编辑按钮组
            //    边界与文档面板对齐，滑动动画效果展示/隐藏
            // ================================================================
            let full_doc_w = (*doc_width).clamp(DOC_MIN_WIDTH, DOC_MAX_WIDTH);
            let effective_doc_w = full_doc_w * slide_progress;

            let doc_area_rect = if slide_progress > 0.001 {
                Rect::from_min_max(
                    pos2((total_rect.right() - effective_doc_w).max(lead_rect.right() + 150.0), total_rect.top()),
                    total_rect.max,
                )
            } else {
                // 折叠时只在最右侧保留“文档”按钮位置
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

                        // [1] “文档”按钮 (在编辑按钮组的左侧显示，边界即文档面板边界)
                        let doc_tip = if *doc_open { "隐藏文档面板" } else { "显示文档面板" };
                        let is_doc_open = *doc_open;
                        let doc_resp = draw_tool_icon_button(ui, TOOL_SIZE, SUB_ICON_SIZE, false, doc_tip, |p, r| {
                            paint_doc_toggle(p, r, is_doc_open, t.text);
                        });
                        if doc_resp.clicked() {
                            *doc_open = !*doc_open;
                        }

                        // [2] 展开状态下且滑动进度大于 0.5 时渲染 StackEdit Markdown 编辑工具组
                        if slide_progress > 0.5 {
                            draw_toolbar_sep(ui);

                            if draw_stackedit_btn(ui, "↶", "撤销编辑").clicked() {
                                doc_action = Some(crate::doc_panel::DocAction::Undo);
                            }
                            if draw_stackedit_btn(ui, "↷", "重做编辑").clicked() {
                                doc_action = Some(crate::doc_panel::DocAction::Redo);
                            }

                            draw_toolbar_sep(ui);

                            if draw_stackedit_btn(ui, "B", "粗体 (Ctrl+Shift+B)").clicked() {
                                doc_action = Some(crate::doc_panel::DocAction::Bold);
                            }
                            if draw_stackedit_btn(ui, "I", "斜体 (Ctrl+Shift+I)").clicked() {
                                doc_action = Some(crate::doc_panel::DocAction::Italic);
                            }
                            if draw_stackedit_btn(ui, "H", "标题 (Ctrl+Shift+H)").clicked() {
                                doc_action = Some(crate::doc_panel::DocAction::Heading);
                            }
                            if draw_stackedit_btn(ui, "~~", "删除线 (Ctrl+Shift+S)").clicked() {
                                doc_action = Some(crate::doc_panel::DocAction::Strikethrough);
                            }

                            draw_toolbar_sep(ui);

                            if draw_stackedit_btn(ui, "•—", "无序列表 (Ctrl+Shift+U)").clicked() {
                                doc_action = Some(crate::doc_panel::DocAction::UnorderedList);
                            }
                            if draw_stackedit_btn(ui, "1.", "有序列表 (Ctrl+Shift+O)").clicked() {
                                doc_action = Some(crate::doc_panel::DocAction::OrderedList);
                            }
                            if draw_stackedit_btn(ui, "☑", "待办清单 (Ctrl+Shift+C)").clicked() {
                                doc_action = Some(crate::doc_panel::DocAction::CheckList);
                            }

                            draw_toolbar_sep(ui);

                            if draw_stackedit_btn(ui, "”", "引用块 (Ctrl+Shift+Q)").clicked() {
                                doc_action = Some(crate::doc_panel::DocAction::Quote);
                            }
                            if draw_stackedit_btn(ui, "< >", "代码块 (Ctrl+Shift+K)").clicked() {
                                doc_action = Some(crate::doc_panel::DocAction::Code);
                            }
                            if draw_stackedit_btn(ui, "⊞", "表格 (Ctrl+Shift+T)").clicked() {
                                doc_action = Some(crate::doc_panel::DocAction::Table);
                            }
                            if draw_stackedit_btn(ui, "🔗", "链接 (Ctrl+Shift+L)").clicked() {
                                doc_action = Some(crate::doc_panel::DocAction::Link);
                            }
                            if draw_stackedit_btn(ui, "🖼", "图片 (Ctrl+Shift+G)").clicked() {
                                doc_action = Some(crate::doc_panel::DocAction::Image);
                            }
                        }
                    });
                },
            );

            // ================================================================
            // 3. 中间区域 (Center Section)：
            //    仅保留必需的 CAD 工具（缩放、平移、测量、帮助），占满剩余空间
            // ================================================================
            let center_rect = Rect::from_min_max(
                pos2(lead_rect.right() + 4.0, total_rect.top()),
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

                                    for group in CAD_TOOL_GROUPS {
                                        draw_toolbar_sep(ui);

                                        for (icon, cmd, label, shortcut) in group.iter() {
                                            let tip = match shortcut {
                                                Some(sc) => format!("{} ({})", label, format_shortcut(sc, mac)),
                                                None => (*label).to_string(),
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
                app.start(cmd);
            }

            // 处理 Markdown 编辑操作
            if let Some(act) = doc_action {
                doc_state.apply_action(act);
            }
        });
}
