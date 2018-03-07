macro_rules! lerp {
  ($bits:expr, $a:expr, $b:expr, $p:expr) => {
      u32::from($a).wrapping_add(u32::from($b).wrapping_sub(u32::from($a)).wrapping_mul(u32::from($p)).wrapping_shr($bits))
  };
}

macro_rules! tu32 {
    ($a:expr) => { u32::from($a) }
}

macro_rules! impl_add {
    ($type: ident, $add: expr) => {
        impl Add<$type> for $type {
            type Output = $type;
            fn add(self, rhs: $type) -> $type {
                $add(self, rhs)
            }
        }
    }
}

macro_rules! impl_sub {
    ($type: ident, $sub: expr) => {
        impl Sub<$type> for $type {
            type Output = $type;
            fn sub(self, rhs: $type) -> $type {
                $sub(self, rhs)
            }
        }
    }
}

macro_rules! impl_mul {
    ($type: ident, $mul: expr) => {
        impl Mul<$type> for $type {
            type Output = $type;
            fn mul(self, rhs: $type) -> $type {
                $mul(self, rhs)
            }
        }
    }
}

macro_rules! impl_div {
    ($type: ident, $div: expr) => {
        impl Div<$type> for $type {
            type Output = $type;
            fn div(self, rhs: $type) -> $type {
                $div(self, rhs)
            }
        }
    }
}

macro_rules! impl_neg {
    ($type: ident, $neg: expr) => {
        impl Neg for $type {
            type Output = $type;
            fn neg(self) -> $type {
                $neg(self)
            }
        }
    }
}
