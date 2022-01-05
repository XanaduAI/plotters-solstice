use plotters_backend::{
    BackendColor, BackendCoord, BackendStyle, BackendTextStyle, DrawingBackend, DrawingErrorKind,
};
use solstice_2d::{Draw, Stroke};

pub struct SolsticeBackend {
    ctx: solstice_2d::solstice::Context,
    gfx: solstice_2d::Graphics,
    font_id: solstice_2d::FontId,
    draw_list: solstice_2d::DrawList<'static>,
}

impl SolsticeBackend {
    #[cfg(target_arch = "wasm32")]
    pub fn with_webgl1(
        webgl1: web_sys::WebGlRenderingContext,
        font_data: solstice_2d::text::FontVec,
        width: f32,
        height: f32,
    ) -> Result<Self, solstice_2d::GraphicsError> {
        let ctx = solstice_2d::solstice::glow::Context::from_webgl1_context(webgl1);
        let mut ctx = solstice_2d::solstice::Context::new(ctx);
        ctx.set_viewport(0, 0, width as _, height as _);
        let mut gfx = solstice_2d::Graphics::new(&mut ctx, width, height)?;
        let font_id = gfx.add_font(font_data);
        Ok(Self {
            ctx,
            gfx,
            font_id,
            draw_list: Default::default(),
        })
    }
}

fn color_into(color: BackendColor) -> solstice_2d::Color {
    let (r, g, b) = color.rgb;
    solstice_2d::Color {
        red: r as f32 / 255.,
        green: g as f32 / 255.,
        blue: b as f32 / 255.,
        alpha: color.alpha as _,
    }
}

impl DrawingBackend for SolsticeBackend {
    type ErrorType = solstice_2d::GraphicsError;

    fn get_size(&self) -> (u32, u32) {
        let vw = self.ctx.viewport();
        let (w, h) = vw.dimensions();
        (w as _, h as _)
    }

    fn ensure_prepared(&mut self) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        Ok(())
    }

    fn present(&mut self) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        self.gfx.process(&mut self.ctx, &self.draw_list);
        self.draw_list = solstice_2d::DrawList::default();
        log::trace!("Presented.");
        Ok(())
    }

    fn draw_pixel(
        &mut self,
        point: BackendCoord,
        color: BackendColor,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        let (x, y) = point;
        self.draw_list.draw_with_color(
            solstice_2d::Rectangle {
                x: x as _,
                y: y as _,
                width: 1.0,
                height: 1.0,
            },
            color_into(color),
        );
        Ok(())
    }

    fn draw_line<S: BackendStyle>(
        &mut self,
        (x1, y1): BackendCoord,
        (x2, y2): BackendCoord,
        style: &S,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        let width = style.stroke_width() as f32;
        let color = color_into(style.color()).into();
        self.draw_list.line_2d(vec![
            solstice_2d::LineVertex {
                position: [x1 as _, y1 as _, 0.],
                width,
                color,
            },
            solstice_2d::LineVertex {
                position: [x2 as _, y2 as _, 0.],
                width,
                color,
            },
        ]);
        Ok(())
    }

    fn draw_rect<S: BackendStyle>(
        &mut self,
        (x1, y1): BackendCoord,
        (x2, y2): BackendCoord,
        style: &S,
        fill: bool,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        let geometry = solstice_2d::Rectangle {
            x: x1 as f32,
            y: y1 as f32,
            width: x2 as f32 - x1 as f32,
            height: y2 as f32 - y1 as f32,
        };
        let color = color_into(style.color());
        self.draw_list.set_line_width(style.stroke_width() as _);
        match fill {
            true => self.draw_list.draw_with_color(geometry, color),
            false => self.draw_list.stroke_with_color(geometry, color),
        }
        Ok(())
    }

    fn draw_path<S: BackendStyle, I: IntoIterator<Item = BackendCoord>>(
        &mut self,
        path: I,
        style: &S,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        let width = style.stroke_width() as f32;
        let color = color_into(style.color()).into();
        self.draw_list.line_2d(
            path.into_iter()
                .map(|(x, y)| solstice_2d::LineVertex {
                    position: [x as f32, y as f32, 0.],
                    width,
                    color,
                })
                .collect::<Vec<_>>(),
        );
        Ok(())
    }

    fn draw_circle<S: BackendStyle>(
        &mut self,
        (x, y): BackendCoord,
        radius: u32,
        style: &S,
        fill: bool,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        let geometry = solstice_2d::Circle {
            x: x as f32,
            y: y as f32,
            radius: radius as f32,
            segments: radius.max(10),
        };
        let color = color_into(style.color());
        self.draw_list.set_line_width(style.stroke_width() as _);
        match fill {
            true => self.draw_list.draw_with_color(geometry, color),
            false => self.draw_list.stroke_with_color(geometry, color),
        }
        Ok(())
    }

    fn draw_text<TStyle: BackendTextStyle>(
        &mut self,
        text: &str,
        style: &TStyle,
        (x, y): BackendCoord,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        let scale = style.size() as f32;
        let bounds = solstice_2d::Rectangle {
            x: x as f32,
            y: y as f32,
            width: self.ctx.viewport().width() as f32,
            height: self.ctx.viewport().height() as f32,
        };
        let text = text.to_owned();
        self.draw_list.set_color(color_into(style.color()));
        self.draw_list.print(text, self.font_id, scale, bounds);
        self.draw_list.set_color([1., 1., 1., 1.]);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
