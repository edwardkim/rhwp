# Task M100 #1315 — 2단계 완료 보고서

## 단계 목표

재조립 HWPX의 패키지(ZIP) 구조 검증기 추가 + 2-round 안정성 검사 + 인벤토리 컬럼 확장.

## 구현 내역

| 파일 | 변경 |
|------|------|
| `src/serializer/hwpx/package_check.rs` | 신규 — `check_package()` + 단위 테스트 8건 |
| `src/serializer/hwpx/mod.rs` | 모듈 등록 1줄 (serializer 본체 무수정) |
| `src/diagnostics/hwpx_roundtrip_batch.rs` | 패키지 검사 + 2-round 컬럼·상태·종료 코드 확장 |

### `check_package(hwpx_bytes, doc)` 검사 항목

1. ZIP 아카이브로 열림
2. `mimetype` — 최초 엔트리 + STORED + 내용 `application/hwp+zip`
3. 필수 엔트리 9종 존재 (version.xml, header.xml, content.hpf, Preview 2종, settings.xml, META-INF 3종)
4. `Contents/section{N}.xml` 수 = IR 섹션 수 (누락·잉여 모두 검출)
5. `content.hpf` manifest가 참조하는 href 전부 ZIP에 실재
6. `BinData/` 엔트리 수·확장자 멀티셋 = IR `bin_data_content` 보존

> serializer가 BinData href를 `BinData/image{N}.{ext}`로 재명명하므로, 원본 ZIP 엔트리 이름이 아닌 **IR 기준 수·확장자**를 보존 기준으로 삼았다 (`context.rs` 확인 결과 반영).

### 2-round 안정성 검사

`round1 IR(doc2)` → 재직렬화 → 재파싱 `IR(doc3)` 비교. `diff_documents(doc2, doc3) == 0`이어야 안정.
트러블슈팅 기록(`hwpx_lineseg_reflow_trap.md`)의 LINE_SEG 류 다회 저장 드리프트를 검출하는 게이트.

### 상태 분류 확장

`PARSE_FAIL → SERIALIZE_FAIL → REPARSE_FAIL → IR_DIFF → PKG_FAIL → ROUND2_FAIL → ROUND2_DIFF → PASS`
하드 실패(파싱/직렬화/재파싱/패키지/2-round 오류) 존재 시 종료 코드 1.

## 전수 측정 결과 (54개 파일, 검사 강화 후)

| 상태 | 건수 | 1단계 대비 |
|------|------|-----------|
| PASS | **52** | 동일 (전부 패키지 검사 + 2-round까지 통과) |
| PKG_FAIL / ROUND2_DIFF / ROUND2_FAIL / IR_DIFF | 0 | — |
| PARSE_FAIL | 1 (`hwpx-01.hwpx`, HWP5 오명명 샘플) | 동일 |
| SERIALIZE_FAIL | 1 (`exam_social.hwpx`, borderFillIDRef 31) | 동일 |

**핵심 확인**: 1단계에서 PASS였던 52건 모두 패키지 구조 및 2-round 반복 저장에서도 결함 0건. 검사 강화로 새로 드러난 실패 없음.

## 검증 결과

- `cargo test --lib` — 통과, 1660 passed / 0 failed / 6 ignored (신규 8건 포함)
- `cargo fmt --check` — 통과
- `cargo clippy --lib -- -D warnings` — 통과
- 전수 배치 재실행 완주 — `output/poc/task1315/inventory.tsv` 갱신 (13컬럼)

## 다음 단계

3단계 — 샘플 등급화 (A=baseline / B=xfail / C=oracle 부적합) + 전수 통합 테스트 `tests/hwpx_roundtrip_baseline.rs`.
