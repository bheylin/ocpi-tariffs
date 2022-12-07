use std::collections::HashSet;

use chrono::{Duration, NaiveDate, NaiveTime, Weekday};

use crate::ocpi::tariff::OcpiTariffRestriction;
use crate::ocpi::Number;
use crate::{ChargePeriod, Error};

pub fn collect_restrictions(
    restriction: &OcpiTariffRestriction,
) -> Result<Vec<Restriction>, Error> {
    let mut collected = Vec::new();

    match (&restriction.start_time, &restriction.end_time) {
        (Some(start_time), Some(end_time)) if end_time < start_time => {
            collected.push(Restriction::WrappingTime {
                start_time: start_time.parse()?,
                end_time: end_time.parse()?,
            })
        }
        (start_time, end_time) => {
            if let Some(start_time) = start_time {
                collected.push(Restriction::StartTime(start_time.parse()?))
            }

            if let Some(end_time) = end_time {
                collected.push(Restriction::EndTime(end_time.parse()?))
            }
        }
    }

    if let Some(start_date) = &restriction.start_date {
        collected.push(Restriction::StartDate(start_date.parse()?))
    }

    if let Some(end_date) = &restriction.end_date {
        collected.push(Restriction::EndDate(end_date.parse()?))
    }

    if let Some(min_kwh) = restriction.min_kwh {
        collected.push(Restriction::MinKwh(min_kwh))
    }

    if let Some(max_kwh) = restriction.max_kwh {
        collected.push(Restriction::MaxKwh(max_kwh))
    }

    if let Some(min_current) = restriction.min_current {
        collected.push(Restriction::MinCurrent(min_current))
    }

    if let Some(max_current) = restriction.max_current {
        collected.push(Restriction::MaxCurrent(max_current))
    }

    if let Some(min_power) = restriction.min_power {
        collected.push(Restriction::MinPower(min_power))
    }

    if let Some(max_power) = restriction.max_power {
        collected.push(Restriction::MaxPower(max_power))
    }

    if let Some(min_duration) = restriction.min_duration {
        collected.push(Restriction::MinDuration(Duration::seconds(min_duration)))
    }

    if let Some(max_duration) = restriction.max_duration {
        collected.push(Restriction::MaxDuration(Duration::seconds(max_duration)))
    }

    if !restriction.day_of_week.is_empty() {
        collected.push(Restriction::DayOfWeek(HashSet::from_iter(
            restriction.day_of_week.iter().copied().map(Into::into),
        )))
    }

    Ok(collected)
}

#[derive(Debug)]
pub enum Restriction {
    StartTime(NaiveTime),
    EndTime(NaiveTime),
    WrappingTime {
        start_time: NaiveTime,
        end_time: NaiveTime,
    },
    StartDate(NaiveDate),
    EndDate(NaiveDate),
    MinKwh(Number),
    MaxKwh(Number),
    MinCurrent(Number),
    MaxCurrent(Number),
    MinPower(Number),
    MaxPower(Number),
    MinDuration(Duration),
    MaxDuration(Duration),
    DayOfWeek(HashSet<Weekday>),
    Reservation,
}

impl Restriction {
    pub fn is_valid(&self, period: &ChargePeriod) -> Option<bool> {
        match self {
            &Self::WrappingTime {
                start_time,
                end_time,
            } => Some(
                period.local_start_time() >= start_time || period.local_start_time() < end_time,
            ),
            &Self::StartTime(start_time) => Some(period.local_start_time() >= start_time),
            &Self::EndTime(end_time) => Some(period.local_start_time() < end_time),
            &Self::StartDate(start_date) => Some(period.local_start_date() >= start_date),
            &Self::EndDate(end_date) => Some(period.local_start_date() < end_date),
            &Self::MinKwh(min_energy) => period
                .start_aggregate
                .energy
                .map(|energy| energy >= min_energy),
            &Self::MaxKwh(max_energy) => period
                .start_aggregate
                .energy
                .map(|energy| energy < max_energy),
            &Self::MinDuration(min_duration) => period
                .start_aggregate
                .duration
                .map(|duration| duration >= min_duration),
            &Self::MaxDuration(max_duration) => period
                .start_aggregate
                .duration
                .map(|duration| duration < max_duration),
            &Self::MinCurrent(min_current) => period
                .state
                .min_current
                .map(|current| current >= min_current),
            &Self::MaxCurrent(max_current) => period
                .state
                .max_current
                .map(|current| current < max_current),
            &Self::MinPower(min_power) => period.state.min_power.map(|power| power >= min_power),
            &Self::MaxPower(max_power) => period.state.max_power.map(|power| power < max_power),
            Self::DayOfWeek(days) => Some(days.contains(&period.local_start_weekday())),
            &Self::Reservation => todo!(),
        }
    }
}
