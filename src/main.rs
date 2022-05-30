use image::{self, ImageBuffer, Pixel};
use rand::{thread_rng, Rng};
use std::{cmp, fs::DirBuilder};
use image::Rgb;
use palette::{FromColor, ColorDifference, Srgb, Lab};
use std::thread;
use std::sync::{Arc, Mutex};

#[derive(Copy, Clone, Debug)]
struct SlopeLine {
    slope: i32,
    origin: i32,
    color: [u8; 3],
    pub score: f64,
    pub count: i32
}

impl SlopeLine {
    fn new(x:u32) -> SlopeLine {
        let mut rng = thread_rng();

        let slope = rng.gen_range(-20..=20);
        let origin = rng.gen_range(-(x as i32)..x as i32*2) as i32;

        let r = rng.gen_range(0..256) as u8;
        let g = rng.gen_range(0..256) as u8;
        let b = rng.gen_range(0..256) as u8;
        SlopeLine {
            slope: slope,
            origin: origin,
            color: [r, g, b],
            score: 0.0,
            count: 1
        }
    }

    fn give_birth(&self) -> SlopeLine {
        let mut rng = thread_rng();

        let mut slope: i32 = self.slope.clone();
        let mut origin = self.origin.clone();
        let color = self.color.clone();
        let mut r = color[0];
        let mut g = color[1];
        let mut b = color[2];

        slope = rng.gen_range(slope - 2..=slope + 2);
        origin = rng.gen_range(origin-10..=origin+10);
        r = rng.gen_range(r - (cmp::min(r, 20) as u8)..=cmp::min(r as i32 + 20, 255) as u8);
        g = rng.gen_range(g - (cmp::min(g, 20) as u8)..=cmp::min(g as i32  + 20, 255) as u8);
        b = rng.gen_range(b - (cmp::min(b, 20) as u8)..=cmp::min(b as i32 + 20, 255) as u8);

        SlopeLine {
            slope: slope,
            origin: origin,
            color: [r, g, b],
            score: 0.0,
            count: 1
        }
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
    let img = match image::open("input.png") {
        Ok(img) => img,
        Err(err) => panic!("{}", err),
    };
    let img = Arc::new(Mutex::new(img.into_rgb8()));

    let (imgx, imgy) = img.lock().unwrap().dimensions();

    let imgbuf = ImageBuffer::new(imgx, imgy);
    
    println!("Img dimensions: {} {}", imgx, imgy);

    DirBuilder::new().recursive(true).create("./output").expect("Error in folder creation");
    imgbuf.save("output/output0.png").expect("Error in file creation");

    let imgbuf = Arc::new(Mutex::new(imgbuf));

    let mut i = 0;

    loop {
        let lines = Arc::new(Mutex::new(vec!()));
        let processed_lines = Arc::new(Mutex::new(vec![])); 
            
        for _ in 0..500 {
            lines.lock().unwrap().push(Arc::new(Mutex::new(SlopeLine::new(imgx))));
        }

        for f in 0..100 {
            let mut handles = vec![];

            for thread_count in 0..5 {
                let processed_lines = Arc::clone(&processed_lines);

                let imgbuf = Arc::clone(&imgbuf);
                let img = Arc::clone(&img);
                let lines = Arc::clone(&lines);

                let handle = thread::spawn(move || {
                    for line_index in (100 * thread_count)..(100 * (thread_count + 1)) {
                        let all_lines = lines.lock().unwrap();
                        let mut line = *all_lines[line_index].lock().unwrap();
                        
                        for x in 0..imgx {
                            let y = (line.slope*x as i32) + line.origin;

                            if y >= 0 && y < imgy as i32 {
                                let prev_img = imgbuf.lock().unwrap();

                                let img = img.lock().unwrap();

                                let difference = color_difference(img.get_pixel(x, y as u32), line.color);
                                let prev_diff = color_difference_pixel(img.get_pixel(x, y as u32), prev_img.get_pixel(x, y as u32));

                                if difference > prev_diff {
                                    line.score += difference - prev_diff;
                                }

                                line.count += 1;

                                processed_lines.lock().unwrap().push(line);
                            }
                        }
                    }
                });

                handles.push(handle);
            }

            for handle in handles {
                handle.join().unwrap();
            }

            let mut processed_lines = processed_lines.lock().unwrap();
    
            // processed_lines.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
            processed_lines.sort_by(|a, b| (b.score/b.count as f64).partial_cmp(&(&a.score / *&a.count as f64)).unwrap());
            processed_lines.truncate(100);

            let mut lines = lines.lock().unwrap();
            lines.truncate(0);

            if f < 99 {
                for index in 0..100 {
                    let mut line = processed_lines[index];

                    line.score = 0.0;
                    line.count = 1;

                    if line.score > 0.0 {
                        lines.push(Arc::new(Mutex::new(line)));
                        lines.push(Arc::new(Mutex::new(line.give_birth())));
                        lines.push(Arc::new(Mutex::new(line.give_birth())));
                        lines.push(Arc::new(Mutex::new(line.give_birth())));
                        lines.push(Arc::new(Mutex::new(line.give_birth())));
                    } else {
                        lines.push(Arc::new(Mutex::new(SlopeLine::new(imgx))));
                        lines.push(Arc::new(Mutex::new(SlopeLine::new(imgx))));
                        lines.push(Arc::new(Mutex::new(SlopeLine::new(imgx))));
                        lines.push(Arc::new(Mutex::new(SlopeLine::new(imgx))));
                        lines.push(Arc::new(Mutex::new(SlopeLine::new(imgx))));
                    }
                }
            }
        }

        lines.lock().unwrap().truncate(0);

        let lines = processed_lines.lock().unwrap();

        let mut any_improvement = false;
        let mut improvement = 0.0;

        for line_index in (0..100).rev() {
            let line = lines[line_index];

            for x in 0..imgx {
                let y = (line.slope*x as i32) + line.origin;

                if y >= 0 && y < imgy as i32 {
                    let imgbuf = Arc::clone(&imgbuf);
                    let mut prev_img = imgbuf.lock().unwrap();

                    let img = img.lock().unwrap();

                    let difference = color_difference(img.get_pixel(x, y as u32), line.color);
                    let prev_diff = color_difference_pixel(img.get_pixel(x, y as u32), prev_img.get_pixel(x, y as u32));

                    if difference > prev_diff {
                        prev_img.put_pixel(x, y as u32, image::Rgb(line.color));
                        any_improvement = true;
                        improvement += difference - prev_diff;
                    }
                }
            }
        }

        i += 1;

        let path = format!("output/output{}.png", i);
    
        imgbuf.lock().unwrap().save(path).unwrap();

        println!("Done saving output{}.png with an improvement of {}", i, improvement);

        if any_improvement == false {
            break;
        }
    }
}