// (c) Copyright 2019-2025 MIT
// this is manually created because it doesn't follow the standard from the introspection output

/// VipsLinear (linear), calculate (a * in + b)
/// inp: `&VipsImage` -> Input image
/// a: `&[f64]` -> Multiply by this. Must have equal len as b
/// b: `&[f64]` -> Add this. Must have equal len as a
/// returns `VipsImage` - Output image
pub fn linear(inp: &VipsImage, a: &mut[f64], b: &mut[f64]) -> Result<VipsImage> {
    unsafe {
        if a.len() != b.len() {
            return Err(Error::LinearError)
        }
        let inp_in: *mut bindings::VipsImage = inp.ctx;
        let a_in: *mut f64 = a.as_mut_ptr();
        let b_in: *mut f64 = b.as_mut_ptr();
        let mut out_out: *mut bindings::VipsImage = null_mut();

        let vips_op_response =
            bindings::vips_linear(inp_in, &mut out_out, a_in, b_in, b.len() as i32, NULL);
        utils::result(
            vips_op_response,
            VipsImage { ctx: out_out },
            Error::LinearError,
        )
    }
}

/// Options for linear operation
pub struct LinearOptions {
    /// uchar: `bool` -> Output should be uchar
    /// default: false
    pub uchar: bool,
}

impl std::default::Default for LinearOptions {
    fn default() -> Self {
        LinearOptions { uchar: false }
    }
}

/// VipsLinear (linear), calculate (a * in + b)
/// inp: `&VipsImage` -> Input image
/// a: `&[f64]` -> Multiply by this. Must have equal len as b
/// b: `&[f64]` -> Add this. Must have equal len as a
/// linear_options: `&LinearOptions` -> optional arguments
/// returns `VipsImage` - Output image
pub fn linear_with_opts(
    inp: &VipsImage,
    a: &mut [f64],
    b: &mut [f64],
    linear_options: &LinearOptions,
) -> Result<VipsImage> {
    unsafe {
        if a.len() != b.len() {
            return Err(Error::LinearError)
        }
        let inp_in: *mut bindings::VipsImage = inp.ctx;
        let a_in: *mut f64 = a.as_mut_ptr();
        let b_in: *mut f64 = b.as_mut_ptr();
        let mut out_out: *mut bindings::VipsImage = null_mut();

        let uchar_in: i32 = if linear_options.uchar { 1 } else { 0 };
        let uchar_in_name = utils::new_c_string("uchar")?;

        let vips_op_response = bindings::vips_linear(
            inp_in,
            &mut out_out,
            a_in,
            b_in,
            b.len() as i32,
            uchar_in_name.as_ptr(),
            uchar_in,
            NULL,
        );
        utils::result(
            vips_op_response,
            VipsImage { ctx: out_out },
            Error::LinearError,
        )
    }
}

/// VipsGetpoint (getpoint), read a point from an image
/// inp: `&VipsImage` -> Input image
/// x: `i32` -> Point to read
/// min: 0, max: 10000000, default: 0
/// y: `i32` -> Point to read
/// min: 0, max: 10000000, default: 0
/// returns `Vec<f64>` - Array of output values
pub fn getpoint(inp: &VipsImage, x: i32, y: i32) -> Result<Vec<f64>> {
    unsafe {
        let inp_in: *mut bindings::VipsImage = inp.ctx;
        let mut out_array_size: i32 = 0;
        let mut out_array: *mut f64 = null_mut();

        let vips_op_response = bindings::vips_getpoint(
            inp_in,
            &mut out_array,
            &mut out_array_size,
            x,
            y,
            NULL,
        );
        utils::result(
            vips_op_response,
            utils::new_double_array(out_array, out_array_size.try_into().unwrap()),
            Error::GetpointError,
        )
    }
}

/// VipsCase (case), use pixel values to pick cases from an array of images
/// index: `&VipsImage` -> Index image
/// cases: `&mut [VipsImage]` -> Array of case images
/// n: `i32` -> number of case images
/// returns `VipsImage` - Output image
pub fn case(index: &VipsImage, cases: &mut [VipsImage], n: i32) -> Result<VipsImage> {
    unsafe {
        let index_in: *mut bindings::VipsImage = index.ctx;
        // Bind the Vec so it outlives the call; `.collect().as_mut_ptr()` on a
        // temporary leaves a dangling pointer (read of freed memory) in vips_case.
        let mut cases_in: Vec<*mut bindings::VipsImage> = cases.iter().map(|v| v.ctx).collect();
        let mut out_out: *mut bindings::VipsImage = null_mut();

        // Bound n to [0, cases.len()] so neither a too-large nor a negative n can
        // make vips_case read past the end of cases_in (out-of-bounds read).
        let n = n.clamp(0, cases_in.len() as i32);
        let vips_op_response = bindings::vips_case(index_in, cases_in.as_mut_ptr(), &mut out_out, n, NULL);
        utils::result(
            vips_op_response,
            VipsImage { ctx: out_out },
            Error::CaseError,
        )
    }
}

/// Options for [`thumbnail_norm`] and [`thumbnail_image_norm`].
///
/// These wrappers fold orientation and colorspace normalization into a single
/// thumbnail pass — the lowest-memory way libvips offers to resize. A thumbnail
/// processes the image with tiled / sequential access (and, for the file/buffer
/// variants, shrink-on-load so the full-resolution image is never decoded),
/// whereas `resize` rasterizes the whole image. Because the defaults autorotate
/// (using EXIF orientation) and convert to sRGB, a caller can drop separate
/// autorotate + ICC-transform passes.
///
/// Differences from the generated [`ThumbnailImageOptions`] / [`ThumbnailOptions`]:
/// * defaults to `no_rotate = false` (autorotate upright) and
///   `output_profile = "srgb"` (convert to sRGB);
/// * an empty `input_profile` / `output_profile` is left unset instead of being
///   forwarded to vips. The generated wrappers always forward them, and vips
///   rejects an empty profile name with `unable to load profile ""`.
#[derive(Clone, Debug)]
pub struct ThumbnailNormOptions {
    /// Size to this height. Large by default so width is the binding dimension.
    pub height: i32,
    /// Only upsize, only downsize, or both. Downsize-only by default.
    pub size: Size,
    /// Don't use orientation tags to rotate the image upright.
    pub no_rotate: bool,
    /// Reduce to fill the target rectangle, then crop.
    pub crop: Interesting,
    /// Reduce in linear light.
    pub linear: bool,
    /// Fallback input profile. Empty = leave unset.
    pub input_profile: String,
    /// Output profile to convert to. Empty = leave unset.
    pub output_profile: String,
    /// Rendering intent.
    pub intent: Intent,
    /// Error level to fail on.
    pub fail_on: FailOn,
}

impl std::default::Default for ThumbnailNormOptions {
    fn default() -> Self {
        ThumbnailNormOptions {
            height: 100_000_000,
            size: Size::Down,
            no_rotate: false,
            crop: Interesting::None,
            linear: false,
            input_profile: String::new(),
            output_profile: String::from("srgb"),
            intent: Intent::Relative,
            fail_on: FailOn::None,
        }
    }
}

/// Issue a `vips_thumbnail*` call, forwarding `input-profile` / `output-profile`
/// only when they are non-empty (vips errors on an empty profile name). The
/// option-name and profile-value `CString`s must be created and kept alive by
/// the caller; their `.as_ptr()` values are passed in.
macro_rules! thumbnail_norm_call {
    ($func:ident, $first:expr, $out:expr, $width:expr, $opts:expr,
     $h_n:expr, $s_n:expr, $nr_n:expr, $c_n:expr, $l_n:expr,
     $ip_n:expr, $op_n:expr, $it_n:expr, $fo_n:expr, $ip_v:expr, $op_v:expr) => {{
        let h = $opts.height;
        let s = $opts.size as i32;
        let nr: i32 = if $opts.no_rotate { 1 } else { 0 };
        let c = $opts.crop as i32;
        let l: i32 = if $opts.linear { 1 } else { 0 };
        let it = $opts.intent as i32;
        let fo = $opts.fail_on as i32;
        match (!$opts.input_profile.is_empty(), !$opts.output_profile.is_empty()) {
            (true, true) => bindings::$func(
                $first, $out, $width, $h_n, h, $s_n, s, $nr_n, nr, $c_n, c, $l_n, l,
                $ip_n, $ip_v, $op_n, $op_v, $it_n, it, $fo_n, fo, NULL,
            ),
            (false, true) => bindings::$func(
                $first, $out, $width, $h_n, h, $s_n, s, $nr_n, nr, $c_n, c, $l_n, l,
                $op_n, $op_v, $it_n, it, $fo_n, fo, NULL,
            ),
            (true, false) => bindings::$func(
                $first, $out, $width, $h_n, h, $s_n, s, $nr_n, nr, $c_n, c, $l_n, l,
                $ip_n, $ip_v, $it_n, it, $fo_n, fo, NULL,
            ),
            (false, false) => bindings::$func(
                $first, $out, $width, $h_n, h, $s_n, s, $nr_n, nr, $c_n, c, $l_n, l,
                $it_n, it, $fo_n, fo, NULL,
            ),
        }
    }};
}

/// VipsThumbnailImage (thumbnail_image), generate a thumbnail from an in-memory
/// image, normalizing orientation and colorspace in the same low-memory pass.
///
/// Compared to [`thumbnail_image_with_opts`], this defaults to autorotate + sRGB
/// and never forwards an empty profile string. See [`ThumbnailNormOptions`].
///
/// inp: `&VipsImage` -> Input image argument
/// width: `i32` -> Size to this width
/// opts: `&ThumbnailNormOptions` -> optional arguments
/// returns `VipsImage` - Output image
pub fn thumbnail_image_norm(inp: &VipsImage, width: i32, opts: &ThumbnailNormOptions) -> Result<VipsImage> {
    unsafe {
        let inp_in: *mut bindings::VipsImage = inp.ctx;
        let mut out_out: *mut bindings::VipsImage = null_mut();

        let height_n = utils::new_c_string("height")?;
        let size_n = utils::new_c_string("size")?;
        let no_rotate_n = utils::new_c_string("no-rotate")?;
        let crop_n = utils::new_c_string("crop")?;
        let linear_n = utils::new_c_string("linear")?;
        let input_profile_n = utils::new_c_string("input-profile")?;
        let output_profile_n = utils::new_c_string("output-profile")?;
        let intent_n = utils::new_c_string("intent")?;
        let fail_on_n = utils::new_c_string("fail-on")?;
        let input_profile_v = utils::new_c_string(&opts.input_profile)?;
        let output_profile_v = utils::new_c_string(&opts.output_profile)?;

        let vips_op_response = thumbnail_norm_call!(
            vips_thumbnail_image, inp_in, &mut out_out, width, opts,
            height_n.as_ptr(), size_n.as_ptr(), no_rotate_n.as_ptr(), crop_n.as_ptr(), linear_n.as_ptr(),
            input_profile_n.as_ptr(), output_profile_n.as_ptr(), intent_n.as_ptr(), fail_on_n.as_ptr(),
            input_profile_v.as_ptr(), output_profile_v.as_ptr()
        );
        utils::result(vips_op_response, VipsImage { ctx: out_out }, Error::ThumbnailImageError)
    }
}

/// VipsThumbnailFile (thumbnail), generate a thumbnail directly from a file,
/// normalizing orientation and colorspace in the same low-memory pass.
///
/// This is the most memory-efficient resize path: vips shrinks on load (for
/// formats that support it) so the full-resolution image is never materialized.
/// Use it as the first operation. The filename may carry loader options, e.g.
/// `"in.gif[n=-1]"` to read every page of an animated GIF.
///
/// filename: `&str` -> Filename to read from
/// width: `i32` -> Size to this width
/// opts: `&ThumbnailNormOptions` -> optional arguments
/// returns `VipsImage` - Output image
pub fn thumbnail_norm(filename: &str, width: i32, opts: &ThumbnailNormOptions) -> Result<VipsImage> {
    unsafe {
        let filename_in: CString = utils::new_c_string(filename)?;
        let mut out_out: *mut bindings::VipsImage = null_mut();

        let height_n = utils::new_c_string("height")?;
        let size_n = utils::new_c_string("size")?;
        let no_rotate_n = utils::new_c_string("no-rotate")?;
        let crop_n = utils::new_c_string("crop")?;
        let linear_n = utils::new_c_string("linear")?;
        let input_profile_n = utils::new_c_string("input-profile")?;
        let output_profile_n = utils::new_c_string("output-profile")?;
        let intent_n = utils::new_c_string("intent")?;
        let fail_on_n = utils::new_c_string("fail-on")?;
        let input_profile_v = utils::new_c_string(&opts.input_profile)?;
        let output_profile_v = utils::new_c_string(&opts.output_profile)?;

        let vips_op_response = thumbnail_norm_call!(
            vips_thumbnail, filename_in.as_ptr(), &mut out_out, width, opts,
            height_n.as_ptr(), size_n.as_ptr(), no_rotate_n.as_ptr(), crop_n.as_ptr(), linear_n.as_ptr(),
            input_profile_n.as_ptr(), output_profile_n.as_ptr(), intent_n.as_ptr(), fail_on_n.as_ptr(),
            input_profile_v.as_ptr(), output_profile_v.as_ptr()
        );
        utils::result(vips_op_response, VipsImage { ctx: out_out }, Error::ThumbnailError)
    }
}
