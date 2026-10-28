//! Pure vector drawings for CADFlow document / Markdown toolbar icons.
//!
//! Drawn on a 24x24 design grid imitating the Material/StackEdit SVG path specifications,
//! using native egui Painter geometry without any external SVG rendering dependencies.

use egui::{Color32, CornerRadius, Painter, Pos2, Rect, Stroke, pos2, vec2};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DocIcon {
    Bold,
    Italic,
    Heading,
    Strikethrough,
    UnorderedList,
    OrderedList,
    CheckList,
    Blockquote,
    Code,
    Table,
    Link,
    Image,
    Play,
    Pause,
    Stop,
    Settings,
}

struct DocPen<'a> {
    p: &'a Painter,
    r: Rect,
    w: f32,
    fg: Color32,
}

impl<'a> DocPen<'a> {
    fn new(p: &'a Painter, r: Rect, fg: Color32) -> Self {
        let s = r.width() / 24.0;
        let w = (s * 1.8).clamp(1.2, 2.2);
        Self { p, r, w, fg }
    }

    fn at(&self, x: f32, y: f32) -> Pos2 {
        let s = self.r.width() / 24.0;
        pos2(self.r.min.x + x * s, self.r.min.y + y * s)
    }

    fn sc(&self) -> f32 {
        self.r.width() / 24.0
    }

    fn st(&self, w_scale: f32) -> Stroke {
        Stroke::new(self.w * w_scale, self.fg)
    }

    fn l(&self, x1: f32, y1: f32, x2: f32, y2: f32) {
        self.p.line_segment([self.at(x1, y1), self.at(x2, y2)], self.st(1.0));
    }

    fn l_w(&self, x1: f32, y1: f32, x2: f32, y2: f32, w_scale: f32) {
        self.p.line_segment([self.at(x1, y1), self.at(x2, y2)], self.st(w_scale));
    }

    fn poly(&self, pts: &[(f32, f32)], closed: bool) {
        let mut v: Vec<Pos2> = pts.iter().map(|(x, y)| self.at(*x, *y)).collect();
        if closed && let Some(f) = v.first().copied() {
            v.push(f);
        }
        self.p.line(v, self.st(1.0));
    }

    fn poly_w(&self, pts: &[(f32, f32)], closed: bool, w_scale: f32) {
        let mut v: Vec<Pos2> = pts.iter().map(|(x, y)| self.at(*x, *y)).collect();
        if closed && let Some(f) = v.first().copied() {
            v.push(f);
        }
        self.p.line(v, self.st(w_scale));
    }

    fn fill_c(&self, x: f32, y: f32, r: f32) {
        self.p.circle_filled(self.at(x, y), r * self.sc(), self.fg);
    }

    fn arc_w(&self, x: f32, y: f32, r: f32, a0: f32, a1: f32, w_scale: f32) {
        let n = (((a1 - a0).abs() / 15.0).ceil() as usize).max(3);
        let pts: Vec<Pos2> = (0..=n)
            .map(|i| {
                let a = (a0 + (a1 - a0) * i as f32 / n as f32).to_radians();
                self.at(x + r * a.cos(), y - r * a.sin())
            })
            .collect();
        self.p.line(pts, self.st(w_scale));
    }

    fn rect_stroke(&self, x: f32, y: f32, w: f32, h: f32, cr: f32) {
        let r = Rect::from_min_max(self.at(x, y), self.at(x + w, y + h));
        self.p.rect_stroke(
            r,
            CornerRadius::same((cr * self.sc()).round() as u8),
            self.st(1.0),
            egui::StrokeKind::Inside,
        );
    }

    fn fill_rect(&self, x: f32, y: f32, w: f32, h: f32) {
        let r = Rect::from_min_max(self.at(x, y), self.at(x + w, y + h));
        self.p.rect_filled(r, CornerRadius::ZERO, self.fg);
    }

    fn fill_rounded_rect(&self, x: f32, y: f32, w: f32, h: f32, rx: f32) {
        let r = Rect::from_min_size(self.at(x, y), vec2(w * self.sc(), h * self.sc()));
        let cr = CornerRadius::same((rx * self.sc()).round().max(1.0) as u8);
        self.p.rect_filled(r, cr, self.fg);
    }

    fn tri(&self, pts: [(f32, f32); 3]) {
        self.p.add(egui::Shape::convex_polygon(
            pts.iter().map(|(x, y)| self.at(*x, *y)).collect(),
            self.fg,
            Stroke::NONE,
        ));
    }
}

/// Paint the requested markdown document icon into `rect`.
pub fn paint_doc_icon(p: &Painter, rect: Rect, icon: DocIcon, fg: Color32) {
    let pen = DocPen::new(p, rect, fg);
    match icon {
        DocIcon::Bold => {
            // 粗体：左侧垂直主干 + 上下双环
            pen.fill_rect(6.0, 4.0, 3.2, 16.0);
            // 上环
            pen.l_w(8.0, 5.2, 13.0, 5.2, 1.2);
            pen.l_w(8.0, 11.2, 13.0, 11.2, 1.2);
            pen.arc_w(13.0, 8.2, 3.0, 90.0, -90.0, 1.2);
            // 下环
            pen.l_w(8.0, 11.2, 14.0, 11.2, 1.2);
            pen.l_w(8.0, 18.4, 14.0, 18.4, 1.2);
            pen.arc_w(14.0, 14.8, 3.6, 90.0, -90.0, 1.2);
        }
        DocIcon::Italic => {
            // 斜体：上下衬线横线 + 倾斜主干
            pen.l(9.5, 5.0, 17.5, 5.0);
            pen.l(6.5, 19.0, 14.5, 19.0);
            pen.l(14.5, 5.0, 9.5, 19.0);
        }
        DocIcon::Heading => {
            // 标题：字母 'H' + 标号 '1'
            pen.l(4.5, 5.0, 4.5, 19.0);
            pen.l(12.0, 5.0, 12.0, 19.0);
            pen.l(4.5, 12.0, 12.0, 12.0);
            pen.l(17.5, 5.0, 17.5, 19.0);
            pen.l(14.5, 8.0, 17.5, 5.0);
            pen.l(15.5, 19.0, 19.5, 19.0);
        }
        DocIcon::Strikethrough => {
            // 删除线：水平贯穿线 + S 字母上下轮廓
            pen.l_w(3.0, 12.0, 21.0, 12.0, 1.2);
            pen.poly(
                &[
                    (15.5, 7.0),
                    (14.0, 5.5),
                    (10.0, 5.5),
                    (8.5, 7.2),
                    (9.0, 9.5),
                    (13.0, 10.8),
                ],
                false,
            );
            pen.poly(
                &[
                    (11.0, 13.2),
                    (15.0, 14.5),
                    (15.5, 16.8),
                    (14.0, 18.5),
                    (10.0, 18.5),
                    (8.5, 17.0),
                ],
                false,
            );
        }
        DocIcon::UnorderedList => {
            // 无序列表：三行圆点 + 条目线
            pen.fill_c(4.5, 6.0, 1.6);
            pen.fill_c(4.5, 12.0, 1.6);
            pen.fill_c(4.5, 18.0, 1.6);
            pen.l(8.5, 6.0, 20.5, 6.0);
            pen.l(8.5, 12.0, 20.5, 12.0);
            pen.l(8.5, 18.0, 20.5, 18.0);
        }
        DocIcon::OrderedList => {
            // 有序列表：三行数字 "1","2","3" + 条目线
            pen.l(9.0, 6.0, 20.5, 6.0);
            pen.l(9.0, 12.0, 20.5, 12.0);
            pen.l(9.0, 18.0, 20.5, 18.0);
            // 序号 "1"
            pen.poly_w(&[(2.5, 5.2), (4.0, 4.0), (4.0, 8.0)], false, 0.85);
            pen.l_w(2.5, 8.0, 5.5, 8.0, 0.85);
            // 序号 "2"
            pen.poly_w(&[(2.5, 10.8), (5.5, 10.8), (5.5, 12.2), (2.5, 14.0), (5.8, 14.0)], false, 0.85);
            // 序号 "3"
            pen.poly_w(&[(2.5, 16.0), (5.5, 16.0), (3.8, 17.8), (5.5, 18.2), (5.5, 20.0), (2.5, 20.0)], false, 0.85);
        }
        DocIcon::CheckList => {
            // 任务列表：方框复选框 + 对勾
            pen.rect_stroke(3.5, 3.5, 17.0, 17.0, 2.5);
            pen.poly_w(&[(6.5, 12.5), (10.0, 16.0), (17.5, 8.0)], false, 1.25);
        }
        DocIcon::Blockquote => {
            // 引用：双引号几何体
            pen.fill_rect(5.0, 7.0, 6.0, 6.0);
            pen.tri([(8.0, 13.0), (11.0, 13.0), (6.0, 17.0)]);
            pen.tri([(11.0, 13.0), (9.0, 17.0), (6.0, 17.0)]);
            pen.fill_rect(13.0, 7.0, 6.0, 6.0);
            pen.tri([(16.0, 13.0), (19.0, 13.0), (14.0, 17.0)]);
            pen.tri([(19.0, 13.0), (17.0, 17.0), (14.0, 17.0)]);
        }
        DocIcon::Code => {
            // 代码块：左右尖括号 < >
            pen.poly_w(&[(9.0, 6.0), (3.5, 12.0), (9.0, 18.0)], false, 1.15);
            pen.poly_w(&[(15.0, 6.0), (20.5, 12.0), (15.0, 18.0)], false, 1.15);
        }
        DocIcon::Table => {
            // 表格：2x2 网格与表头
            pen.rect_stroke(3.5, 4.0, 17.0, 16.0, 2.0);
            pen.l(3.5, 9.0, 20.5, 9.0);
            pen.l(3.5, 14.5, 20.5, 14.5);
            pen.l(12.0, 4.0, 12.0, 20.0);
        }
        DocIcon::Link => {
            // 链接：双环链条结构
            pen.arc_w(7.0, 12.0, 3.5, 90.0, 270.0, 1.15);
            pen.l_w(7.0, 8.5, 10.5, 8.5, 1.15);
            pen.l_w(7.0, 15.5, 10.5, 15.5, 1.15);
            pen.arc_w(17.0, 12.0, 3.5, -90.0, 90.0, 1.15);
            pen.l_w(13.5, 8.5, 17.0, 8.5, 1.15);
            pen.l_w(13.5, 15.5, 17.0, 15.5, 1.15);
            pen.l_w(8.5, 12.0, 15.5, 12.0, 1.15);
        }
        DocIcon::Image => {
            // 图片：相框 + 连绵山峦 + 太阳
            pen.rect_stroke(3.5, 3.5, 17.0, 17.0, 2.0);
            pen.poly_w(&[(5.0, 18.0), (8.5, 13.5), (11.0, 16.5), (14.5, 12.0), (19.0, 18.0)], false, 1.15);
            pen.fill_c(8.5, 8.0, 1.5);
        }
        DocIcon::Play => {
            // 播放图标：M8 5.5v13L18 12 8 5.5Z
            pen.tri([(8.0, 5.5), (18.0, 12.0), (8.0, 18.5)]);
        }
        DocIcon::Pause => {
            // 暂停图标：双圆角竖条
            pen.fill_rounded_rect(6.25, 5.0, 4.5, 14.0, 1.0);
            pen.fill_rounded_rect(13.25, 5.0, 4.5, 14.0, 1.0);
        }
        DocIcon::Stop => {
            // 停止图标：圆角矩形
            pen.fill_rounded_rect(7.0, 7.0, 10.0, 10.0, 1.6);
        }
        DocIcon::Settings => {
            // 设置图标：6 齿对称齿轮与中心圆孔
            let st = Stroke::new((1.85 * pen.sc()).clamp(1.0, 2.2), pen.fg);
            let mut pts = Vec::with_capacity(36);
            for k in 0..6 {
                let base_ang = (k as f32 * 60.0 - 90.0).to_radians();
                let a1 = base_ang - 12.0_f32.to_radians();
                let a2 = base_ang + 12.0_f32.to_radians();
                let a3 = base_ang + 19.0_f32.to_radians();
                let a4 = base_ang + 41.0_f32.to_radians();

                pts.push(pen.at(12.0 + 8.55 * a1.cos(), 12.0 + 8.55 * a1.sin()));
                pts.push(pen.at(12.0 + 8.55 * a2.cos(), 12.0 + 8.55 * a2.sin()));
                pts.push(pen.at(12.0 + 6.75 * a3.cos(), 12.0 + 6.75 * a3.sin()));
                let mid_val = base_ang + 30.0_f32.to_radians();
                pts.push(pen.at(12.0 + 6.55 * mid_val.cos(), 12.0 + 6.55 * mid_val.sin()));
                pts.push(pen.at(12.0 + 6.75 * a4.cos(), 12.0 + 6.75 * a4.sin()));
            }
            if let Some(first) = pts.first().copied() {
                pts.push(first);
            }
            pen.p.line(pts, st);
            pen.p.circle_stroke(pen.at(12.0, 12.0), 2.85 * pen.sc(), st);
        }
    }
}

