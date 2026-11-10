//! 고정형: HWP 구역 정의의 바탕쪽 목록 수와 메모 모양 번호라는 독립 바이너리 필드를 검사한다.
//! 화면 배치의 px 고정 검사가 아니며, 댓글 보존은 편집 전후의 내용·부모 관계를 대조한다.
//! 근거: PR #8866의 한컴 12.30 바탕쪽 저장본과 PR #8885의 한컴 12.30 메모 저장본.
//! 댓글 검사는 관계형이며, 날짜를 포함한 전체 조회 결과를 같은 저장 형식의 무편집 대조군과 비교한다.
//! HWP FILETIME의 현지 시각 변환과 한컴 HWPX의 리터럴 Z 표기를 동일한 UTC 문자열로 가정하지 않는다.
//! 주석 갱신: 2026-11-10T18:21:54+09:00.

use rhwp::document_core::DocumentCore;

const SHAPE: &str = r##"{"width":19842,"lineType":2,"lineWidth":7,"lineColor":"#C75252","fillColor":"#DFEAF5","activeColor":"#FAEDD2"}"##;

fn add_master_pages(core: &mut DocumentCore) {
    core.create_master_page_native(0, r#"{"applyTo":0}"#)
        .unwrap();
    core.create_master_page_native(0, r#"{"kind":"last"}"#)
        .unwrap();
    core.create_master_page_native(0, r#"{"kind":"page","pageNumber":2,"overlap":true}"#)
        .unwrap();
}

#[test]
fn memo_shape_and_tail_master_count_keep_independent_section_extra_slots() {
    for memo_first in [false, true] {
        let mut core = DocumentCore::new_empty();
        core.create_blank_document_native().unwrap();
        if memo_first {
            core.set_memo_shape_json(Some(0), SHAPE).unwrap();
        }
        add_master_pages(&mut core);
        if !memo_first {
            core.set_memo_shape_json(Some(0), SHAPE).unwrap();
        }

        // HWPX 경유 저장도 같은 독립 필드로 돌아와야 한다.
        for bytes in [
            core.export_hwp_native().unwrap(),
            core.export_hwpx_native().unwrap(),
        ] {
            let reopened = DocumentCore::from_bytes(&bytes).unwrap();
            assert_eq!(reopened.get_memo_shape_json(0).unwrap(), SHAPE);
            let hwp = reopened.export_hwp_native().unwrap();
            let hwp = DocumentCore::from_bytes(&hwp).unwrap();
            let section = &hwp.document().sections[0].section_def;
            assert_eq!(
                section.master_pages.len(),
                3,
                "memo_first={memo_first}; HWP/HWPX 재저장 뒤 바탕쪽 수",
            );
            assert_eq!(section.memo_shape_id, 1);
            let extra = &section.raw_ctrl_extra;
            assert_eq!(u16::from_le_bytes([extra[2], extra[3]]), 2);
            assert_eq!(u16::from_le_bytes([extra[6], extra[7]]), 1);
            assert_eq!(hwp.get_memo_shape_json(0).unwrap(), SHAPE);
        }
    }
}

#[test]
fn memo_replies_survive_master_page_creation_and_both_save_formats() {
    for bytes in [
        include_bytes!("../fixtures/memo_hancom/reply.hwp").as_slice(),
        include_bytes!("../fixtures/memo_hancom/reply.hwpx").as_slice(),
    ] {
        let mut core = DocumentCore::from_bytes(bytes).unwrap();
        let memos = core.get_memo_list_json();
        // 기존 메모 시각 계약은 FILETIME을 실행 환경의 현지 시각으로 변환한다.
        // KST에서 저장한 HWPX의 리터럴 Z와 UTC 호스트의 HWP 조회 문자열은 다를 수 있다.
        // 날짜 필드를 제거하지 않고, 같은 입력·저장 형식의 무편집 대조군을 먼저 고정한다.
        let unedited = [
            core.export_hwp_native().unwrap(),
            core.export_hwpx_native().unwrap(),
        ]
        .map(|saved| {
            DocumentCore::from_bytes(&saved)
                .unwrap()
                .get_memo_list_json()
        });
        add_master_pages(&mut core);
        core.set_memo_shape_json(Some(0), SHAPE).unwrap();
        assert_eq!(
            core.get_memo_list_json(),
            memos,
            "바탕쪽 편집 직후 메모 보존"
        );
        for (saved, expected_memos) in [
            core.export_hwp_native().unwrap(),
            core.export_hwpx_native().unwrap(),
        ]
        .into_iter()
        .zip(unedited)
        {
            let reopened = DocumentCore::from_bytes(&saved).unwrap();
            assert_eq!(reopened.get_memo_list_json(), expected_memos);
            assert_eq!(reopened.get_memo_shape_json(0).unwrap(), SHAPE);
            assert_eq!(
                reopened.document().sections[0]
                    .section_def
                    .master_pages
                    .len(),
                3
            );
        }
    }
}
