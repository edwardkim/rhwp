# Task #1552 최종 결과보고서 — HWP5 roundtrip 무손실 게이트(hwp5-roundtrip CLI) 신설

- 이슈: #1552 (M100)
- 브랜치: `local/task1552` (from `devel`)
- 일자: 2026-06-26

## 1. 목표 및 결과

`hwpx-roundtrip`에 대응하는 **HWP5 동일포맷 roundtrip 무손실 게이트**를 신설했다.
`serialize_document`(= `export_hwp_native`, HWP5 "저장하기")가 IR 뼈대뿐 아니라
**BinData 그림 스트림·페이지수·CFB 구조**까지 보존하는지 자동 검사하고 회귀로 고정한다.

- CLI `rhwp hwp5-roundtrip`(단일/`--batch`/`-o`/`inventory.tsv`/종료코드) 신설
- 회귀 테스트 `tests/hwp5_roundtrip_baseline.rs` (신규 샘플 자동 포함, XFAIL 등급화)
- 매뉴얼 `mydocs/manual/hwp5_roundtrip_baseline.md`

## 2. 검사 항목 (C1~C5)

| # | 검사 | 잡는 결함 |
|---|------|-----------|
| C1 | IR 뼈대 diff(`diff_documents`, 포맷 무관) | 구조 손실 |
| C2 | **BinData 보존**(decompressed 내용 멀티셋) | **그림 스트림 드롭(F1)** |
| C3 | 페이지수 복원(`DocumentCore::from_bytes`, rhwp 자기 일관) | 페이지 변형 |
| C4 | CFB 구조(필수 스트림 + 섹션 수 = IR) | 구조 회귀 |
| C5 | 2-round 안정성 | 비결정성 |

## 3. 전수 측정 결과 (`samples/*.hwp` 319건, 27.5s)

| 분류 | 건수 | 비고 |
|------|----:|------|
| **A (PASS)** | 297 | C1~C5 전부 통과 |
| **B (xfail) BinData 드롭(F1)** | 9 | serialize_document 그림 스트림 드롭 — 후속 수정 |
| 자동 제외 — HWP3 | 10 | `HWP Document File V3.00` (별도 포맷, 교차변환 페이지 폭증) |
| 자동 제외 — 배포용 | 3 | `serialize_document` 직접 적용 시 DISTRIBUTE_DOC_DATA 누락 |

xfail 9건(전부 편집가능·배포용 아님): `img-start-001`(20/20), `BookReview`(7/10),
`Worldcup_FIFA2010_32`(13/47), `exam_social`(2/8), `NewYear_s_Day`(2/4),
`곡선이있는분산형`(2/3), `pic-crop-01`(2/3), `interview`(1/3), `BlogForm_Recipe`(1/3).

## 4. 핵심 발견 — 손실 원인 분리 (선행 조사 정정)

선행 조사(`output/poc/fidelity/report.md`)는 `convert` CLI로 저장본을 만들었는데,
`convert`는 `serialize_document` 전에 **`convert_to_editable_native()`를 호출**한다.
본 게이트는 `serialize_document`를 직접 호출하여 두 손실 원인을 분리했다:

| 손실 | 원인 | 본 게이트 |
|------|------|-----------|
| **F1 그림 드롭** | `serialize_document` 자체(편집가능 문서 9건, interview 1/3·Worldcup 13/47 등) | **C2 가 정탐 → xfail** |
| **F2' 그림+페이지 붕괴** | `convert_to_editable`/`convert` 경로(KTX 3/3 이미지·27→1쪽) | 범위 밖(별도 이슈) |

KTX 교차 검증: `serialize_document` 저장본은 BinData 3/3 보존 + 한글 페이지 27=27
보존(붕괴 없음). 즉 **F2(KTX 페이지 붕괴)는 serialize_document 결함이 아니라 convert
경로 결함**임이 게이트 신설로 규명됨. (선행 보고서의 KTX·convert 기반 수치는 본 보고서로 정정)

## 5. 검증

- `cargo build --release` 성공.
- `cargo test --lib hwp5_roundtrip`: 14 passed (단위).
- `cargo test --test hwp5_roundtrip_baseline`: 통과 (A=297 baseline, xfail 9 여전히 실패 확인).
- 전수 배치 종료 코드 1(하드 실패 존재 = xfail 결함 노출) — CI 게이트로 사용 가능.

## 6. 변경 파일

- 신규: `src/diagnostics/hwp5_roundtrip_batch.rs`, `tests/hwp5_roundtrip_baseline.rs`,
  `mydocs/manual/hwp5_roundtrip_baseline.md`
- 수정: `src/diagnostics/mod.rs`(모듈 등록), `src/main.rs`(서브커맨드+help)
- 계획/보고: `mydocs/plans/task_m100_1552{,_impl}.md`, `mydocs/working/task_m100_1552_stage{1..4}.md`

## 7. 후속 이슈 후보

- **F1**: `serialize_document` BinData 그림 스트림 드롭 수정 (xfail 9건 승격 목표)
- **F2'**: `convert_to_editable`/`convert` 경로 이미지·페이지 손실 (KTX 27→1)
- **F3**: HWPX serializer 컨트롤 슬롯 8유닛 시프트 (실문서 노출, `hwpx-roundtrip` 소관)
- (선택) 배포용 문서 roundtrip 지원(convert_to_editable 경유 게이트 분기)

## 8. 한계

- C3 는 rhwp 자기 일관 기준 — 외부 한글에서만 나타나는 페이지 붕괴는 미검출.
  `output/poc/fidelity/` 한글 harness(T3 재열림·T4 PDF)가 보조.
- 게이트 통과 = 구조+BinData+페이지(자기일관) 보존이며 시각 충실도 보장이 아니다.
