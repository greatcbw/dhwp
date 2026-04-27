use docx_rs::{
    AlignmentType, Docx, LineSpacing, Paragraph, Run, RunFonts, SpecialIndentType, Table,
    TableCell, TableRow, VAlignType, VertAlignType,
};
use rhwp::model::control::Control;
use rhwp::model::document::Document;
use rhwp::model::paragraph::Paragraph as HwpParagraph;
use rhwp::model::style::{Alignment, UnderlineType};
use rhwp::DocumentCore;
use std::io::Cursor;
use std::path::Path;

const HWPUNIT_PER_TWIP: f64 = 7200.0 / 1440.0; // 1 twip = 1/1440 inch

fn hwpunit_to_twip(hwpunit: i32) -> u32 {
    ((hwpunit as f64) / HWPUNIT_PER_TWIP).round().max(0.0) as u32
}

fn color_to_hex(r: u8, g: u8, b: u8) -> String {
    format!("{:02X}{:02X}{:02X}", r, g, b)
}

pub fn ensure_docx_path(path: &Path) -> Result<(), String> {
    match path.extension().and_then(|e| e.to_str()) {
        Some(ext) if ext.eq_ignore_ascii_case("docx") => Ok(()),
        _ => Err(format!(
            "DOCX 내보내기 경로는 .docx 확장자여야 합니다: {}",
            path.display()
        )),
    }
}

pub fn export_core_to_docx(core: &DocumentCore, path: &Path) -> Result<(), String> {
    let doc = core.document();
    let docx = build_docx(doc)?;

    let mut buf: Vec<u8> = Vec::new();
    docx.build()
        .pack(Cursor::new(&mut buf))
        .map_err(|e| format!("DOCX 패킹 실패: {}", e))?;

    atomic_write(path, &buf)
}

fn build_docx(doc: &Document) -> Result<Docx, String> {
    let mut docx = Docx::new();

    for section in &doc.sections {
        for para in &section.paragraphs {
            // 표 컨트롤이 있는 문단은 표로 변환
            let table_control = para.controls.iter().find_map(|c| {
                if let Control::Table(t) = c { Some(t.as_ref()) } else { None }
            });

            if let Some(table) = table_control {
                docx = docx.add_table(convert_table(table, doc)?);
            } else {
                docx = docx.add_paragraph(convert_paragraph(para, doc)?);
            }
        }
    }

    Ok(docx)
}

fn convert_paragraph(
    para: &HwpParagraph,
    doc: &Document,
) -> Result<Paragraph, String> {
    let para_shape = doc
        .doc_info
        .para_shapes
        .get(para.para_shape_id as usize)
        .cloned()
        .unwrap_or_default();

    let align = match para_shape.alignment {
        Alignment::Left => AlignmentType::Left,
        Alignment::Right => AlignmentType::Right,
        Alignment::Center => AlignmentType::Center,
        Alignment::Justify | Alignment::Distribute | Alignment::Split => AlignmentType::Both,
    };

    let indent_left = hwpunit_to_twip(para_shape.margin_left);
    let indent_right = hwpunit_to_twip(para_shape.margin_right);
    let special_indent = if para_shape.indent < 0 {
        Some(SpecialIndentType::Hanging(hwpunit_to_twip(-para_shape.indent) as i32))
    } else if para_shape.indent > 0 {
        Some(SpecialIndentType::FirstLine(hwpunit_to_twip(para_shape.indent) as i32))
    } else {
        None
    };
    let space_before = hwpunit_to_twip(para_shape.spacing_before);
    let space_after = hwpunit_to_twip(para_shape.spacing_after);

    let mut docx_para = Paragraph::new()
        .align(align)
        .indent(Some(indent_left as i32), special_indent, Some(indent_right as i32), None)
        .line_spacing(LineSpacing::new().before(space_before).after(space_after));

    // 텍스트를 글자모양 구간별로 Run으로 분할
    for run in build_runs(para, doc) {
        docx_para = docx_para.add_run(run);
    }

    Ok(docx_para)
}

fn build_runs(para: &HwpParagraph, doc: &Document) -> Vec<Run> {
    if para.text.is_empty() {
        return vec![];
    }

    let chars: Vec<char> = para.text.chars().collect();
    let utf16_offsets: Vec<u32> = para.char_offsets.clone();

    // char_shapes: 각 구간의 시작 UTF-16 위치와 글자모양 ID
    // 구간 경계를 char index 기준으로 재계산
    let shape_segments = build_shape_segments(&chars, &utf16_offsets, &para.char_shapes);

    shape_segments
        .into_iter()
        .filter_map(|(text, shape_id)| {
            if text.is_empty() {
                return None;
            }
            let char_shape = doc
                .doc_info
                .char_shapes
                .get(shape_id)
                .cloned()
                .unwrap_or_default();
            Some(apply_char_shape(Run::new().add_text(&text), &char_shape, doc))
        })
        .collect()
}

fn build_shape_segments(
    chars: &[char],
    utf16_offsets: &[u32],
    char_shapes: &[rhwp::model::paragraph::CharShapeRef],
) -> Vec<(String, usize)> {
    if chars.is_empty() {
        return vec![];
    }

    // char_shapes가 없으면 기본 모양(0)으로 전체 처리
    if char_shapes.is_empty() {
        let text: String = chars.iter().collect();
        return vec![(text, 0)];
    }

    let mut segments: Vec<(String, usize)> = Vec::new();
    let mut current_shape_idx = 0usize;
    let mut segment_start = 0usize;

    for (char_idx, &ch) in chars.iter().enumerate() {
        // 제어 문자 스킵 (0x00~0x1F 중 탭/줄바꿈 제외)
        if (ch as u32) < 0x20 && ch != '\t' && ch != '\n' {
            continue;
        }

        let utf16_pos = utf16_offsets.get(char_idx).copied().unwrap_or(char_idx as u32);

        // 다음 글자모양 구간으로 전환 여부 확인
        let next_shape_idx = char_shapes
            .iter()
            .rposition(|cs| cs.start_pos <= utf16_pos)
            .unwrap_or(0);

        if next_shape_idx != current_shape_idx {
            // 이전 구간 텍스트 수집
            let text: String = chars[segment_start..char_idx]
                .iter()
                .filter(|&&c| !((c as u32) < 0x20 && c != '\t' && c != '\n'))
                .collect();
            let shape_id = char_shapes
                .get(current_shape_idx)
                .map(|cs| cs.char_shape_id as usize)
                .unwrap_or(0);
            if !text.is_empty() {
                segments.push((text, shape_id));
            }
            current_shape_idx = next_shape_idx;
            segment_start = char_idx;
        }
    }

    // 마지막 구간
    let text: String = chars[segment_start..]
        .iter()
        .filter(|&&c| !((c as u32) < 0x20 && c != '\t' && c != '\n'))
        .collect();
    let shape_id = char_shapes
        .get(current_shape_idx)
        .map(|cs| cs.char_shape_id as usize)
        .unwrap_or(0);
    if !text.is_empty() {
        segments.push((text, shape_id));
    }

    segments
}

fn apply_char_shape(
    mut run: Run,
    cs: &rhwp::model::style::CharShape,
    doc: &Document,
) -> Run {
    if cs.bold {
        run = run.bold();
    }
    if cs.italic {
        run = run.italic();
    }
    if cs.underline_type != UnderlineType::None {
        run = run.underline("single");
    }
    if cs.strikethrough {
        run = run.strike();
    }
    if cs.superscript {
        run.run_property = run.run_property.vert_align(VertAlignType::SuperScript);
    } else if cs.subscript {
        run.run_property = run.run_property.vert_align(VertAlignType::SubScript);
    }

    // 글자 크기: base_size / 100 = pt, DOCX half-point 단위
    if cs.base_size > 0 {
        let half_pt = (cs.base_size / 50).max(1) as usize;
        run = run.size(half_pt);
    }

    // 글자 색 (검정이면 생략); text_color는 BGR u32 (0x00BBGGRR)
    let color = cs.text_color;
    let (r, g, b) = ((color & 0xFF) as u8, ((color >> 8) & 0xFF) as u8, ((color >> 16) & 0xFF) as u8);
    if r != 0 || g != 0 || b != 0 {
        run = run.color(&color_to_hex(r, g, b));
    }

    // 글꼴: 한글 폰트(index 0), 영어 폰트(index 1)
    let korean_font = font_name(doc, 0, cs.font_ids[0]);
    let english_font = font_name(doc, 1, cs.font_ids[1]);
    if let Some(name) = &korean_font {
        let mut fonts = RunFonts::new();
        fonts = fonts.east_asia(name);
        if let Some(en) = &english_font {
            fonts = fonts.ascii(en);
        }
        run = run.fonts(fonts);
    }

    run
}

fn font_name(doc: &Document, lang_idx: usize, font_id: u16) -> Option<String> {
    doc.doc_info
        .font_faces
        .get(lang_idx)?
        .get(font_id as usize)
        .map(|f| f.name.clone())
        .filter(|n| !n.is_empty())
}

fn convert_table(
    table: &rhwp::model::table::Table,
    doc: &Document,
) -> Result<Table, String> {
    let row_count = table.row_count as usize;

    // rowspan이 있는 셀은 아랫 행에 vMerge::Continue 셀을 반드시 삽입해야 함.
    // continuations[row] = Vec<(col_start, col_span)>
    let mut continuations: Vec<Vec<(usize, usize)>> = vec![vec![]; row_count];
    for cell in &table.cells {
        if cell.row_span > 1 {
            let start_row = cell.row as usize;
            let col_span = (cell.col_span as usize).max(1);
            for r in 1..(cell.row_span as usize) {
                let target_row = start_row + r;
                if target_row < row_count {
                    continuations[target_row].push((cell.col as usize, col_span));
                }
            }
        }
    }
    for cont in &mut continuations {
        cont.sort_by_key(|&(col, _)| col);
    }

    let mut rows: Vec<TableRow> = Vec::with_capacity(row_count);

    for row_idx in 0..row_count {
        // 이 행에서 시작하는 실제 셀 (col 순 정렬)
        let mut row_cells: Vec<&rhwp::model::table::Cell> = table
            .cells
            .iter()
            .filter(|c| c.row as usize == row_idx)
            .collect();
        row_cells.sort_by_key(|c| c.col);

        // 실제 셀과 Continue 셀을 컬럼 순서대로 병합
        let cont = &continuations[row_idx];
        let mut real_iter = row_cells.iter().peekable();
        let mut cont_iter = cont.iter().peekable();
        let mut cells: Vec<TableCell> = Vec::new();

        loop {
            let next_real = real_iter.peek().map(|c| c.col as usize);
            let next_cont = cont_iter.peek().map(|c| c.0);

            match (next_real, next_cont) {
                (None, None) => break,
                (Some(_), None) => {
                    let cell = *real_iter.next().unwrap();
                    cells.push(build_table_cell(cell, doc)?);
                }
                (None, Some(_)) => {
                    let &(_, col_span) = cont_iter.next().unwrap();
                    cells.push(make_continue_cell(col_span));
                }
                (Some(rc), Some(cc)) => {
                    if rc <= cc {
                        let cell = *real_iter.next().unwrap();
                        cells.push(build_table_cell(cell, doc)?);
                    } else {
                        let &(_, col_span) = cont_iter.next().unwrap();
                        cells.push(make_continue_cell(col_span));
                    }
                }
            }
        }

        rows.push(TableRow::new(cells));
    }

    Ok(Table::new(rows))
}

/// rowspan 연속 행에 삽입하는 빈 Continue 셀
fn make_continue_cell(col_span: usize) -> TableCell {
    let mut cell = TableCell::new().vertical_merge(docx_rs::VMergeType::Continue);
    if col_span > 1 {
        cell = cell.grid_span(col_span);
    }
    cell
}

/// 실제 내용이 있는 일반 셀 빌드
fn build_table_cell(
    cell: &rhwp::model::table::Cell,
    doc: &Document,
) -> Result<TableCell, String> {
    let mut docx_cell = TableCell::new();

    for para in &cell.paragraphs {
        docx_cell = docx_cell.add_paragraph(convert_paragraph(para, doc)?);
    }

    if cell.col_span > 1 {
        docx_cell = docx_cell.grid_span(cell.col_span as usize);
    }

    if cell.row_span > 1 {
        docx_cell = docx_cell.vertical_merge(docx_rs::VMergeType::Restart);
    }

    let valign = match cell.vertical_align {
        rhwp::model::table::VerticalAlign::Top => VAlignType::Top,
        rhwp::model::table::VerticalAlign::Center => VAlignType::Center,
        rhwp::model::table::VerticalAlign::Bottom => VAlignType::Bottom,
    };
    docx_cell = docx_cell.vertical_align(valign);

    Ok(docx_cell)
}

fn atomic_write(path: &Path, data: &[u8]) -> Result<(), String> {
    let parent = path.parent().ok_or_else(|| {
        format!("DOCX 저장 경로의 상위 디렉터리를 찾을 수 없습니다: {}", path.display())
    })?;

    let tmp_path = parent.join(format!(
        ".hop-docx-{}.tmp",
        uuid::Uuid::new_v4().simple()
    ));

    std::fs::write(&tmp_path, data)
        .map_err(|e| format!("DOCX 임시 파일 쓰기 실패: {}", e))?;

    std::fs::rename(&tmp_path, path).map_err(|e| {
        let _ = std::fs::remove_file(&tmp_path);
        format!("DOCX 파일 교체 실패: {}", e)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hwpunit_to_twip_converts_correctly() {
        // 7200 HWPUNIT = 1 inch = 1440 twip
        assert_eq!(hwpunit_to_twip(7200), 1440);
        assert_eq!(hwpunit_to_twip(0), 0);
    }

    #[test]
    fn ensure_docx_path_accepts_docx_extension() {
        assert!(ensure_docx_path(Path::new("/tmp/out.docx")).is_ok());
        assert!(ensure_docx_path(Path::new("/tmp/out.DOCX")).is_ok());
    }

    #[test]
    fn ensure_docx_path_rejects_other_extensions() {
        assert!(ensure_docx_path(Path::new("/tmp/out.pdf")).is_err());
        assert!(ensure_docx_path(Path::new("/tmp/out.hwp")).is_err());
    }

    #[test]
    fn color_to_hex_formats_correctly() {
        assert_eq!(color_to_hex(255, 0, 0), "FF0000");
        assert_eq!(color_to_hex(0, 128, 255), "0080FF");
    }
}
