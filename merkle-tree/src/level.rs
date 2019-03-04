#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Level {
    pub start: usize,
    pub end: usize,
}

impl Level {
    pub fn new(start: usize, end: usize) -> Self {
        Level { start, end }
    }

    pub fn down(&self) -> Option<Level> {
        if self.is_empty() {
            return None;
        }

        let half = (self.len() + 1) >> 1;
        Some(Level::new(self.end, self.end + half))
    }

    #[inline]
    pub fn contains(&self, index: usize) -> bool {
        index >= self.start && index < self.end
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        return self.start >= self.end;
    }

    #[inline]
    pub fn len(&self) -> usize {
        if self.is_empty() {
            return 0;
        }
        self.end - self.start
    }
}

#[derive(Clone, Debug)]
pub(crate) struct IndexedLevel {
    pub index: usize,
    pub level: Level,
}

impl IndexedLevel {
    pub fn new(index: usize, start: usize, end: usize) -> Option<Self> {
        let level = Level { start, end };

        if level.contains(index) && !level.is_empty() {
            Some(IndexedLevel { index, level })
        } else {
            None
        }
    }

    pub fn down(&self) -> Option<IndexedLevel> {
        if let Some(level) = self.level.down() {
            Some(IndexedLevel {
                index: self.parent(),
                level,
            })
        } else {
            None
        }
    }

    pub fn siblings(&self) -> [Option<usize>; 2] {
        let left;
        let right;

        if (self.index - self.level.start) & 1 == 1 {
            left = Some(self.index - 1);
            right = Some(self.index);
        } else {
            left = Some(self.index);
            right = if self.index == self.level.end - 1 {
                None
            } else {
                Some(self.index + 1)
            };
        }

        [left, right]
    }

    pub fn sibling(&self) -> Option<usize> {
        if (self.index - self.level.start) & 1 == 1 {
            Some(self.index - 1)
        } else if self.index == self.level.end - 1 {
            None
        } else {
            Some(self.index + 1)
        }
    }

    pub fn parent(&self) -> usize {
        self.level.end + ((self.index - self.level.start) >> 1)
    }
}

impl PartialEq for IndexedLevel {
    fn eq(&self, other: &IndexedLevel) -> bool {
        self.level == other.level && self.index == other.index
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_level_len() {
        let mut level;

        level = Level::new(0, 0);
        assert_eq!(level.is_empty(), true);
        assert_eq!(level.len(), 0);

        level = Level::new(100, 0);
        assert_eq!(level.is_empty(), true);
        assert_eq!(level.len(), 0);

        level = Level::new(1, 101);
        assert_eq!(level.is_empty(), false);
        assert_eq!(level.len(), 100);
    }

    #[test]
    fn test_level_contains() {
        let mut level;

        level = Level::new(0, 0);
        assert_eq!(level.contains(0), false);
        assert_eq!(level.contains(1), false);

        level = Level::new(100, 0);
        assert_eq!(level.contains(0), false);
        assert_eq!(level.contains(50), false);
        assert_eq!(level.contains(100), false);

        level = Level::new(1, 101);
        assert_eq!(level.contains(0), false);
        assert_eq!(level.contains(1), true);
        assert_eq!(level.contains(100), true);
        assert_eq!(level.contains(101), false);
    }

    #[test]
    fn test_level_lower() {
        let mut level;

        level = Level::new(0, 0);
        assert_eq!(level.down(), None);

        level = Level::new(0, 1);
        assert_eq!(level.down(), Some(Level::new(1, 2)));

        level = Level::new(13, 13);
        assert_eq!(level.down(), None);

        level = Level::new(100, 0);
        assert_eq!(level.down(), None);

        level = Level::new(10, 20);
        assert_eq!(level.down(), Some(Level::new(20, 25)));
        assert_eq!(level.down().unwrap().len(), 5);

        level = Level::new(10, 19);
        assert_eq!(level.down(), Some(Level::new(19, 24)));
        assert_eq!(level.down().unwrap().len(), 5);
    }

    #[test]
    fn test_indexed_level_new() {
        assert_eq!(IndexedLevel::new(1, 2, 3).is_some(), false);
        assert_eq!(IndexedLevel::new(2, 2, 2).is_some(), false);
        assert_eq!(IndexedLevel::new(1, 3, 2).is_some(), false);
    }

    #[test]
    fn test_indexed_level_siblings() {
        let mut level;

        level = IndexedLevel::new(3, 3, 13).unwrap();
        assert_eq!(level.siblings(), [Some(3), Some(4)]);

        level = IndexedLevel::new(8, 3, 13).unwrap();
        assert_eq!(level.siblings(), [Some(7), Some(8)]);

        level = IndexedLevel::new(12, 3, 13).unwrap();
        assert_eq!(level.siblings(), [Some(11), Some(12)]);

        level = IndexedLevel::new(11, 3, 12).unwrap();
        assert_eq!(level.siblings(), [Some(11), None]);
    }

    #[test]
    fn test_indexed_level_parent() {
        let mut level;

        level = IndexedLevel::new(3, 3, 5).unwrap();
        assert_eq!(level.parent(), 5);

        level = IndexedLevel::new(3, 3, 13).unwrap();
        assert_eq!(level.parent(), 13);

        level = IndexedLevel::new(4, 3, 13).unwrap();
        assert_eq!(level.parent(), 13);

        level = IndexedLevel::new(5, 3, 13).unwrap();
        assert_eq!(level.parent(), 14);

        level = IndexedLevel::new(6, 3, 13).unwrap();
        assert_eq!(level.parent(), 14);

        level = IndexedLevel::new(7, 3, 13).unwrap();
        assert_eq!(level.parent(), 15);
    }
}
