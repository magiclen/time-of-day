use core::fmt;

use crate::{
    FormatOptionError, FormatOptions, Resolution, ResolutionKind, TimeOfDay, TrailingZeros,
};

/// A borrowing display wrapper for a typed time of day.
pub struct FormattedTimeOfDay<'a, R: Resolution> {
    value:   &'a TimeOfDay<R>,
    options: FormatOptions,
}

impl<'a, R: Resolution> FormattedTimeOfDay<'a, R> {
    #[inline]
    pub(crate) const fn new(value: &'a TimeOfDay<R>, options: FormatOptions) -> Self {
        Self {
            value,
            options,
        }
    }
}

impl<R: Resolution> fmt::Display for FormattedTimeOfDay<'_, R> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        format_time(*self.value, self.options, f)
    }
}

pub(crate) fn format_options<R: Resolution>(
    options: FormatOptions,
) -> Result<(), FormatOptionError> {
    if let TrailingZeros::TrimAfter(minimum) = options.trailing_zeros
        && minimum > R::KIND
    {
        return Err(FormatOptionError {
            minimum,
            value_resolution: R::KIND,
        });
    }

    Ok(())
}

pub(crate) fn format_time<R: Resolution>(
    value: TimeOfDay<R>,
    options: FormatOptions,
    f: &mut fmt::Formatter<'_>,
) -> fmt::Result {
    write!(f, "{:02}:{:02}", value.hour(), value.minute())?;

    let canonical_digits = R::CANONICAL_FRACTIONAL_DIGITS;

    let (minimum, keep) = match options.trailing_zeros {
        TrailingZeros::Keep => (R::KIND, true),
        TrailingZeros::TrimAfter(minimum) => (minimum, false),
    };

    let subsecond = value.subsecond_nanoseconds();

    if !keep && minimum == ResolutionKind::Minute && value.second() == 0 && subsecond == 0 {
        return Ok(());
    }

    if R::KIND == ResolutionKind::Minute {
        return Ok(());
    }

    write!(f, ":{:02}", value.second())?;

    if canonical_digits == 0 {
        return Ok(());
    }

    let mut output_digits = canonical_digits;
    let minimum_digits = minimum.fractional_digits().min(canonical_digits);
    let fraction = subsecond / 10_u32.pow(9 - u32::from(canonical_digits));

    if !keep {
        // Trimming stops at the requested minimum even when additional digits are zero.
        while output_digits > minimum_digits
            && fraction.is_multiple_of(10_u32.pow(u32::from(canonical_digits - output_digits + 1)))
        {
            output_digits -= 1;
        }
    }

    if output_digits == 0 {
        return Ok(());
    }

    let displayed = fraction / 10_u32.pow(u32::from(canonical_digits - output_digits));

    write!(f, ".{:0width$}", displayed, width = usize::from(output_digits))
}
