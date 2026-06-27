use libvips_rs::ops::{self, Interpretation};
use libvips_rs::{VipsApp, VipsImage, VipsInterpolate, VipsSource};

fn srgb_image(w: i32, h: i32) -> VipsImage {
    let black = ops::black(w, h).expect("black");
    ops::colourspace(&black, Interpretation::Srgb).expect("colourspace srgb")
}

// All assertions live in one test because only one VipsApp may exist per process.
#[test]
fn memory_safety_regressions() {
    let app = VipsApp::new("mem-safety-test", false).expect("init libvips");
    app.cache_set_max(0);
    app.cache_set_max_mem(0);

    let base = srgb_image(512, 512);

    // (1) Clone must be reference-counted. With the previous derived Clone, the
    // raw pointer was bit-copied without a ref, so dropping the clones freed the
    // shared object and `base` became a dangling use-after-free (SIGSEGV). With
    // refcounted Clone the object survives until every wrapper is dropped.
    for _ in 0..2000 {
        let a = base.clone();
        let b = a.clone();
        let c = b.clone();
        drop(a);
        drop(c);
        drop(b);
    }
    assert_eq!((base.get_width(), base.get_height()), (512, 512), "base freed under clone churn");
    let probe = ops::thumbnail_image(&base, 64).expect("op on base after clone churn");
    assert_eq!(probe.get_width(), 64, "base unusable after clone churn");

    // (2) Buffer save path (new_byte_array = copy + g_free) must not corrupt or
    // leak across many iterations.
    let mut len = 0usize;
    for _ in 0..200 {
        let buf = base.image_write_to_buffer(".png").expect("image_write_to_buffer");
        assert!(!buf.is_empty(), "empty encoded buffer");
        len = buf.len();
    }
    assert!(len > 0);

    // (3) VipsBlob -> Vec<u8> path (profile_load = copy + vips_area_unref). The
    // old code stole the blob's internal buffer and then g_object_unref'd a
    // VipsArea (wrong unref); this loop must run clean and leak-free.
    for _ in 0..200 {
        let profile = ops::profile_load("srgb").expect("profile_load srgb");
        assert!(!profile.is_empty(), "empty profile");
    }

    // (4) VipsInterpolate static singletons are BORROWED ("no need to unref");
    // the wrapper must take its own ref. Under the old code each drop unref'd the
    // shared singleton, freed it, and left libvips' cached pointer dangling -> the
    // next construct/drop was a use-after-free. Churn + clone, then use a real op
    // that interpolates to prove the singleton is still alive.
    for _ in 0..1000 {
        let i = VipsInterpolate::new();
        let j = i.clone();
        drop(i);
        drop(VipsInterpolate::new_from_neasest_static());
        drop(VipsInterpolate::new_from_bilinear_static());
        drop(j);
    }
    let small = ops::thumbnail_image(&base, 48).expect("thumbnail for affine");
    let affined = ops::affine(&small, 2.0, 0.0, 0.0, 2.0).expect("affine after interpolate churn");
    assert!(affined.get_width() > 0 && affined.get_height() > 0, "affine produced empty image");

    // (5) getpoint drives utils::new_double_array (copy + g_free).
    for _ in 0..200 {
        let px = ops::getpoint(&base, 10, 10).expect("getpoint");
        assert_eq!(px.len(), 3, "sRGB pixel should have 3 components, got {}", px.len());
    }

    // (6) VipsSource read()/map() — the FFI fixes (no NULL deref, no mismatched
    // allocator). `data` is kept alive for the whole block.
    {
        let data: Vec<u8> = (0..1024u32).map(|i| (i % 251) as u8).collect();
        let mut src = VipsSource::new_from_memory(&data).expect("source from memory");
        let head = src.read(16).expect("read 16");
        assert_eq!(head.len(), 16, "read should return 16 bytes");
        assert_eq!(&head[..], &data[..16], "read content mismatch");

        let src2 = VipsSource::new_from_memory(&data).expect("source from memory (2)");
        let mapped = src2.map().expect("map");
        assert_eq!(mapped.len(), data.len(), "mapped length mismatch");
        assert_eq!(mapped, &data[..], "mapped content mismatch");
    }

    // (7) image_write now builds a real memory destination (was a NULL deref).
    let written = base.image_write().expect("image_write");
    assert_eq!((written.get_width(), written.get_height()), (512, 512), "image_write produced wrong dims");

    // With the cache disabled, vips-tracked pixel memory returns to ~baseline.
    let tracked = app.tracked_get_mem();
    assert!(tracked < 64 * 1024 * 1024, "tracked vips memory unexpectedly high: {tracked} bytes");

    drop(app);
}
