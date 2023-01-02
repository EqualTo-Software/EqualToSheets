from __future__ import annotations

from functools import cached_property
from typing import TYPE_CHECKING

from equalto.exceptions import CellReferenceError
from equalto.reference import parse_cell_reference
from equalto.sheet import WorkbookSheets

if TYPE_CHECKING:
    from equalto._pycalc import PyCalcModel
    from equalto.cell import Cell


class Workbook:
    def __init__(self, model: PyCalcModel):
        self._model = model

    def __getitem__(self, key: str) -> Cell:
        """Get cell by Excel reference."""
        sheet_name, row, column = parse_cell_reference(key)
        if sheet_name is None:
            raise CellReferenceError(f'"{key}" reference is missing the sheet name')
        return self.sheets[sheet_name].cell(row, column)

    def __delitem__(self, key: str) -> None:
        """Delete the cell content and style."""
        self[key].delete()

    def cell(self, sheet_index: int, row: int, column: int) -> Cell:
        return self.sheets[sheet_index].cell(row, column)

    @cached_property
    def sheets(self) -> WorkbookSheets:
        return WorkbookSheets(self)