//! [#4430] 내용 손실은 stderr의 시간적 상태가 아니라 성공한 산출물의 값이다.
//!
//! 실제 표 셀 안 그림과 같은 BinData를 여러 중첩 owner가 공유하는 문서를 사용해
//! HWP→HWPX→HWP, HWPX→HWP→HWPX 경계를 모두 지난다. 첫 저장에서만 정확히 한
//! 자원 손실이 보고되고, live IR은 바뀌지 않으며, placeholder를 다시 저장하는 두 번째
//! 경계는 이전 보고서를 되풀이하지 않아야 한다.
#![cfg(not(target_arch = "wasm32"))]

use std::cmp::Reverse;
use std::collections::BTreeMap;
use std::sync::Arc;

use rhwp::document_core::DocumentCore;
use rhwp::model::bin_data::{BinDataBytes, BinDataResolver};
use rhwp::model::control::Control;
use rhwp::model::paragraph::CharShapeRef;
use rhwp::serializer::{
    ContentLossCode, ContentLossReason, ContentLossReport, ContentLossSubject, SerializedFormat,
};
use rhwp::wasm_api::{DocumentExport, HwpDocument};

const NESTED_HWP: &str = "samples/pic-in-table-01.hwp";
const MULTIPLEXED_HWPX: &str = "samples/hwpx_sample2.hwpx";

#[derive(Debug)]
struct UnreadableResource;

impl BinDataResolver for UnreadableResource {
    fn resolve(&self, _key: &str) -> Vec<u8> {
        Vec::new()
    }

    fn resolve_limited(&self, _key: &str, _max_bytes: usize) -> Option<Vec<u8>> {
        None
    }

    fn resolved_len(&self, _key: &str) -> usize {
        0
    }

    fn resolved_is_empty(&self, _key: &str) -> bool {
        true
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PictureOwner {
    table_depth: usize,
    caption_depth: usize,
    resource_id: u16,
}

fn collect_picture_owners(
    controls: &[Control],
    table_depth: usize,
    caption_depth: usize,
    out: &mut Vec<PictureOwner>,
) {
    for control in controls {
        match control {
            Control::Picture(picture) => {
                out.push(PictureOwner {
                    table_depth,
                    caption_depth,
                    resource_id: picture.image_attr.bin_data_id,
                });
                if let Some(caption) = &picture.caption {
                    for paragraph in &caption.paragraphs {
                        collect_picture_owners(
                            &paragraph.controls,
                            table_depth,
                            caption_depth + 1,
                            out,
                        );
                    }
                }
            }
            Control::Table(table) => {
                for cell in &table.cells {
                    for paragraph in &cell.paragraphs {
                        collect_picture_owners(
                            &paragraph.controls,
                            table_depth + 1,
                            caption_depth,
                            out,
                        );
                    }
                }
                if let Some(caption) = &table.caption {
                    for paragraph in &caption.paragraphs {
                        collect_picture_owners(
                            &paragraph.controls,
                            table_depth,
                            caption_depth + 1,
                            out,
                        );
                    }
                }
            }
            _ => {}
        }
    }
}

fn picture_owners(core: &DocumentCore) -> Vec<PictureOwner> {
    let mut owners = Vec::new();
    for section in &core.document().sections {
        for paragraph in &section.paragraphs {
            collect_picture_owners(&paragraph.controls, 0, 0, &mut owners);
        }
    }
    owners
}

type OwnerTopology = BTreeMap<(usize, usize), usize>;

fn owner_topology(core: &DocumentCore, resource_id: u16) -> OwnerTopology {
    let mut topology = OwnerTopology::new();
    for owner in picture_owners(core)
        .into_iter()
        .filter(|owner| owner.resource_id == resource_id)
    {
        *topology
            .entry((owner.table_depth, owner.caption_depth))
            .or_default() += 1;
    }
    topology
}

fn select_nested_resource(
    core: &DocumentCore,
    minimum_depth: usize,
    minimum_owners: usize,
) -> (u16, usize) {
    let mut counts = BTreeMap::<u16, usize>::new();
    for owner in picture_owners(core) {
        if owner.table_depth >= minimum_depth {
            *counts.entry(owner.resource_id).or_default() += 1;
        }
    }
    let selected = counts
        .into_iter()
        .filter(|(_, count)| *count >= minimum_owners)
        .max_by_key(|(resource_id, count)| (*count, Reverse(*resource_id)))
        .unwrap_or_else(|| {
            panic!(
                "table depth {minimum_depth}에서 {minimum_owners}개 이상 owner가 공유하는 그림 자원이 없음"
            )
        });
    selected
}

fn find_nested_owner_paragraph(
    controls: &[Control],
    resource_id: u16,
    table_depth: usize,
    minimum_depth: usize,
) -> Option<rhwp::model::paragraph::Paragraph> {
    for control in controls {
        let Control::Table(table) = control else {
            continue;
        };
        for cell in &table.cells {
            for paragraph in &cell.paragraphs {
                if table_depth + 1 >= minimum_depth
                    && paragraph.controls.iter().any(|control| {
                        matches!(
                            control,
                            Control::Picture(picture)
                                if picture.image_attr.bin_data_id == resource_id
                        )
                    })
                {
                    return Some(paragraph.clone());
                }
                if let Some(found) = find_nested_owner_paragraph(
                    &paragraph.controls,
                    resource_id,
                    table_depth + 1,
                    minimum_depth,
                ) {
                    return Some(found);
                }
            }
        }
    }
    None
}

/// 이미 유효한 셀 문단 전체를 body로 복제해 control marker/char offset도 함께 보존한다.
fn add_body_owner_for_nested_resource(
    core: &mut DocumentCore,
    resource_id: u16,
    minimum_depth: usize,
) {
    let owner_paragraph = core
        .document()
        .sections
        .iter()
        .flat_map(|section| section.paragraphs.iter())
        .find_map(|paragraph| {
            find_nested_owner_paragraph(&paragraph.controls, resource_id, 0, minimum_depth)
        })
        .unwrap_or_else(|| panic!("BinData {resource_id}의 중첩 owner 문단 없음"));
    let section = core
        .document_mut()
        .sections
        .first_mut()
        .expect("본문 구역 없음");
    section.raw_stream = None;
    section.paragraphs.push(owner_paragraph);
}

fn replace_with_unreadable_resource(core: &mut DocumentCore, resource_id: u16) -> String {
    let content = core
        .document_mut()
        .bin_data_content
        .iter_mut()
        .find(|content| content.id == resource_id)
        .unwrap_or_else(|| panic!("BinDataContent {resource_id} 없음"));
    let extension = content.extension.clone();
    content.data = BinDataBytes::Lazy {
        resolver: Arc::new(UnreadableResource),
        key: format!("unreadable-resource-{resource_id}"),
    };
    extension
}

fn assert_nested_resource(
    core: &DocumentCore,
    resource_id: u16,
    expected_topology: &OwnerTopology,
) {
    let actual_topology = owner_topology(core, resource_id);
    assert_eq!(
        &actual_topology, expected_topology,
        "BinData {resource_id}의 body/table/caption owner topology가 왕복에서 바뀜"
    );
    let content = core
        .document()
        .bin_data_content
        .iter()
        .find(|content| content.id == resource_id)
        .unwrap_or_else(|| panic!("왕복 뒤 BinDataContent {resource_id} 없음"));
    assert!(
        content.data.is_empty(),
        "보고된 BinData {resource_id}는 왕복 뒤 빈 placeholder여야 함"
    );
}

fn assert_single_loss(
    report: &ContentLossReport,
    format: SerializedFormat,
    resource_id: u16,
    path: &str,
    reason: ContentLossReason,
) {
    assert_eq!(report.output_format(), format);
    assert_eq!(
        report.len(),
        1,
        "손실 자원 하나는 owner 수와 무관하게 한 번 보고"
    );
    let loss = &report.losses()[0];
    assert_eq!(loss.code, ContentLossCode::BinaryContentEmptied);
    assert_eq!(loss.subject, ContentLossSubject::BinaryData);
    assert_eq!(loss.resource_id, Some(resource_id));
    assert_eq!(loss.path, path);
    assert_eq!(loss.reason, reason);
}

#[test]
fn hwp_to_hwpx_to_hwp_reports_nested_resource_once_without_mutating_source() {
    let bytes = std::fs::read(NESTED_HWP).expect("중첩 그림 HWP 읽기");
    let mut source = DocumentCore::from_bytes(&bytes).expect("중첩 그림 HWP 파싱");
    let (resource_id, _) = select_nested_resource(&source, 2, 1);
    add_body_owner_for_nested_resource(&mut source, resource_id, 2);
    let topology = owner_topology(&source, resource_id);
    assert!(topology.get(&(0, 0)).copied().unwrap_or(0) >= 1);
    assert!(topology.keys().any(|(table_depth, _)| *table_depth >= 2));
    let extension = replace_with_unreadable_resource(&mut source, resource_id);
    let source_before = format!("{:#?}", source.document());

    let first = source
        .export_hwpx_native_with_report()
        .expect("HWP→HWPX reported 저장");
    assert_single_loss(
        first.content_loss(),
        SerializedFormat::Hwpx,
        resource_id,
        &format!("BinData/image{resource_id}.{extension}"),
        ContentLossReason::ResourceReadFailedOrLimitExceeded,
    );
    assert_eq!(
        format!("{:#?}", source.document()),
        source_before,
        "reported HWPX 저장은 live HWP IR을 바꾸면 안 됨"
    );

    let middle = DocumentCore::from_bytes(first.bytes()).expect("중간 HWPX 재파싱");
    assert_nested_resource(&middle, resource_id, &topology);
    let second = middle
        .export_hwp_with_adapter_snapshot_with_report()
        .expect("HWPX→HWP reported 저장");
    assert!(
        second.content_loss().is_empty(),
        "이미 materialize된 placeholder가 첫 저장의 보고서를 되풀이하면 안 됨"
    );

    let final_hwp = DocumentCore::from_bytes(second.bytes()).expect("최종 HWP 재파싱");
    assert_nested_resource(&final_hwp, resource_id, &topology);
}

#[test]
fn hwpx_to_hwp_to_hwpx_reports_one_shared_resource_across_many_nested_owners() {
    let bytes = std::fs::read(MULTIPLEXED_HWPX).expect("multiplexed HWPX 읽기");
    let mut source = DocumentCore::from_bytes(&bytes).expect("multiplexed HWPX 파싱");
    let (resource_id, _) = select_nested_resource(&source, 2, 2);
    add_body_owner_for_nested_resource(&mut source, resource_id, 2);
    let topology = owner_topology(&source, resource_id);
    assert!(topology.get(&(0, 0)).copied().unwrap_or(0) >= 1);
    assert!(topology
        .iter()
        .any(|((table_depth, _), count)| *table_depth >= 2 && *count >= 2));
    let extension = replace_with_unreadable_resource(&mut source, resource_id);
    let source_before = format!("{:#?}", source.document());

    let first = source
        .export_hwp_with_adapter_snapshot_with_report()
        .expect("HWPX→HWP reported 저장");
    assert_single_loss(
        first.content_loss(),
        SerializedFormat::Hwp,
        resource_id,
        &format!("/BinData/BIN{resource_id:04X}.{extension}"),
        ContentLossReason::RawPassthroughUnavailable,
    );
    assert_eq!(
        format!("{:#?}", source.document()),
        source_before,
        "snapshot adapter와 HWP 저장은 live HWPX IR을 바꾸면 안 됨"
    );

    let middle = DocumentCore::from_bytes(first.bytes()).expect("중간 HWP 재파싱");
    assert_nested_resource(&middle, resource_id, &topology);
    let second = middle
        .export_hwpx_native_with_report()
        .expect("HWP→HWPX reported 저장");
    assert!(
        second.content_loss().is_empty(),
        "한 자원의 여러 owner가 이전 저장의 손실을 중복/재보고하면 안 됨"
    );

    let final_hwpx = DocumentCore::from_bytes(second.bytes()).expect("최종 HWPX 재파싱");
    assert_nested_resource(&final_hwpx, resource_id, &topology);
}

#[test]
fn password_reported_wasm_exports_keep_report_attached_to_each_encrypted_artifact() {
    let bytes = std::fs::read(NESTED_HWP).expect("중첩 그림 HWP 읽기");
    let mut source = HwpDocument::from_bytes(&bytes).expect("중첩 그림 HWP 파싱");
    let (resource_id, _) = select_nested_resource(&source, 2, 1);
    add_body_owner_for_nested_resource(&mut source, resource_id, 2);
    let topology = owner_topology(&source, resource_id);
    let extension = replace_with_unreadable_resource(&mut source, resource_id);
    let source_before = format!("{:#?}", source.document());
    let password = std::process::id().to_string();

    let mut hwp = source
        .export_hwp_with_password_and_report_wasm(password.as_str())
        .expect("비밀번호 HWP reported 저장");
    let hwp_report: serde_json::Value =
        serde_json::from_str(&hwp.content_loss()).expect("HWP 보고서 JSON");
    assert_eq!(hwp_report["count"], 1);
    assert_eq!(hwp_report["losses"][0]["resourceId"], resource_id);
    assert_eq!(
        hwp_report["losses"][0]["path"],
        format!("/BinData/BIN{resource_id:04X}.{extension}")
    );
    let hwp_bytes = hwp.take_bytes().expect("암호 HWP 바이트 가져오기");
    let hwp_reloaded = DocumentCore::from_bytes_with_password(&hwp_bytes, password.as_bytes())
        .expect("암호 HWP 재파싱");
    assert_nested_resource(&hwp_reloaded, resource_id, &topology);

    let mut hwpx = source
        .export_hwpx_with_password_and_report_wasm(password.as_str())
        .expect("비밀번호 HWPX reported 저장");
    let hwpx_report: serde_json::Value =
        serde_json::from_str(&hwpx.content_loss()).expect("HWPX 보고서 JSON");
    assert_eq!(hwpx_report["count"], 1);
    assert_eq!(hwpx_report["losses"][0]["resourceId"], resource_id);
    assert_eq!(
        hwpx_report["losses"][0]["path"],
        format!("BinData/image{resource_id}.{extension}")
    );
    let hwpx_bytes = hwpx.take_bytes().expect("암호 HWPX 바이트 가져오기");
    let hwpx_reloaded = DocumentCore::from_bytes_with_password(&hwpx_bytes, password.as_bytes())
        .expect("암호 HWPX 재파싱");
    assert_nested_resource(&hwpx_reloaded, resource_id, &topology);

    assert_eq!(
        format!("{:#?}", source.document()),
        source_before,
        "비밀번호 reported 저장도 두 형식 모두 live IR을 바꾸면 안 됨"
    );
}

#[test]
fn document_export_report_survives_byte_take_and_failure_has_no_artifact() {
    let source = HwpDocument::create_empty();
    let serialized = source
        .export_hwpx_native_with_report()
        .expect("빈 문서 reported 저장");
    let mut exported: DocumentExport = serialized.into();
    let report_before_take = exported.content_loss();
    assert!(exported.has_bytes());
    let bytes = exported.take_bytes().expect("바이트 한 번 가져오기");
    assert!(!bytes.is_empty());
    assert!(!exported.has_bytes());
    assert_eq!(exported.content_loss(), report_before_take);

    let mut invalid = HwpDocument::create_empty();
    invalid.document_mut().sections[0].paragraphs[0]
        .char_shapes
        .push(CharShapeRef {
            start_pos: 0,
            char_shape_id: 42,
        });
    let failed = invalid.export_hwpx_native_with_report();
    assert!(failed.is_err(), "미등록 charShape 참조 저장은 실패해야 함");
    assert_eq!(
        exported.content_loss(),
        report_before_take,
        "실패한 저장은 이전 artifact 보고서를 변경하거나 새 결과로 가장하면 안 됨"
    );
}
