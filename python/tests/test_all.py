import pytest
import tipping


def test_sum_as_string():
    assert tipping.sum_as_string(1, 1) == "2"
