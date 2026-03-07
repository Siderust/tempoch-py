# tempoch-py

Python bindings for [tempoch](https://github.com/Siderust/tempoch) — astronomical time primitives powered by Rust.

## Features

- **Julian Date** and **Modified Julian Date** objects with full arithmetic
- **UTC conversion** (ISO 8601 strings and Python `datetime` objects)
- **Time-scale conversion** between 11 astronomical scales (JD, MJD, TT, TDB, TAI, TCG, TCB, GPS, UnixTime, UT, JDE)
- **Time periods** (intervals) with duration, intersection, and containment
- **Python exceptions** for errors (no raw FFI/status codes)
- **Pickle** and **hash** support
- All computation in Rust (no reimplementation in Python)

## Installation

```bash
# Development install
pip install maturin
maturin develop

# Or build a wheel
maturin build --release
pip install target/wheels/tempoch-*.whl
```

## Quick Start

```python
from tempoch import JulianDate, ModifiedJulianDate, TimePeriod, TimeScale, convert_timescale

# Julian Date basics
j2000 = JulianDate.j2000()          # J2000.0 epoch
print(j2000)                         # JulianDate(2451545.0)
print(j2000.to_utc())               # 2000-01-01T11:58:55...+00:00

# Modified Julian Date
mjd = j2000.to_mjd()                # ModifiedJulianDate(51544.5)
jd_back = mjd.to_jd()               # JulianDate(2451545.0)

# UTC conversion
jd = JulianDate.from_utc("2024-06-21T12:00:00Z")
dt = jd.to_datetime()               # Python datetime object (UTC)

# Arithmetic
tomorrow = j2000 + 1.0              # Add 1 day
diff = tomorrow - j2000             # 1.0 (days)

# Julian centuries since J2000
centuries = jd.julian_centuries()

# Time-scale conversion
tdb = convert_timescale(2451545.0, TimeScale.JD, TimeScale.TDB)
tai = convert_timescale(2451545.0, TimeScale.JD, TimeScale.TAI)

# Time periods
period = TimePeriod(59000.0, 59010.0)  # MJD-based
print(period.duration_days())           # 10.0
print(period.contains(59005.0))         # True

# Period intersection
p1 = TimePeriod(59000.0, 59010.0)
p2 = TimePeriod(59005.0, 59015.0)
overlap = p1.intersection(p2)           # TimePeriod(59005.0, 59010.0)
```

## API Reference

### Classes

| Class | Description |
|-------|-------------|
| `JulianDate` | Julian Date (continuous day count) |
| `ModifiedJulianDate` | Modified Julian Date (JD − 2,400,000.5) |
| `TimePeriod` | Time interval defined by start/end MJD |
| `TimeScale` | Enum of astronomical time scales |

### JulianDate

| Method | Description |
|--------|-------------|
| `JulianDate(value)` | Create from day number |
| `JulianDate.j2000()` | J2000.0 epoch constant |
| `JulianDate.from_utc(str)` | Create from UTC string |
| `JulianDate.from_datetime(dt)` | Create from Python datetime |
| `.value` | Raw day number (float) |
| `.to_mjd()` | Convert to ModifiedJulianDate |
| `.to_utc()` | Convert to UTC string (ISO 8601) |
| `.to_datetime()` | Convert to Python datetime (UTC) |
| `.add_days(n)` | Add n days |
| `.difference(other)` | Days between two JDs |
| `.julian_centuries()` | Centuries since J2000.0 |
| `.julian_years()` | Years since J2000.0 |
| `.julian_millennia()` | Millennia since J2000.0 |

### ModifiedJulianDate

| Method | Description |
|--------|-------------|
| `ModifiedJulianDate(value)` | Create from MJD day number |
| `ModifiedJulianDate.from_utc(str)` | Create from UTC string |
| `ModifiedJulianDate.from_datetime(dt)` | Create from Python datetime |
| `.value` | Raw MJD day number (float) |
| `.to_jd()` | Convert to JulianDate |
| `.to_utc()` | Convert to UTC string |
| `.to_datetime()` | Convert to Python datetime |
| `.add_days(n)` | Add n days |
| `.difference(other)` | Days between two MJDs |

### TimePeriod

| Method | Description |
|--------|-------------|
| `TimePeriod(start_mjd, end_mjd)` | Create from MJD values |
| `TimePeriod.from_mjd(start, end)` | Create from MJD objects |
| `TimePeriod.from_jd(start, end)` | Create from JD objects |
| `TimePeriod.from_utc(start, end)` | Create from UTC strings |
| `.start` / `.end` | Start/end as ModifiedJulianDate |
| `.start_mjd` / `.end_mjd` | Start/end as float |
| `.duration_days()` | Duration in days |
| `.duration_hours()` | Duration in hours |
| `.duration_seconds()` | Duration in seconds |
| `.to_utc()` | Start/end as UTC strings |
| `.intersection(other)` | Intersection with another period |
| `.contains(mjd)` | Check if MJD is within period |

### Functions

| Function | Description |
|----------|-------------|
| `convert_timescale(value, from, to)` | Convert between time scales |
| `tai_minus_utc(jd)` | TAI − UTC leap seconds (seconds) |
| `intersect_periods(periods, bounds)` | Intersect period list with bounds |

### Exceptions

| Exception | Base | Description |
|-----------|------|-------------|
| `NonFiniteTimeError` | `ValueError` | NaN or infinite time value |
| `InvalidIntervalError` | `ValueError` | Period start after end |
| `ConversionError` | `ValueError` | UTC/scale conversion out of range |

## Time Scales

| Scale | Description |
|-------|-------------|
| `TimeScale.JD` | Julian Date (identity) |
| `TimeScale.JDE` | Julian Ephemeris Day |
| `TimeScale.MJD` | Modified Julian Date |
| `TimeScale.TDB` | Barycentric Dynamical Time |
| `TimeScale.TT` | Terrestrial Time |
| `TimeScale.TAI` | International Atomic Time |
| `TimeScale.TCG` | Geocentric Coordinate Time |
| `TimeScale.TCB` | Barycentric Coordinate Time |
| `TimeScale.GPS` | GPS Time |
| `TimeScale.UnixTime` | Unix/POSIX Time |
| `TimeScale.UT` | Universal Time (Earth rotation) |

## Development

```bash
# Create virtual environment
python -m venv .venv
source .venv/bin/activate

# Install in development mode
pip install maturin pytest
maturin develop

# Run tests
pytest tests/ -v
```

## License

AGPL-3.0 — see [LICENSE](LICENSE).
