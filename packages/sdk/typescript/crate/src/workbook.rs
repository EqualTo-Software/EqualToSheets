use wasm_bindgen::{
    prelude::{wasm_bindgen, JsError},
    JsValue,
};

use equalto_calc::{
    cell::CellValue,
    model::{Environment, Model},
};

#[cfg(feature = "xlsx")]
use equalto_xlsx::import::load_xlsx_from_memory;

use crate::error::WorkbookError;

#[wasm_bindgen(js_name = "SheetDimension")]
pub struct WasmSheetDimension {
    #[wasm_bindgen(js_name = "minColumn")]
    pub min_column: i32,
    #[wasm_bindgen(js_name = "maxColumn")]
    pub max_column: i32,
    #[wasm_bindgen(js_name = "minRow")]
    pub min_row: i32,
    #[wasm_bindgen(js_name = "maxRow")]
    pub max_row: i32,
}

#[wasm_bindgen]
pub struct WasmWorkbook {
    model: Model,
}

#[wasm_bindgen]
impl WasmWorkbook {
    #[wasm_bindgen(constructor)]
    pub fn new(locale: &str, timezone: &str) -> Result<WasmWorkbook, JsError> {
        let env = Environment::default();
        let model =
            Model::new_empty("workbook", locale, timezone, env).map_err(WorkbookError::from)?;
        Ok(WasmWorkbook { model })
    }

    #[wasm_bindgen(js_name=loadFromMemory)]
    #[cfg(feature = "xlsx")]
    pub fn load_from_memory(
        data: &mut [u8],
        locale: &str,
        timezone: &str,
    ) -> Result<WasmWorkbook, JsError> {
        let env = Environment::default();
        let model = load_xlsx_from_memory("workbook", data, locale, timezone, env)
            .map_err(WorkbookError::from)?;
        Ok(WasmWorkbook { model })
    }

    pub fn evaluate(&mut self) -> Result<(), JsError> {
        self.model.evaluate();
        Ok(())
    }

    #[wasm_bindgen(js_name = "getWorksheetNames")]
    pub fn get_worksheet_names(&self) -> Result<js_sys::Array, JsError> {
        Ok(self
            .model
            .workbook
            .get_worksheet_names()
            .into_iter()
            .map(JsValue::from)
            .collect())
    }

    #[wasm_bindgen(js_name = "getWorksheetIds")]
    pub fn get_worksheet_ids(&self) -> Result<js_sys::Array, JsError> {
        Ok(self
            .model
            .workbook
            .get_worksheet_ids()
            .into_iter()
            .map(JsValue::from)
            .collect())
    }

    #[wasm_bindgen(js_name = "addSheet")]
    pub fn add_sheet(&mut self, name: &str) -> Result<(), JsError> {
        self.model
            .add_sheet(name)
            .map_err(WorkbookError::from)
            .map_err(JsError::from)
    }

    #[wasm_bindgen(js_name = "newSheet")]
    pub fn new_sheet(&mut self) -> Result<(), JsError> {
        self.model.new_sheet();
        Ok(())
    }

    // TODO: Should be by sheetId
    #[wasm_bindgen(js_name = "renameSheetBySheetIndex")]
    pub fn rename_sheet_by_sheet_index(
        &mut self,
        sheet: i32,
        new_name: &str,
    ) -> Result<(), JsError> {
        self.model
            .rename_sheet_by_index(sheet.try_into().unwrap(), new_name)
            .map_err(WorkbookError::from)
            .map_err(JsError::from)
    }

    #[wasm_bindgen(js_name = "deleteSheetBySheetId")]
    pub fn delete_sheet_by_sheet_id(&mut self, sheet_id: i32) -> Result<(), JsError> {
        self.model
            .delete_sheet_by_sheet_id(sheet_id.try_into().unwrap())
            .map_err(WorkbookError::from)
            .map_err(JsError::from)
    }

    #[wasm_bindgen(js_name = "getCellValueByIndex")]
    pub fn get_cell_value_by_index(
        &self,
        sheet: i32,
        row: i32,
        column: i32,
    ) -> Result<JsValue, JsError> {
        Ok(
            match self
                .model
                .get_cell_value_by_index(sheet.try_into().unwrap(), row, column)
                .map_err(WorkbookError::from)?
            {
                CellValue::String(s) => JsValue::from(s),
                CellValue::Number(f) => JsValue::from(f),
                CellValue::Boolean(b) => JsValue::from(b),
            },
        )
    }

    #[wasm_bindgen(js_name = "getFormattedCellValue")]
    pub fn formatted_cell_value(
        &self,
        sheet: i32,
        row: i32,
        column: i32,
    ) -> Result<String, JsError> {
        self.model
            .formatted_cell_value(sheet.try_into().unwrap(), row, column)
            .map_err(WorkbookError::from)
            .map_err(JsError::from)
    }

    #[wasm_bindgen(js_name = "updateCellWithText")]
    pub fn update_cell_with_text(
        &mut self,
        sheet: i32,
        row: i32,
        column: i32,
        value: &str,
    ) -> Result<(), JsError> {
        self.model
            .update_cell_with_text(sheet.try_into().unwrap(), row, column, value);
        Ok(())
    }

    #[wasm_bindgen(js_name = "updateCellWithNumber")]
    pub fn update_cell_with_number(
        &mut self,
        sheet: i32,
        row: i32,
        column: i32,
        value: f64,
    ) -> Result<(), JsError> {
        self.model
            .update_cell_with_number(sheet.try_into().unwrap(), row, column, value);
        Ok(())
    }

    #[wasm_bindgen(js_name = "updateCellWithBool")]
    pub fn update_cell_with_bool(
        &mut self,
        sheet: i32,
        row: i32,
        column: i32,
        value: bool,
    ) -> Result<(), JsError> {
        self.model
            .update_cell_with_bool(sheet.try_into().unwrap(), row, column, value);
        Ok(())
    }

    #[wasm_bindgen(js_name = "getCellFormula")]
    pub fn cell_formula(
        &self,
        sheet: i32,
        row: i32,
        column: i32,
    ) -> Result<Option<String>, JsError> {
        self.model
            .cell_formula(sheet.try_into().unwrap(), row, column)
            .map_err(WorkbookError::from)
            .map_err(JsError::from)
    }

    #[wasm_bindgen(js_name = "updateCellWithFormula")]
    pub fn update_cell_with_formula(
        &mut self,
        sheet: i32,
        row: i32,
        column: i32,
        formula: String,
    ) -> Result<(), JsError> {
        self.model
            .update_cell_with_formula(sheet.try_into().unwrap(), row, column, formula)
            .map_err(WorkbookError::from)
            .map_err(JsError::from)
    }

    #[wasm_bindgen(js_name = "setCellEmpty")]
    pub fn set_cell_empty(&mut self, sheet: i32, row: i32, column: i32) -> Result<(), JsError> {
        self.model
            .set_cell_empty(sheet.try_into().unwrap(), row, column)
            .map_err(WorkbookError::from)
            .map_err(JsError::from)
    }

    #[wasm_bindgen(js_name = "deleteCell")]
    pub fn delete_cell(&mut self, sheet: i32, row: i32, column: i32) -> Result<(), JsError> {
        self.model
            .delete_cell(sheet.try_into().unwrap(), row, column)
            .map_err(WorkbookError::from)
            .map_err(JsError::from)
    }

    #[wasm_bindgen(js_name = "getColumnWidth")]
    pub fn column_width(&self, sheet_index: u32, column: i32) -> Result<f64, JsError> {
        self.model
            .workbook
            .worksheet(sheet_index)
            .map_err(WorkbookError::from)?
            .column_width(column)
            .map_err(WorkbookError::from)
            .map_err(JsError::from)
    }

    #[wasm_bindgen(js_name = "getRowHeight")]
    pub fn row_height(&self, sheet_index: u32, row: i32) -> Result<f64, JsError> {
        self.model
            .workbook
            .worksheet(sheet_index)
            .map_err(WorkbookError::from)?
            .row_height(row)
            .map_err(WorkbookError::from)
            .map_err(JsError::from)
    }

    #[wasm_bindgen(js_name = "setColumnWidth")]
    pub fn set_column_width(
        &mut self,
        sheet_index: u32,
        column: i32,
        width: f64,
    ) -> Result<(), JsError> {
        self.model
            .workbook
            .worksheet_mut(sheet_index)
            .map_err(WorkbookError::from)?
            .set_column_width(column, width)
            .map_err(WorkbookError::from)
            .map_err(JsError::from)
    }

    #[wasm_bindgen(js_name = "setRowHeight")]
    pub fn set_row_height(
        &mut self,
        sheet_index: u32,
        row: i32,
        height: f64,
    ) -> Result<(), JsError> {
        self.model
            .workbook
            .worksheet_mut(sheet_index)
            .map_err(WorkbookError::from)?
            .set_row_height(row, height)
            .map_err(WorkbookError::from)
            .map_err(JsError::from)
    }

    #[wasm_bindgen(js_name = "getSheetDimensions")]
    pub fn sheet_dimensions(&self, sheet_index: u32) -> Result<WasmSheetDimension, JsError> {
        let dimension = self
            .model
            .workbook
            .worksheet(sheet_index)
            .map_err(WorkbookError::from)?
            .dimension();

        Ok(WasmSheetDimension {
            min_row: dimension.min_row,
            min_column: dimension.min_column,
            max_row: dimension.max_row,
            max_column: dimension.max_column,
        })
    }
}