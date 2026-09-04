# Task M100 #1380 최종 보고서 — HWPX serializer lineseg 원본 보존

- 이슈: #1380 (M100, #1315 serializer 충실도 한계 후속)
- 브랜치: `local/task1380`
- 계획서: `mydocs/plans/task_m100_1380.md` / `task_m100_1380_impl.md` (4단계)
- 단계 보고서: `mydocs/working/task_m100_1380_stage{1..3}.md`

## 1. 결함 본질과 해소

**결함**: 원본 HWPX에 `<hp:linesegarray>` 가 없는 문단(한컴 생산 파일에 실재)을
파서가 zero-default LineSeg 1개로 합성 주입하고, serializer 가 이를 방출하여
**원본 무 → RT 유 합성 비대칭**이 발생 (H1 샘플: business_overview 0→38,
expense_report 0→55). 또한 IR 이 빈 경우의 fallback 정적 합성(vertsize=1000 계열)이
존재했다. lineseg 는 게이트 비교 항목이 아니어서 검출 사각이었다.

**해소** (재계산 금지 원칙 — reflow trap):

| 축 | 내용 |
|----|------|
| 파서 | zero-default 주입 제거 — linesegarray 부재 문단은 IR `line_segs` 빈 채 유지 |
| serializer | 빈 IR 문단의 `<hp:linesegarray>` 요소 방출 생략 (단일 지점 `render_paragraph_parts`) + fallback 정적 합성 완전 제거 |
| document_core | 로드 시 자동 reflow 를 HWPX 한정 `include_empty` 로 정합 (에디터·WASM 경로 동작 보존, HWP5 불간섭) |
| 게이트 | `diff_linesegs` 9필드 비교를 `IrDifference::ParagraphLinesegs` 로 baseline 게이트 동승 — **xfail 0** |

## 2. 단계 요약

| 단계 | 내용 | 커밋 |
|------|------|------|
| 1 | `diff_linesegs` 측정 도구 + `--lineseg-report` TSV + 전수 진단·분류 (잔존 결함 = H1 단일 패턴 확정) | `1fc703e8` |
| 2 | 파서 주입 제거 + serializer 방출 생략 + fallback 제거 + document_core 정합 (HWP5 페이지 수 게이트 회귀 발견 → HWPX 한정 플래그로 정정) | `b9fbfd9d` |
| 3 | 게이트 동승 (`ParagraphLinesegs` variant + `diff_documents` 변환) + 합성 비대칭 검출·2-round 안정성 테스트 | `e2324438` |
| 4 | 전수 검증 + 페이지 수·SVG 귀속 + 매뉴얼 갱신 + 본 보고서 | (본 커밋) |

## 3. 검증 결과 (4단계)

### 3.1 전수 게이트·배치 (output/poc/task1380)

```
hwpx-roundtrip --batch → 총 54: PASS 48 / IR_DIFF 1(#1382) / SERIALIZE_FAIL 4(#1384) / PARSE_FAIL 1(EXCLUDED)
lineseg_diff.tsv → round1=0 round2=0 파일=0   (lineseg 잔존 0 — 기지 이슈 제외 없이도 0)
baseline 게이트 4/4 그린 — XFAIL 5건 불변 (전부 lineseg 무관: #1382 ×1, #1384 ×4)
```

H1 결함 해소 확정: business_overview 0→0 (구 0→38), expense_report 0→0 (구 0→55).

### 3.2 이슈 기재 4샘플 페이지 수 (완료 조건 2항)

| 샘플 | 이슈 기재 | 실측 (정정 후) | 귀속 |
|------|----------|---------------|------|
| form-002 | 10→17쪽 | 10→**15**쪽 | 잔존 차이 **전량 #1388** — RT 의 secPr 여백만 원본 값으로 패치하면 **10=10 일치** (실증) |
| 2025-1Q 보도자료 | 9→13쪽 | 9→13쪽 | 동일 실증 — 여백 패치 시 **9=9 일치** |
| math-001 | 1~2px 시프트 | 1=1쪽, SVG md5 동일 | 해소 |
| footnote-01 | 전 페이지 차이 | 6=6쪽 | 균일 +37.8px = 여백 차 (#1388, 1단계 실측) |

**이슈에 기재된 페이지 수 변화는 전부 #1388(secPr 여백 하드코딩) 기인으로 확정** —
lineseg 기인 페이지 수 차이는 0 (1단계 분류와 정합, margin 패치 대조로 실증 보강).

### 3.3 대표 SVG 비교 (완료 조건 3항)

- **H1 2샘플** (2단계): 원본 렌더링 수정 전후 md5 동일 + RT 렌더링 구/신 코드 md5
  동일 — 본 변경의 렌더링 영향 0
- **form-002** (4단계, 여백 #1388 통제 후): 전 10쪽 텍스트 요소 수 동일
  (841/857/1122…), 텍스트 좌표 2,551건 중 2,281건 동일. 잔존 차이 2종 — 전부
  lineseg 무관 기지/신규 귀속:
  - 셀 배경 rect 소실 — 기지 #1315 계열 serializer 충실도 한계
  - 짝수 페이지 절 제목 27자 y +5.6~7.4px 시프트 — **표 pageBreak 속성 미보존**
    (#1393, 4절 신규 발견) 기인 개연. `ir-diff` 잔존 차이가 tbl page_break 5건뿐임을 확인

### 3.4 CI급 (완료 조건 7항)

| 항목 | 결과 |
|------|------|
| `cargo test --lib` | 1716 passed |
| `cargo test --tests` | 2226 passed / 0 failed (baseline·sample16 페이지 수 게이트 포함) |
| `cargo fmt --check` | 통과 |
| `cargo clippy --lib --tests` | 경고 0 |

4단계는 문서·측정만으로 소스 변경 없음 — 3단계 검증 수치가 최종 상태와 동일.

## 4. 신규 발견 (범위 밖 — 별도 이슈 등록 완료)

| 이슈 | 발견 | 증상 | 발견 시점 |
|------|------|------|----------|
| #1391 | MEMO 필드(fieldBegin) subList 소실 | aift `type="MEMO"` 2건의 parameters+메모 본문 문단 RT 소실 — 게이트 사각 | 1단계 |
| #1392 | shapeComment 소실 | aift `hp:shapeComment` 15→0 | 1단계 |
| #1393 | 표 pageBreak 속성 미보존 | 원본 `pageBreak="CELL"/"NONE"` → RT 일괄 `"TABLE"` 방출. IR page_break 변형 (form-002 5건, RowBreak→CellBreak) — 표 분할 배치 시프트 기여. 게이트 사각(컨트롤 내부 속성 비교 없음) | 4단계 |

## 5. 한컴 편집기 판정 요청 (완료 조건 4항, 작업지시자)

`output/poc/task1380/` 의 rt 파일 중 대표 3건의 한컴 편집기 열람 판정을 요청드립니다:

1. `business_overview.rt.hwpx` — linesegarray 부재 문단 38건 보존 (방출 생략) 본문 줄 배치
2. `expense_report.rt.hwpx` — 부재 문단 55건 보존 본문 줄 배치
3. `form-002.rt.hwpx` — 양식(표) 문서, lineseg 보존 + 기지 #1388 여백 변형 혼재 상태

판정 관점: 본문 줄 간격·줄바꿈이 원본과 동일한가 (linesegarray 생략 문단을 한컴이
재계산으로 정상 표시하는가). #1388 여백·#1387 캡션 차이는 기지이므로 제외.

## 6. 잔존 한계와 이슈 귀속 (완료 조건 종합)

| 한계 | 귀속 |
|------|------|
| 페이지 수 차이 (form-002·보도자료·footnote-01) | #1388 (전량, 실증) |
| 캡션 문단·lineseg 차이 (ta-pic-001-r −1, mel-001 −2, 143E… −1, aift −13) | #1387 |
| 셀 배경 rect 소실, 그림 크기 | #1315 계열 / #1389 |
| autoNum 폭 비일관 / borderFillIDRef | #1382 / #1384 (xfail 유지) |
| MEMO subList / shapeComment / tbl pageBreak | #1391 / #1392 / #1393 (4절) |

## 7. 산출물

- 소스: parser/hwpx/section.rs, serializer/hwpx/{section,table,shape,mod,roundtrip}.rs,
  diagnostics/hwpx_roundtrip_batch.rs, document_core/commands/document.rs
- 문서: 단계 보고서 3건 + 본 보고서 + `manual/hwpx_roundtrip_baseline.md` 갱신
  (게이트 비교 항목 lineseg 추가, known limitations #1380 해소, pageBreak 신규 행)
- 측정: `output/poc/task1380/` (inventory.tsv, lineseg_diff.tsv, *.rt.hwpx — git 미포함)
