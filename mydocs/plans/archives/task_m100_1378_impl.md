# Task M100 #1378 구현계획서 — HWPX serializer 문단 run 분할 보존

- 수행계획서: `mydocs/plans/task_m100_1378.md` (승인 2026-06-11)
- 브랜치: `local/task1378`

## 1. 설계 핵심

### 1.1 run 분할 모델

파서(`src/parser/hwpx/section.rs:398-415`)는 각 `<hp:run charPrIDRef>` 시작 위치에서
`(utf16_pos, char_shape_id)` 를 기록하고, 연속 동일 id 를 dedup 하여
`para.char_shapes: Vec<CharShapeRef>` 로 보존한다.

serializer 는 이 역방향: `char_shapes[i].start_pos` (i ≥ 1) 를 **run 경계(cut point)** 로 삼아
문단 콘텐츠를 다중 `<hp:run charPrIDRef={char_shapes[i].char_shape_id}>` 로 분할 출력한다.

문단 콘텐츠는 UTF-16 위치 축 위에서 네 종류 이벤트로 구성된다:

| 이벤트 | 위치 결정 | 처리 |
|--------|----------|------|
| 텍스트 문자 | `char_offsets[idx]` | 현재 run 의 텍스트 버퍼에 누적 |
| 컨트롤 슬롯 | `expected_utf16_pos` (8 유닛 폭) | 버퍼 flush 후 슬롯 방출 (기존 로직) |
| 필드 begin/end | `field_ranges` (기존 로직) | 현재 run 내 방출 |
| **run 경계 (신규)** | `char_shapes[i].start_pos` | 버퍼 flush → `</hp:run>` → `<hp:run charPrIDRef={새 id}>` |

### 1.2 경계 규칙 (단위 테스트로 고정)

1. **동일 위치에서 경계와 슬롯이 겹치면 경계 먼저** — 파서가 run 시작 위치에서 경계를
   기록하므로, 그 위치의 컨트롤은 새 run 소속이다.
   처리 순서: `boundary.start_pos <= slot_start` 이면 cut 후 슬롯 방출.
2. **연속 동일 id 경계는 출력 시 skip** — 파서 dedup 과 왕복 정합. (IR 은 이미 dedup
   상태이나 방어적으로 유지)
3. **`char_shapes` 가 비어있으면** 단일 run `charPrIDRef="0"` (현행 유지).
4. **`char_shapes[0].start_pos > 0`** 인 비정상 IR 은 첫 run 을 위치 0 부터 시작 (관용 처리).
5. 빈 세그먼트(경계 직후 또 경계)는 빈 `<hp:t/>` 없이 run 태그만 출력하지 않고 **skip** —
   재파싱 시 dedup/동일 위치 기록과 충돌하지 않도록 단위 테스트로 검증 후 확정.

### 1.3 본문 경로 구조 전환 (`src/serializer/hwpx/section.rs`)

현재 `render_run_content()` 는 run **내부 콘텐츠만** 반환하고 호출부(`write_section`)가
`<hp:run charPrIDRef>` 로 감싼다. 이를 **완전한 run 시퀀스를 반환하는 `render_runs()`** 로 전환:

- `write_section` 첫 문단: 현재의 2중 치환(TEXT_SLOT 치환 → anchor `<hp:run charPrIDRef="0">`+첫_t 재치환, 55-95행)을
  템플릿의 텍스트 run 전체 `<hp:run charPrIDRef="0"><hp:t/></hp:run>` 1회 치환으로 단순화.
  (템플릿 확인 완료: 첫 run 은 secPr/colPr 전용, 텍스트 run 은 이 문자열이 유일하게 1회 등장)
- 추가 문단 루프(98-112행): `<hp:run charPrIDRef="{cs}">` + 콘텐츠 + `</hp:run>` 조립을
  `render_runs()` 결과로 대체.
- `first_run_char_shape_id()` 는 `render_runs()` 내부로 흡수.

### 1.4 셀·글상자 경로 (text-only 분할)

`table.rs write_sub_list`(254-282행)·`shape.rs write_draw_text_paragraph`(238-250행)는
컨트롤을 출력하지 않으므로(컨트롤 보존은 #1379 범위) **텍스트만 char_shapes 경계로 분할**:

- 공유 헬퍼: `text + char_offsets + char_shapes + tab_extended` → run 분할된 XML 문자열.
  char_offsets 로 문자 idx → UTF-16 위치를 매핑하므로 IR 내 컨트롤(8 유닛 갭)이 있어도
  경계 위치가 어긋나지 않는다.
- `shape.rs` 는 현재 탭/lineBreak 처리 없는 단순 escape 출력 — 공유 헬퍼 적용으로
  `render_hp_t_content` 기반(탭/lineBreak 포함)으로 정렬된다. 출력 변화 범위는 단위
  테스트로 명시.

### 1.5 ID 참조 무결성

`ctx.assert_all_refs_resolved()` 정합을 위해 **모든 run 의 char_shape_id 를
`ctx.char_shape_ids.reference()`** 한다. (현재 셀 경로는 첫 run 만 reference —
`table.rs:254-256`. 본문 경로는 미참조 → 함께 정비)
`shape.rs write_draw_text_paragraph` 는 ctx 를 받지 않으므로 시그니처에 ctx 추가.

### 1.6 게이트 강화 (`roundtrip.rs diff_documents`)

- `IrDifference::ParagraphCharShapes { section, paragraph, detail }` 추가 —
  문단별 `char_shapes` 시퀀스 `(start_pos, char_shape_id)` 전체 비교.
- 1단계는 본문(top-level) 문단만 비교. 셀·글상자 내부 문단 재귀 비교는 3단계에서 확장
  (2단계 완료 전 셀 경로 미수정 상태에서 게이트가 본 타스크 외 실패를 만들지 않도록).
- **테스트 그린 유지 전략**: 게이트 강화로 실패 전환되는 A등급 샘플은 1단계에서
  `tests/hwpx_roundtrip_baseline.rs` 의 xfail 목록에 **임시 등록** (주석: `#1378 2~3단계에서
  해소 예정`). 2~4단계에서 xfail 제거가 곧 해소의 증명이 된다. 측정 결과 본 타스크
  범위 밖 원인(예: 다른 serializer 한계)이 섞여 있으면 분류하여 승인 요청.

## 2. 단계별 계획 (4단계)

### 1단계 — 게이트 강화 + 전수 측정

| 항목 | 내용 |
|------|------|
| 소스 | `roundtrip.rs`: `ParagraphCharShapes` 차이 항목 + 본문 문단 char_shapes 비교 |
| 테스트 | `roundtrip.rs` 단위 테스트 (char_shapes 차이 검출/무차이 통과), baseline 임시 xfail 등록 |
| 측정 | `samples/hwpx` 54건 전수: 다중 char_shapes 문단 보유 샘플 수·문단 수, 강화 게이트 실패 샘플 목록 (본문/셀/글상자 기인 분류) |
| 보고 | stage1 보고서 — 측정표 + 임시 xfail 목록 + 경계 케이스 목록(1.2 규칙 적용 대상) 승인 요청 |

완료 조건: `cargo test --tests` 그린 (임시 xfail 포함), 측정표 보고.

### 2단계 — 본문 경로 다중 run 출력

| 항목 | 내용 |
|------|------|
| 소스 | `section.rs`: `render_runs()` 전환, `write_section` 치환 단순화, run id 전체 reference |
| 테스트 | 단위 테스트 — 2-run 분할, 경계=슬롯 동일 위치(규칙 1), 경계가 슬롯 사이, 탭/lineBreak 포함 run, 빈 문단, dedup(규칙 2), 필드 begin/end 와 경계 교차, roundtrip char_shapes 일치 |
| 게이트 | 본문 평탄화 기인 임시 xfail 해소 (목록에서 제거) |

완료 조건: `cargo test --lib`/`--tests` 그린, 본문 기인 xfail 0건.

### 3단계 — 셀·글상자 경로 + 게이트 재귀 확장

| 항목 | 내용 |
|------|------|
| 소스 | 공유 헬퍼(1.4) 추출, `table.rs write_sub_list`·`shape.rs write_draw_text_paragraph` 적용, ctx 시그니처 정비 |
| 게이트 | `diff_documents` char_shapes 비교를 셀(Table)·글상자(Shape/TextBox)·각주/미주 내부 문단까지 재귀 확장 |
| 테스트 | 셀 다중 run, 글상자 다중 run, 셀 내 탭 포함, roundtrip 일치 |

완료 조건: 잔여 임시 xfail 전부 해소 (본 타스크 범위 밖으로 분류 승인된 것 제외).

### 4단계 — 전수 검증 + 시각 자료 + 문서화

| 항목 | 내용 |
|------|------|
| 전수 | `cargo test --test hwpx_roundtrip_baseline` + `rhwp hwpx-roundtrip --batch samples/hwpx -o output/poc/task1378` |
| 시각 | 대표 샘플(run 분할 빈도 상위 + #1315 대표 8건 교집합) SVG byte 비교 — 원본 vs rt |
| 판정 | rt 파일 한컴에디터 열기 판정 요청 (작업지시자) |
| CI급 | `cargo test --lib` / `--tests` / `fmt --check` / `clippy --lib --tests -- -D warnings` / wasm check |
| 문서 | `manual/hwpx_roundtrip_baseline.md` 게이트 강화 반영, 최종 보고서, 오늘할일 갱신 |

## 3. 비수정 범위 (확전 금지)

- 셀·글상자 **컨트롤 출력** (#1379), **lineseg 원본 보존** (#1380), exam_social xfail (#1381)
- `tests/hwpx_roundtrip_integration.rs` (누적-만-가능 원칙)
- PR #1366 범위 (pageBreak/margin/colPr/notePr) — `section.rs`·`table.rs` 수정은 run 출력 구조에 한정

## 4. 위험 대응 (수행계획서 5절 구체화)

| 위험 | 구현 대응 |
|------|----------|
| 템플릿 anchor 치환 취약성 | 1.3 — 텍스트 run 전체 1회 치환으로 단순화, 템플릿 내 유일성 확인 완료 |
| 슬롯·경계 교차 | 1.2 규칙 1 — 2단계 단위 테스트 선행 고정 |
| 게이트 강화로 신규 실패 | 1.6 — 임시 xfail + 측정 보고 + 분류 승인 |
| 한컴 호환 | 다중 run 은 한컴 원본 구조와 동일 형태 — 4단계 한컴에디터 판정으로 확정 |
| 2-round 드리프트 | dedup 출력(규칙 2)으로 1-round == 2-round 보장, baseline 2-round 검사로 검증 |
