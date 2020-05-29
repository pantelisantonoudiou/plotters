use super::{AsRangedCoord, DiscreteRanged, KeyPointHint, Ranged, ValueFormatter};
use std::ops::Range;

/// The ranged value spec that needs to be grouped.
/// This is useful, for example, when we have an X axis is a integer and denotes days.
/// And we are expecting the tick mark denotes weeks, in this way we can make the range
/// spec grouping by 7 elements.
/// With the help of the GroupBy decorator, this can be archived quite easily:
///```rust
///use plotters::prelude::*;
///let mut buf = vec![0;1024*768*3];
///let area = BitMapBackend::with_buffer(buf.as_mut(), (1024, 768)).into_drawing_area();
///let chart = ChartBuilder::on(&area)
///    .build_ranged((0..100).group_by(7), 0..100)
///    .unwrap();
///```
#[derive(Clone)]
pub struct GroupBy<T: DiscreteRanged>(T, usize);

/// The trait that provides method `Self::group_by` function which creates a
/// `GroupBy` decorated ranged value.
pub trait ToGroupByRange: AsRangedCoord + Sized
where
    Self::CoordDescType: DiscreteRanged,
{
    /// Make a grouping ranged value, see the documentation for `GroupBy` for details.
    ///
    /// - `value`: The number of values we want to group it
    /// - **return**: The newly created grouping range specification
    fn group_by(self, value: usize) -> GroupBy<<Self as AsRangedCoord>::CoordDescType> {
        GroupBy(self.into(), value)
    }
}

impl<T: AsRangedCoord + Sized> ToGroupByRange for T where T::CoordDescType: DiscreteRanged {}

impl<T: DiscreteRanged> DiscreteRanged for GroupBy<T> {
    fn size(&self) -> usize {
        (self.0.size() + self.1 - 1) / self.1
    }
    fn index_of(&self, value: &Self::ValueType) -> Option<usize> {
        self.0.index_of(value).map(|idx| idx / self.1)
    }
    fn from_index(&self, index: usize) -> Option<Self::ValueType> {
        self.0.from_index(index * self.1)
    }
}

impl<T, R: DiscreteRanged<ValueType = T> + ValueFormatter<T>> ValueFormatter<T> for GroupBy<R> {
    fn format(value: &T) -> String {
        R::format(value)
    }
}

impl<T: DiscreteRanged> Ranged for GroupBy<T> {
    type FormatOption = crate::coord::ranged::NoDefaultFormatting;
    type ValueType = T::ValueType;
    fn map(&self, value: &T::ValueType, limit: (i32, i32)) -> i32 {
        self.0.map(value, limit)
    }
    fn range(&self) -> Range<T::ValueType> {
        self.0.range()
    }
    // TODO: See issue issue #88
    fn key_points<HintType: KeyPointHint>(&self, hint: HintType) -> Vec<T::ValueType> {
        let range = 0..(self.0.size() + self.1) / self.1;
        //let logic_range: RangedCoordusize = range.into();

        let interval =
            ((range.end - range.start + hint.bold_points() - 1) / hint.bold_points()).max(1);
        let count = (range.end - range.start) / interval;

        let idx_iter = (0..hint.bold_points()).map(|x| x * interval);

        if hint.weight().allow_light_points() && count < hint.bold_points() * 2 {
            let outter_ticks = idx_iter;
            let outter_tick_size = interval * self.1;
            let inner_ticks_per_group = hint.max_num_points() / outter_ticks.len();
            let inner_ticks =
                (outter_tick_size + inner_ticks_per_group - 1) / inner_ticks_per_group;
            let inner_ticks: Vec<_> = (0..(outter_tick_size / inner_ticks))
                .map(move |x| x * inner_ticks)
                .collect();
            let size = self.0.size();
            return outter_ticks
                .into_iter()
                .map(|base| inner_ticks.iter().map(move |&ofs| base * self.1 + ofs))
                .flatten()
                .take_while(|&idx| idx < size)
                .map(|x| self.0.from_index(x).unwrap())
                .collect();
        }

        idx_iter
            .into_iter()
            .map(|x| self.0.from_index(x * self.1).unwrap())
            .collect()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_group_by() {
        let coord = (0..100).group_by(10);
        assert_eq!(coord.size(), 11);
        for (idx, val) in (0..).zip(coord.values()) {
            assert_eq!(val, idx * 10);
            assert_eq!(coord.from_index(idx as usize), Some(val));
        }
    }
}
