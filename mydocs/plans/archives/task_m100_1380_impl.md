# Task M100 #1380 구현계획서 — HWPX serializer lineseg 원본 보존

- 수행계획서: `mydocs/plans/task_m100_1380.md` (승인됨)
- 브랜치: `local/task1380`
- 단계: 4단계

## 0. 사전 조사 확정 사항

### 0.1 serializer 경로 (serializer/hwpx/section.rs — 코드 확인 완료)

#1379 이후 본문·셀·글상자·각주/미주·머리말/꼬리말 전 경로가
`render_paragraph_parts()`(section.rs:162) 공유:

- `para.line_segs` 비어있지 않음 → `render_lineseg_array_from_ir`(1184) —
  9개 필드 IR 값 그대로 출력 (#177). **값 가공 없음 확인**
- 비어있음 → `render_lineseg_array_fallback`(1221) → `push_lineseg_static`(1259) —
  `vertsize=1000 / textheight=1000 / baseline=850 / spacing=600 / horzpos=0` 합성
- 섹션에 문단이 하나도 없을 때도 fallback (section.rs:75)

### 0.2 parser 경로 (parser/hwpx/section.rs — 코드 확인 완료)

- `parse_lineseg_array`(1383) + `parse_lineseg_element`(1410) — 9개 필드
  (textpos/vertpos/vertsize/textheight/baseline/spacing/horzpos/horzsize/flags)
  전부 그대로 적재. **파싱 시점 값 변형 없음 확인**
- **빈 문단 기본값 주입**(729): `linesegarray`가 없는 문단은 파서가
  `LineSeg { text_start: 0, tag: TAG_SINGLE_SEGMENT_LINE, ..Default::default() }`
  (수치 필드 전부 0) 1개를 합성 주입

### 0.3 사전 조사로 좁혀진 잔존 결함 가설

파서·serializer 모두 "값 그대로" 경로가 확인되므로, spot 실측에서 관찰된
`vertsize=1000 … horzsize=0` 합성 잔존과 #1315 당시 값 변형의 출처 후보는:

| 가설 | 메커니즘 | 1단계 판별 방법 |
|------|---------|----------------|
| H1. 원본에 linesegarray 없는 문단 | 파서 기본값(0) 주입 → IR 비어있지 않으므로 fallback 미발동, **0값 방출** — 그러나 관찰값은 1000 계열이므로 별도 경로 존재 가능 | 원본 XML에서 linesegarray 부재 문단 인벤토리 |
| H2. 파서가 적재하지 않는 문단 유형 | `parse_paragraph`를 거치지 않는 서브 경로(머리말 등)가 line_segs 빈 채 IR 생성 → serializer fallback 발동 | RT의 1000 계열 문단 ↔ IR line_segs 상태 대조 |
| H3. 문단 자체 소실·신설 | #1387 캡션 소실 등으로 문단 대응이 어긋나 diff가 값 변형처럼 보임 | 문단 수 대조 후 diff에서 기지 이슈 분리 |
| H4. #1315 당시 관찰된 값 변형이 #1378/#1379로 이미 해소 | 당시 셀·글상자 경로 합성이 본문 측정에 혼입되었을 가능성 | 전수 재측정으로 현재 기준 확정 |

1단계 전수 측정으로 가설을 확정하고 2·3단계 수정 범위를 분류한다.

## 1단계 — 측정 도구 + 전수 진단 + 분류 승인

### 1.1 lineseg 비교 함수 (`roundtrip.rs`)

- `diff_linesegs(doc1, doc2) -> Vec<LinesegDiff>` 신설 — 문단별 `line_segs`
  9개 필드 시퀀스 비교. #1378의 재귀 패턴(본문 + 셀·글상자 Group 재귀 +
  각주/미주)을 그대로 따라 동일 경로를 순회
- `LinesegDiff { section, paragraph, path, kind }` — kind:
  `CountMismatch { expected, actual }` / `ValueMismatch { index, field, expected, actual }`
- **게이트(`diff_documents`)에는 동승하지 않음** — 1단계는 측정 전용.
  동승 여부는 측정 결과 보고 시 제안 (수행계획서 2절 "게이트 공백")

### 1.2 인벤토리 산출 (CLI)

- `rhwp hwpx-roundtrip --batch`의 `-o` 산출에 `lineseg_diff.tsv` 추가
  (옵션 `--lineseg-report`, 기본 비활성 — 기존 inventory.tsv 13컬럼 불변)
- 컬럼: 파일 / 섹션 / 문단 path / kind / 필드 / 원본값 / RT값
- round1(원본 vs RT)과 round2(RT vs RT²) 모두 산출 — 드리프트 분리 관찰

### 1.3 전수 측정 + 분류

- 54건 전수 실행 → 불일치를 0.3 가설(H1~H4)별로 분류
- 기지 이슈 기인분 분리: #1387(캡션 문단 소실), #1382(autoNum 경계),
  #1384(SERIALIZE_FAIL 4건은 측정 불가 — xfail 상태 기재)
- 이슈 4샘플 페이지 수 재측정: form-002, 2025년 1분기 보도자료, math-001,
  footnote-01 — 원본 vs RT `export-svg` 페이지 수 대조

### 1.4 보고 + 승인 요청

- 측정 결과 + 분류 + 2·3단계 수정 범위 확정 + 게이트 동승 여부 제안
  (불일치 규모·2-round 안정성 영향 포함)

완료 조건: `_stage1.md` 보고 + 승인. `cargo test --lib`/`--tests` 그린
(측정 도구는 기존 게이트에 영향 없음).

## 2단계 — serializer 방출 정정

1단계 분류 결과에 따라 확정하되, 현 시점 예상 작업:

### 2.1 fallback 발동 경로 축소

- H2 확정 시: line_segs 빈 IR을 만드는 serializer 측 경로(빈 섹션 템플릿 등)의
  fallback 합성을 원본 보존 가능 형태로 정정
- fallback 자체는 유지 (IR에 정말 없을 때의 최후 수단 — `Document::default()` 등
  비파싱 IR). **재계산 도입 금지** (reflow trap 원칙)

### 2.2 값 방출 정정

- 1단계에서 ValueMismatch가 확인되면 해당 필드 매핑 정정
  (0.1 확인 기준으로는 가공 없음 — H3/H4로 소거될 가능성 높음)

### 2.3 단위 테스트 (section.rs)

| 테스트 | 고정하는 동작 |
|--------|--------------|
| 빈 문단 lineseg 원본 보존 | 원본 `linesegarray` 값(0 아님)이 RT에 그대로 |
| linesegarray 없는 문단 roundtrip | round1 → round2 동일 출력 (드리프트 0) |
| fallback 발동 조건 | 파싱 IR에서는 fallback 미발동 |

완료 조건: `_stage2.md` 보고 + 승인. `cargo test --lib` 그린.

## 3단계 — 파서 적재 정합 + (승인 시) 게이트 동승

### 3.1 파서 기본값 주입 정합 (parser/hwpx/section.rs:729)

1단계 분류에 따라 택일 (승인 결과 반영):

- A안: 기본값 주입 유지 + serializer가 "주입분"을 식별해 원본과 동일하게 방출
  (원본에 linesegarray 없으면 RT에도 방출 생략) — 보존 우선
- B안: 주입 값을 원본 의미에 맞게 보정 — 렌더러가 최소 1개 lineseg를
  기대하는 경로(layout 등)가 있는지 확인 후 결정

어느 쪽이든 **HWP5 파서 경로·렌더러 공통 코드는 수정하지 않는다**.

### 3.2 게이트 동승 (1단계 승인 결과에 따라)

- 승인 시: `diff_linesegs`를 baseline 게이트(`diff_documents` 또는 별도 검사)에
  추가 + 잔존 불일치 샘플 xfail 등록(사유 명기) + still-fail 가드
- 비승인 시: 측정 도구로만 유지 (매뉴얼에 한계 기재)

### 3.3 단위 테스트

| 테스트 | 고정하는 동작 |
|--------|--------------|
| linesegarray 부재 문단 파싱 | 3.1 택일안의 IR 상태 |
| 2-round 안정성 | 정정 후 round2 diff 0 |

완료 조건: `_stage3.md` 보고 + 승인. `cargo test --lib`/`--tests` 그린.

## 4단계 — 전수 검증 + 한컴 판정 요청

1. `rhwp hwpx-roundtrip --batch samples/hwpx -o output/poc/task1380` 전수 —
   PASS 수·xfail 정합 확인 + `lineseg_diff.tsv` 잔존 0(기지 이슈 제외) 확인
2. 페이지 수 비교: 이슈 4샘플 필수 (form-002, 2025-1Q 보도자료, math-001,
   footnote-01) — 완료 조건 "페이지 수 일치" 판정
3. 대표 샘플 SVG 비교 (#1379 4단계 절차) — lineseg 기인 시프트 해소 확인
   (#1387/#1388/#1389 기지 차이는 분리 기재)
4. 한컴에디터 판정 요청 (작업지시자): 대표 rt 열기 + 본문 줄 배치
5. 매뉴얼 `manual/hwpx_roundtrip_baseline.md` 갱신 — known limitations #1380 행
   해소 반영 + (동승 시) 게이트 비교 항목 갱신
6. 최종 보고서 `report/task_m100_1380_report.md` + 오늘할일 갱신
7. CI급: `cargo test --lib` / `--tests` / `cargo fmt --check` / clippy 경고 0

완료 조건: 전 항목 + 잔존 한계의 이슈 귀속 명기.

## 위험 관리 (수행계획서 5절 보강)

| 위험 | 대응 |
|------|------|
| 1단계 측정에서 가설 외 출처 발견 | 분류 보고에 포함 → 승인 후 범위 재확정 (임의 확대 금지) |
| 파서 기본값 주입 제거가 렌더러를 깨뜨림 | 3.1에서 빈 line_segs 의존 경로를 먼저 확인 — 공통 코드 수정 금지, A안(방출 측 식별) 우선 검토 |
| 게이트 동승 시 대량 xfail | 1단계 사전 측정 후 승인 요청 (#1378/#1379 패턴) |
| 2-round 드리프트 | 1.2에서 round2 diff를 별도 산출해 정정 전후 비교 |
| inventory.tsv 형식 변경으로 기존 도구 호환 파손 | `--lineseg-report` 옵션 분리, 기존 13컬럼 불변 |
| PR #1366 충돌 | 변경을 lineseg 함수 내부로 한정, 드랍 권장 안내 완료 상태 유지 |
