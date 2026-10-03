"""The downstream extension consumes tempoch-py's public Rust API."""

import datetime

import pytest

downstream = pytest.importorskip(
    "_tempoch_downstream", reason="downstream fixture is built by the integration-test job"
)


def test_downstream_roundtrip_normalizes_offset_and_preserves_microseconds():
    source = datetime.datetime(
        2024,
        6,
        21,
        17,
        34,
        56,
        123456,
        tzinfo=datetime.timezone(datetime.timedelta(hours=5)),
    )
    expected = datetime.datetime(
        2024,
        6,
        21,
        12,
        34,
        56,
        123456,
        tzinfo=datetime.timezone.utc,
    )

    result = downstream.roundtrip(source)

    assert abs(result - expected) < datetime.timedelta(microseconds=50)
    assert result.microsecond > 123400
    assert result.utcoffset() == datetime.timedelta(0)


def test_downstream_rejects_naive_datetime():
    with pytest.raises(ValueError, match="naive"):
        downstream.roundtrip(datetime.datetime(2024, 6, 21, 12, 0))
