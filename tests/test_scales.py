"""Tests for time-scale conversion and module-level API surface."""

import pytest

from tempoch import (
    InvalidIntervalError,
    JulianDate,
    ModifiedJulianDate,
    NonFiniteTimeError,
    TimePeriod,
    TimeScale,
    __version__,
    convert_timescale,
    tai_minus_utc,
)


class TestModuleExports:
    def test_all_exports_present(self):
        import tempoch

        for name in (
            "JulianDate",
            "ModifiedJulianDate",
            "TimePeriod",
            "TimeScale",
            "NonFiniteTimeError",
            "InvalidIntervalError",
            "ConversionError",
            "convert_timescale",
            "tai_minus_utc",
            "intersect_periods",
            "__version__",
        ):
            assert hasattr(tempoch, name), f"tempoch.{name} missing"

    def test_version_is_string(self):
        assert isinstance(__version__, str)
        assert __version__ == "0.1.0"


class TestTimeScaleEnum:
    def test_all_scales_exist(self):
        assert TimeScale.JD is not None
        assert TimeScale.JDE is not None
        assert TimeScale.MJD is not None
        assert TimeScale.TDB is not None
        assert TimeScale.TT is not None
        assert TimeScale.TAI is not None
        assert TimeScale.TCG is not None
        assert TimeScale.TCB is not None
        assert TimeScale.GPS is not None
        assert TimeScale.UnixTime is not None
        assert TimeScale.UT is not None

    def test_scale_equality(self):
        assert TimeScale.JD == TimeScale.JD
        assert TimeScale.JD != TimeScale.MJD

    def test_scale_hashable(self):
        s = {TimeScale.JD, TimeScale.MJD, TimeScale.JD}
        assert len(s) == 2

    def test_scale_repr(self):
        r = repr(TimeScale.JD)
        assert "JD" in r

    def test_scale_str(self):
        assert str(TimeScale.JD) == "JD"
        assert str(TimeScale.MJD) == "MJD"
        assert str(TimeScale.TDB) == "TDB"


class TestConvertTimescale:
    def test_jd_to_mjd(self):
        jd = 2451545.0  # J2000
        mjd = convert_timescale(jd, TimeScale.JD, TimeScale.MJD)
        assert abs(mjd - 51544.5) < 1e-10

    def test_mjd_to_jd(self):
        mjd = 51544.5
        jd = convert_timescale(mjd, TimeScale.MJD, TimeScale.JD)
        assert abs(jd - 2451545.0) < 1e-10

    def test_identity_conversion(self):
        value = 2451545.0
        result = convert_timescale(value, TimeScale.JD, TimeScale.JD)
        assert abs(result - value) < 1e-15

    def test_jd_to_tt(self):
        # JD = TT for epoch-counter identity
        jd = 2451545.0
        tt = convert_timescale(jd, TimeScale.JD, TimeScale.TT)
        assert abs(tt - jd) < 1e-10

    def test_tt_to_tai(self):
        # TT = TAI + 32.184s, so TAI = TT − 32.184/86400 days
        tt = 2451545.0
        tai = convert_timescale(tt, TimeScale.TT, TimeScale.TAI)
        diff_seconds = (tt - tai) * 86400
        assert abs(diff_seconds - 32.184) < 0.001

    def test_jd_to_tdb_roundtrip(self):
        jd = 2451545.0
        tdb = convert_timescale(jd, TimeScale.JD, TimeScale.TDB)
        back = convert_timescale(tdb, TimeScale.TDB, TimeScale.JD)
        assert abs(back - jd) < 1e-6  # TDB correction is ~1.7ms

    def test_jd_to_gps(self):
        # GPS epoch is 1980-01-06 (JD 2444244.5), GPS = TAI − 19s
        jd = 2451545.0
        gps = convert_timescale(jd, TimeScale.JD, TimeScale.GPS)
        assert gps != jd  # GPS is offset

    def test_jd_to_ut(self):
        # UT involves ΔT correction
        jd = 2451545.0
        ut = convert_timescale(jd, TimeScale.JD, TimeScale.UT)
        # ΔT at J2000 is about 63.83 seconds
        diff_seconds = (jd - ut) * 86400
        assert abs(diff_seconds - 63.83) < 1.0


class TestTaiMinusUtc:
    def test_j2000(self):
        # At J2000 (JD 2451545.0), TAI − UTC = 32s
        result = tai_minus_utc(2451545.0)
        assert abs(result - 32.0) < 1.0

    def test_before_1972(self):
        # Before 1972 (JD ~2441317.5), TAI − UTC = 10s
        result = tai_minus_utc(2441317.5)
        assert result >= 10.0

    def test_recent(self):
        # After 2017 (JD ~2457754.5), TAI − UTC = 37s
        result = tai_minus_utc(2457754.5 + 365.0)
        assert abs(result - 37.0) < 1.0


class TestErrorHandling:
    def test_non_finite_is_value_error(self):
        """NonFiniteTimeError is a subclass of ValueError."""
        with pytest.raises(ValueError):
            JulianDate(float("nan"))

    def test_invalid_interval_is_value_error(self):
        """InvalidIntervalError is a subclass of ValueError."""
        with pytest.raises(ValueError):
            TimePeriod(59001.0, 59000.0)

    def test_exception_messages(self):
        with pytest.raises(NonFiniteTimeError, match="finite"):
            JulianDate(float("nan"))

        with pytest.raises(InvalidIntervalError, match="start"):
            TimePeriod(59001.0, 59000.0)


class TestNoFFIExposure:
    """Verify that no raw FFI types leak into the Python API."""

    def test_jd_is_clean(self):
        jd = JulianDate(2451545.0)
        assert type(jd).__name__ == "JulianDate"
        assert hasattr(jd, "value")
        assert hasattr(jd, "to_mjd")
        assert hasattr(jd, "to_utc")
        assert hasattr(jd, "to_datetime")

    def test_mjd_is_clean(self):
        mjd = ModifiedJulianDate(51544.5)
        assert type(mjd).__name__ == "ModifiedJulianDate"
        assert hasattr(mjd, "value")
        assert hasattr(mjd, "to_jd")
        assert hasattr(mjd, "to_utc")
        assert hasattr(mjd, "to_datetime")

    def test_period_is_clean(self):
        p = TimePeriod(59000.0, 59001.0)
        assert type(p).__name__ == "TimePeriod"
        assert hasattr(p, "start")
        assert hasattr(p, "end")
        assert hasattr(p, "duration_days")
        assert hasattr(p, "intersection")

    def test_no_status_codes(self):
        """Errors are Python exceptions, not integer status codes."""
        try:
            JulianDate(float("nan"))
        except Exception as e:
            assert isinstance(e, ValueError)
            assert not isinstance(e, (SystemError, OSError))


class TestEndToEnd:
    def test_jd_mjd_utc_workflow(self):
        """Full workflow: JD → MJD → UTC → JD roundtrip."""
        jd = JulianDate(2451545.0)
        mjd = jd.to_mjd()
        utc = mjd.to_utc()
        jd2 = JulianDate.from_utc(utc)
        # Allow ~1s tolerance for UTC string parsing precision
        assert abs(jd2.value - jd.value) < 1e-4

    def test_period_workflow(self):
        """Full workflow: create period, check duration, intersect."""
        p1 = TimePeriod(59000.0, 59010.0)
        p2 = TimePeriod(59005.0, 59015.0)
        overlap = p1.intersection(p2)
        assert overlap is not None
        assert abs(overlap.duration_days() - 5.0) < 1e-12
        assert overlap.contains(59007.0)

    def test_scale_conversion_workflow(self):
        """Convert JD through multiple scales and back."""
        jd = 2451545.0
        tdb = convert_timescale(jd, TimeScale.JD, TimeScale.TDB)
        tai = convert_timescale(tdb, TimeScale.TDB, TimeScale.TAI)
        back = convert_timescale(tai, TimeScale.TAI, TimeScale.JD)
        assert abs(back - jd) < 1e-6
