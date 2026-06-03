# Task #280 최종 결과보고서 — 수식 SVG 폰트 스택 재정렬

## 요약

`samples/equation-lim.hwp` 의 수식이 한컴 PDF(HyhwpEQ 폰트) 대비 "볼드처럼 두껍게" 렌더링되던 문제를, 수식 전용 폰트 스택에서 `Cambria Math` 를 제거하고 `Times New Roman` 등 얇은 세리프 폴백을 추가하는 1줄 수정(2파일)으로 해소. 레이아웃 계산은 불변. WASM 빌드 포함 전 회귀 테스트 통과.

## 이슈

- [#280](https://github.com/edwardkim/rhwp/issues/280) — 수식 SVG 렌더링: 폰트가 볼드처럼 보임 (Cambria Math 폴백) — 폰트 스택 재정렬
- 브랜치: `local/task280`
- 베이스: `devel` (a4bf19d)

## 원인

### 1차 원인: Cambria Math 매칭

변경 전 폰트 스택:
```
'Latin Modern Math', 'STIX Two Math', 'Cambria Math', 'Pretendard', serif
```

Windows 기본 환경에서 앞 두 폰트(Latin Modern Math / STIX Two Math) 는 미설치 → **Cambria Math** (Office 설치 시 자동 포함)가 매칭. Cambria Math 는 수학 디스플레이용 heavy-stroke 폰트라 일반 Times 세리프보다 확연히 두꺼움.

### 파생 조사 결과 (이 타스크에서 함께 확인)

사용자가 "크기도 다르다" 라고 언급해 PDF 내부 분석(`BT .../F1 110 Tf ...`) 으로 `lim=110pt`, 본체=92pt, 첨자=62pt 차이를 확인. "함수명 1.2x 확대 규칙" 이 있는지 의심했으나, 다음 근거로 **규칙 아님** 확정:

- `exam_math.hwp` 의 `"b= log 2"` bbox=1125 HWPUNIT (font_size * 1.02)
- 단순 `"1"`, `"f(1)"` 과 동일 → `log`, `sin`, `cos` 는 본문과 같은 크기
- PDF 의 110/92 비율은 HyhwpEQ 폰트의 ASCII 글리프(l, i, m) vs PUA 수식 글리프(f, h, (, ) 등) em 박스 차이를 HWP 엔진이 보정한 것

→ **`lim` 에 1.2x 배율은 적용하지 않음**. 단계 4 `exam_math_p013.png` (lim, sin, ln, ∫ 동시 렌더) 에서 모두 본문과 동일 크기로 정상 표시됨을 재확인.

## 변경 내역

### 코드 (2파일, 각 1줄)

**`src/renderer/equation/svg_render.rs:11`**:
```rust
// 변경 전
const EQ_FONT_FAMILY: &str = " font-family=\"'Latin Modern Math', 'STIX Two Math', 'Cambria Math', 'Pretendard', serif\"";
// 변경 후
const EQ_FONT_FAMILY: &str = " font-family=\"'Latin Modern Math', 'STIX Two Text', 'STIX Two Math', 'Times New Roman', 'Times', serif\"";
```

**`src/renderer/equation/canvas_render.rs:223`**:
동일 순서의 폰트 스택을 `set_font` 함수의 Canvas font shorthand 문자열로 반영.

### 설계 근거

- `Latin Modern Math` 첫 번째 유지 → `src/renderer/svg.rs:332` 의 폰트 임베딩 파이프라인(`--embed-fonts` 옵션) 이 "Latin Modern Math" 키를 고정 사용. LaTeX 설치된 환경에서는 기존처럼 LM Math 가 매칭되어 최고 품질 렌더링 유지.
- `STIX Two Text` 추가 → Mac/STIX 프로젝트 설치 환경용. Math 변형보다 얇은 본문용.
- `Times New Roman`, `Times` 추가 → Windows/Mac/Linux 공통 기본 세리프. Windows 에서 여기서 stop 되어 볼드 인상 해소.
- **`Cambria Math` 제거** → Windows 볼드 인상의 근본 원인.
- **`Pretendard` 제거** → 산세리프(한글+라틴 sans). 수식 렌더링에 부적합.

### 폴백 시나리오

| 환경 | 매칭 폰트 | 렌더링 품질 |
|------|-----------|-----------|
| LaTeX 설치 | Latin Modern Math | PDF 와 가장 유사 (기존 유지) |
| Mac + STIX | STIX Two Text | 얇고 깔끔한 세리프 |
| Windows 기본 | **Times New Roman** | 볼드 인상 해소 |
| 기타 Unix | serif (liberation 등) | 시스템 기본 세리프 |

### 문서

- `mydocs/plans/task_m100_280.md` — 수행계획서
- `mydocs/plans/task_m100_280_impl.md` — 구현계획서
- `mydocs/working/task_m100_280_stage{1,2,3,4}.md` — 단계별 완료보고서
- `mydocs/working/task_m100_280_stage{1,4}/*.png, *.svg` — 시각 검증 근거
- `samples/equation-lim.{hwp,pdf}` — 재현 샘플 커밋

## 검증

### 자동 회귀

| 검증 | 결과 |
|------|------|
| `cargo test --lib equation` | ✅ **48 passed** / 0 failed (수식 전 단위 테스트 통과) |
| `cargo test --test svg_snapshot` | ✅ 3 passed / 0 failed |
| `cargo clippy --lib -- -D warnings` | ✅ clean |
| `cargo check --target wasm32-unknown-unknown --lib` | ✅ clean |
| `cargo build --release` | ✅ success |

> `cargo test --lib` 전체에서 14건 실패하나, `git stash` 로 수정 취소 후에도 **동일한 14건** 실패 → 이번 변경과 **무관한** 기존 CFB writer Windows path 문제. 별도 이슈 조사 후보.

### 시각 회귀

- `samples/equation-lim.hwp` : before/after/pdf 3종 비교 → "볼드 인상" 해소 확인 (단계4 보고서)
- `samples/exam_math.hwp` : 5개 페이지(20 중) 육안 검증 → 레이아웃 변화 없이 폰트만 얇아짐

## Phase 2 후속 이슈 후보

이 타스크의 Phase 1 범위 밖으로 남긴 항목 (별도 이슈 등록):

1. **괄호 `(` `)` SVG path 폭 조정** — `draw_stretch_bracket` 의 `paren_w = fs * 0.3` 가 폰트 글리프 대비 다소 큼. 본문 글자보다 파렌이 크게 보여 부자연스러움. PDF 의 HyhwpEQ 폰트 글리프 비율에 맞게 0.25~0.28 범위로 조정 검토.
2. **두 렌더러 폰트 스택 중복 제거** — `svg_render.rs::EQ_FONT_FAMILY` 와 `canvas_render.rs::set_font` 가 각자 하드코딩. 공용 상수로 추출하는 리팩터.
3. **폰트 임베딩 파이프라인 유연화** — `src/renderer/svg.rs:332` 가 "Latin Modern Math" 키를 고정 사용. 다중 폰트 서브셋 지원 시 재검토.

## 교훈

1. **사용자 인상 ≠ 측정값** — "볼드" 이면서 "크기도 다르다" 는 제보를 받고 1.2x 스케일 규칙 의심으로 출발했으나, 샘플 bbox 데이터 비교(`"b= log 2"` = 1125 = font_size * 1.02)로 함수명 스케일 규칙 없음을 확인. **PDF 콘텐츠 스트림의 수치(110 vs 92)만 보고 즉단했으면 잘못된 방향으로 수정할 뻔함.** 단일 근거가 아닌 여러 샘플 교차 검증이 핵심.

2. **독점 폰트(HyhwpEQ) 접근 불가 상황** — 한컴 PDF 와 픽셀 단위 일치는 구조적으로 불가능. "충분히 근접" 을 기준으로 목표 수준을 명시하고 진행한 것이 효율적.

3. **범위를 좁게 정했지만 조사는 끝까지** — 소스 1줄 수정이지만 조사 과정에서 `svg.rs:332` 의 폰트 임베딩 로직, `canvas_render.rs` 의 중복, PDF 내부 구조까지 훑어 **의도치 않은 깨짐이 없음을 확신** 할 수 있었음. 1차 원인 확인 후 바로 수정하지 않고 영향 범위 조사를 Phase 1 안에 포함한 결정이 유효했음.

## 산출물 목록

### 커밋

| 커밋 | 내용 |
|------|------|
| `4497754` | 단계1 — 기준선(before/pdf 이미지) + 샘플 + 계획서 |
| `85a2300` | 단계2 — canvas_render.rs 영향도 조사 |
| `171570b` | 단계3 — 폰트 스택 실제 수정 (svg_render.rs + canvas_render.rs) |
| `c3cae27` | 단계4 — after 이미지 + exam_math 회귀 |
| (이후) | 단계5 — 최종 보고서 + orders 갱신 + Phase 2 이슈 등록 |

### 변경 파일

- 코드: 2 파일, 각 1줄 (주석 보강 제외)
- 샘플: `samples/equation-lim.{hwp,pdf}` 신규
- 문서: `mydocs/plans/`, `mydocs/working/`, `mydocs/report/` 하위 다수
