"""Tests for tempoch Python bindings — JulianDate and ModifiedJulianDate."""

import pickle

import pytest

from tempoch import JulianDate, ModifiedJulianDate, NonFiniteTimeError


class TestJulianDateCreation:
    def test_basic_creation(self):
        jd = JulianDate(2451545.0)
        assert jd.value == 2451545.0

    def test_j2000_constant(self):
        j2000 = JulianDate.j2000()
        assert j2000.value == 2451545.0

    def test_non_finite_raises(self):
        with pytest.raises(NonFiniteTimeError):
            JulianDate(float("nan"))

    def test_infinity_raises(self):
        with pytest.raises(NonFiniteTimeError):
            JulianDate(float("inf"))

    def test_negative_infinity_raises(self):
        with pytest.raises(NonFiniteTimeError):
            JulianDate(float("-inf"))

    def test_negative_value(self):
        jd = JulianDate(-100.0)
        assert jd.value == -100.0

    def test_zero(self):
        jd = JulianDate(0.0)
        assert jd.value == 0.0

    def test_large_value(self):
        jd = JulianDate(1e10)
        assert jd.value == 1e10


class TestJulianDateConversion:
    def test_to_mjd(self):
        jd = JulianDate(2451545.0)
        mjd = jd.to_mjd()
        assert isinstance(mjd, ModifiedJulianDate)
        assert abs(mjd.value - 51544.5) < 1e-10

    def test_to_mjd_roundtrip(self):
        original = JulianDate(2451545.0)
        mjd = original.to_mjd()
        back = mjd.to_jd()
        assert isinstance(back, JulianDate)
        assert abs(back.value - original.value) < 1e-10

    def test_to_utc_j2000(self):
        j2000 = JulianDate.j2000()
        utc = j2000.to_utc()
        assert isinstance(utc, str)
        assert "2000-01" in utc

    def test_from_utc_roundtrip(self):
        jd = JulianDate.from_utc("2020-06-15T12:00:00Z")
        utc_str = jd.to_utc()
        assert "2020-06-15" in utc_str

    def test_to_datetime(self):
        import datetime

        j2000 = JulianDate.j2000()
        dt = j2000.to_datetime()
        assert isinstance(dt, datetime.datetime)
        assert dt.tzinfo is not None  # UTC-aware

    def test_from_datetime(self):
        import datetime

        dt = datetime.datetime(2000, 1, 1, 12, 0, 0, tzinfo=datetime.timezone.utc)
        jd = JulianDate.from_datetime(dt)
        # J2000 is approximately JD 2451545.0
        assert abs(jd.value - 2451545.0) < 0.01  # within ~15 minutes (ΔT is ~64s)

    def test_from_datetime_normalizes_non_utc_offset(self):
        import datetime

        offset = datetime.timezone(datetime.timedelta(hours=5, minutes=30))
        local = datetime.datetime(2024, 6, 21, 17, 30, 0, 123456, tzinfo=offset)
        expected = datetime.datetime(2024, 6, 21, 12, 0, 0, 123456, tzinfo=datetime.timezone.utc)

        result = JulianDate.from_datetime(local).to_datetime()

        assert abs(result - expected) < datetime.timedelta(microseconds=50)
        assert result.utcoffset() == datetime.timedelta(0)

    def test_from_datetime_rejects_naive_datetime(self):
        import datetime

        with pytest.raises(ValueError, match="naive"):
            JulianDate.from_datetime(datetime.datetime(2024, 6, 21, 12, 0))


class TestJulianDateArithmetic:
    def test_add_days(self):
        jd = JulianDate(2451545.0)
        result = jd.add_days(1.0)
        assert abs(result.value - 2451546.0) < 1e-12

    def test_add_operator(self):
        jd = JulianDate(2451545.0)
        result = jd + 1.0
        assert abs(result.value - 2451546.0) < 1e-12

    def test_subtract_days_operator(self):
        jd = JulianDate(2451546.0)
        result = jd - 1.0
        assert abs(result.value - 2451545.0) < 1e-12

    def test_subtract_jd_operator(self):
        jd1 = JulianDate(2451546.0)
        jd2 = JulianDate(2451545.0)
        diff = jd1 - jd2
        assert isinstance(diff, float)
        assert abs(diff - 1.0) < 1e-12

    def test_difference(self):
        jd1 = JulianDate(2451546.0)
        jd2 = JulianDate(2451545.0)
        assert abs(jd1.difference(jd2) - 1.0) < 1e-12


class TestJulianDateComparisons:
    def test_equality(self):
        a = JulianDate(2451545.0)
        b = JulianDate(2451545.0)
        assert a == b

    def test_inequality(self):
        a = JulianDate(2451545.0)
        b = JulianDate(2451546.0)
        assert a != b

    def test_less_than(self):
        a = JulianDate(2451545.0)
        b = JulianDate(2451546.0)
        assert a < b
        assert not (b < a)

    def test_less_equal(self):
        a = JulianDate(2451545.0)
        b = JulianDate(2451545.0)
        assert a <= b

    def test_greater_than(self):
        a = JulianDate(2451546.0)
        b = JulianDate(2451545.0)
        assert a > b

    def test_greater_equal(self):
        a = JulianDate(2451545.0)
        b = JulianDate(2451545.0)
        assert a >= b


class TestJulianDateEpoch:
    def test_julian_centuries(self):
        j2000 = JulianDate.j2000()
        assert abs(j2000.julian_centuries()) < 1e-15

        # One Julian century after J2000
        jd = JulianDate(2451545.0 + 36525.0)
        assert abs(jd.julian_centuries() - 1.0) < 1e-12

    def test_julian_years(self):
        j2000 = JulianDate.j2000()
        assert abs(j2000.julian_years()) < 1e-15

        # One Julian year after J2000
        jd = JulianDate(2451545.0 + 365.25)
        assert abs(jd.julian_years() - 1.0) < 1e-12

    def test_julian_millennia(self):
        j2000 = JulianDate.j2000()
        assert abs(j2000.julian_millennia()) < 1e-15


class TestJulianDateDisplay:
    def test_repr(self):
        jd = JulianDate(2451545.0)
        assert "2451545" in repr(jd)
        assert "JulianDate" in repr(jd)

    def test_str(self):
        jd = JulianDate(2451545.0)
        assert "2451545" in str(jd)

    def test_float(self):
        jd = JulianDate(2451545.0)
        assert float(jd) == 2451545.0


class TestJulianDatePickle:
    def test_roundtrip(self):
        jd = JulianDate(2451545.0)
        restored = pickle.loads(pickle.dumps(jd))
        assert restored.value == jd.value

    def test_hash(self):
        a = JulianDate(2451545.0)
        b = JulianDate(2451545.0)
        c = JulianDate(2451546.0)
        assert hash(a) == hash(b)
        assert hash(a) != hash(c)

    def test_set_membership(self):
        s = {JulianDate(2451545.0), JulianDate(2451545.0), JulianDate(2451546.0)}
        assert len(s) == 2


class TestModifiedJulianDateCreation:
    def test_basic_creation(self):
        mjd = ModifiedJulianDate(51544.5)
        assert mjd.value == 51544.5

    def test_non_finite_raises(self):
        with pytest.raises(NonFiniteTimeError):
            ModifiedJulianDate(float("nan"))

    def test_infinity_raises(self):
        with pytest.raises(NonFiniteTimeError):
            ModifiedJulianDate(float("inf"))


class TestModifiedJulianDateConversion:
    def test_to_jd(self):
        mjd = ModifiedJulianDate(51544.5)
        jd = mjd.to_jd()
        assert isinstance(jd, JulianDate)
        assert abs(jd.value - 2451545.0) < 1e-10

    def test_to_utc(self):
        mjd = ModifiedJulianDate(51544.5)
        utc = mjd.to_utc()
        assert isinstance(utc, str)
        assert "2000-01" in utc

    def test_from_utc_roundtrip(self):
        mjd = ModifiedJulianDate.from_utc("2020-06-15T12:00:00Z")
        utc_str = mjd.to_utc()
        assert "2020-06-15" in utc_str

    def test_to_datetime(self):
        import datetime

        mjd = ModifiedJulianDate(51544.5)
        dt = mjd.to_datetime()
        assert isinstance(dt, datetime.datetime)

    def test_from_datetime(self):
        import datetime

        dt = datetime.datetime(2020, 6, 15, 12, 0, 0, tzinfo=datetime.timezone.utc)
        mjd = ModifiedJulianDate.from_datetime(dt)
        assert abs(mjd.value - 59015.50080074074) < 1e-9


class TestModifiedJulianDateArithmetic:
    def test_add_days(self):
        mjd = ModifiedJulianDate(51544.5)
        result = mjd.add_days(1.0)
        assert abs(result.value - 51545.5) < 1e-12

    def test_add_operator(self):
        mjd = ModifiedJulianDate(51544.5)
        result = mjd + 1.0
        assert abs(result.value - 51545.5) < 1e-12

    def test_subtract_days_operator(self):
        mjd = ModifiedJulianDate(51545.5)
        result = mjd - 1.0
        assert abs(result.value - 51544.5) < 1e-12

    def test_subtract_mjd_operator(self):
        a = ModifiedJulianDate(51545.5)
        b = ModifiedJulianDate(51544.5)
        diff = a - b
        assert isinstance(diff, float)
        assert abs(diff - 1.0) < 1e-12

    def test_difference(self):
        a = ModifiedJulianDate(51545.5)
        b = ModifiedJulianDate(51544.5)
        assert abs(a.difference(b) - 1.0) < 1e-12


class TestModifiedJulianDateComparisons:
    def test_equality(self):
        a = ModifiedJulianDate(51544.5)
        b = ModifiedJulianDate(51544.5)
        assert a == b

    def test_ordering(self):
        a = ModifiedJulianDate(51544.5)
        b = ModifiedJulianDate(51545.5)
        assert a < b
        assert b > a
        assert a <= b
        assert b >= a


class TestModifiedJulianDateDisplay:
    def test_repr(self):
        mjd = ModifiedJulianDate(51544.5)
        assert "51544.5" in repr(mjd)
        assert "ModifiedJulianDate" in repr(mjd)

    def test_float(self):
        mjd = ModifiedJulianDate(51544.5)
        assert float(mjd) == 51544.5


class TestModifiedJulianDatePickle:
    def test_roundtrip(self):
        mjd = ModifiedJulianDate(51544.5)
        restored = pickle.loads(pickle.dumps(mjd))
        assert restored.value == mjd.value

    def test_hash(self):
        a = ModifiedJulianDate(51544.5)
        b = ModifiedJulianDate(51544.5)
        assert hash(a) == hash(b)
