// 셀 경계 개체 회귀의 원본 생성·조회 도우미. 검사 모듈 안에 포함한다.

type Path = Vec<(usize, usize, usize)>;

fn fixture(depth: usize, tables: bool) -> (DocumentCore, Path) {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    for (pi, text) in ["머리", "가🦦나다라", "중간", "앞뒤끝말", ""]
        .iter()
        .enumerate()
    {
        core.insert_text_native(0, pi, 0, text).unwrap();
        if pi < 4 {
            core.split_paragraph_native(0, pi, text.chars().count(), None)
                .unwrap();
        }
    }
    for (pi, at, name) in [
        (1, 1, "첫 문단 밖"),
        (1, 3, "첫 문단 안"),
        (3, 1, "끝 문단 안"),
        (3, 3, "끝 문단 밖"),
    ] {
        let result: serde_json::Value = serde_json::from_str(
            &core
                .insert_picture_native(
                    0,
                    pi,
                    at,
                    &[],
                    include_bytes!("../../assets/logo/logo-16.png"),
                    1200,
                    1400,
                    16,
                    16,
                    "png",
                    name,
                    None,
                    None,
                )
                .unwrap(),
        )
        .unwrap();
        core.set_picture_properties_native(
            0,
            pi,
            result["controlIdx"].as_u64().unwrap() as usize,
            r#"{"treatAsChar":true,"transparency":25}"#,
        )
        .unwrap();
    }
    core.apply_char_format_native(0, 1, 0, 2, r#"{"bold":true}"#)
        .unwrap();
    core.apply_char_format_native(0, 3, 2, 4, r#"{"italic":true}"#)
        .unwrap();
    core.create_table_native(0, 4, 0, 1, 2).unwrap();
    let mut document = core.document().clone();
    let template = document.sections[0].paragraphs[4]
        .controls
        .iter()
        .find_map(|control| {
            if let Control::Table(table) = control {
                Some((**table).clone())
            } else {
                None
            }
        })
        .unwrap();
    let mut paragraphs = document.sections[0].paragraphs[1..=3].to_vec();
    let plain = document.sections[0].paragraphs[2].clone();
    let host = document.sections[0].paragraphs[4].clone();
    if tables {
        for paragraph in &mut paragraphs {
            for control in &mut paragraph.controls {
                if let Control::Picture(picture) = control {
                    let mut table = template.clone();
                    table.common = picture.common.clone();
                    *control = Control::Table(Box::new(table));
                }
            }
        }
    }
    // 중간 단계의 문단 1에만 자식 표를 두어 최내측 경로 해석도 검증한다.
    let mut path = vec![(0, 0, 1); depth];
    path.last_mut().unwrap().2 = 0;
    for _ in 0..depth {
        let mut table = template.clone();
        table.cells[0].paragraphs = paragraphs;
        table.cells[1].paragraphs = vec![plain.clone()];
        let mut owner = host.clone();
        owner.controls = vec![Control::Table(Box::new(table))];
        paragraphs = vec![plain.clone(), owner];
    }
    let outer = paragraphs.pop().unwrap();
    document.sections[0].paragraphs = vec![
        document.sections[0].paragraphs[0].clone(),
        outer,
        Paragraph::default(),
    ];
    document.sections[0].raw_stream = None;
    core.set_document(document);
    (core, path)
}

fn cell<'a>(document: &'a Document, path: &[(usize, usize, usize)]) -> &'a [Paragraph] {
    let mut paragraphs = &document.sections[0].paragraphs;
    let mut pi = 1;
    for &(ci, cei, cpi) in path {
        let Control::Table(table) = &paragraphs[pi].controls[ci] else {
            panic!("표 경로")
        };
        paragraphs = &table.cells[cei].paragraphs;
        pi = cpi;
    }
    paragraphs
}

fn cell_mut<'a>(document: &'a mut Document, path: &Path) -> &'a mut Vec<Paragraph> {
    let mut paragraphs = &mut document.sections[0].paragraphs;
    let mut pi = 1;
    for &(ci, cei, cpi) in path {
        let Control::Table(table) = &mut paragraphs[pi].controls[ci] else {
            panic!("표 경로")
        };
        paragraphs = &mut table.cells[cei].paragraphs;
        pi = cpi;
    }
    paragraphs
}

fn siblings(document: &Document, path: &Path) -> Vec<Vec<Paragraph>> {
    let mut result = Vec::new();
    let mut paragraphs = &document.sections[0].paragraphs;
    let mut pi = 1;
    for &(ci, cei, cpi) in path {
        let Control::Table(table) = &paragraphs[pi].controls[ci] else {
            panic!("표 경로")
        };
        result.push(table.cells[1].paragraphs.clone());
        paragraphs = &table.cells[cei].paragraphs;
        pi = cpi;
    }
    result
}

fn images(document: &Document) -> Vec<(u16, String, Vec<u8>)> {
    document
        .bin_data_content
        .iter()
        .map(|b| (b.id, b.extension.clone(), b.data.load()))
        .collect()
}

fn names(paragraphs: &[Paragraph]) -> Vec<String> {
    paragraphs
        .iter()
        .flat_map(|p| &p.controls)
        .filter_map(|control| match control {
            Control::Picture(p) => Some(p.common.description.clone()),
            Control::Table(t) => Some(t.common.description.clone()),
            _ => None,
        })
        .collect()
}

fn style_at(paragraph: &Paragraph, at: usize) -> u32 {
    let utf16 = paragraph.char_offsets[at];
    paragraph
        .char_shapes
        .iter()
        .rfind(|s| s.start_pos <= utf16)
        .unwrap()
        .char_shape_id
}

fn delete(
    core: &mut DocumentCore,
    path: &Path,
    flat: bool,
    start: (usize, usize),
    end: (usize, usize),
) -> Result<String, rhwp::error::HwpError> {
    if flat {
        core.delete_range_native(0, start.0, start.1, end.0, end.1, Some((1, 0, 0)))
    } else {
        core.delete_range_in_cell_by_path(0, 1, path, start.0, start.1, end.0, end.1)
    }
}

fn copy(core: &mut DocumentCore, path: &Path, flat: bool) {
    if flat {
        core.copy_selection_in_cell_native(0, 1, 0, 0, 0, 2, 2, 2)
            .unwrap();
    } else {
        core.copy_selection_in_cell_by_path_native(0, 1, path, 0, 2, 2, 2)
            .unwrap();
    }
}
