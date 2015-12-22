#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct U4x4(u16);
impl U4x4 {
    fn new() -> Self { U4x4(0) }
    fn get(self, i: u8) -> u8 {
        assert!(i < 4);
        (self.0 >> (i * 4) & 0xF) as u8
    }
    fn clear(self, i: u8) -> Self {
        assert!(i < 4);
        U4x4(self.0 & !(0xF << (i * 4)))
    }
    fn set(self, i: u8, val: u8) -> Self {
        assert!(val < 16);
        U4x4(self.clear(i).0 | (val as u16) << (i * 4))
    }
    fn set_if_zero(self, i: u8, val: u8) -> Option<Self> {
        if self.get(i) == 0 {
            Some(U4x4(self.0 | (val as u16) << (i * 4)))
        } else {
            None
        }
    }
    fn checked_decr_at(self, i: u8) -> Option<Self> {
        assert!(i < 4);
        let bits = self.0 - (1 << (i * 4));
        if !bits & 0xF << (i * 4) == 0 {
            None
        } else {
            Some(U4x4(bits))
        }
    }
}
