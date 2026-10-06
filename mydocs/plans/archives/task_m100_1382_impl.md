# Task M100 #1382 구현계획서 — HWPX 파서 autoNum 폭 비일관 정정

- 수행계획서: `mydocs/plans/task_m100_1382.md` (승인 완료)
- 브랜치: `local/task1382`
- 작성일: 2026-06-12
- 단계: 4단계

## 0. 사전 조사 확정 사항 (코드 확인 완료)

### 0.1 파서의 세 위치 축 (parser/hwpx/section.rs)

| 축 | 위치 | `\u{0012}` 처리 | 용도 |
|----|------|----------------|------|
| char_shapes 경계 | `calc_utf16_len_from_parts`(`:4918`) — run 시작마다 호출(`:414`, `:559`) | `_` 분기 **1유닛** ← **결함** | `char_shape_changes.push((utf16_pos, id))` |
| char_offsets | 조립 루프(`:637~`) | placeholder 공백 push + **8유닛** jump (#1050) | 슬롯 추론·serializer 재구성 |
| visible 문자 인덱스 | field_ranges 루프(`:620`) | visible_char_idx += 1 | 필드 범위 (visual_text 인덱스 축 — **별개 축, 정상**) |

`\u{0012}` 주입원: autoNum 파싱 2곳 (`:3817`, `:3903`).

### 0.2 정정 지점

`calc_utf16_len_from_parts`의 8유닛 분기(`"\u{0002}" | "\u{0003}" | "\u{0004}"`)에
`"\u{0012}"` 추가 — char_shapes 경계가 offsets 축과 일치하게 된다.
(예: 143E 각주 run2 경계 2 → 9 = autoNum 8 + 공백 1)

### 0.3 회귀 위험 소비처 (1단계 전수 조사 대상)

- serializer `RunSplitter`/슬롯 시스템 — offsets 축 전제이므로 일관화가 정합 방향
- renderer `composer.rs`/`layout.rs` — char_shapes.start_pos를 char_offsets 매핑으로
  소비하는지(HWP5와 공유 경로면 8유닛 축 전제) 확인. HWP5 파서는 CHAR_SHAPE 레코드
  위치(8유닛 축 포함)를 그대로 적재하므로 공유 렌더 경로는 8유닛 축 전제일 개연이 높다.

## 1단계 — 메커니즘 추적 + 소비처 축 조사

코드 수정 없음 (조사 전용).

### 1.1 발현 2패턴 정밀 추적

- 143E 각주 문단·ta-pic 캡션 문단의 char_shapes/char_offsets/text를 파싱 직후 덤프
  (임시 단위 테스트) — 1유닛 축 경계가 RunSplitter cut·슬롯 방출을 각각 어떻게
  변위시키는지 단계별 기록 (경계 시프트 vs 끝 방출의 분기 원인 규명).

### 1.2 스펙 교차 확정

- HWP 5.0 PARA_TEXT 표 — AUTO_NUMBER(0x12) 확장 컨트롤 8 WCHAR 점유 확인.
- HWP5 파서(parser/hwp5)가 같은 문서의 char_shapes를 어떤 축으로 적재하는지 대조
  (hancom-hwp 쌍 샘플 — business_overview 등 hwp/hwpx 쌍 활용 가능 시 ir-diff).

### 1.3 소비처 축 의존성 전수 조사

- `char_shapes`·`start_pos` 소비처 sweep (renderer/composer·layout·style_resolver,
  document_core, serializer) — 각 소비처가 offsets 매핑 경유인지 직접 인덱스인지 분류.

### 1.4 전수 분포 + 동승 영향

- samples/hwpx 전수에서 `\u{0012}` 보유 문단·run 경계 동반 케이스 정량화 (영향 범위).
- xfail 변동 예측: 143E 해소 확인, 신규 노출 유무.
- 보고: `mydocs/working/task_m100_1382_stage1.md` → 승인 요청

## 2단계 — 파서 정정

### 2.1 `calc_utf16_len_from_parts` 정정

- `"\u{0012}"`를 8유닛 분기에 추가 + doc 주석 (#1382, offsets 축 정합 근거).
- 두 함수(calc vs offsets 루프) 분기 1:1 대조 — 동류 비일관 토큰 잔존 여부 일괄
  점검 (탭/서로게이트 처리 차이 포함), 발견 시 같은 단계에서 정정 또는 이슈 분리.

### 2.2 단위 테스트 (parser/hwpx)

- calc 함수: `\u{0012}` 포함 parts → 8유닛 집계
- 143E 패턴 재현: autoNum+공백 후 run 경계 → char_shapes `[(0,a),(9,b)]`
- 실샘플: 143E 각주 문단 char_shapes 파싱 값 고정

### 2.3 보고 + 승인 요청 (`_stage2.md`)

- spot: 143E roundtrip char_shapes 대칭 + ta-pic 캡션 슬롯 mid-text 방출 확인 수치

## 3단계 — xfail·테스트 승격

### 3.1 baseline 승격

- `tests/hwpx_roundtrip_baseline.rs` `XFAIL_1378_RECURSIVE`에서 143E 제거 —
  `xfail_1378_recursive_entries_still_fail` 가드가 강제하는 동시 처리.

### 3.2 #1387 테스트 승격

- `task1387_ta_pic_001_r_roundtrip_preserves_caption`의 trim_end 완화 제거 —
  텍스트 완전 일치로 승격 (#1382 귀속 주석 제거).
- 캡션 autoNum 슬롯 위치 회귀 테스트: RT XML에서 ctrl이 mid-text 위치
  (`<hp:t>&lt;그림 </hp:t><hp:ctrl>…` 패턴) 방출 확인.

### 3.3 보고 + 승인 요청 (`_stage3.md`)

- baseline 전수 (xfail 변동 명세)

## 4단계 — 전수 검증 + 문서 + 한컴 판정 요청

1. `hwpx-roundtrip --batch samples/hwpx` 전수 → `output/poc/task1382/`
2. ta-pic SVG 좌표 대조 — 캡션 행 -3.5px 시프트 해소 (#1387 4단계 잔존분), 완전 일치 기대
3. 143E SVG 비교 (각주 영역 회귀 없음)
4. 매뉴얼 `hwpx_roundtrip_baseline.md` 갱신 (#1382 해소 — autoNum 변위 행 + xfail 표)
5. CI급: `cargo test --profile release-test --tests` + fmt + clippy
6. 최종 보고서 + 한컴 판정 요청 (ta-pic-001-r.rt — 캡션 번호 "&lt;그림 1&gt;" 위치)

## 위험 관리 (수행계획서 6절 보강)

| 위험 | 단계 | 대응 |
|------|------|------|
| 렌더러 소비처가 1유닛 축 전제로 동작 중이었을 가능성 | 1 | 소비처 전수 분류 후 진행 — 직접 인덱스 소비처 발견 시 구현계획 보정 승인 |
| 동류 토큰 추가 비일관 | 2 | 두 함수 분기 1:1 대조 일괄 점검 |
| 143E 외 잠재 xfail 변동 | 1·3 | 1단계 전수 분포로 사전 예측, 3단계 baseline 전수로 확정 |
| 자기 정합 ≠ 한컴 호환 (#1058 선례) | 4 | 한컴 판정 게이트 필수 |
