# Arbitrary-Precision Floating-Point Library

ARPFloat is an implementation of arbitrary precision floating point data
structures and utilities. The library can be used to emulate floating point
operation, in software, or create new floating point data types.

### Example
```
fn test_readme_example() {
    // Create a new type: 15 bits exponent, 112 significand.
    type FP128 = Float<15, 112>;

    // Use Newton-Raphson to find the square root of 5.
    let n = FP128::from_u64(5);

    let two = FP128::from_u64(2);
    let mut x = n;

    for _ in 0..1000 {
        x = (x + (n / x))/two;
    }

    println!("fp128: {}", x);
    println!("fp64:  {}", x.as_f64());
}```

The program above will print this output:
```
fp128: 2.2360679774997896964091736687312763
fp64:  2.23606797749979
```

The library also provides API that exposes rounding modes, and low-level
operations.
```rust
    // Explicit control over rounding modes:
    let val = FP128::mul_with_rm(y, z, RoundingMode::NearestTiesToEven);

    // View the internals of the float:
    let fp = FP16::from_i64(15);
    let m = fp.get_mantissa();

    // Prints FP[+ E=+3 M=11110000000]
    fp.dump();
```


#### License

Licensed under Apache-2.0
