// Copyright (c) 2019, The rav1e contributors. All rights reserved
//
// This source code is subject to the terms of the BSD 2 Clause License and
// the Alliance for Open Media Patent License 1.0. If the BSD 2 Clause License
// was not distributed with this source code in the LICENSE file, you can
// obtain it at www.aomedia.org/license/software. If the Alliance for Open
// Media Patent License 1.0 was not distributed with this source code in the
// PATENTS file, you can obtain it at www.aomedia.org/license/patent.

#[cfg(all(target_arch = "x86_64", feature = "nasm"))]
pub use self::nasm::*;
#[cfg(any(not(target_arch = "x86_64"), not(feature = "nasm")))]
pub use self::native::*;

use crate::tiling::*;
use crate::util::Pixel;

use std::ops;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MotionVector {
  pub row: i16,
  pub col: i16
}

impl ops::Add<MotionVector> for MotionVector {
    type Output = MotionVector;

    fn add(self, _rhs: MotionVector) -> MotionVector {
        MotionVector{row: self.row + _rhs.row, col: self.col + _rhs.col}
    }
}

impl ops::Div<i16> for MotionVector {
    type Output = MotionVector;

    fn div(self, _rhs: i16) -> MotionVector {
        MotionVector{row: self.row  / _rhs, col: self.col / _rhs}
    }
}

impl MotionVector {
  pub fn quantize_to_fullpel(self) -> Self {
    Self {
      row: (self.row / 8) * 8,
      col: (self.col / 8) * 8
    }
  }

  pub fn is_zero(self) -> bool {
    self.row == 0 && self.col == 0
  }
}

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
#[allow(unused)]
pub enum FilterMode {
  REGULAR = 0,
  SMOOTH = 1,
  SHARP = 2,
  BILINEAR = 3
}

pub const SUBPEL_FILTER_SIZE: usize = 8;

const SUBPEL_FILTERS: [[[i32; SUBPEL_FILTER_SIZE]; 16]; 6] = [
  [
    [0, 0, 0, 128, 0, 0, 0, 0],
    [0, 2, -6, 126, 8, -2, 0, 0],
    [0, 2, -10, 122, 18, -4, 0, 0],
    [0, 2, -12, 116, 28, -8, 2, 0],
    [0, 2, -14, 110, 38, -10, 2, 0],
    [0, 2, -14, 102, 48, -12, 2, 0],
    [0, 2, -16, 94, 58, -12, 2, 0],
    [0, 2, -14, 84, 66, -12, 2, 0],
    [0, 2, -14, 76, 76, -14, 2, 0],
    [0, 2, -12, 66, 84, -14, 2, 0],
    [0, 2, -12, 58, 94, -16, 2, 0],
    [0, 2, -12, 48, 102, -14, 2, 0],
    [0, 2, -10, 38, 110, -14, 2, 0],
    [0, 2, -8, 28, 116, -12, 2, 0],
    [0, 0, -4, 18, 122, -10, 2, 0],
    [0, 0, -2, 8, 126, -6, 2, 0]
  ],
  [
    [0, 0, 0, 128, 0, 0, 0, 0],
    [0, 2, 28, 62, 34, 2, 0, 0],
    [0, 0, 26, 62, 36, 4, 0, 0],
    [0, 0, 22, 62, 40, 4, 0, 0],
    [0, 0, 20, 60, 42, 6, 0, 0],
    [0, 0, 18, 58, 44, 8, 0, 0],
    [0, 0, 16, 56, 46, 10, 0, 0],
    [0, -2, 16, 54, 48, 12, 0, 0],
    [0, -2, 14, 52, 52, 14, -2, 0],
    [0, 0, 12, 48, 54, 16, -2, 0],
    [0, 0, 10, 46, 56, 16, 0, 0],
    [0, 0, 8, 44, 58, 18, 0, 0],
    [0, 0, 6, 42, 60, 20, 0, 0],
    [0, 0, 4, 40, 62, 22, 0, 0],
    [0, 0, 4, 36, 62, 26, 0, 0],
    [0, 0, 2, 34, 62, 28, 2, 0]
  ],
  [
    [0, 0, 0, 128, 0, 0, 0, 0],
    [-2, 2, -6, 126, 8, -2, 2, 0],
    [-2, 6, -12, 124, 16, -6, 4, -2],
    [-2, 8, -18, 120, 26, -10, 6, -2],
    [-4, 10, -22, 116, 38, -14, 6, -2],
    [-4, 10, -22, 108, 48, -18, 8, -2],
    [-4, 10, -24, 100, 60, -20, 8, -2],
    [-4, 10, -24, 90, 70, -22, 10, -2],
    [-4, 12, -24, 80, 80, -24, 12, -4],
    [-2, 10, -22, 70, 90, -24, 10, -4],
    [-2, 8, -20, 60, 100, -24, 10, -4],
    [-2, 8, -18, 48, 108, -22, 10, -4],
    [-2, 6, -14, 38, 116, -22, 10, -4],
    [-2, 6, -10, 26, 120, -18, 8, -2],
    [-2, 4, -6, 16, 124, -12, 6, -2],
    [0, 2, -2, 8, 126, -6, 2, -2]
  ],
  [
    [0, 0, 0, 128, 0, 0, 0, 0],
    [0, 0, 0, 120, 8, 0, 0, 0],
    [0, 0, 0, 112, 16, 0, 0, 0],
    [0, 0, 0, 104, 24, 0, 0, 0],
    [0, 0, 0, 96, 32, 0, 0, 0],
    [0, 0, 0, 88, 40, 0, 0, 0],
    [0, 0, 0, 80, 48, 0, 0, 0],
    [0, 0, 0, 72, 56, 0, 0, 0],
    [0, 0, 0, 64, 64, 0, 0, 0],
    [0, 0, 0, 56, 72, 0, 0, 0],
    [0, 0, 0, 48, 80, 0, 0, 0],
    [0, 0, 0, 40, 88, 0, 0, 0],
    [0, 0, 0, 32, 96, 0, 0, 0],
    [0, 0, 0, 24, 104, 0, 0, 0],
    [0, 0, 0, 16, 112, 0, 0, 0],
    [0, 0, 0, 8, 120, 0, 0, 0]
  ],
  [
    [0, 0, 0, 128, 0, 0, 0, 0],
    [0, 0, -4, 126, 8, -2, 0, 0],
    [0, 0, -8, 122, 18, -4, 0, 0],
    [0, 0, -10, 116, 28, -6, 0, 0],
    [0, 0, -12, 110, 38, -8, 0, 0],
    [0, 0, -12, 102, 48, -10, 0, 0],
    [0, 0, -14, 94, 58, -10, 0, 0],
    [0, 0, -12, 84, 66, -10, 0, 0],
    [0, 0, -12, 76, 76, -12, 0, 0],
    [0, 0, -10, 66, 84, -12, 0, 0],
    [0, 0, -10, 58, 94, -14, 0, 0],
    [0, 0, -10, 48, 102, -12, 0, 0],
    [0, 0, -8, 38, 110, -12, 0, 0],
    [0, 0, -6, 28, 116, -10, 0, 0],
    [0, 0, -4, 18, 122, -8, 0, 0],
    [0, 0, -2, 8, 126, -4, 0, 0]
  ],
  [
    [0, 0, 0, 128, 0, 0, 0, 0],
    [0, 0, 30, 62, 34, 2, 0, 0],
    [0, 0, 26, 62, 36, 4, 0, 0],
    [0, 0, 22, 62, 40, 4, 0, 0],
    [0, 0, 20, 60, 42, 6, 0, 0],
    [0, 0, 18, 58, 44, 8, 0, 0],
    [0, 0, 16, 56, 46, 10, 0, 0],
    [0, 0, 14, 54, 48, 12, 0, 0],
    [0, 0, 12, 52, 52, 12, 0, 0],
    [0, 0, 12, 48, 54, 14, 0, 0],
    [0, 0, 10, 46, 56, 16, 0, 0],
    [0, 0, 8, 44, 58, 18, 0, 0],
    [0, 0, 6, 42, 60, 20, 0, 0],
    [0, 0, 4, 40, 62, 22, 0, 0],
    [0, 0, 4, 36, 62, 26, 0, 0],
    [0, 0, 2, 34, 62, 30, 0, 0]
  ]
];

#[cfg(all(target_arch = "x86_64", feature = "nasm"))]
mod nasm {
  use super::*;
  use crate::frame::*;

  use std::mem;

  type PutFn = unsafe extern fn(
    *mut u8,
    libc::ptrdiff_t,
    *const u8,
    libc::ptrdiff_t,
    i32,
    i32,
    i32,
    i32
  );

  macro_rules! decl_mc_fns {
    ($($func_name:ident),+) => {
      extern {
        $(
          fn $func_name(
            dst: *mut u8, dst_stride: libc::ptrdiff_t, src: *const u8,
            src_stride: libc::ptrdiff_t, w: i32, h: i32, mx: i32, my: i32
          );
        )*
      }
    }
  }

  decl_mc_fns!(
    rav1e_put_8tap_regular_avx2,
    rav1e_put_8tap_regular_smooth_avx2,
    rav1e_put_8tap_regular_sharp_avx2,
    rav1e_put_8tap_smooth_avx2,
    rav1e_put_8tap_smooth_regular_avx2,
    rav1e_put_8tap_smooth_sharp_avx2,
    rav1e_put_8tap_sharp_avx2,
    rav1e_put_8tap_sharp_regular_avx2,
    rav1e_put_8tap_sharp_smooth_avx2,
    rav1e_put_bilin_avx2
  );

  fn select_put_fn_avx2(mode_x: FilterMode, mode_y: FilterMode) -> PutFn {
    use self::FilterMode::*;
    match (mode_x, mode_y) {
      (REGULAR, REGULAR) => rav1e_put_8tap_regular_avx2,
      (REGULAR, SMOOTH) => rav1e_put_8tap_regular_smooth_avx2,
      (REGULAR, SHARP) => rav1e_put_8tap_regular_sharp_avx2,
      (SMOOTH, REGULAR) => rav1e_put_8tap_smooth_regular_avx2,
      (SMOOTH, SMOOTH) => rav1e_put_8tap_smooth_avx2,
      (SMOOTH, SHARP) => rav1e_put_8tap_smooth_sharp_avx2,
      (SHARP, REGULAR) => rav1e_put_8tap_sharp_regular_avx2,
      (SHARP, SMOOTH) => rav1e_put_8tap_sharp_smooth_avx2,
      (SHARP, SHARP) => rav1e_put_8tap_sharp_avx2,
      (BILINEAR, BILINEAR) => rav1e_put_bilin_avx2,
      (_, _) => unreachable!()
    }
  }

  type PrepFn =
    unsafe extern fn(*mut i16, *const u8, libc::ptrdiff_t, i32, i32, i32, i32);

  macro_rules! decl_mct_fns {
    ($($func_name:ident),+) => {
      extern {
        $(
          fn $func_name(
            tmp: *mut i16, src: *const u8, src_stride: libc::ptrdiff_t, w: i32,
            h: i32, mx: i32, my: i32
          );
        )*
      }
    }
  }

  decl_mct_fns!(
    rav1e_prep_8tap_regular_avx2,
    rav1e_prep_8tap_regular_smooth_avx2,
    rav1e_prep_8tap_regular_sharp_avx2,
    rav1e_prep_8tap_smooth_avx2,
    rav1e_prep_8tap_smooth_regular_avx2,
    rav1e_prep_8tap_smooth_sharp_avx2,
    rav1e_prep_8tap_sharp_avx2,
    rav1e_prep_8tap_sharp_regular_avx2,
    rav1e_prep_8tap_sharp_smooth_avx2,
    rav1e_prep_bilin_avx2
  );

  fn select_prep_fn_avx2(mode_x: FilterMode, mode_y: FilterMode) -> PrepFn {
    use self::FilterMode::*;
    match (mode_x, mode_y) {
      (REGULAR, REGULAR) => rav1e_prep_8tap_regular_avx2,
      (REGULAR, SMOOTH) => rav1e_prep_8tap_regular_smooth_avx2,
      (REGULAR, SHARP) => rav1e_prep_8tap_regular_sharp_avx2,
      (SMOOTH, REGULAR) => rav1e_prep_8tap_smooth_regular_avx2,
      (SMOOTH, SMOOTH) => rav1e_prep_8tap_smooth_avx2,
      (SMOOTH, SHARP) => rav1e_prep_8tap_smooth_sharp_avx2,
      (SHARP, REGULAR) => rav1e_prep_8tap_sharp_regular_avx2,
      (SHARP, SMOOTH) => rav1e_prep_8tap_sharp_smooth_avx2,
      (SHARP, SHARP) => rav1e_prep_8tap_sharp_avx2,
      (BILINEAR, BILINEAR) => rav1e_prep_bilin_avx2,
      (_, _) => unreachable!()
    }
  }

  extern {
    fn rav1e_avg_avx2(
      dst: *mut u8, dst_stride: libc::ptrdiff_t, tmp1: *const i16,
      tmp2: *const i16, w: i32, h: i32
    );
  }

  pub fn put_8tap<T: Pixel>(
    dst: &mut PlaneRegionMut<'_, T>, src: PlaneSlice<'_, T>, width: usize,
    height: usize, col_frac: i32, row_frac: i32, mode_x: FilterMode,
    mode_y: FilterMode, bit_depth: usize
  ) {
    if mem::size_of::<T>() == 1 && is_x86_feature_detected!("avx2") {
      debug_assert!(bit_depth == 8);
      let dst_stride = dst.plane_cfg.stride as isize;
      let src_stride = src.plane.cfg.stride as isize;
      unsafe {
        select_put_fn_avx2(mode_x, mode_y)(
          dst.data_ptr_mut() as *mut _,
          dst_stride,
          src.as_ptr() as *const _,
          src_stride,
          width as i32,
          height as i32,
          col_frac,
          row_frac
        );
      }
      return;
    }
    super::native::put_8tap(
      dst, src, width, height, col_frac, row_frac, mode_x, mode_y, bit_depth,
    );
  }

  pub fn prep_8tap<T: Pixel>(
    tmp: &mut [i16], src: PlaneSlice<'_, T>, width: usize, height: usize,
    col_frac: i32, row_frac: i32, mode_x: FilterMode, mode_y: FilterMode,
    bit_depth: usize
  ) {
    if mem::size_of::<T>() == 1 && is_x86_feature_detected!("avx2") {
      debug_assert!(bit_depth == 8);
      let src_stride = src.plane.cfg.stride as isize;
      unsafe {
        select_prep_fn_avx2(mode_x, mode_y)(
          tmp.as_mut_ptr(),
          src.as_ptr() as *const _,
          src_stride,
          width as i32,
          height as i32,
          col_frac,
          row_frac
        );
      }
      return;
    }
    super::native::prep_8tap(
      tmp, src, width, height, col_frac, row_frac, mode_x, mode_y, bit_depth
    );
  }

  pub fn mc_avg<T: Pixel>(
    dst: &mut PlaneRegionMut<'_, T>, tmp1: &[i16], tmp2: &[i16], width: usize,
    height: usize, bit_depth: usize
  ) {
    if mem::size_of::<T>() == 1 && is_x86_feature_detected!("avx2") {
      debug_assert!(bit_depth == 8);
      let dst_stride = dst.plane_cfg.stride as isize;
      unsafe {
        rav1e_avg_avx2(
          dst.data_ptr_mut() as *mut _,
          dst_stride,
          tmp1.as_ptr(),
          tmp2.as_ptr(),
          width as i32,
          height as i32
        );
      }
      return;
    }
    super::native::mc_avg(dst, tmp1, tmp2, width, height, bit_depth);
  }
}

mod native {
  use super::*;
  use num_traits::*;
  use crate::frame::*;
  use crate::util::*;

  unsafe fn run_filter<T: AsPrimitive<i32>>(
    src: *const T, stride: usize, filter: [i32; 8]
  ) -> i32 {
    filter
      .iter()
      .enumerate()
      .map(|(i, f)| {
        let p = src.add(i * stride);
        f * (*p).as_()
      })
      .sum::<i32>()
  }

  fn get_filter(
    mode: FilterMode, frac: i32, length: usize
  ) -> [i32; SUBPEL_FILTER_SIZE] {
    let filter_idx = if mode == FilterMode::BILINEAR || length > 4 {
      mode as usize
    } else {
      (mode as usize).min(1) + 4
    };
    SUBPEL_FILTERS[filter_idx][frac as usize]
  }

  pub fn put_8tap<T: Pixel>(
    dst: &mut PlaneRegionMut<'_, T>, src: PlaneSlice<'_, T>, width: usize,
    height: usize, col_frac: i32, row_frac: i32, mode_x: FilterMode,
    mode_y: FilterMode, bit_depth: usize
  ) {
    let ref_stride = src.plane.cfg.stride;
    let y_filter = get_filter(mode_y, row_frac, height);
    let x_filter = get_filter(mode_x, col_frac, width);
    let max_sample_val = ((1 << bit_depth) - 1) as i32;
    let intermediate_bits = 4 - if bit_depth == 12 { 2 } else { 0 };
    match (col_frac, row_frac) {
      (0, 0) => {
        for r in 0..height {
          let src_slice = &src[r];
          let dst_slice = &mut dst[r];
          dst_slice[..width].copy_from_slice(&src_slice[..width]);
        }
      }
      (0, _) => {
        let offset_slice = src.go_up(3);
        for r in 0..height {
          let src_slice = &offset_slice[r];
          let dst_slice = &mut dst[r];
          for c in 0..width {
            dst_slice[c] = T::cast_from(round_shift(
              unsafe {
                run_filter(
                  src_slice[c..].as_ptr(),
                  ref_stride,
                  y_filter
                )
              },
              7
            )
            .max(0)
            .min(max_sample_val));
          }
        }
      }
      (_, 0) => {
        let offset_slice = src.go_left(3);
        for r in 0..height {
          let src_slice = &offset_slice[r];
          let dst_slice = &mut dst[r];
          for c in 0..width {
            dst_slice[c] = T::cast_from(round_shift(
              round_shift(
                unsafe { run_filter(src_slice[c..].as_ptr(), 1, x_filter) },
                7 - intermediate_bits
              ),
              intermediate_bits
            )
            .max(0)
            .min(max_sample_val));
          }
        }
      }
      (_, _) => {
        let mut intermediate = [0 as i16; 8 * (128 + 7)];

        let offset_slice = src.go_left(3).go_up(3);
        for cg in (0..width).step_by(8) {
          for r in 0..height + 7 {
            let src_slice = &offset_slice[r];
            for c in cg..(cg + 8).min(width) {
              intermediate[8 * r + (c - cg)] = round_shift(
                unsafe { run_filter(src_slice[c..].as_ptr(), 1, x_filter) },
                7 - intermediate_bits
              ) as i16;
            }
          }

          for r in 0..height {
            let dst_slice = &mut dst[r];
            for c in cg..(cg + 8).min(width) {
              dst_slice[c] = T::cast_from(round_shift(
                unsafe { run_filter(intermediate[8 * r + c - cg..].as_ptr(), 8, y_filter) },
                7 + intermediate_bits
              )
              .max(0)
              .min(max_sample_val));
            }
          }
        }
      }
    }
  }

  pub fn prep_8tap<T: Pixel>(
    tmp: &mut [i16], src: PlaneSlice<'_, T>, width: usize, height: usize,
    col_frac: i32, row_frac: i32, mode_x: FilterMode, mode_y: FilterMode,
    bit_depth: usize
  ) {
    let ref_stride = src.plane.cfg.stride;
    let y_filter = get_filter(mode_y, row_frac, height);
    let x_filter = get_filter(mode_x, col_frac, width);
    let intermediate_bits = 4 - if bit_depth == 12 { 2 } else { 0 };
    match (col_frac, row_frac) {
      (0, 0) => {
        for r in 0..height {
          let src_slice = &src[r];
          for c in 0..width {
            tmp[r * width + c] =
              i16::cast_from(src_slice[c]) << intermediate_bits;
          }
        }
      }
      (0, _) => {
        let offset_slice = src.go_up(3);
        for r in 0..height {
          let src_slice = &offset_slice[r];
          for c in 0..width {
            tmp[r * width + c] = round_shift(
              unsafe {
                run_filter(
                  src_slice[c..].as_ptr(),
                  ref_stride,
                  y_filter
                )
              },
              7 - intermediate_bits
            ) as i16;
          }
        }
      }
      (_, 0) => {
        let offset_slice = src.go_left(3);
        for r in 0..height {
          let src_slice = &offset_slice[r];
          for c in 0..width {
            tmp[r * width + c] = round_shift(
              unsafe { run_filter(src_slice[c..].as_ptr(), 1, x_filter) },
              7 - intermediate_bits
            ) as i16;
          }
        }
      }
      (_, _) => {
        let mut intermediate = [0 as i16; 8 * (128 + 7)];

        let offset_slice = src.go_left(3).go_up(3);
        for cg in (0..width).step_by(8) {
          for r in 0..height + 7 {
            let src_slice = &offset_slice[r];
            for c in cg..(cg + 8).min(width) {
              intermediate[8 * r + (c - cg)] = round_shift(
                unsafe { run_filter(src_slice[c..].as_ptr(), 1, x_filter) },
                7 - intermediate_bits
              ) as i16;
            }
          }

          for r in 0..height {
            for c in cg..(cg + 8).min(width) {
              tmp[r * width + c] = round_shift(
                unsafe { run_filter(intermediate[8 * r + c - cg..].as_ptr(), 8, y_filter) },
                7
              ) as i16;
            }
          }
        }
      }
    }
  }

  pub fn mc_avg<T: Pixel>(
    dst: &mut PlaneRegionMut<'_, T>, tmp1: &[i16], tmp2: &[i16], width: usize,
    height: usize, bit_depth: usize
  ) {
    let max_sample_val = ((1 << bit_depth) - 1) as i32;
    let intermediate_bits = 4 - if bit_depth == 12 { 2 } else { 0 };
    for r in 0..height {
      let dst_slice = &mut dst[r];
      for c in 0..width {
        dst_slice[c] = T::cast_from(round_shift(
          tmp1[r * width + c] as i32 + tmp2[r * width + c] as i32,
          intermediate_bits + 1
        )
        .max(0)
        .min(max_sample_val));
      }
    }
  }
}
