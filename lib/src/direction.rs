const DEGREES: i32 = 360;

/// Rotation, in degrees.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Rotation(i32);

/// Direction, in degrees.
///
/// Invariant: value in [0, DEGREES).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Direction(u32);

impl Direction {
    pub fn new(mut dir: i32) -> Self {
        while dir < 0 {
            dir += DEGREES;
        }
        Self((dir % DEGREES) as u32)
    }
}

impl std::ops::Add<Rotation> for Direction {
    type Output = Self;
    fn add(self, other: Rotation) -> Self {
        Self::new(self.0 as i32 + other.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        let values = [
            (10, 10, 20),
            (10, -10, 0),
            (350, 10, 0),
            (350, -10, 340),
            (350, 20, 10),
            (350, 380, 10),
        ];
        for (start, delta, want) in values {
            let start = Direction::new(start);
            let delta = Rotation(delta);
            let want = Direction::new(want);
            let got = start + delta;
            assert_eq!(got, want, "{start:?} + {delta:?} should be {want:?}");
        }
    }
}
