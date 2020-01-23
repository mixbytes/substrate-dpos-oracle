use codec::{Decode, Encode};
use rstd::ops::{Add, Div, Mul, Rem, Shr, Sub};
use sr_primitives::traits::One;

#[derive(Encode, Decode, Clone, Eq, Default, PartialEq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct PeriodHandler<Time>
{
    start: Time,
    calculate_period: Time,
    aggregate_period: Time,
    last_sources_update: Time,
}

impl<Time: Default + PartialOrd<Time>> PeriodHandler<Time>
{
    pub fn new(
        now: Time,
        calculate_period: Time,
        aggregate_period: Time,
    ) -> Result<PeriodHandler<Time>, &'static str>
    {
        if calculate_period <= aggregate_period
        {
            Err("Wrong period params.")
        }
        else
        {
            Ok(PeriodHandler {
                calculate_period,
                aggregate_period,
                start: now,
                last_sources_update: Time::default(),
            })
        }
    }
}

impl<
        Time: One
            + Add<Time, Output = Time>
            + Sub<Time, Output = Time>
            + Mul<Time, Output = Time>
            + Div<Time, Output = Time>
            + PartialOrd<Time>
            + Ord
            + Copy,
    > PeriodHandler<Time>
{
    pub fn get_period(&self, now: Time) -> Time
    {
        (now - self.start) / self.calculate_period
    }

    pub fn is_aggregate_time(&self, now: Time) -> bool
    {
        let next_period = self.get_period(now) + One::one();
        let next_period_begin = self.start + next_period * self.calculate_period;

        (next_period_begin - now) <= self.aggregate_period
    }

    pub fn is_calculate_time(&self, last_update_time: Option<Time>, now: Time) -> bool
    {
        match last_update_time
        {
            Some(last_changed) => self.get_period(now) > self.get_period(last_changed),
            None => true,
        }
    }

    pub fn update_source_time(&mut self, now: Time)
    {
        self.last_sources_update = now;
    }

    pub fn is_source_update_needed(&self, now: Time) -> bool
    {
        self.is_aggregate_time(now)
            && self.get_period(self.last_sources_update) < self.get_period(now)
    }
}

#[cfg(test)]
mod tests
{
    type PeriodHandler = super::PeriodHandler<u8>;

    #[test]
    fn get_period()
    {
        let handler = PeriodHandler::new(100, 10, 5).unwrap();

        assert_eq!(handler.get_period(100), 0);
        assert_eq!(handler.get_period(109), 0);
        assert_eq!(handler.get_period(110), 1);
        assert_eq!(handler.get_period(121), 2);
    }

    #[test]
    fn is_aggregate_time()
    {
        let handler = PeriodHandler::new(100, 10, 5).unwrap();

        (200..=204).for_each(|now| assert!(!handler.is_aggregate_time(now)));
        (205..=209).for_each(|now| assert!(handler.is_aggregate_time(now)));
    }

    #[test]
    fn is_calculate_time()
    {
        let handler = PeriodHandler::new(100, 10, 5).unwrap();

        assert!(handler.is_calculate_time(None, 100));
        assert!(handler.is_calculate_time(Some(100), 110));
        assert!(!handler.is_calculate_time(Some(100), 101));
    }

    #[test]
    fn is_source_update_needed()
    {
        let mut handler = PeriodHandler::new(100, 10, 5).unwrap();
        handler.update_source_time(105);

        (106..=114).for_each(|now| assert!(!handler.is_source_update_needed(now)));
        assert!(handler.is_source_update_needed(115));
    }
}
