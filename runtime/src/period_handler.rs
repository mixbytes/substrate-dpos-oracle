use codec::{Decode, Encode};
use sr_primitives::traits::One;
use sr_primitives::traits::SimpleArithmetic;

#[derive(Encode, Decode, Clone, Eq, Default, PartialEq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct PeriodHandler<Time>
{
    start: Time,
    calculate_period: Time,
    aggregate_period: Time,
    last_sources_update: Time,
}

impl<Time: Default> PeriodHandler<Time>
{
    pub fn new(now: Time, calculate_period: Time, aggregate_period: Time) -> PeriodHandler<Time>
    {
        PeriodHandler {
            calculate_period,
            aggregate_period,
            start: now,
            last_sources_update: Time::default(),
        }
    }
}

impl<Time: SimpleArithmetic + Copy> PeriodHandler<Time>
{
    pub fn get_period(&self, now: Time) -> Time
    {
        (now - self.start) % self.calculate_period
    }

    pub fn is_aggregate_time(&self, now: Time) -> bool
    {
        ((self.get_period(now) + One::one()) * self.calculate_period - now) < self.aggregate_period
    }

    pub fn is_calculate_time(&self, last_update_time: Option<Time>, now: Time) -> bool
    {
        match last_update_time
        {
            Some(last_changed) => self.get_period(now) > self.get_period(last_changed),
            None => true,
        }
    }

    pub fn is_source_update_time(&self, now: Time) -> bool
    {
        self.is_aggregate_time(now)
            && self.get_period(self.last_sources_update) < self.get_period(now)
    }
}
