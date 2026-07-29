use crate::{
    ComponentRangeError, ParseMode, ParseTimeOfDayError, QuantizeMode, Resolution, ResolutionLoss,
    TimeOfDay, value::NANOSECONDS_PER_DAY,
};

pub(crate) struct ParsedTime {
    pub(crate) fractional_digits: u8,
    pub(crate) has_seconds:       bool,
    pub(crate) nanoseconds:       u64,
}

pub(crate) fn parse_for<R: Resolution>(
    input: &str,
    mode: ParseMode,
) -> Result<TimeOfDay<R>, ParseTimeOfDayError> {
    // The raw parser validates the original value against 24:00 before any lossy quantization.
    let parsed = parse_raw(input)?;

    if mode == ParseMode::Strict && !is_canonical_for::<R>(input, &parsed) {
        return Err(ParseTimeOfDayError::InvalidSyntax);
    }

    let quantize = match mode {
        ParseMode::Strict | ParseMode::Relaxed => QuantizeMode::Exact,
        ParseMode::Truncate => QuantizeMode::Truncate,
        ParseMode::Ceil => QuantizeMode::Ceil,
        ParseMode::Round => QuantizeMode::Round,
    };

    quantize_nanoseconds::<R>(parsed.nanoseconds, quantize)
}

pub(crate) fn parse_raw(input: &str) -> Result<ParsedTime, ParseTimeOfDayError> {
    let bytes = input.as_bytes();
    if bytes.len() < 5
        || bytes[2] != b':'
        || !bytes[0..2].iter().all(u8::is_ascii_digit)
        || !bytes[3..5].iter().all(u8::is_ascii_digit)
    {
        return Err(ParseTimeOfDayError::InvalidSyntax);
    }

    let hour = decimal_pair(&bytes[0..2]);
    let minute = decimal_pair(&bytes[3..5]);
    let mut second = 0;
    let mut nanosecond = 0;
    let mut has_seconds = false;
    let mut fractional_digits = 0;

    if bytes.len() > 5 {
        if bytes.len() < 8 || bytes[5] != b':' || !bytes[6..8].iter().all(u8::is_ascii_digit) {
            return Err(ParseTimeOfDayError::InvalidSyntax);
        }

        has_seconds = true;

        second = decimal_pair(&bytes[6..8]);

        if bytes.len() > 8 {
            let fraction = bytes.get(9..).ok_or(ParseTimeOfDayError::InvalidSyntax)?;

            if bytes[8] != b'.'
                || fraction.is_empty()
                || fraction.len() > 9
                || !fraction.iter().all(u8::is_ascii_digit)
            {
                return Err(ParseTimeOfDayError::InvalidSyntax);
            }

            fractional_digits = fraction.len() as u8;

            // Right padding converts the declared decimal fraction into a nanosecond fraction without allocation.
            nanosecond =
                fraction.iter().fold(0_u32, |value, digit| value * 10 + u32::from(*digit - b'0'))
                    * 10_u32.pow(9 - u32::from(fractional_digits));
        }
    }

    validate_parsed_components(hour, minute, second, nanosecond)?;

    let nanoseconds = u64::from(hour) * 3_600_000_000_000
        + u64::from(minute) * 60_000_000_000
        + u64::from(second) * 1_000_000_000
        + u64::from(nanosecond);

    if nanoseconds > NANOSECONDS_PER_DAY {
        return Err(ParseTimeOfDayError::AfterEndOfDay);
    }

    Ok(ParsedTime {
        fractional_digits,
        has_seconds,
        nanoseconds,
    })
}

fn decimal_pair(bytes: &[u8]) -> u8 {
    (bytes[0] - b'0') * 10 + bytes[1] - b'0'
}

fn validate_parsed_components(
    hour: u8,
    minute: u8,
    second: u8,
    nanosecond: u32,
) -> Result<(), ParseTimeOfDayError> {
    let error = if hour > 24 {
        Some(ComponentRangeError::Hour(hour))
    } else if minute > 59 {
        Some(ComponentRangeError::Minute(minute))
    } else if second > 59 {
        Some(ComponentRangeError::Second(second))
    } else if hour == 24 && (minute != 0 || second != 0 || nanosecond != 0) {
        return Err(ParseTimeOfDayError::AfterEndOfDay);
    } else {
        None
    };

    match error {
        Some(error) => Err(ParseTimeOfDayError::ComponentOutOfRange(error)),
        None => Ok(()),
    }
}

fn is_canonical_for<R: Resolution>(input: &str, parsed: &ParsedTime) -> bool {
    match R::KIND {
        crate::ResolutionKind::Minute => !parsed.has_seconds && input.len() == 5,
        crate::ResolutionKind::Second => {
            parsed.has_seconds && parsed.fractional_digits == 0 && input.len() == 8
        },
        _ => {
            parsed.has_seconds
                && parsed.fractional_digits == R::CANONICAL_FRACTIONAL_DIGITS
                && input.len() == 9 + usize::from(R::CANONICAL_FRACTIONAL_DIGITS)
        },
    }
}

fn quantize_nanoseconds<R: Resolution>(
    nanoseconds: u64,
    mode: QuantizeMode,
) -> Result<TimeOfDay<R>, ParseTimeOfDayError> {
    let tick = R::NANOSECONDS_PER_TICK;
    let quotient = nanoseconds / tick;
    let remainder = nanoseconds % tick;

    // The domain is nonnegative, so half-up rounding always moves a tie toward the end of day.
    let ticks = match mode {
        QuantizeMode::Exact if remainder != 0 => {
            return Err(ParseTimeOfDayError::ResolutionLoss(ResolutionLoss {
                nanoseconds,
                target_tick_nanoseconds: tick,
            }));
        },
        QuantizeMode::Exact | QuantizeMode::Truncate => quotient,
        QuantizeMode::Ceil => quotient + u64::from(remainder != 0),
        QuantizeMode::Round => quotient + u64::from(remainder >= tick - remainder),
    };

    if ticks > R::TICKS_PER_DAY {
        return Err(ParseTimeOfDayError::RoundingOverflow);
    }

    Ok(TimeOfDay::from_valid_ticks(ticks))
}
