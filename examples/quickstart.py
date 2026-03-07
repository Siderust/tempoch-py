#!/usr/bin/env python3
"""Quick start example for tempoch-py.

Shows the core workflow: creating Julian Dates, converting to MJD/UTC,
arithmetic, and period operations.
"""

from tempoch import JulianDate, TimePeriod, TimeScale, convert_timescale

print("=== tempoch-py Quick Start ===\n")

# 1. Julian Date basics
j2000 = JulianDate.j2000()
print(f"J2000.0 epoch: {j2000}")
print(f"  value: {j2000.value}")
print(f"  UTC:   {j2000.to_utc()}")

# 2. Modified Julian Date
mjd = j2000.to_mjd()
print(f"\nAs MJD: {mjd}")
print(f"  back to JD: {mjd.to_jd()}")

# 3. Arithmetic
tomorrow = j2000 + 1.0
print(f"\nJ2000 + 1 day: {tomorrow}")
print(f"  difference:  {tomorrow.difference(j2000)} days")

# 4. Julian centuries
print(f"\nJulian centuries since J2000: {j2000.julian_centuries()}")

# 5. UTC conversion
jd = JulianDate.from_utc("2024-03-20T12:00:00Z")
print(f"\n2024-03-20 noon UTC → JD: {jd}")
print(f"  round-trip UTC: {jd.to_utc()}")

# 6. Time periods
p = TimePeriod(59000.0, 59010.0)
print(f"\nPeriod: {p}")
print(f"  duration: {p.duration_days()} days = {p.duration_hours()} hours")
start_utc, end_utc = p.to_utc()
print(f"  UTC: {start_utc} to {end_utc}")

# 7. Period intersection
p1 = TimePeriod(59000.0, 59010.0)
p2 = TimePeriod(59005.0, 59015.0)
overlap = p1.intersection(p2)
print(f"\nIntersection of {p1} and {p2}:")
print(f"  {overlap}  ({overlap.duration_days()} days)")

# 8. Time-scale conversion
jd_val = 2451545.0  # J2000
tdb = convert_timescale(jd_val, TimeScale.JD, TimeScale.TDB)
tai = convert_timescale(jd_val, TimeScale.JD, TimeScale.TAI)
print(f"\nScale conversions from JD {jd_val}:")
print(f"  TDB: {tdb}")
print(f"  TAI: {tai}")
print(f"  TAI-JD offset: {(jd_val - tai) * 86400:.3f} seconds")

print("\n=== Done ===")
