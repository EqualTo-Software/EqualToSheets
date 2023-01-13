mod _rels;
mod doc_props;
mod escape;
mod shared_strings;
mod styles;
mod workbook;
mod workbook_xml_rels;
mod worksheets;
mod xml_constants;

use std::io::BufWriter;
use std::{fs, io::Write};

use equalto_calc::expressions::utils::number_to_column;
use equalto_calc::model::Model;
use equalto_calc::types::Workbook;

use self::xml_constants::XML_DECLARATION;

use crate::error::XlsxError;

#[cfg(test)]
mod test;

fn get_content_types_xml(workbook: &Workbook) -> String {
    // A list of all files in the zip
    let mut content = vec![
        r#"<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">"#.to_string(),
        r#"<Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>"#.to_string(),
        r#"<Default Extension="xml" ContentType="application/xml"/>"#.to_string(),
        r#"<Override PartName="/xl/workbook.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml"/>"#.to_string(),
    ];
    for worksheet in 0..workbook.worksheets.len() {
        let sheet = format!(
            r#"<Override PartName="/xl/worksheets/sheet{}.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml"/>"#,
            worksheet + 1
        );
        content.push(sheet);
    }
    // we skip the theme and calcChain
    // r#"<Override PartName="/xl/theme/theme1.xml" ContentType="application/vnd.openxmlformats-officedocument.theme+xml"/>"#,
    // r#"<Override PartName="/xl/calcChain.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.calcChain+xml"/>"#,
    content.extend([
        r#"<Override PartName="/xl/styles.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.styles+xml"/>"#.to_string(),
        r#"<Override PartName="/xl/sharedStrings.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.sharedStrings+xml"/>"#.to_string(),
        r#"<Override PartName="/docProps/core.xml" ContentType="application/vnd.openxmlformats-package.core-properties+xml"/>"#.to_string(),
        r#"<Override PartName="/docProps/app.xml" ContentType="application/vnd.openxmlformats-officedocument.extended-properties+xml"/>"#.to_string(),
        r#"</Types>"#.to_string(),
    ]);
    format!("{XML_DECLARATION}\n{}", content.join(""))
}

/// Exports a model to an xlsx file
pub fn save_to_xlsx(model: &Model, file_name: &str) -> Result<(), XlsxError> {
    let workbook = &model.workbook;
    let file_path = std::path::Path::new(&file_name);
    if file_path.exists() {
        return Err(XlsxError::IO(format!("file {} already exists", file_name)));
    }
    let file = fs::File::create(file_path).unwrap();
    let writer = BufWriter::new(file);
    let mut zip = zip::ZipWriter::new(writer);

    let options =
        zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Stored);

    // root folder
    zip.start_file("[Content_Types].xml", options)?;
    zip.write_all(get_content_types_xml(workbook).as_bytes())?;

    zip.add_directory("docProps", options)?;
    zip.start_file("docProps/app.xml", options)?;
    zip.write_all(doc_props::get_app_xml(workbook).as_bytes())?;
    zip.start_file("docProps/core.xml", options)?;
    zip.write_all(doc_props::get_core_xml(workbook).as_bytes())?;

    // Package-relationship item
    zip.add_directory("_rels", options)?;
    zip.start_file("_rels/.rels", options)?;
    zip.write_all(_rels::get_dot_rels(workbook).as_bytes())?;

    zip.add_directory("xl", options)?;
    zip.start_file("xl/sharedStrings.xml", options)?;
    zip.write_all(shared_strings::get_shared_strings_xml(workbook).as_bytes())?;
    zip.start_file("xl/styles.xml", options)?;
    zip.write_all(styles::get_styles_xml(workbook).as_bytes())?;
    zip.start_file("xl/workbook.xml", options)?;
    zip.write_all(workbook::get_workbook_xml(workbook).as_bytes())?;

    zip.add_directory("xl/_rels", options)?;
    zip.start_file("xl/_rels/workbook.xml.rels", options)?;
    zip.write_all(workbook_xml_rels::get_workbook_xml_rels(workbook).as_bytes())?;

    zip.add_directory("xl/worksheets", options)?;
    for (sheet_index, worksheet) in workbook.worksheets.iter().enumerate() {
        let id = sheet_index + 1;
        zip.start_file(&format!("xl/worksheets/sheet{id}.xml"), options)?;
        let (row_min, column_min, row_max, column_max) =
            model.get_sheet_dimension(sheet_index as u32);
        let column_min_str = number_to_column(column_min).unwrap();
        let column_max_str = number_to_column(column_max).unwrap();
        let sheet_dimension_str = &format!("{column_min_str}{row_min}:{column_max_str}{row_max}");
        zip.write_all(
            worksheets::get_worksheet_xml(
                worksheet,
                &model.parsed_formulas[sheet_index],
                sheet_dimension_str,
            )
            .as_bytes(),
        )?;
    }

    zip.finish()?;

    Ok(())
}

/// Exports an internal representation of a workbook into an equivalent EqualTo json format
pub fn save_to_json(workbook: Workbook, output: &str) {
    let s = serde_json::to_string(&workbook).unwrap();
    let file_path = std::path::Path::new(output);
    let mut file = fs::File::create(file_path).unwrap();
    file.write_all(s.as_bytes()).unwrap();
}
