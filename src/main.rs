use pixels::{Color, PixelFormatEnum};
use sdl2::gfx::primitives::DrawRenderer;
use sdl2::pixels;
use sdl2::{event::Event, keyboard::Keycode, render::BlendMode, video::DisplayMode, EventPump};

use geo::{LineString, Polygon};

use lyon::{
    lyon_tessellation::{
        BuffersBuilder, FillAttributes, FillOptions, FillTessellator, VertexBuffers,
    },
    math::{point, Point},
    path::Builder,
};

fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let mut events = sdl_context.event_pump()?;
    let video_subsys = sdl_context.video()?;

    let mut window: sdl2::video::Window = video_subsys
        .window("Polygon drawer", 512, 512)
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;

    //let renderer = RendererBuilder::new(window);

    window.set_display_mode(DisplayMode::new(
        PixelFormatEnum::RGBA8888,
        512 as i32,
        512 as i32,
        60,
    ))?;

    let mut canvas = window
        .into_canvas()
        .accelerated()
        .present_vsync()
        .build()
        .map_err(|e| e.to_string())?;

    canvas.set_blend_mode(BlendMode::None);

    let outerring = LineString::from(vec![
        (0.8898926, 0.796875),
        (0.8876953, 0.79541016),
        (0.8845215, 0.80029297),
        (0.8857422, 0.8010254),
        (0.88427734, 0.80371094),
        (0.88500977, 0.8041992),
        (0.88378906, 0.8059082),
        (0.8847656, 0.8063965),
        (0.8898926, 0.796875),
    ]);

    let poly = Polygon::new(outerring.clone(), vec![]);
    let triangles = polygon(&poly);

    while !should_quit(&mut events) {
        canvas.set_draw_color(Color::BLACK);
        canvas.clear();

        let vx = outerring
            .0
            .iter()
            .map(|p| x_to_pixel(p.x))
            .collect::<Vec<_>>();
        let vy = outerring
            .0
            .iter()
            .map(|p| y_to_pixel(p.y))
            .collect::<Vec<_>>();
        canvas.filled_polygon(&vx, &vy, Color::RGB(255, 0, 0))?;

        triangles.indices.windows(3).for_each(|triangle| {
            let xs = Vec::from(triangle)
                .iter()
                .map(|index| triangles.vertices[*index as usize].0)
                .map(x_to_pixel)
                .collect::<Vec<_>>();

            let ys = Vec::from(triangle)
                .iter()
                .map(|index| triangles.vertices[*index as usize].1)
                .map(y_to_pixel)
                .collect::<Vec<_>>();

            canvas
                .filled_polygon(&xs, &ys, Color::RGB(0, 255, 0))
                .unwrap();
        });

        canvas.present();
    }

    Ok(())
}

const SCALE: f32 = 30000.0;
fn x_to_pixel(x: f32) -> i16 {
    ((x - 0.88) * SCALE) as i16
}

fn y_to_pixel(y: f32) -> i16 {
    ((y - 0.79) * SCALE) as i16
}

#[derive(Debug)]
pub struct TriangulatedGeometry {
    pub vertices: Vec<(f32, f32)>,
    pub indices: Vec<u32>,
}

pub fn polygon(poly: &Polygon<f32>) -> TriangulatedGeometry {
    let mut builder = Builder::with_capacity(1000, 2000);

    let mut add_ring = |ring: &LineString<f32>| {
        let c0 = ring.0[0];

        builder.move_to(point(c0.x, c0.y));
        ring.0.iter().skip(1).take(ring.0.len()).for_each(|c| {
            builder.line_to(point(c.x, c.y));
        });
        builder.close();
    };

    add_ring(poly.exterior());
    poly.interiors().iter().for_each(add_ring);

    let path = builder.build();

    // Will contain the result of the tessellation.
    let mut geometry: VertexBuffers<(f32, f32), u32> = VertexBuffers::new();

    let mut tessellator = FillTessellator::new();

    // Compute the tessellation.
    tessellator
        .tessellate(
            &path,
            &FillOptions::default(),
            &mut BuffersBuilder::new(&mut geometry, |pos: Point, _: FillAttributes| {
                (pos.x, pos.y)
            }),
        )
        .unwrap();

    TriangulatedGeometry {
        vertices: geometry.vertices,
        indices: geometry.indices,
    }
}

fn should_quit(events: &mut EventPump) -> bool {
    let maybe_event = events.poll_event();
    if let Some(event) = maybe_event {
        match event {
            Event::Quit { .. } => true,

            Event::KeyDown {
                keycode: Some(keycode),
                ..
            } => {
                if keycode == Keycode::Escape {
                    true
                } else {
                    false
                }
            }

            _ => false,
        }
    } else {
        false
    }
}
