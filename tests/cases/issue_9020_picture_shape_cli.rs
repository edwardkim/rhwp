//! 관계형(그림·도형 컨트롤의 문단·칸 소속, 저장 → 다시 열기 → 재저장 → 다시 열기 뒤 소속 보존, 거부한 편집 ↔ 출력 파일 없음; 실물 px 검사 아님): #9020 회귀.
//! 주석 갱신: 2026-11-12T23:55:00+09:00
//! 이 파일은 조판 회귀가 아닌 CLI 명령 계약 검사다. 새 명령은 코어의 기존 편집 함수
//! (`set_shape_properties_native`·`relocate_floating_picture_native`·`insert_picture_in_cell_native`)를
//! 부르기만 하고 조판 규칙과 그림 배치 계산을 바꾸지 않으므로, 독립 PDF 시각 측정 대상이 아니며 시각
//! 점수를 추정하지 않는다.
//!
//! [#9020] 도형 속성 변경, 떠 있는 그림을 다른 문단·표 칸으로 옮기기, 표 칸 문단 안에 그림 넣기를
//! CLI·MCP 로 할 수 없었다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

use rhwp::model::control::Control;
use rhwp::model::document::Document;
use rhwp::model::paragraph::Paragraph;
use rhwp::model::shape::{HorzRelTo, TextWrap, VertRelTo};
use rhwp::wasm_api::HwpDocument;
use serde_json::Value;

static TEMP_SERIAL: AtomicU64 = AtomicU64::new(0);

/// 시험 문서의 표 2 (구역 0 문단 2, 1행×2열).
const TABLE_NO: &str = "2";
const TABLE_PARA: usize = 2;

fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}

fn temp(tag: &str, ext: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "rhwp-9020-{tag}-{}-{}.{ext}",
        std::process::id(),
        TEMP_SERIAL.fetch_add(1, Ordering::Relaxed),
    ))
}

fn repo_path(rel: &str) -> String {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(rel)
        .to_string_lossy()
        .into_owned()
}

fn sample() -> String {
    repo_path("samples/paper-square-table-tac-host.hwpx")
}

fn image() -> String {
    repo_path("samples/images/moogung.jpg")
}

fn run(args: &[&str]) -> Output {
    Command::new(rhwp_bin()).args(args).output().unwrap()
}

fn ok_envelope(args: &[&str]) -> Value {
    let out = run(args);
    assert_eq!(
        out.status.code(),
        Some(0),
        "{args:?}\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
    serde_json::from_slice(&out.stdout).unwrap()
}

fn open(path: &Path) -> HwpDocument {
    HwpDocument::from_bytes(&std::fs::read(path).unwrap()).unwrap()
}

/// 저장 → 다시 열기 → 재저장(같은 형식) → 다시 열기.
fn reopen_twice(path: &Path) -> HwpDocument {
    let mut doc = open(path);
    let bytes = if path.extension().is_some_and(|e| e == "hwpx") {
        doc.export_hwpx_native().unwrap()
    } else {
        doc.export_hwp_with_adapter().unwrap()
    };
    HwpDocument::from_bytes(&bytes).unwrap()
}

fn cell_paragraph(doc: &Document, table_ctrl: usize, cell: usize) -> &Paragraph {
    match &doc.sections[0].paragraphs[TABLE_PARA].controls[table_ctrl] {
        Control::Table(table) => &table.cells[cell].paragraphs[0],
        other => panic!("표가 아니다: {other:?}"),
    }
}

fn table_ctrl(doc: &Document) -> usize {
    doc.sections[0].paragraphs[TABLE_PARA]
        .controls
        .iter()
        .position(|c| matches!(c, Control::Table(_)))
        .expect("표")
}

fn picture_count(p: &Paragraph) -> usize {
    p.controls
        .iter()
        .filter(|c| matches!(c, Control::Picture(_)))
        .count()
}

fn index(v: &Value, key: &str) -> usize {
    v[key]
        .as_u64()
        .unwrap_or_else(|| panic!("결과 JSON 에 {key} 가 없다: {v}")) as usize
}

/// 본문 문단 1 에 떠 있는 그림을 넣고 (파일, 컨트롤 번호) 를 돌려준다.
fn with_body_picture(ext: &str) -> (PathBuf, usize) {
    let path = temp("body", ext);
    let env = ok_envelope(&[
        "edit",
        "insert-picture",
        &sample(),
        "--image",
        &image(),
        "--para",
        "1",
        "--width",
        "6000",
        "--x",
        "3000",
        "--y",
        "4000",
        "--json",
        "-o",
        path.to_str().unwrap(),
    ]);
    (path, index(&env, "ctrl"))
}

#[test]
fn insert_picture_into_table_cell_stays_in_the_cell_after_resave() {
    for ext in ["hwpx", "hwp"] {
        let path = temp("cell", ext);
        let env = ok_envelope(&[
            "edit",
            "insert-picture",
            &sample(),
            "--image",
            &image(),
            "--table",
            TABLE_NO,
            "--row",
            "0",
            "--col",
            "1",
            "--width",
            "3000",
            "--json",
            "-o",
            path.to_str().unwrap(),
        ]);
        assert!(
            env["ctrl"].is_null(),
            "칸 안 그림은 본문 ctrl 이 없다: {env}"
        );
        assert_eq!(env["paragraph"], TABLE_PARA as u64, "{env}");
        let cell_ctrl = index(&env, "cellCtrl");
        for doc in [open(&path), reopen_twice(&path)] {
            let doc = doc.document();
            let para = cell_paragraph(doc, table_ctrl(doc), 1);
            assert!(
                matches!(para.controls.get(cell_ctrl), Some(Control::Picture(_))),
                "{ext}: 칸 문단의 cellCtrl 이 그림이 아니다"
            );
            assert_eq!(picture_count(para), 1, "{ext}");
        }
        let _ = std::fs::remove_file(&path);
    }

    // 칸 지정과 본문 문단 지정을 함께 주면 인자 오류다.
    let out = run(&[
        "edit",
        "insert-picture",
        &sample(),
        "--image",
        &image(),
        "--table",
        TABLE_NO,
        "--row",
        "0",
        "--col",
        "1",
        "--para",
        "1",
    ]);
    assert_eq!(out.status.code(), Some(2), "{out:?}");
}

#[test]
fn move_picture_to_another_paragraph_and_into_a_cell() {
    for ext in ["hwpx", "hwp"] {
        let (src, ctrl) = with_body_picture(ext);
        let before = open(&src);
        let before_pics = picture_count(&before.document().sections[0].paragraphs[3]);

        // 본문 문단 1 → 3. --x/--y 를 생략하면 거리는 그대로다.
        let moved = temp("moved", ext);
        let env = ok_envelope(&[
            "edit",
            "move-picture",
            src.to_str().unwrap(),
            "--section",
            "0",
            "--para",
            "1",
            "--ctrl",
            &ctrl.to_string(),
            "--to-para",
            "3",
            "--json",
            "-o",
            moved.to_str().unwrap(),
        ]);
        let to_ctrl = index(&env, "toCtrl");
        for doc in [open(&moved), reopen_twice(&moved)] {
            let paras = &doc.document().sections[0].paragraphs;
            assert_eq!(
                picture_count(&paras[1]),
                0,
                "{ext}: 원래 문단에 그림이 남았다"
            );
            assert_eq!(picture_count(&paras[3]), before_pics + 1, "{ext}");
            match &paras[3].controls[to_ctrl] {
                Control::Picture(pic) => {
                    assert_eq!(pic.common.horizontal_offset, 3000, "{ext}");
                    assert_eq!(pic.common.vertical_offset, 4000, "{ext}");
                }
                other => panic!("{ext}: toCtrl 이 그림이 아니다: {other:?}"),
            }
        }

        // 본문 문단 1 → 표 칸. 기준이 칸 문단으로 바뀐다.
        let in_cell = temp("in-cell", ext);
        let env = ok_envelope(&[
            "edit",
            "move-picture",
            src.to_str().unwrap(),
            "--section",
            "0",
            "--para",
            "1",
            "--ctrl",
            &ctrl.to_string(),
            "--table",
            TABLE_NO,
            "--row",
            "0",
            "--col",
            "0",
            "--json",
            "-o",
            in_cell.to_str().unwrap(),
        ]);
        assert!(env["toCtrl"].is_null(), "{env}");
        let cell_ctrl = index(&env, "cellCtrl");
        for doc in [open(&in_cell), reopen_twice(&in_cell)] {
            let doc = doc.document();
            assert_eq!(picture_count(&doc.sections[0].paragraphs[1]), 0, "{ext}");
            match &cell_paragraph(doc, table_ctrl(doc), 0).controls[cell_ctrl] {
                Control::Picture(pic) => {
                    assert_eq!(pic.common.vert_rel_to, VertRelTo::Para, "{ext}");
                    assert_eq!(pic.common.horz_rel_to, HorzRelTo::Column, "{ext}");
                }
                other => panic!("{ext}: cellCtrl 이 그림이 아니다: {other:?}"),
            }
        }
        for p in [&src, &moved, &in_cell] {
            let _ = std::fs::remove_file(p);
        }
    }
}

#[test]
fn move_picture_rejects_missing_or_conflicting_target_and_non_pictures() {
    let (src, ctrl) = with_body_picture("hwpx");
    let ctrl = ctrl.to_string();
    let table_para = TABLE_PARA.to_string();
    // (문단, 컨트롤, 옮길 자리, 기대 종료 코드)
    let cases: [(&str, &str, Vec<&str>, i32); 3] = [
        ("1", ctrl.as_str(), vec![], 2),
        (
            "1",
            ctrl.as_str(),
            vec![
                "--to-para",
                "3",
                "--table",
                TABLE_NO,
                "--row",
                "0",
                "--col",
                "0",
            ],
            2,
        ),
        // 표를 담은 문단의 첫 컨트롤은 그림이 아니라 표다.
        (table_para.as_str(), "0", vec!["--to-para", "3"], 1),
    ];
    for (para, target_ctrl, target, code) in cases {
        let out_path = temp("rejected", "hwpx");
        let mut args = vec![
            "edit",
            "move-picture",
            src.to_str().unwrap(),
            "--section",
            "0",
            "--para",
            para,
            "--ctrl",
            target_ctrl,
        ];
        args.extend(target);
        args.extend(["--dry-run", "-o", out_path.to_str().unwrap()]);
        let out = run(&args);
        assert_eq!(out.status.code(), Some(code), "{args:?}: {out:?}");
        assert!(out.stdout.is_empty(), "{args:?}");
        assert!(!out_path.exists(), "{args:?}");
    }
    let _ = std::fs::remove_file(&src);
}

#[test]
fn set_shape_changes_shape_and_rejects_bad_props() {
    let with_shape = temp("shape", "hwpx");
    ok_envelope(&[
        "edit",
        "insert-shape",
        &sample(),
        "--para",
        "1",
        "--width",
        "5000",
        "--height",
        "3000",
        "--json",
        "-o",
        with_shape.to_str().unwrap(),
    ]);
    let doc = open(&with_shape);
    let ctrl = doc.document().sections[0].paragraphs[1]
        .controls
        .iter()
        .position(|c| matches!(c, Control::Shape(_)))
        .expect("넣은 도형");
    let ctrl_s = ctrl.to_string();

    let changed = temp("shape-set", "hwpx");
    ok_envelope(&[
        "edit",
        "set-shape",
        with_shape.to_str().unwrap(),
        "--section",
        "0",
        "--para",
        "1",
        "--ctrl",
        &ctrl_s,
        "--props",
        r#"{"textWrap": "TopAndBottom", "horzOffset": 1200}"#,
        "--json",
        "-o",
        changed.to_str().unwrap(),
    ]);
    for doc in [open(&changed), reopen_twice(&changed)] {
        match &doc.document().sections[0].paragraphs[1].controls[ctrl] {
            Control::Shape(shape) => {
                assert_eq!(shape.common().text_wrap, TextWrap::TopAndBottom);
                assert_eq!(shape.common().horizontal_offset, 1200);
            }
            other => panic!("도형이 아니다: {other:?}"),
        }
    }

    for (props, target_ctrl, code) in [
        (r#"{"textWrap":"topAndBottom"}"#, ctrl_s.as_str(), 2),
        (r#"{"brightness":10}"#, ctrl_s.as_str(), 2),
        (r#"{"horzOffset":-1}"#, ctrl_s.as_str(), 2),
        // 문단 1 에는 표가 없고, 도형이 아닌 자리는 실행 오류다.
        (r#"{"textWrap":"TopAndBottom"}"#, "99", 1),
    ] {
        let out_path = temp("shape-rejected", "hwpx");
        let out = run(&[
            "edit",
            "set-shape",
            with_shape.to_str().unwrap(),
            "--section",
            "0",
            "--para",
            "1",
            "--ctrl",
            target_ctrl,
            "--props",
            props,
            "--dry-run",
            "-o",
            out_path.to_str().unwrap(),
        ]);
        assert_eq!(out.status.code(), Some(code), "{props}: {out:?}");
        assert!(out.stdout.is_empty(), "{props}");
        assert!(!out_path.exists(), "{props}");
    }
    let _ = std::fs::remove_file(&with_shape);
    let _ = std::fs::remove_file(&changed);
}

#[test]
fn check_shape_props_accepts_setter_values_and_rejects_the_rest() {
    use rhwp::document_core::check_shape_props;
    assert!(check_shape_props(
        r#"{"textWrap":"TopAndBottom","horzOffset":1000,"fillType":"solid"}"#
    )
    .is_ok());
    for json in [
        r#"{"horzOffset":-1}"#,
        r#"{"fillType":"Solid"}"#,
        r#"{"lineType":64}"#,
        r#"{"brightness":10}"#,
        r#"{"tbVerticalAlign":"Middle"}"#,
    ] {
        assert!(check_shape_props(json).is_err(), "거부해야 함: {json}");
    }
}

/// `src` 에서 `fn name(` 으로 시작하는 함수 본문(다음 함수 정의 앞까지)을 잘라 낸다.
fn fn_body<'a>(src: &'a str, name: &str) -> &'a str {
    let start = src
        .find(&format!("fn {name}("))
        .unwrap_or_else(|| panic!("함수 {name} 이 없다"));
    let rest = &src[start + 3..];
    let end = ["\n    pub fn ", "\n    pub(crate) fn ", "\n    fn "]
        .iter()
        .filter_map(|marker| rest.find(marker))
        .min()
        .unwrap_or(rest.len());
    &src[start..start + 3 + end]
}

/// 검사가 받는 키와 도형 setter 가 읽는 키가 같아야 한다 (#9019 의 그림 검사와 같은 이유).
#[test]
fn shape_prop_list_matches_the_setter() {
    let shape = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/document_core/commands/object_ops/shape.rs"
    ));
    let common = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/document_core/commands/object_ops/common.rs"
    ));
    let mut read = std::collections::BTreeSet::new();
    for body in [
        fn_body(shape, "set_shape_properties_native"),
        fn_body(shape, "apply_shape_caption_props"),
        fn_body(common, "apply_common_obj_attr_from_json"),
    ] {
        for (i, _) in body.match_indices("props_json, \"") {
            let rest = &body[i + "props_json, \"".len()..];
            read.insert(rest[..rest.find('"').unwrap()].to_string());
        }
    }
    let err = rhwp::document_core::check_shape_props(r#"{"__unknown__":1}"#).unwrap_err();
    let list = &err[err.find("받는 키: ").expect("받는 키 목록") + "받는 키: ".len()..];
    let listed: std::collections::BTreeSet<String> = list[..list.rfind(')').unwrap()]
        .split(", ")
        .map(str::to_string)
        .collect();
    assert_eq!(listed, read);
}
