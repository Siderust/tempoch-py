"""Tests for tempoch TimePeriod and period operations."""

import pickle

import pytest

from tempoch import (
    InvalidIntervalError,
    JulianDate,
    ModifiedJulianDate,
    TimePeriod,
    intersect_periods,
)


class TestTimePeriodCreation:
    def test_basic_creation(self):
        p = TimePeriod(59000.0, 59001.0)
        assert p.start_mjd == 59000.0
        assert p.end_mjd == 59001.0

    def test_start_after_end_raises(self):
        with pytest.raises(InvalidIntervalError):
            TimePeriod(59001.0, 59000.0)

    def test_zero_duration_allowed(self):
        p = TimePeriod(59000.0, 59000.0)
        assert p.duration_days() == 0.0

    def test_from_mjd(self):
        start = ModifiedJulianDate(59000.0)
        end = ModifiedJulianDate(59001.0)
        p = TimePeriod.from_mjd(start, end)
        assert p.start_mjd == 59000.0
        assert p.end_mjd == 59001.0

    def test_from_jd(self):
        start = JulianDate(2451545.0)
        end = JulianDate(2451546.0)
        p = TimePeriod.from_jd(start, end)
        assert abs(p.duration_days() - 1.0) < 1e-10

    def test_from_utc(self):
        p = TimePeriod.from_utc(
            "2020-01-01T00:00:00Z",
            "2020-01-02T00:00:00Z",
        )
        assert abs(p.duration_days() - 1.0) < 0.01  # within tolerance for ΔT


class TestTimePeriodAccessors:
    def test_start_end_objects(self):
        p = TimePeriod(59000.0, 59001.0)
        assert isinstance(p.start, ModifiedJulianDate)
        assert isinstance(p.end, ModifiedJulianDate)
        assert p.start.value == 59000.0
        assert p.end.value == 59001.0


class TestTimePeriodDuration:
    def test_duration_days(self):
        p = TimePeriod(59000.0, 59001.0)
        assert abs(p.duration_days() - 1.0) < 1e-12

    def test_duration_seconds(self):
        p = TimePeriod(59000.0, 59001.0)
        assert abs(p.duration_seconds() - 86400.0) < 1e-6

    def test_duration_hours(self):
        p = TimePeriod(59000.0, 59001.0)
        assert abs(p.duration_hours() - 24.0) < 1e-9


class TestTimePeriodUtc:
    def test_to_utc(self):
        p = TimePeriod(59000.0, 59001.0)
        start_utc, end_utc = p.to_utc()
        assert isinstance(start_utc, str)
        assert isinstance(end_utc, str)
        # MJD 59000 is approximately 2020-05-28
        assert "2020" in start_utc


class TestTimePeriodIntersection:
    def test_overlapping(self):
        a = TimePeriod(59000.0, 59002.0)
        b = TimePeriod(59001.0, 59003.0)
        c = a.intersection(b)
        assert c is not None
        assert abs(c.start_mjd - 59001.0) < 1e-12
        assert abs(c.end_mjd - 59002.0) < 1e-12

    def test_no_overlap(self):
        a = TimePeriod(59000.0, 59001.0)
        b = TimePeriod(59002.0, 59003.0)
        c = a.intersection(b)
        assert c is None

    def test_adjacent(self):
        a = TimePeriod(59000.0, 59001.0)
        b = TimePeriod(59001.0, 59002.0)
        c = a.intersection(b)
        # Half-open: adjacent periods have no intersection
        assert c is None

    def test_contained(self):
        outer = TimePeriod(59000.0, 59003.0)
        inner = TimePeriod(59001.0, 59002.0)
        c = outer.intersection(inner)
        assert c is not None
        assert abs(c.start_mjd - 59001.0) < 1e-12
        assert abs(c.end_mjd - 59002.0) < 1e-12


class TestTimePeriodContains:
    def test_contains_mjd_value(self):
        p = TimePeriod(59000.0, 59002.0)
        assert p.contains(59001.0)
        assert not p.contains(58999.0)
        assert not p.contains(59002.0)  # half-open interval
        assert not p.contains(59003.0)

    def test_contains_mjd_object(self):
        p = TimePeriod(59000.0, 59002.0)
        mjd = ModifiedJulianDate(59001.0)
        assert p.contains_mjd(mjd)


class TestTimePeriodEquality:
    def test_equal(self):
        a = TimePeriod(59000.0, 59001.0)
        b = TimePeriod(59000.0, 59001.0)
        assert a == b

    def test_not_equal(self):
        a = TimePeriod(59000.0, 59001.0)
        b = TimePeriod(59000.0, 59002.0)
        assert a != b


class TestTimePeriodDisplay:
    def test_repr(self):
        p = TimePeriod(59000.0, 59001.0)
        r = repr(p)
        assert "TimePeriod" in r
        assert "59000" in r
        assert "59001" in r

    def test_str(self):
        p = TimePeriod(59000.0, 59001.0)
        s = str(p)
        assert "MJD" in s


class TestTimePeriodPickle:
    def test_roundtrip(self):
        p = TimePeriod(59000.0, 59001.0)
        restored = pickle.loads(pickle.dumps(p))
        assert restored.start_mjd == p.start_mjd
        assert restored.end_mjd == p.end_mjd

    def test_hash(self):
        a = TimePeriod(59000.0, 59001.0)
        b = TimePeriod(59000.0, 59001.0)
        assert hash(a) == hash(b)


class TestIntersectPeriods:
    def test_basic_intersection(self):
        periods = [
            TimePeriod(59000.0, 59002.0),
            TimePeriod(59003.0, 59005.0),
        ]
        bounds = TimePeriod(59001.0, 59004.0)
        result = intersect_periods(periods, bounds)
        assert len(result) == 2
        assert abs(result[0].start_mjd - 59001.0) < 1e-12
        assert abs(result[0].end_mjd - 59002.0) < 1e-12
        assert abs(result[1].start_mjd - 59003.0) < 1e-12
        assert abs(result[1].end_mjd - 59004.0) < 1e-12

    def test_no_intersection(self):
        periods = [TimePeriod(59000.0, 59001.0)]
        bounds = TimePeriod(59002.0, 59003.0)
        result = intersect_periods(periods, bounds)
        assert len(result) == 0
