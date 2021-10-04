use core::f32::NAN;
use core::intrinsics;

pub trait Float {
    fn sqrt(self) -> Self;
    fn powi(self, x: i32) -> Self;
    fn sin(self) -> Self;
    fn asin(self) -> Self;
    fn cos(self) -> Self;
    fn acos(self) -> Self;
    fn atan2(self, x: Self) -> Self;
    fn tan(self) -> Self;
    fn pow(self, x: Self) -> Self;
    fn exp(self) -> Self;
    fn exp2(self) -> Self;
    fn log(self) -> Self;
    fn log10(self) -> Self;
    fn log2(self) -> Self;
    fn fma(self, b: Self, c: Self) -> Self;
    fn abs(self) -> Self;
    fn copysign(self, y: Self) -> Self;
    fn floor(self) -> Self;
    fn ceil(self) -> Self;
    fn trunc(self) -> Self;
    fn round(self) -> Self;
    fn fract(self) -> Self;
}

extern "C" {
    fn tan(x: f64) -> f64;
    fn asin(x: f64) -> f64;
    fn acos(x: f64) -> f64;
    fn atan2(x: f64, y: f64) -> f64;
}

impl Float for f32 {
    #[inline]
    fn sqrt(self) -> Self {
        if self < 0.0 {
            NAN
        } else {
            unsafe { intrinsics::sqrtf32(self) }
        }
    }

    #[inline]
    fn powi(self, x: i32) -> Self {
        unsafe { intrinsics::powif32(self, x) }
    }

    #[inline]
    fn sin(self) -> Self {
        unsafe { intrinsics::sinf32(self) }
    }

    #[inline]
    fn asin(self) -> Self {
        unsafe { asin(self as f64) as f32 }
    }

    #[inline]
    fn cos(self) -> Self {
        unsafe { intrinsics::cosf32(self) }
    }

    #[inline]
    fn acos(self) -> Self {
        unsafe { acos(self as f64) as f32 }
    }

    #[inline]
    fn atan2(self, x: Self) -> Self {
        unsafe { atan2(self as f64, x as f64) as f32 }
    }

    #[inline]
    fn tan(self) -> Self {
        unsafe { tan(self as f64) as f32 }
    }

    #[inline]
    fn pow(self, x: Self) -> Self {
        unsafe { intrinsics::powf32(self, x) }
    }

    #[inline]
    fn exp(self) -> Self {
        unsafe { intrinsics::expf32(self) }
    }

    #[inline]
    fn exp2(self) -> Self {
        unsafe { intrinsics::exp2f32(self) }
    }

    #[inline]
    fn log(self) -> Self {
        unsafe { intrinsics::logf32(self) }
    }

    #[inline]
    fn log10(self) -> Self {
        unsafe { intrinsics::log10f32(self) }
    }

    #[inline]
    fn log2(self) -> Self {
        unsafe { intrinsics::log2f32(self) }
    }

    #[inline]
    fn fma(self, b: Self, c: Self) -> Self {
        unsafe { intrinsics::fmaf32(self, b, c) }
    }

    #[inline]
    fn abs(self) -> Self {
        unsafe { intrinsics::fabsf32(self) }
    }

    #[inline]
    fn copysign(self, y: Self) -> Self {
        unsafe { intrinsics::copysignf32(self, y) }
    }

    #[inline]
    fn floor(self) -> Self {
        unsafe { intrinsics::floorf32(self) }
    }

    #[inline]
    fn ceil(self) -> Self {
        unsafe { intrinsics::ceilf32(self) }
    }

    #[inline]
    fn trunc(self) -> Self {
        unsafe { intrinsics::truncf32(self) }
    }

    #[inline]
    fn round(self) -> Self {
        unsafe { intrinsics::roundf32(self) }
    }

    #[inline]
    fn fract(self) -> Self {
        self - self.trunc()
    }
}
