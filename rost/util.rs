pub fn range(start: uint, end: uint, f: |uint|) {
    let mut i = start;
    while i < end {
        f(i);
        i += 1;
    }
}

static CHARS: &'static str = "0123456789abcdef";

pub fn convert(value: u32, f: |char|) {
    convert_radix(value, 10, f);
}

pub fn convert_radix(value: u32, radix: u32, f: |char|) {
    let mut result: [u8, ..20] = ['0' as u8, ..20];

    match radix {
        2 => { f('0'); f('b'); },
        16 => { f('0'); f('x'); }
        _ => {}
    }

    let mut n = value;
    if (n == 0) {
        f('0');
    } else if (n < 0) {
        n = -n;
        f('-');
    }

    let mut length = 0;
    while n > 0 {
        result[length] = CHARS[n % radix] as u8;
        n /= radix;
        length += 1;
    }

    while (length > 0) {
        f(result[length - 1] as char);
        length -= 1;
    }
}