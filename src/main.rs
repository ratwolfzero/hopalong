use image::{ImageFormat, RgbImage};
use minifb::{Key, Window, WindowOptions};
use rayon::prelude::*;
use std::env;
use std::path::Path;
use std::time::Instant;

const WIDTH: usize = 1000;
const HEIGHT: usize = 1000;
const SCALE_FACTOR: f64 = 0.95;
const OUTPUT_DIR: &str = "/Users/ralf/Projects/hopalong_pictures";

/// Independent orbit segments advanced in lock-step inside one task. The map is a serial
/// dependency chain (sqrt latency), so interleaving several segments lets the CPU overlap them.
const LANES: usize = 4;

/// Below this many points the parallel machinery isn't worth its set-up cost.
const PARALLEL_THRESHOLD: usize = 1_000_000;

/// Upper bound on the points plotted by one task, so that its `u32` pixel counters cannot
/// overflow. The per-task histograms are summed into `u64`, so `num` itself is not limited.
const MAX_POINTS_PER_TASK: usize = 1 << 31;

const NUMCOL: usize = COLOR_PALETTE.len();

const COLOR_PALETTE: [(u8, u8, u8, &str); 28] = [
    (0, 0, 0, "Black"),
    (28, 0, 0, "Dark Red"),
    (56, 0, 0, "Darker Red"),
    (85, 0, 0, "Deep Red"),
    (113, 0, 0, "Dark Red"),
    (141, 0, 0, "Red"),
    (170, 0, 0, "Red"),
    (198, 0, 0, "Red"),
    (226, 0, 0, "Bright Red"),
    (255, 0, 0, "Bright Red"),
    (255, 28, 0, "Orange-Red"),
    (255, 56, 0, "Orange"),
    (255, 85, 0, "Orange"),
    (255, 113, 0, "Orange"),
    (255, 141, 0, "Orange"),
    (255, 170, 0, "Yellow-Orange"),
    (255, 198, 0, "Yellow-Orange"),
    (255, 226, 0, "Bright Orange"),
    (255, 255, 0, "Yellow"),
    (255, 255, 28, "Light Yellow"),
    (255, 255, 56, "Pale Yellow"),
    (255, 255, 85, "Soft Yellow"),
    (255, 255, 113, "Yellowish White"),
    (255, 255, 141, "Yellowish White"),
    (255, 255, 170, "Light Yellowish White"),
    (255, 255, 198, "Almost White"),
    (255, 255, 226, "Almost White"),
    (255, 255, 255, "White"),
];

// ---------------------------------------------------------------------------------------------
// The map
// ---------------------------------------------------------------------------------------------

#[derive(Clone, Copy)]
struct Params {
    a: f64,
    b: f64,
    c: f64,
}

/// One Hopalong iteration.
#[inline(always)]
fn step(x: f64, y: f64, p: Params) -> (f64, f64) {
    (
        y - f64::signum(x) * f64::sqrt((p.b * x - p.c).abs()),
        p.a - x,
    )
}

// ---------------------------------------------------------------------------------------------
// Orbit plan: the orbit of `num` points is cut into `segments` consecutive pieces. Pass 1 walks
// the whole orbit once (serially), collecting the bounds and the state at the start of every
// piece. Pass 2 can then replay the pieces independently - and in parallel - and visits exactly
// the same points as a single sequential run would.
// ---------------------------------------------------------------------------------------------

struct Plan {
    segments: usize,
    seg: usize, // points per segment
    rem: usize, // leftover points, handled by the last segment
}

impl Plan {
    fn new(num: usize, tasks: usize) -> Self {
        let segments = tasks * LANES;
        Plan {
            segments,
            seg: num / segments,
            rem: num % segments,
        }
    }

    fn len(&self, k: usize) -> usize {
        self.seg + if k + 1 == self.segments { self.rem } else { 0 }
    }
}

struct Bounds {
    min_x: f64,
    max_x: f64,
    min_y: f64,
    max_y: f64,
}

/// Pass 1: bounds of the orbit plus the start state of every segment.
fn first_pass(plan: &Plan, p: Params) -> (Bounds, Vec<(f64, f64)>) {
    let (mut x, mut y) = (0.0f64, 0.0f64);
    let mut bounds = Bounds {
        min_x: 0.0,
        max_x: 0.0,
        min_y: 0.0,
        max_y: 0.0,
    };
    let mut checkpoints = Vec::with_capacity(plan.segments);

    for k in 0..plan.segments {
        checkpoints.push((x, y));
        for _ in 0..plan.len(k) {
            (x, y) = step(x, y, p);
            bounds.min_x = bounds.min_x.min(x);
            bounds.max_x = bounds.max_x.max(x);
            bounds.min_y = bounds.min_y.min(y);
            bounds.max_y = bounds.max_y.max(y);
        }
    }
    (bounds, checkpoints)
}

// ---------------------------------------------------------------------------------------------
// Continuous -> discrete grid mapping (y axis is flipped, as in the Julia/Python versions)
// ---------------------------------------------------------------------------------------------

struct Mapping {
    min_x: f64,
    max_y: f64,
    x_scale: f64,
    y_scale: f64,
    x_offset: f64,
    y_offset: f64,
}

impl Mapping {
    fn new(b: &Bounds) -> Self {
        let span = |lo: f64, hi: f64| {
            let s = hi - lo;
            if s.is_finite() && s > 0.0 { s } else { 1.0 }
        };
        let (sx, sy) = (span(b.min_x, b.max_x), span(b.min_y, b.max_y));
        let x_scale = (WIDTH as f64 / sx) * SCALE_FACTOR;
        let y_scale = (HEIGHT as f64 / sy) * SCALE_FACTOR;
        Mapping {
            min_x: b.min_x,
            max_y: b.max_y,
            x_scale,
            y_scale,
            x_offset: (WIDTH as f64 - sx * x_scale) / 2.0,
            y_offset: (HEIGHT as f64 - sy * y_scale) / 2.0,
        }
    }

    /// Index into the flat `WIDTH * HEIGHT` grid, or `None` if the point falls outside it.
    #[inline(always)]
    fn pixel_index(&self, x: f64, y: f64) -> Option<usize> {
        let px = ((x - self.min_x) * self.x_scale + self.x_offset).round();
        let py = ((self.max_y - y) * self.y_scale + self.y_offset).round();
        // Range-check in f64 *before* casting: `f64 as usize` saturates negatives and NaN to 0,
        // which would pile stray points into row/column 0 instead of discarding them.
        if px >= 0.0 && px < WIDTH as f64 && py >= 0.0 && py < HEIGHT as f64 {
            Some(py as usize * WIDTH + px as usize)
        } else {
            None
        }
    }
}

// ---------------------------------------------------------------------------------------------
// Pass 2: replay segments and count hits per pixel
// ---------------------------------------------------------------------------------------------

/// Advance `LANES` consecutive segments in lock-step, counting hits in `hist`.
/// `tail` is the number of extra points to append to the last lane (non-zero only for the task
/// that owns the very last segment of the orbit).
fn plot_lanes(
    start: [(f64, f64); LANES],
    seg: usize,
    tail: usize,
    p: Params,
    m: &Mapping,
    hist: &mut [u32],
) {
    let mut xs = start.map(|s| s.0);
    let mut ys = start.map(|s| s.1);

    for _ in 0..seg {
        for l in 0..LANES {
            let (nx, ny) = step(xs[l], ys[l], p);
            xs[l] = nx;
            ys[l] = ny;
            if let Some(i) = m.pixel_index(nx, ny) {
                hist[i] += 1;
            }
        }
    }

    let (mut x, mut y) = (xs[LANES - 1], ys[LANES - 1]);
    for _ in 0..tail {
        (x, y) = step(x, y, p);
        if let Some(i) = m.pixel_index(x, y) {
            hist[i] += 1;
        }
    }
}

/// Each rayon task fills its own (cache-friendly) `u32` histogram, which is then added into a
/// `u64` accumulator. `tasks` is chosen so that no task plots more than `MAX_POINTS_PER_TASK`
/// points, which keeps the `u32` counters safe for any `num`.
fn density_histogram(
    tasks: usize,
    plan: &Plan,
    checkpoints: &[(f64, f64)],
    p: Params,
    m: &Mapping,
) -> Vec<u64> {
    (0..tasks)
        .into_par_iter()
        .map(|t| {
            let start: [(f64, f64); LANES] =
                checkpoints[t * LANES..(t + 1) * LANES].try_into().unwrap();
            let tail = if t + 1 == tasks { plan.rem } else { 0 };
            let mut hist = vec![0u32; WIDTH * HEIGHT];
            plot_lanes(start, plan.seg, tail, p, m, &mut hist);
            hist
        })
        .fold(
            || vec![0u64; WIDTH * HEIGHT],
            |mut acc, h| {
                acc.iter_mut().zip(&h).for_each(|(a, &c)| *a += c as u64);
                acc
            },
        )
        .reduce_with(|mut a, b| {
            a.iter_mut().zip(&b).for_each(|(x, y)| *x += *y);
            a
        })
        .unwrap()
}

// ---------------------------------------------------------------------------------------------
// Colouring and output
// ---------------------------------------------------------------------------------------------

fn colorize(hist: &[u64], buffer: &mut [u32]) {
    let packed: [u32; NUMCOL] = std::array::from_fn(|i| {
        let (r, g, b, _name) = COLOR_PALETTE[i];
        0xFF000000 | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
    });
    let max_density = hist.iter().copied().max().unwrap_or(1).max(1);

    for (out, &d) in buffer.iter_mut().zip(hist) {
        // `.min(NUMCOL - 1)` instead of `% NUMCOL`: the densest pixel must map to the last
        // palette entry (white), not wrap around to index 0 (black).
        let idx = (((d as f64 / max_density as f64) * NUMCOL as f64) as usize).min(NUMCOL - 1);
        *out = packed[idx];
    }
}

fn save_png(buffer: &[u32], path: &Path) -> Result<(), image::ImageError> {
    let mut rgb = Vec::with_capacity(WIDTH * HEIGHT * 3);
    for &px in buffer {
        rgb.extend_from_slice(&[(px >> 16) as u8, (px >> 8) as u8, px as u8]);
    }
    let img = RgbImage::from_raw(WIDTH as u32, HEIGHT as u32, rgb)
        .expect("buffer length matches image dimensions");
    img.save_with_format(path, ImageFormat::Png)
}

// ---------------------------------------------------------------------------------------------
// main
// ---------------------------------------------------------------------------------------------

fn parse_f64(s: &str, name: &str) -> Result<f64, String> {
    s.parse::<f64>()
        .map_err(|_| format!("Invalid value for '{name}': {s}"))
}

fn parse_num(s: &str) -> Result<usize, String> {
    let n = if let Ok(n) = s.parse::<usize>() {
        n
    } else if let Ok(f) = s.parse::<f64>() {
        f as usize // accepts things like 1e7
    } else {
        return Err(format!("Invalid value for 'num': {s}"));
    };
    if n == 0 {
        return Err("'num' must be at least 1".into());
    }
    Ok(n)
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 5 {
        eprintln!("Usage: {} <a> <b> <c> <num>", args[0]);
        return;
    }

    let parsed = (|| -> Result<(Params, usize), String> {
        let p = Params {
            a: parse_f64(&args[1], "a")?,
            b: parse_f64(&args[2], "b")?,
            c: parse_f64(&args[3], "c")?,
        };
        Ok((p, parse_num(&args[4])?))
    })();
    let (p, num) = match parsed {
        Ok(v) => v,
        Err(e) => {
            eprintln!("{e}");
            return;
        }
    };
    let num_str = &args[4];

    // --- compute -------------------------------------------------------------------------
    let tasks = if num >= PARALLEL_THRESHOLD {
        (2 * rayon::current_num_threads()).max(num.div_ceil(MAX_POINTS_PER_TASK))
    } else {
        1
    };
    let plan = Plan::new(num, tasks);

    let t0 = Instant::now();
    let (bounds, checkpoints) = first_pass(&plan, p);
    let mapping = Mapping::new(&bounds);
    let t1 = Instant::now();
    let hist = density_histogram(tasks, &plan, &checkpoints, p, &mapping);
    let t2 = Instant::now();

    let mut buffer = vec![0u32; WIDTH * HEIGHT];
    colorize(&hist, &mut buffer);
    println!(
        "pass 1 (bounds): {:.1?}, pass 2 (plot): {:.1?}, colour: {:.1?}",
        t1 - t0,
        t2 - t1,
        t2.elapsed()
    );

    // --- save ----------------------------------------------------------------------------
    let image_name = format!("num={}_a={}_b={}_c={}.png", num_str, p.a, p.b, p.c);
    if let Err(e) = std::fs::create_dir_all(OUTPUT_DIR) {
        eprintln!("Could not create {OUTPUT_DIR}: {e}");
    }
    let save_path = Path::new(OUTPUT_DIR).join(image_name);
    match save_png(&buffer, &save_path) {
        Ok(()) => println!("Image saved to: {}", save_path.display()),
        Err(e) => eprintln!("Error saving image: {e}"),
    }

    // --- display (optional) --------------------------------------------------------------
    let title = format!(
        "Hopalong - num  = {}  , a = {} , b = {} , c = {}",
        num_str, p.a, p.b, p.c
    );
    match Window::new(
        &title,
        WIDTH,
        HEIGHT,
        WindowOptions {
            resize: false,
            ..WindowOptions::default()
        },
    ) {
        Ok(mut window) => {
            // The picture never changes, so don't spin a core at 100 % redrawing it.
            window.set_target_fps(60);
            while window.is_open() && !window.is_key_down(Key::Escape) {
                window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
            }
        }
        Err(e) => eprintln!("Could not open a window ({e}); the image was still saved."),
    }
}
