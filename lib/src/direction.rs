const DEGREES: i32 = 360;

/// Rotation, in degrees.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Rotation(i32);

/// Direction, in degrees.
///
/// Invariant: value in [0, DEGREES).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Direction(u32);

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

impl std::ops::AddAssign<Rotation> for Direction {
    fn add_assign(&mut self, other: Rotation) {
        self.0 = Direction::new(self.0 as i32 + other.0).0;
    }
}

impl std::ops::Sub<Direction> for Direction {
    type Output = Rotation;
    fn sub(self, other: Direction) -> Rotation {
        Rotation(self.0 as i32 - other.0 as i32)
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
