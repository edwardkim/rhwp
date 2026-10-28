# 최종 결과보고서 — Task M100 #1916

## 이슈

[#1916 HWP5 저장: 표(tbl) flowWithText 플래그 소실 (10k 서베이 12건, #1655의 HWP5·표 축)](https://github.com/edwardkim/rhwp/issues/1916)

## 요약

flowWithText 는 빙산의 일각이었다. `serialize_table` 이 `raw_ctrl_data` 부재 시
CTRL_HEADER 데이터를 **빈 값**으로 방출해, 재파스 시 표의 **CommonObjAttr
전체**(treat_as_char/text_wrap/flowWithText/크기/여백)가 기본값으로 붕괴했다.
부재 시 IR `common` 을 합성 방출하도록 수정 (다른 GSO 컨트롤과 동일 계약).

## 판별 (이슈 전제 보정)

- 서베이 12건은 **전부 ZIP(HWPX) 실체의 .hwp 파일**(역방향 확장자 위장 —
  매직 바이트 전수 확인). "원본 v5.1.0.0"은 HWPX 파서가 합성하는 IR 버전
  필드의 판독이다. 이들이 `hwp5-roundtrip`(어댑터 미경유 plain serialize)에
  들어가 표 attr 가 붕괴한 것 — 게이트 유입 분류는 #1914(FORMAT_SKIP,
  PR #1922)로 별도 해소됐다.
- **제품 변환 경로는 정상**: `export_hwp_with_adapter` 는 어댑터 Stage 2 가
  표 raw_ctrl_data 를 합성하므로 flowWithText=true 가 보존된다
  (hwp5-anchor-trace 실측, properties bit 13 유지).
- **남는 실경로** = raw_ctrl_data 없는 IR 의 plain HWP5 저장: 편집기에서
  신설한 표를 가진 HWP5-native 문서의 저장(`export_hwp_native`) 등. 이 경로가
  표의 모든 공통속성을 잃고 있었다.

## 수정

`src/serializer/control.rs` `serialize_table`:

```
raw_ctrl_data 있음 → 그대로 (HWP5 파스본·어댑터 경로 — 불변)
raw_ctrl_data 없음 → serialize_common_obj_attr(&table.common) 합성
```

attr=0 인 IR 은 `pack_common_attr_bits` 경유로 flow_with_text(bit 13)·
treat_as_char(bit 0)·wrap(bit 21-23) 등이 인코딩된다.

## 검증

- `tests/issue_1916.rs`: raw_ctrl_data 없는 표(flowWithText/tac/TopAndBottom/
  크기)의 plain HWP5 왕복 보존 핀 — 수정 전 전항목 붕괴, 수정 후 PASS.
- 타깃 게이트: issue_1916 + hwpx/hwp5_roundtrip_baseline PASS (풀 스위트는
  PR CI `Build default-feature tests` 가 담당 — 로컬 이중 실행 생략, 작업지시자
  지시에 따른 배치 검증 체계)

## 산출물

- 수정: src/serializer/control.rs
- 테스트: tests/issue_1916.rs
- 문서: plans/task_m100_1916.md, plans/task_m100_1916_impl.md, 본 보고서
