use image::{self, ImageBuffer, Pixel, Rgb};
use rand::{thread_rng, Rng};
use core::panic;
use std::{cmp, fs::DirBuilder};
use palette::{FromColor, ColorDifference, Srgb, Lab};
use std::{thread, fs};
use std::sync::{Arc, Mutex};
use std::time::Instant;

#[derive(Copy, Clone, Debug)]
struct SlopeLine {
    slope: f32,
    origin: i32,
    color: [u8; 3],
    pub score: f64,
    pub count: i32,
    pub xmin: u32,
    pub xmax: u32
}

impl SlopeLine {
    fn new(x: u32, y: u32) -> SlopeLine {
        let mut rng = thread_rng();

        let slope: f32 = ((rng.gen_range(-5.0..=5.0) * 1000.0) as f32).floor() / 1000.0;

        let origin: i32;

        if slope > 0.0 {
            origin = rng.gen_range(0..(x as i32 + (x as f32 * slope) as i32)) as i32;
        } else {
            origin = rng.gen_range((0 + (x as f32 * slope) as i32)..x as i32);
        }

        let r = rng.gen_range(0..256) as u8;
        let g = rng.gen_range(0..256) as u8;
        let b = rng.gen_range(0..256) as u8;

        let xmin = cmp::max(((0 - origin) as f32 / slope) as i32, 0) as u32;
        let xmax = ((y as i32 - origin) as f32 / slope) as u32;

        let xmax = cmp::min(xmax, x);

        SlopeLine {
            slope: slope,
            origin: origin,
            color: [r, g, b],
            score: 0.0,
            count: 1,
            xmin: xmin,
            xmax: xmax
        }
    }

    fn give_birth(&self, x: u32, y: u32) -> SlopeLine {
        let mut rng = thread_rng();

        let mut slope: f32 = self.slope.clone();
        let mut origin = self.origin.clone();
        let color = self.color.clone();
        let mut r = color[0];
        let mut g = color[1];
        let mut b = color[2];

        slope = ((rng.gen_range((slope - 3.0)..=(slope + 3.0)) * 1000.0) as f32).floor() / 1000.0;
        origin = rng.gen_range(origin-10..=origin+10);

        r = rng.gen_range(r - (cmp::min(r, 20) as u8)..=cmp::min(r as i32 + 20, 255) as u8);
        g = rng.gen_range(g - (cmp::min(g, 20) as u8)..=cmp::min(g as i32  + 20, 255) as u8);
        b = rng.gen_range(b - (cmp::min(b, 20) as u8)..=cmp::min(b as i32 + 20, 255) as u8);

        let xmin = cmp::max(((0 - origin) as f32 / slope) as i32, 0) as u32;
        let xmax = ((y as i32 - origin) as f32 / slope) as u32;

        let xmax = cmp::min(xmax, x);

        let new_line = SlopeLine {
            slope: slope,
            origin: origin,
            color: [r, g, b],
            score: 0.0,
            count: 1,
            xmin: xmin,
            xmax: xmax
        };

        return new_line;

    }
}

fn color_difference(rgb1: &Rgb<u8>, rgb2: [u8; 3]) -> f64 {
    let pixel_goal = rgb1.channels();

    let r1 = pixel_goal[0] as f64;
    let g1 = pixel_goal[1] as f64;
    let b1 = pixel_goal[2] as f64;
    let r2 = rgb2[0] as f64;
    let g2 = rgb2[1] as f64;
    let b2 = rgb2[2] as f64;

    let rgb1 = Srgb::new(r1/255.0, g1/255.0, b1/255.0).into_linear();
    let rgb2 = Srgb::new(r2/255.0, g2/255.0, b2/255.0).into_linear();

    let rgb1 = Lab::from_color(rgb1);
    let rgb2 = Lab::from_color(rgb2);

    let difference = rgb2.get_color_difference(&rgb1);

    return 100.0 - difference;
}

fn color_difference_pixel(rgb1: &Rgb<u8>, rgb2: &Rgb<u8>) -> f64 {
    let pixel_goal = rgb1.channels();
    let pixel = rgb2.channels();

    let r1 = pixel_goal[0] as f64;
    let g1 = pixel_goal[1] as f64;
    let b1 = pixel_goal[2] as f64;
    let r2 = pixel[0] as f64;
    let g2 = pixel[1] as f64;
    let b2 = pixel[2] as f64;

    let rgb1 = Srgb::new(r1/255.0, g1/255.0, b1/255.0).into_linear();
    let rgb2 = Srgb::new(r2/255.0, g2/255.0, b2/255.0).into_linear();

    let rgb1 = Lab::from_color(rgb1);
    let rgb2 = Lab::from_color(rgb2);

    let difference = rgb2.get_color_difference(&rgb1);

    return 100.0 - difference;
}

fn main() {
    let input = fs::read_dir("./input").expect("Folder not found");

    let first_img_path = fs::read_dir("./input").expect("Folder not found").next().expect("Folder is empty").expect("Folder is empty");
    let first_img_file = image::open(first_img_path.path()).unwrap();
    let first_img_rgb = first_img_file.into_rgb8();
    let (first_imgx, first_imgy) = first_img_rgb.dimensions();

    if first_imgx == 0 || first_imgy == 0 {
        panic!("Input is empty");
    }

    let mut imgbuf = ImageBuffer::new(first_imgx, first_imgy);

    let mut input_count = 0;

    let mut t = 0;

    for entry in input {
        let path = entry.unwrap().path();

        let img = match image::open(path) {
            Ok(img) => img,
            Err(err) => {
                println!("{}", err);
                break;
            },
        };

        input_count += 1;

        let img = Arc::new(Mutex::new(img.into_rgb8()));

        let (imgx, imgy) = img.lock().unwrap().dimensions();

        if first_imgx != imgx {
            break;
        }

        if first_imgy != imgy {
            break;
        }

        let perfect_score = (imgx * imgy) as f64 * 100.0;

        if t == 0 {
            println!("Img dimensions: {} {}", imgx, imgy);

            DirBuilder::new().recursive(true).create("./output").expect("Error in folder creation");
            imgbuf.save("output/output0.png").expect("Error in file creation");
        }

        let imgbuffer = Arc::new(Mutex::new(imgbuf));

        let mut i = 0;

        loop {
            let now = Instant::now();
            let mut lines = Arc::new(Mutex::new(vec!()));
            let processed_lines = Arc::new(Mutex::new(vec![]));
                
            for _ in 0..500 {
                lines.lock().unwrap().push(Arc::new(Mutex::new(SlopeLine::new(imgx, imgy))));
            }

            for f in 0..100 {
                let mut handles = vec![];

                for thread_count in 0..5 {
                    let processed_lines = Arc::clone(&processed_lines);

                    let imgbuffer = Arc::clone(&imgbuffer);
                    let img = Arc::clone(&img);
                    let lines = Arc::clone(&lines);

                    let handle = thread::spawn(move || {
                        for line_index in (100 * thread_count)..(100 * (thread_count + 1)) {
                            let all_lines = &mut *lines.lock().unwrap();
                            let mut line = *all_lines[line_index].lock().unwrap();

                            if line.score > 0.0 {
                                processed_lines.lock().unwrap().push(line);
                                continue;
                            }

                            for x in line.xmin..line.xmax {
                                let y = (line.slope*x as f32) as i32 + line.origin;

                                if y >= 0 && y < imgy as i32 {
                                    let prev_img = imgbuffer.lock().unwrap();

                                    let img = img.lock().unwrap();

                                    let difference = color_difference(img.get_pixel(x, y as u32), line.color);
                                    let prev_diff = color_difference_pixel(img.get_pixel(x, y as u32), prev_img.get_pixel(x, y as u32));

                                    if difference > prev_diff {
                                        line.score += difference - prev_diff;
                                    }

                                    line.count += 1;
                                }
                            }

                            processed_lines.lock().unwrap().push(line);
                        }
                    });

                    handles.push(handle);
                }

                for handle in handles {handle.join().unwrap();}

                let mut p_lines = &mut *processed_lines.lock().unwrap();

                p_lines.sort_by(|a, b| (b.score/b.count as f64).partial_cmp(&(&a.score / *&a.count as f64)).unwrap());

                let mut slopes: Vec<(i32, f32)> = vec![];
                let mut filtered_lines = vec![];

                for i in 0..p_lines.len() {
                    let mut line = p_lines[i];

                    if line.score == 0.0 {
                        continue;
                    }

                    if slopes.contains(&(line.origin, line.slope)) {
                        line.score = 0.0;
                        filtered_lines.push(line);
                        continue;
                    } 

                    slopes.push((line.origin, line.slope));
                    filtered_lines.push(line);
                }

                p_lines = &mut filtered_lines;
                
                p_lines.sort_by(|a, b| (b.score/b.count as f64).partial_cmp(&(&a.score / *&a.count as f64)).unwrap());

                p_lines.truncate(100);
                let mut new_lines = vec![];

                if f < 99 {
                    for index in 0..cmp::min(100, p_lines.len()) {
                        let line = p_lines[index];

                        if line.score == 0.0 {
                            break;
                        }

                        new_lines.push(Arc::new(Mutex::new(line)));
                        new_lines.push(Arc::new(Mutex::new(line.give_birth(imgx, imgy))));
                        new_lines.push(Arc::new(Mutex::new(line.give_birth(imgx, imgy))));
                        new_lines.push(Arc::new(Mutex::new(line.give_birth(imgx, imgy))));
                        new_lines.push(Arc::new(Mutex::new(line.give_birth(imgx, imgy))));
                    }

                    while new_lines.len() < 500 {
                        new_lines.push(Arc::new(Mutex::new(SlopeLine::new(imgx, imgy))));
                    }

                    lines = Arc::new(Mutex::new(new_lines));
                } else {
                    for index in 0..cmp::min(100, (cmp::max(imgx, 1000) + i + t) / 100) {
                        let line = p_lines[index as usize];

                        if line.score > 0.0 {
                            new_lines.push(Arc::new(Mutex::new(line)));
                        }
                    }

                    lines = Arc::new(Mutex::new(new_lines));
                }
            }

            let lines = &mut *lines.lock().unwrap();
            let mut improvement = 0.0;

            for line_index in (0..lines.len()).rev() {
                let line = lines[line_index].lock().unwrap();

                for x in line.xmin..line.xmax {
                    let y = (line.slope*x as f32) as i32 + line.origin;

                    if y >= 0 && y < imgy as i32 {
                        let imgbuffer = Arc::clone(&imgbuffer);
                        let mut prev_img = imgbuffer.lock().unwrap();

                        let img = img.lock().unwrap();
                        let img_pixel = img.get_pixel(x, y as u32);

                        let difference = color_difference(img_pixel, line.color);
                        let prev_diff = color_difference_pixel(img_pixel, prev_img.get_pixel(x, y as u32));

                        if difference > prev_diff {
                            prev_img.put_pixel(x, y as u32, Rgb(line.color));
                            improvement += difference - prev_diff;
                        }
                    }
                }
            }

            i += 1;
            t += 1;

            let path = format!("output/output{}_{}.png", input_count, i);
        
            imgbuffer.lock().unwrap().save(path).unwrap();

            println!("Done saving output{}_{}.png\tlines: {}\ttime: {:.2}s\timprovement: {:.2} ({:.2}%)",input_count, i, lines.len(), now.elapsed().as_secs_f32(), improvement, (improvement / perfect_score) * 100.0);

            if improvement == 0.0 {
                println!("Completed due to no more improvements to be made");
                imgbuf = imgbuffer.lock().unwrap().clone();
                break;
            }

            if i % 10 == 0 {
                let mut total_score: f64 = 0.0;
                for (x, y, pixel) in imgbuffer.lock().unwrap().enumerate_pixels() {
                    total_score += color_difference_pixel(img.lock().unwrap().get_pixel(x, y as u32), pixel);
                }

                let percent_change = (total_score / perfect_score) * 100.0;

                println!("Generated image is {:.2}% of the original", percent_change);

                if percent_change > 98.0 {
                    println!("Completed due to being too close to the source image");
                    imgbuf = imgbuffer.lock().unwrap().clone();
                    break;
                }
            }
        }
    }
}