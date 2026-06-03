# Task #280 구현 계획서 — 수식 SVG 폰트 스택 재정렬

## 참고

- 수행계획서: [`task_m100_280.md`](task_m100_280.md)
- 이슈: [#280](https://github.com/edwardkim/rhwp/issues/280)

## 단계 구성 (5단계)

### 단계 1 — 기준선 확보 (변경 전 스냅샷)

**목적**: 변경 전/후 비교 근거 확보. 재현 샘플을 리포지토리에 포함.

**작업**:
- `samples/equation-lim.hwp`, `samples/equation-lim.pdf` 커밋 (제보 재현 샘플)
- 변경 전 상태에서 `cargo build --release` 후 SVG 생성:
  - `target/release/rhwp export-svg samples/equation-lim.hwp -o output/equation-lim/`
- SVG → PNG 렌더 (Chrome headless, `mydocs/working/task_m100_280_stage1/` 에 보관):
  - `before.svg` (원본 SVG 사본)
  - `before.png` (Chrome headless 렌더)
  - `pdf.png` (한컴 PDF를 Chrome PDF viewer 로 렌더)
- `samples/exam_math.hwp` 의 모든 수식에 대한 기준 SVG 도 생성해 `output/exam_math_before/` 에 보관 (회귀 판단 근거, 커밋 불필요)

**커밋**: `samples/equation-lim.{hwp,pdf}` + 단계 1 보고서 `mydocs/working/task_m100_280_stage1.md`

**완료 조건**: 작업지시자 승인

---

### 단계 2 — `canvas_render.rs` 영향도 조사

**목적**: Phase 1 범위 경계 결정. `svg_render.rs` 만 고칠지, `canvas_render.rs` 도 함께 손볼지 판단.

**작업**:
- `src/renderer/equation/canvas_render.rs` 의 폰트 지정 방식 확인
- Canvas 2D API의 `ctx.font` 에 font-family 문자열이 어떻게 전달되는지 추적
- 동일 상수를 공유하는지, 별도 정의인지 확인
- 판단 결과:
  - (A) 동일 상수 공유 → 1줄 수정으로 양쪽 해결
  - (B) 별도 정의 → 양쪽 동시 수정 (범위 유지)
  - (C) Canvas 는 다른 전략 사용 (예: 개별 글자마다 폰트 지정) → 범위 유지하며 같은 방향 재정렬

**산출물**: 단계 2 보고서 `mydocs/working/task_m100_280_stage2.md` — 조사 결과와 판단 근거

**완료 조건**: 작업지시자 승인

---

### 단계 3 — 폰트 스택 변경 + 회귀 테스트

**목적**: 실제 코드 수정.

**작업**:
- `src/renderer/equation/svg_render.rs:11` `EQ_FONT_FAMILY` 상수 변경:
  ```rust
  const EQ_FONT_FAMILY: &str = " font-family=\"'STIX Two Text', 'Latin Modern Roman', 'Times New Roman', 'Times', 'Cambria', serif\"";
  ```
- (단계 2 결과에 따라) `canvas_render.rs` 도 같은 기조로 수정
- 회귀:
  - `cargo test --lib` — 수식 단위 테스트(`test_simple_text_svg`, `test_fraction_svg`, `test_paren_svg`, `test_eq01_svg`) 포함 전체 통과
  - `cargo test --test svg_snapshot` — 스냅샷 회귀 (영향 있으면 스냅샷 업데이트 필요)
  - `cargo clippy --lib -- -D warnings`
- 스냅샷이 `font-family` 문자열을 포함해 깨지면:
  - 수식 렌더링 결과가 의도대로 바뀐 것이므로 스냅샷 업데이트
  - 업데이트 전 변경 내용이 font-family 속성 문자열 외 다른 부분에 영향 없는지 확인

**커밋**: 코드 변경 + 스냅샷 업데이트(있으면) + `mydocs/working/task_m100_280_stage3.md`

**완료 조건**: 작업지시자 승인

---

### 단계 4 — 시각 비교 검증

**목적**: 변경이 실제로 "볼드 인상" 을 해소했는지, 회귀 없는지 육안 확인.

**작업**:
- 변경 후 SVG 생성 → PNG 렌더:
  - `samples/equation-lim.hwp` → `after.svg`, `after.png`
  - `mydocs/working/task_m100_280_stage4/` 에 `before.png` / `after.png` / `pdf.png` 3종 배치
- `samples/exam_math.hwp` 회귀 — 수식 여러 개 (`sin`, `cos`, `log`, `sqrt`, `int`, `frac`, 단순 숫자) SVG 샘플 렌더해 의도한 방향으로 바뀌었는지 확인
- 확인 항목:
  - [ ] `lim` 및 본문 글자가 가는 세리프로 바뀜
  - [ ] 전체 너비, 첨자 위치, 분수선 위치는 변화 없음 (레이아웃 동일)
  - [ ] 특수 기호(→, √, ∫, ∑ 등) 가 브라우저 폴백으로 정상 표시됨
  - [ ] 다른 수식 샘플도 깨지지 않음

**산출물**: 단계 4 보고서 `mydocs/working/task_m100_280_stage4.md` — 비교 이미지 참조 + 확인 체크리스트

**완료 조건**: 작업지시자 승인

---

### 단계 5 — 최종 보고서 + 오늘할일 갱신

**목적**: 타스크 종결.

**작업**:
- 최종 보고서 `mydocs/report/task_m100_280_report.md` — 요약, 변경 내역, 검증 결과, Phase 2 후속 이슈 후보 기록
- `mydocs/orders/20260424.md` 에 Task #280 섹션 추가
- Phase 2 후속 이슈 등록 (괄호 path 폭, 마이너스 위치 등) — 제목만 등록, 상세는 후속 타스크에서
- **주의**: 단계 5 는 타스크 브랜치에서 커밋 (`local/task280`). merge 전 `git status` 로 미커밋 파일 없는지 확인.

**커밋**: 최종 보고서 + orders 갱신 + (후속 이슈 참조 기록)

**완료 조건**: 작업지시자 승인 → 타스크 브랜치를 `devel` 에 merge → `gh issue close 280`

---

## 전체 변경 파일 예상

| 파일 | 단계 | 변경 유형 |
|------|------|-----------|
| `samples/equation-lim.hwp` | 1 | 신규 |
| `samples/equation-lim.pdf` | 1 | 신규 |
| `src/renderer/equation/svg_render.rs` | 3 | 상수 1줄 |
| `src/renderer/equation/canvas_render.rs` | 3 | 단계 2 결과에 따라 |
| 테스트 스냅샷 | 3 | 필요 시 업데이트 |
| `mydocs/working/task_m100_280_stage{1,2,3,4}.md` | 각 단계 | 단계별 보고서 |
| `mydocs/report/task_m100_280_report.md` | 5 | 최종 보고서 |
| `mydocs/orders/20260424.md` | 5 | Task #280 섹션 추가 |

## 리스크

- **스냅샷 업데이트 파급** — 기존 SVG 스냅샷에 `font-family` 문자열이 리터럴로 들어있을 수 있음. 단계 3 에서 실제 범위 확인 후 대응.
- **브라우저 폰트 폴백 동작 차이** — Chrome/Firefox/Safari 각 브라우저가 수학 기호 폴백을 다르게 처리할 수 있음. 단계 4 에서 최소 Chrome 기준으로 확인하고 다른 브라우저는 후속 이슈에서 추적.
- **작업지시자 기대치** — "PDF와 동일한 모양" 은 HyhwpEQ 독점 폰트 탓에 불가. 단계 4 시각 비교에서 "충분히 근접" 수준으로 수용되는지 확인 필요.

## 예상 소요

- 단계 1: ~15분 (샘플 커밋 + 기준 스냅샷)
- 단계 2: ~20분 (canvas 조사)
- 단계 3: ~20분 (코드 수정 + 회귀)
- 단계 4: ~30분 (시각 비교 + 회귀 샘플 여러 개)
- 단계 5: ~20분 (최종 보고서 + orders)
- **총 예상**: 1.5~2시간
