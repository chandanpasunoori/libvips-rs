use libvips_rs::ops::{self, Interpretation, ThumbnailNormOptions};
use libvips_rs::{VipsApp, VipsImage};
use std::process::Command;

fn srgb_image(w: i32, h: i32) -> VipsImage {
    let black = ops::black(w, h).expect("black");
    ops::colourspace(&black, Interpretation::Srgb).expect("colourspace srgb")
}

fn has(tool: &str) -> bool {
    // Detection only cares whether the binary can be spawned, not the probe's
    // exit code (e.g. `exiftool -version` exits non-zero; the real flag is -ver).
    Command::new(tool).arg("-ver").output().is_ok()
}

#[test]
fn thumbnail_norm_methods() {
    let app = VipsApp::new("thumbnail-norm-test", false).expect("init libvips");
    let err = || app.error_buffer().unwrap_or("").to_string();

    let src = srgb_image(400, 300);
    assert_eq!(src.get_bands(), 3, "synthetic sRGB image should be 3-band");

    // 1. Default opts: input_profile is empty (must be skipped, not forwarded as
    //    "") and output_profile = "srgb". The generated thumbnail_image_with_opts
    //    fails here with `unable to load profile ""`; thumbnail_image_norm must not.
    let out = ops::thumbnail_image_norm(&src, 200, &ThumbnailNormOptions::default()).unwrap_or_else(|_| panic!("thumbnail_image_norm default failed: {}", err()));
    assert_eq!(out.get_width(), 200, "width should be 200");
    assert!((out.get_height() - 150).abs() <= 1, "height ~150, got {}", out.get_height());

    // 2. Both profiles empty -> no ICC forwarded at all, must still succeed.
    let no_profile = ThumbnailNormOptions {
        input_profile: String::new(),
        output_profile: String::new(),
        ..Default::default()
    };
    let out2 = ops::thumbnail_image_norm(&src, 100, &no_profile).unwrap_or_else(|_| panic!("thumbnail_image_norm (no profiles) failed: {}", err()));
    assert_eq!(out2.get_width(), 100);

    // 3. Default size = Down -> never upscale past the source width.
    let out3 = ops::thumbnail_image_norm(&src, 800, &ThumbnailNormOptions::default()).unwrap_or_else(|_| panic!("thumbnail_image_norm upscale-guard failed: {}", err()));
    assert_eq!(out3.get_width(), 400, "Size::Down must not upscale past source 400");

    // 4. File-based thumbnail_norm (shrink-on-load path).
    let tmp = std::env::temp_dir().join("libvips_rs_thumbnail_norm.jpg");
    let tmp_s = tmp.to_str().unwrap();
    ops::jpegsave(&src, tmp_s).expect("jpegsave");
    let outf = ops::thumbnail_norm(tmp_s, 150, &ThumbnailNormOptions::default()).unwrap_or_else(|_| panic!("thumbnail_norm (file) failed: {}", err()));
    assert_eq!(outf.get_width(), 150);
    let _ = std::fs::remove_file(&tmp);

    // 5. Autorotate by default: an EXIF orientation=6 image (stored 400x300,
    //    displays 300x400) must come out portrait by default and landscape with
    //    no_rotate=true. Needs magick (author the jpeg) + exiftool (set the tag);
    //    skipped if either is missing or the tag didn't take.
    if has("magick") && has("exiftool") {
        let oriented = std::env::temp_dir().join("libvips_rs_oriented.jpg");
        let op = oriented.to_str().unwrap();
        let made = Command::new("magick").args(["-size", "400x300", "xc:red", op]).status().map(|s| s.success()).unwrap_or(false)
            && Command::new("exiftool").args(["-overwrite_original", "-n", "-Orientation=6", op]).status().map(|s| s.success()).unwrap_or(false);
        let tagged = made && ops::jpegload(op).map(|i| i.get_orientation() == 6).unwrap_or(false);
        if tagged {
            let rotated = ops::thumbnail_norm(op, 150, &ThumbnailNormOptions::default()).unwrap_or_else(|_| panic!("thumbnail_norm autorotate failed: {}", err()));
            assert!(rotated.get_height() > rotated.get_width(), "autorotate (default) should yield portrait, got {}x{}", rotated.get_width(), rotated.get_height());

            let no_rot = ThumbnailNormOptions { no_rotate: true, ..Default::default() };
            let kept = ops::thumbnail_norm(op, 150, &no_rot).unwrap_or_else(|_| panic!("thumbnail_norm no_rotate failed: {}", err()));
            assert!(kept.get_width() > kept.get_height(), "no_rotate=true should keep landscape, got {}x{}", kept.get_width(), kept.get_height());
        } else {
            eprintln!("skipping autorotate assertion: could not author an EXIF-oriented fixture");
        }
        let _ = std::fs::remove_file(&oriented);
    } else {
        eprintln!("skipping autorotate assertion: magick/exiftool not installed");
    }

    drop(app);
}
