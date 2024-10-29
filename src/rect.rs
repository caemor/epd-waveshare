//! Rectangle operations for bigger displays with multiple _windows_
use core::cmp;

/// A rectangle
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq)]
pub struct Rect {
    /// Origin X
    pub x: u32,
    /// Origin Y
    pub y: u32,
    /// Width
    pub w: u32,
    /// Height
    pub h: u32,
}

impl Rect {
    /// Construct a new rectangle
    pub const fn new(x: u32, y: u32, w: u32, h: u32) -> Rect {
        Rect { x, y, w, h }
    }
    /// Compute intersection with another rectangle
    pub fn intersect(&self, other: Rect) -> Rect {
        let x = cmp::max(self.x, other.x);
        let y = cmp::max(self.y, other.y);
        let w = cmp::min(self.x + self.w, other.x + other.w).saturating_sub(x);
        let h = cmp::min(self.y + self.h, other.y + other.h).saturating_sub(y);
        Rect { x, y, w, h }
    }
    /// Move rectangle by (-dx,-dy)
    pub fn sub_offset(&self, dx: u32, dy: u32) -> Rect {
        Rect {
            x: self.x - dx,
            y: self.y - dy,
            w: self.w,
            h: self.h,
        }
    }
    /// Test whether the rectangle is empty.
    pub fn is_empty(&self) -> bool {
        self.w == 0 || self.h == 0
    }
}

#[test]
fn test_intersect() {
    let r1 = Rect::new(0, 0, 10, 10);
    let r2 = Rect::new(6, 3, 10, 10);
    let r3 = r1.intersect(r2);
    assert!(matches!(
        r3,
        Rect {
            x: 6,
            y: 3,
            w: 4,
            h: 7
        }
    ));

    let r1 = Rect::new(0, 0, 10, 10);
    let r2 = Rect::new(10, 11, 10, 10);
    let r3 = r1.intersect(r2);
    assert!(matches!(
        r3,
        Rect {
            x: _,
            y: _,
            w: 0,
            h: 0
        }
    ));
}

#[test]
fn sub_offset() {
    let r1 = Rect::new(10, 10, 10, 10);
    let r2 = r1.sub_offset(10, 5);
    assert!(matches!(
        r2,
        Rect {
            x: 0,
            y: 5,
            w: 10,
            h: 10
        }
    ));
}
