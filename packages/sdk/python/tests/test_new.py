from datetime import timezone

import pytest
from pytest_mock import MockerFixture

import equalto
from equalto.exceptions import WorkbookError


def test_create_new_workbook() -> None:
    workbook = equalto.new("en-US", timezone.utc)

    # perform a few basic operations in order to confirm that the workbook works as expected
    workbook["Sheet1!A1"].value = 1
    workbook["Sheet1!A2"].value = 2
    workbook["Sheet1!B1"].formula = "=A1+A2"
    assert workbook["Sheet1!B1"].value == 3


def test_create_new_workbook_error_handling(mocker: MockerFixture) -> None:
    """Errors coming from pycalc should be wrapped in WorkbookError."""
    mocker.patch("equalto.create", side_effect=Exception("cannot create a new workbook"))
    with pytest.raises(WorkbookError, match="cannot create a new workbook"):
        equalto.new("en-US", timezone.utc)