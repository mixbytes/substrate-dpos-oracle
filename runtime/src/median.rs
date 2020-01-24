use rstd::prelude::*;

extern crate alloc;

use rstd::cmp::Ord;

#[derive(PartialEq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub enum Median<T>
{
    Value(T),
    Pair(T, T),
}

pub fn get_median<T: Ord + Copy>(mut values: Vec<T>) -> Option<Median<T>>
{
    values.sort();

    let middle = values.len() / 2;
    match values.len()
    {
        0 | 1 => None,
        len if len % 2 == 0 => Some(Median::Pair(values[middle - 1], values[middle + 1])),
        _len => Some(Median::Value(values[middle])),
    }
}

#[cfg(test)]
mod tests
{
    use super::{get_median, Median};

    #[test]
    fn simple()
    {
        let array: Vec<u8> = (0..=10).collect();
        let median = array[5];
        assert_eq!(get_median(array), Some(Median::Value(median)));
    }
}
