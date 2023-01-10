//!
//! ARPFloat is an implementation of arbitrary precision
//![floating point](https://en.wikipedia.org/wiki/IEEE_754) data
//!structures and utilities. The library can be used to emulate floating point
//!operation, in software, or create new floating point data types.

//!### Example
//!```
//!  use arpfloat::Float;
//!  use arpfloat::FP128;
//!
//!  // Create the number '5' in FP128 format.
//!  let n = Float::from_u64(FP128, 5);
//!
//!  // Use Newton-Raphson to find the square root of 5.
//!  let mut x = n.clone();
//!  for _ in 0..20 {
//!      x = (x.clone() + (&n / &x))/2;
//!  }
//!
//!  println!("fp128: {}", x);
//!  println!("fp64:  {}", x.as_f64());
//! ```
//!
//!
//!The program above will print this output:
//!```console
//!fp128: 2.2360679774997896964091736687312763
//!fp64:  2.23606797749979
//!```
//!
//!The library also provides API that exposes rounding modes, and low-level
//!operations.
//!
//!```
//!    use arpfloat::{FP16, FP128, RoundingMode};
//!    use arpfloat::Float;
//!
//!    let x = Float::from_u64(FP128, 1<<53);
//!    let y = Float::from_f64(1000.0).cast(FP128);
//!    let val = Float::mul_with_rm(&x, &y, RoundingMode::NearestTiesToEven);
//! ```
//!
//! View the internal representation of numbers:
//! ```
//!    use arpfloat::{Float, FP16, FP128, RoundingMode};
//!
//!    let fp = Float::from_i64(FP16, 15);
//!    let m = fp.get_mantissa();
//!
//!    // Prints FP[+ E=+3 M=11110000000]
//!    fp.dump();
//!```
//!
//! Control the rounding mode for type conversion:
//!```
//!    use arpfloat::{FP16, FP32, RoundingMode, Float};
//!    let x = Float::from_u64(FP32, 2649);
//!    let b = x.cast_with_rm(FP16, RoundingMode::Zero);
//!    println!("{}", b); // Prints 2648!
//!```

#![no_std]

#[cfg(feature = "std")]
extern crate std;

mod arithmetic;
mod bigint;
mod cast;
mod float;
mod functions;
mod string;
mod utils;

pub use self::bigint::BigInt;
pub use self::float::Float;
pub use self::float::RoundingMode;
pub use self::float::{FP128, FP16, FP256, FP32, FP64};
