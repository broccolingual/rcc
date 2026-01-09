use core::fmt;

pub(crate) trait AlignUp {
    fn align_up(&self, align: usize) -> usize;
}

impl AlignUp for usize {
    // alignの倍数に切り上げる
    fn align_up(&self, align: usize) -> usize {
        (*self + align - 1) & !(align - 1)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub(crate) struct Span {
    pub(crate) start: usize,
    pub(crate) end: usize,
}

impl Span {
    pub(crate) const fn new(start: usize, end: usize) -> Self {
        Span { start, end }
    }

    #[allow(dead_code)]
    pub(crate) const fn len(&self) -> usize {
        self.end.saturating_sub(self.start)
    }

    #[allow(dead_code)]
    pub(crate) const fn is_empty(&self) -> bool {
        self.start == self.end
    }

    #[allow(dead_code)]
    pub(crate) fn contains(&self, pos: usize) -> bool {
        pos >= self.start && pos < self.end
    }

    #[allow(dead_code)]
    pub(crate) fn merge(self, other: Span) -> Span {
        Span { start: self.start.min(other.start), end: self.end.max(other.end) }
    }
}

impl fmt::Debug for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}..{}", self.start, self.end)
    }
}

impl From<(usize, usize)> for Span {
    fn from((start, end): (usize, usize)) -> Self {
        Span { start, end }
    }
}

impl From<Span> for (usize, usize) {
    fn from(span: Span) -> Self {
        (span.start, span.end)
    }
}
