# Task M100 #488 — 최종 보고서

수식 토크나이저 폰트 스타일 키워드 prefix 분리 + 렌더러 italic 파라미터 honor

## 1. 배경

`samples/exam_science.hwp` 페이지 1 의 화학 수식이 `rmK`, `rmCa`, `rmmol`, `itl`, `itaq` 등 raw 식별자 그대로 SVG에 출력되던 버그를 시작으로, 동일 코드 경로의 두 가지 본질 결함을 함께 정정.

## 2. 진단

### 결함 A — 토크나이저 prefix 분리 누락

hwpeq 문법: `rm` / `it` / `bold` 폰트 스타일 키워드는 식별자에 공백 없이 붙여 쓸 수 있고 (예: `rmK` = `rm` + `K`, `itl` = `it` + `l`, `rmmol` = `rm` + `mol`), 키워드 길이만큼만 소비된 뒤 나머지는 다음 토큰이 된다.

`src/renderer/equation/tokenizer.rs:80 read_command()` 가 영문자/숫자를 끝까지 탐욕적으로 읽기 때문에 `rmK`, `rmCa` 등이 단일 토큰이 되어 `symbols.rs:181 ("rm", FontStyleKind::Roman)` 매핑과 매칭에 실패. 결과: SVG 에 `rm` / `it` prefix 가 글자 그대로 남음.

### 결함 B — SVG/Canvas 렌더러가 `italic` 파라미터를 honor 하지 않음

`src/renderer/equation/svg_render.rs:50` `Text` arm 이 `italic` 파라미터(line 29)를 받지만 무시하고 비-CJK 텍스트는 무조건 italic 적용:

```rust
let style = if has_cjk { "" } else { " font-style=\"italic\"" };
```

`FontStyle::Roman` 분기(line 274 `(false, false)`) 가 자식 렌더링에 `italic=false` 를 전달해도 효과가 없음. 결과: 결함 A 수정 후에도 `rm K` 의 K 가 italic. 동일 결함이 `canvas_render.rs:47` 에도 있음.

또한 진입점 `render_equation_svg` / `render_equation_canvas` 의 default 가 `italic=false` 로 시작되는데, 이는 hwpeq 변수 기본 스타일(italic) 과 다르고 Text arm 의 강제 italic 으로 우연히 결과가 맞춰지던 상태.

## 3. 수정 내역

### 3.1 토크나이저 (`src/renderer/equation/tokenizer.rs`)

- `matches_at(kw)` 헬퍼 추가: 현재 위치에서 키워드가 prefix 로 매치되는지 검사
- `read_command()` 진입 시 `bold` → `it` → `rm` 순으로 prefix 매치를 시도하고, 키워드 직후가 ASCII 영문자/숫자(식별자 연속) 면 키워드 길이만큼만 소비하고 토큰 종료. 직후가 공백/기호/EOF 면 기존 동작(전체 식별자 한 토큰).
- 단위 테스트 8건 추가

### 3.2 SVG 렌더러 (`src/renderer/equation/svg_render.rs`)

- `Text` arm: `let italic_attr = if !has_cjk && italic { ... } else { "" };` — `italic` 파라미터 honor + bold 함께 처리
- 진입점 `render_equation_svg` default 를 `italic=true` 로 변경 (hwpeq 변수 기본 스타일)
- 단위 테스트 6건 추가 (default italic / `rm` 적용 / `it` 적용 / CJK 미적용 / `rmK`·`rmCa` prefix 분리)

### 3.3 Canvas 렌더러 (`src/renderer/equation/canvas_render.rs`)

- `Text` arm: `set_font(ctx, fi, !has_cjk && italic, bold)` — `italic` 파라미터 honor
- 진입점 `render_equation_canvas` default 를 `italic=true` 로 변경

## 4. 검증

### 4.1 단위 테스트

- 토크나이저 모듈: 20 passed (기존 12 + 신규 8)
- SVG 렌더러 모듈: 신규 6 passed
- 수식 모듈 전체: 66 passed
- **라이브러리 전체: 1092 passed, 0 failed, 1 ignored** (회귀 없음)

### 4.2 시각 검증 (`samples/exam_science.hwp`)

#### 페이지 1 표 #3 (이온 결합 화합물)

| 셀 | 원본 스크립트 | 수정 전 | 수정 후 |
|----|--------------|--------|--------|
| (가) 구성 이온 | `rmK ^{+}` , `rmX ^{-}` | `rmK`, `rmX` (italic) | **K** ⁺ , **X** ⁻ (roman) |
| (나) 구성 이온 | `rmK ^{+}` , `rmY ^{-}` | `rmK`, `rmY` (italic) | **K** ⁺ , **Y** ⁻ (roman) |
| (다) 구성 이온 | `rmCa ^{2+}` , `rmO ^{2-}` | `rmCa`, `rmO` (italic) | **Ca** ²⁺ , **O** ²⁻ (roman) |
| 헤더 mol | `1\`rmmol`, `LEFT ( rmmol RIGHT )` | `1 rmmol`, `(rmmol)` | 1 mol , (mol) |

#### 페이지 1 5번 문제 본문

`rmH _{2} O\` LEFT ( itl RIGHT )` (5종 위치) 모두 H 가 roman 으로 정상 표시. O 와 l 의 italic 잔존은 hwpeq 작성자 스크립트 형식(공백 없는 prefix `rmH`, `itl` 만 사용; 그룹화 없음) 에 따른 표준 동작 — `rm` 은 직후 single token 만 적용 (parser.rs:414 `parse_single_or_group`).

### 4.3 회귀 검증 (광범위 샘플)

| 샘플 | 페이지 수 | raw prefix 잔존 |
|------|----------|----------------|
| `eq-01.hwp` | 1 | 0건 |
| `equation-lim.hwp` | 1 | 0건 |
| `atop-equation-01.hwp` | 1 | 0건 |
| `exam_math.hwp` | 20 | 0건 |
| `exam_kor.hwp` | 20 | 0건 |
| `exam_eng.hwp` | 8 | 0건 |
| `exam_social.hwp` | 4 | 0건 |
| `exam_science.hwp` | 4 | 0건 (수정 전 60건) |
| **합계** | **59 페이지** | **0건** |

수식 spot check 결과 정상 출력 확인 (`eq-01.hwp` 평점/입찰가격평가/×, `equation-lim.hwp` lim/→, `exam_math.hwp` 페이지 1 italic 변수 29건).

## 5. 변경 파일

| 파일 | 변경 |
|------|------|
| `src/renderer/equation/tokenizer.rs` | +86 (헬퍼 1 + 로직 보강 + 테스트 8) |
| `src/renderer/equation/svg_render.rs` | +124 / -7 (Text arm + 진입점 + 테스트 6) |
| `src/renderer/equation/canvas_render.rs` | +6 / -2 (Text arm + 진입점) |

## 6. 별도 이슈로 분리

본 task 진행 중 시각 검증에서 발견된 4건은 본질 영역(레이아웃 / 페이지네이션 / 셀 정렬) 이 다르므로 별도 이슈로 등록:

- [#489](https://github.com/edwardkim/rhwp/issues/489) — 페이지 1 5번 그림이 본문 첫 줄을 가림
- [#490](https://github.com/edwardkim/rhwp/issues/490) — 페이지 1 3번 표 28/36 셀 중앙정렬
- [#491](https://github.com/edwardkim/rhwp/issues/491) — 페이지 1 2번 답안지 위치 약간 아래로
- [#492](https://github.com/edwardkim/rhwp/issues/492) — 페이지 1 컬럼 2 5번 밑단 짤림

## 7. 위험 요소 및 회귀 대비

- 변경 영역이 수식 모듈 3개 파일로 한정.
- hwpeq 룰(키워드 prefix 분리 + italic 파라미터 honor) 에 부합하는 본질 정정.
- `symbols.rs` 에 `rm`/`it`/`bold` 와 prefix 를 공유하는 다른 명령어 없음 → 키워드 매핑 충돌 영역 없음.
- 라이브러리 1092건 + 핵심 8개 샘플 59페이지 검증 완료.
- 롤백: 각 단계 별 단일 커밋 (`6d8d0e1` Stage 1, `1af552d` Stage 2). `git revert` 로 단계별 롤백 가능.

## 8. 결론

Task #488 의 본질 스코프 — hwpeq 토크나이저의 폰트 스타일 키워드 prefix 분리 및 SVG/Canvas 렌더러의 `italic` 파라미터 honor — 가 완료. 사용자 보고의 핵심 항목 (1번: K⁺/Ca²⁺/O²⁻ 등 화학 기호 정상 표시) 해결. 부수적으로 발견된 4건의 레이아웃 영역 결함은 별도 이슈로 분리하여 우선순위에 따라 후속 진행.

`local/task488` → `local/devel` merge 승인 요청.
