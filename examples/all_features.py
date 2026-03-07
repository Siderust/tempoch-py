#!/usr/bin/env python3
"""Comprehensive example demonstrating all tempoch-py features.

Covers: JD/MJD creation, UTC conversion, datetime interop, arithmetic,
comparisons, epoch helpers, time scales, periods, intersections, pickling,
error handling, and end-to-end workflows.
"""

import datetime
import pickle

from tempoch import (
    InvalidIntervalError,
    JulianDate,
    ModifiedJulianDate,
    NonFiniteTimeError,
    TimePeriod,
    TimeScale,
    convert_timescale,
    intersect_periods,
    tai_minus_utc,
)


def demo_julian_date():
    print("── Julian Date ─────────────────────────────────────────")
    j2000 = JulianDate.j2000()
    print(f"J2000.0:         {j2000}")
    print(f"  value:         {j2000.value}")
    print(f"  float():       {float(j2000)}")
    print(f"  as MJD:        {j2000.to_mjd()}")
    print(f"  UTC string:    {j2000.to_utc()}")
    print(f"  UTC datetime:  {j2000.to_datetime()}")
    print(f"  centuries:     {j2000.julian_centuries()}")
    print(f"  years:         {j2000.julian_years()}")
    print(f"  millennia:     {j2000.julian_millennia()}")
    print()


def demo_modified_julian_date():
    print("── Modified Julian Date ────────────────────────────────")
    mjd = ModifiedJulianDate(51544.5)
    print(f"MJD 51544.5:     {mjd}")
    print(f"  as JD:         {mjd.to_jd()}")
    print(f"  UTC string:    {mjd.to_utc()}")
    print(f"  UTC datetime:  {mjd.to_datetime()}")
    print()


def demo_utc_conversion():
    print("── UTC Conversion ──────────────────────────────────────")
    jd = JulianDate.from_utc("2024-06-21T12:00:00Z")
    print(f"2024 summer solstice noon → JD: {jd}")
    print(f"  round-trip:    {jd.to_utc()}")

    mjd = ModifiedJulianDate.from_utc("2024-06-21T12:00:00Z")
    print(f"  as MJD:        {mjd}")
    print()


def demo_datetime_interop():
    print("── Python datetime interop ─────────────────────────────")
    dt = datetime.datetime(2024, 3, 20, 12, 0, 0, tzinfo=datetime.timezone.utc)
    jd = JulianDate.from_datetime(dt)
    print(f"datetime({dt}) → JD: {jd}")

    back = jd.to_datetime()
    print(f"  back to datetime: {back}")
    print()


def demo_arithmetic():
    print("── Arithmetic ──────────────────────────────────────────")
    jd = JulianDate(2451545.0)
    print(f"JD:              {jd}")
    print(f"  + 365.25 days: {jd + 365.25}")
    print(f"  - 100 days:    {jd - 100.0}")

    jd2 = JulianDate(2451545.0 + 365.25)
    print(f"  diff (days):   {jd2 - jd}")

    mjd = ModifiedJulianDate(51544.5)
    print(f"\nMJD:             {mjd}")
    print(f"  + 10 days:     {mjd + 10.0}")
    print()


def demo_scales():
    print("── Time Scale Conversions ──────────────────────────────")
    jd = 2451545.0
    for scale in [
        TimeScale.MJD,
        TimeScale.TDB,
        TimeScale.TT,
        TimeScale.TAI,
        TimeScale.GPS,
        TimeScale.UT,
    ]:
        val = convert_timescale(jd, TimeScale.JD, scale)
        print(f"  JD → {str(scale):10s}: {val:.6f}")

    print(f"\nTAI − UTC at J2000: {tai_minus_utc(jd)} seconds")
    print()


def demo_periods():
    print("── Time Periods ────────────────────────────────────────")
    p = TimePeriod(59000.0, 59010.0)
    print(f"Period:          {p}")
    print(f"  duration:      {p.duration_days()} days")
    print(f"  duration:      {p.duration_hours()} hours")
    print(f"  duration:      {p.duration_seconds()} seconds")
    print(f"  contains 59005: {p.contains(59005.0)}")
    print(f"  contains 59020: {p.contains(59020.0)}")

    start_utc, end_utc = p.to_utc()
    print(f"  UTC: {start_utc}")
    print(f"    to {end_utc}")
    print()


def demo_period_intersection():
    print("── Period Intersection ─────────────────────────────────")
    p1 = TimePeriod(59000.0, 59010.0)
    p2 = TimePeriod(59005.0, 59015.0)
    overlap = p1.intersection(p2)
    print(f"  {p1} ∩ {p2}")
    print(f"  = {overlap}")

    p3 = TimePeriod(59020.0, 59030.0)
    no_overlap = p1.intersection(p3)
    print(f"\n  {p1} ∩ {p3}")
    print(f"  = {no_overlap}")

    # Multi-period intersection
    periods = [
        TimePeriod(59000.0, 59005.0),
        TimePeriod(59010.0, 59015.0),
        TimePeriod(59020.0, 59025.0),
    ]
    bounds = TimePeriod(59003.0, 59022.0)
    result = intersect_periods(periods, bounds)
    print(f"\n  {len(result)} intersections within bounds:")
    for r in result:
        print(f"    {r}")
    print()


def demo_pickle():
    print("── Pickle Support ──────────────────────────────────────")
    jd = JulianDate(2451545.0)
    jd2 = pickle.loads(pickle.dumps(jd))
    print(f"JulianDate pickle:       {jd.value} → {jd2.value} ✓")

    mjd = ModifiedJulianDate(51544.5)
    mjd2 = pickle.loads(pickle.dumps(mjd))
    print(f"ModifiedJulianDate:      {mjd.value} → {mjd2.value} ✓")

    p = TimePeriod(59000.0, 59001.0)
    p2 = pickle.loads(pickle.dumps(p))
    print(
        f"TimePeriod:              [{p.start_mjd}, {p.end_mjd}] → [{p2.start_mjd}, {p2.end_mjd}] ✓"
    )
    print()


def demo_error_handling():
    print("── Error Handling ──────────────────────────────────────")
    try:
        JulianDate(float("nan"))
    except NonFiniteTimeError as e:
        print(f"NonFiniteTimeError:    {e}")

    try:
        TimePeriod(59001.0, 59000.0)
    except InvalidIntervalError as e:
        print(f"InvalidIntervalError:  {e}")

    # All our custom exceptions are ValueError subclasses
    try:
        JulianDate(float("inf"))
    except ValueError as e:
        print(f"ValueError (base):     {e}")
    print()


if __name__ == "__main__":
    demo_julian_date()
    demo_modified_julian_date()
    demo_utc_conversion()
    demo_datetime_interop()
    demo_arithmetic()
    demo_scales()
    demo_periods()
    demo_period_intersection()
    demo_pickle()
    demo_error_handling()
    print("All demos completed successfully.")
