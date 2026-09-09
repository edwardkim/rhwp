# Task #1554 최종 결과보고서 — serialize_document BinData 고아 스트림 드롭 수정 (F1, XFAIL 9건 승격)

- 이슈: #1554 (M100)
- 브랜치: `local/task1554` (from `devel`)
- 일자: 2026-06-26
- 선행: #1552(PR #1553) — `hwp5-roundtrip` 게이트가 본 결함을 정탐

## 1. 목표 및 결과

Task #1552의 `hwp5-roundtrip` 게이트 **C2(BinData 보존)** 가 검출한 F1 결함
— `serialize_document`(= HWP5 "저장하기")가 일부 BinData 그림 스트림을 **통째 드롭**
— 을 수정했다. 영향 9건의 BinData 가 저장본에 전수 보존되며, `tests/hwp5_roundtrip_baseline.rs`
의 XFAIL 9건이 baseline 으로 승격(목록에서 제거)되었다.

## 2. 근본 원인 — "storage_id 충돌"이 아니라 "레코드 없는 고아 스트림"

이슈의 1차 가설(storage_id 충돌)은 **오진**이었다. 진단 결과 실제 원인은
**대응 `HWPTAG_BIN_DATA` 레코드가 없는 고아(orphan) `/BinData` 스트림**이다.

| 파일 | 원본 `/BinData` 스트림 | DocInfo BinData 레코드 | 고아 |
|------|----------------------|----------------------|------|
| `interview.hwp` | `BIN0001.jpg`,`BIN0002.jpg`,`BIN0003.gif` (3) | storage_id=2,3 (2) | `BIN0001.jpg` (1) |
| `img-start-001.hwp` | `BIN0001`~`BIN0014` (20) | **0개** | 20 전건 |

`load_bin_data_content`(파서)와 `collect_extra_streams`(추가 스트림 보존) **둘 다
`bin_data_list`(레코드) 기준으로만** 동작한다. 따라서 레코드 없는 스트림은 파싱 단계에서
버려지고, 직렬화 시 재생성되지 못해 저장본에서 사라졌다. IR diff=0·텍스트 동일이라
기존 자동검사는 전건 false PASS였다(데이터 손실).

> 유지되는 그림의 크기 변동은 무손실 재압축(decompressed 바이트 동일)으로 확인됨.
> 손실은 **드롭된 고아 스트림**에 한정.

## 3. 수정 (`src/parser/mod.rs`)

`collect_extra_streams` 가 **직렬화기가 `bin_data_content` 로부터 재생성할 `/BinData`
경로 집합**을 계산하고, 그 집합에 들지 않는 `/BinData` 스트림(= 고아)을 `extra_streams`
로 **원본 바이트 그대로 보존**하도록 변경했다.

- 신규 헬퍼 `serialized_bin_name()` — 직렬화기 `cfb_writer::find_bin_data_info_with_compress`
  의 명명 규칙(매칭 레코드 우선, 없으면 content 자체값)을 미러링하여 "재생성될 경로"를
  정확히 산출 → 중복 기록 없이 고아만 선별.
- `collect_extra_streams` 의 `/BinData/` 일괄 제외를 "재생성 대상만 제외"로 완화.
- 호출부에 `&bin_data_content` 전달(파싱 순서상 이미 산출되어 있음).

`extra_streams` 는 기존 Scripts/DocOptions 보존에 쓰던 메커니즘으로, 렌더링·IR·레이아웃
경로를 건드리지 않는 **격리된 저위험** 방식이다. 직렬화기(`cfb_writer`)의 extra_streams
기록 루프가 원본 경로 그대로 출력하며, 재생성 스트림과 경로가 분리(disjoint)되어 충돌 없다.

## 4. 검증

| 항목 | 결과 |
|------|------|
| 영향 9건 `rhwp hwp5-roundtrip` | `BINDATA_LOSS` → **전건 PASS** |
| `interview.hwp` 저장본 스트림 | `BIN0001/0002/0003` 전수 보존(decompressed 동일) |
| `cargo test --test hwp5_roundtrip_baseline` | 3건 통과(`baseline_all_samples` 포함 → 유지 그림 재압축 무손실 회귀 없음, `xfail_entries_still_fail` 통과) |
| `cargo test --lib cfb_writer`(17) / `bin_data`(16) | 통과 |
| `cargo clippy --release` | 무경고 |
| 변경 파일 `cargo fmt` | 적용 |

XFAIL 9건 전건 승격: `img-start-001`(20/20), `BookReview`(7/10),
`Worldcup_FIFA2010_32`(13/47), `exam_social`(2/8), `NewYear_s_Day`(2/4),
`곡선이있는분산형`(2/3), `pic-crop-01`(2/3), `interview`(1/3), `BlogForm_Recipe`(1/3).

## 5. 수용 기준 대조

- [x] 영향 9건 BinData 저장본 전수 보존(내용 멀티셋 동일)
- [x] `tests/hwp5_roundtrip_baseline.rs` XFAIL 9건 baseline 승격(목록 비움)
- [x] 유지 그림 재압축 무손실 회귀 없음, `cargo test --test hwp5_roundtrip_baseline` 통과

## 6. 범위 밖 (별개)

- **F2'**: `convert`/`convert_to_editable` 경로의 이미지·페이지 손실(KTX 27→1쪽). 본 이슈
  (F1, `serialize_document` 직접)와 무관 — `serialize_document` 는 KTX 보존 확인됨.
- lenient CFB 폴백 경로(`parse_hwp_with_lenient`)는 `extra_streams` 를 비워 두는 기존
  한계 유지(깨진 CFB 전용 폴백, 9건은 모두 strict 경로로 파싱).

## 7. 변경 파일

| 파일 | 변경 |
|------|------|
| `src/parser/mod.rs` | `collect_extra_streams` 고아 `/BinData` 보존 + `serialized_bin_name` 헬퍼 |
| `tests/hwp5_roundtrip_baseline.rs` | XFAIL 9건 제거(baseline 승격) |

관련: #1552, PR #1553. 게이트·근거: `mydocs/manual/hwp5_roundtrip_baseline.md`,
`mydocs/report/task_m100_1552_report.md`.
