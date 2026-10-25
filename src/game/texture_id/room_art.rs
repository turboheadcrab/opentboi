/// The style of a room's walls and floor.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RoomArt {
    Basement,
    Shop,
    Boss,
    Secret,
    /// One of the five styles (`hid2`).
    SuperSecret(u8),
    Curse,
    Sacrifice,
    Library,
}

impl RoomArt {
    pub const ALL: [Self; 12] = [
        Self::Basement,
        Self::Shop,
        Self::Boss,
        Self::Secret,
        Self::SuperSecret(0),
        Self::SuperSecret(1),
        Self::SuperSecret(2),
        Self::SuperSecret(3),
        Self::SuperSecret(4),
        Self::Curse,
        Self::Sacrifice,
        Self::Library,
    ];
}
