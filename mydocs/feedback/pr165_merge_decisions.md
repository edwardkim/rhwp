---
문서: PR #165 (Skia + Layered Renderer) 를 local/pr165-merge-test 로 devel merge 시 충돌 16파일 결정 요청
작성일: 2026-04-21
브랜치: local/pr165-merge-test (PR #165 head + local/devel merge 시도 중)
충돌 파일: 16건 · 31 hunk
원칙: 파일별 판단 (다 · 작업지시자 확정)
---

# 읽는 법

각 파일 블록에서 마지막 `**결정**:` 줄 옆 대괄호 `[ ]` 에 **A / B / C / 원하는 지시** 하나를 적어 주시면 됩니다.

- **A = HEAD (PR #165 · skia 쪽)**
- **B = devel (로컬 devel · 최근 기여 포함)**
- **C = 양쪽 통합 (구체적 지시 필요)**
- **자유 지시** = "둘 다 버리고 XX 로" 같은 임의 지시

작성 후 한 번에 넘겨주시면 순서대로 반영하고 빌드/테스트 진행합니다.

# 충돌 개요

```
grp 1 (단순 · 자동 가능)  : README.md, rhwp-studio/package-lock.json
grp 2 (한쪽 우선 명확)    : main.rs, model/table.rs, parser/control/tests.rs, parser/hwpx/section.rs,
                           document_core/commands/{document,object_ops}.rs,
                           equation/{layout,parser}.rs
grp 3 (신중 통합)         : renderer/composer.rs, renderer/layout/paragraph_layout.rs,
                           renderer/style_resolver.rs, renderer/svg.rs,
                           wasm_api/tests.rs, rhwp-studio/src/core/wasm-bridge.ts
```

---

# 그룹 1 — 단순 · 자동 가능

## [1] README.md · 1 hunk

**내용**: 최근 변경 섹션. HEAD 는 짧은 "793+ 테스트" 한 줄, devel 은 891+ 테스트 + v0.7.3 릴리즈 노트 전체(+기여자 목록).

**차이 요약**:
- HEAD: 테스트 수치만 과거 상태
- devel: v0.7.3 릴리즈 전체 변경 이력, 외부 기여자 크레딧

**기본 추천**: **B (devel 우선)** — 최신 릴리즈 정보 보존

**결정**: [ B ]

---

## [2] rhwp-studio/package-lock.json · 2 hunks

**내용**: dependabot 버전 업(#211/#212) + PR #165 의 canvaskit-wasm 추가

**차이 요약**: 기계 생성 파일. 충돌 해결 의미 적음.

**기본 추천**: **B + 재생성** — devel 쪽 유지 후 `cd rhwp-studio && rm package-lock.json && npm install` 로 canvaskit-wasm 재반영

**결정**: [ B ]

---

# 그룹 2 — 한쪽 우선 명확

## [3] src/main.rs · 2 hunks

**내용**: `dump` 커맨드 출력 포맷
- hunk 1: 도형 출력에 `s.shape_name()` 추가 (devel 쪽) / HEAD 는 단순 태그
- hunk 2: 수식 출력에 `size={}x{} tac={}` 추가 (devel 쪽) / HEAD 는 더 적음

**기본 추천**: **B (devel 우선)** — devel 의 dump 포맷이 정보 풍부, skia 쪽은 단순 linting 차이

**결정**: [ B ]

---

## [4] src/model/table.rs · 7 hunks

**내용**: 모든 hunk 가 동일한 clippy 포맷 차이
- HEAD: `self.cells.sort_by(|a, b| (a.row, a.col).cmp(&(b.row, b.col)))`
- devel: `self.cells.sort_by_key(|c| (c.row, c.col))` (동등 동작, 더 관용적)

**기본 추천**: **B (devel 우선)** — 더 관용적 Rust, 기능 동일

**결정**: [ B ]

---

## [5] src/parser/control/tests.rs · 1 hunk

**내용**: dump 출력 시 Shape variant 매칭
- HEAD: Line/Rectangle/Ellipse/Arc/Polygon/Curve/Group/Picture 만
- devel: 위 + **Chart/Ole 추가** (PR #221 반영)

**기본 추천**: **B (devel 우선)** — #221 Chart/Ole variant 반영 필수 (HEAD 대로면 #221 코드 컴파일 실패)

**결정**: [ B ]

---

## [6] src/parser/hwpx/section.rs · 2 hunks

**내용**: HWPX char_offsets 처리 (#213 기여자 @jskang 의 fix)

**차이 요약**:
- HEAD: 이전 char_offsets 계산 (controls 전체 앞에 푸시)
- devel: interleaved 순서 보존 fix 반영 (#213 cherry-pick · `77f37b6`)

**기본 추천**: **B (devel 우선)** — #213 fix 가 중요 버그 수정 · 최신 로직

**결정**: [ B ]

---

## [7] src/document_core/commands/document.rs · 2 hunks

**내용**: 모듈 use 선언 순서/구성
- HEAD: 간결한 기본 use
- devel: `validation::{CellPath, ValidationReport, ValidationWarning, WarningKind}` 추가 (#177 관련)

**기본 추천**: **B (devel 우선)** — validation 모듈은 #177 (HWPX 비표준 대응) 의 핵심

**결정**: [ B ]

---

## [8] src/document_core/commands/object_ops.rs · 1 hunk

**내용**: shape 속성 update 시 최소 크기 클램프
- HEAD: `c.width = w; c.height = h;`
- devel: `c.width = w.max(MIN_SHAPE_SIZE); c.height = h.max(MIN_SHAPE_SIZE);` (#153 @seunghan91 기여)

**기본 추천**: **B (devel 우선)** — 안전장치 (음수/너무 작은 값 방지)

**결정**: [ B ]

---

## [9] src/renderer/equation/layout.rs · 3 hunks

**내용**: 수식 렌더링 상수
- hunk 1: `FRAC_LINE_PAD` 0.15 ↔ 0.2, devel 에 `AXIS_HEIGHT`, `TEXT_BASELINE` 추가 (수식 baseline alignment)
- hunk 2: `width: w + fs * 0.1` ↔ `w + fs * 0.02` (공간 측정 미세 조정)
- hunk 3: `layout_big_op` 시그니처 스타일

**차이 성격**: 양쪽 모두 수식 렌더 품질 조정. devel 은 baseline alignment 정합성 개선 (더 최신 작업).

**기본 추천**: **B (devel 우선)** — AXIS_HEIGHT 기반 정확한 수식 높이 계산, 최신 개선 보존

**결정**: [ B ]

---

## [10] src/renderer/equation/parser.rs · 3 hunks

**내용**: 수식 AST 파서
- hunk 1: top-level OVER 분수 감지 분기 (HEAD) ↔ 단순 parse_expression (devel)
- hunk 2: `&&` 탭 공간 처리 (devel 에 추가)
- hunk 3: RIGHT 위치 파싱 (HEAD 에 추가)

**차이 성격**: 양쪽 모두 수식 파서 개선. 일부 기능은 HEAD 에만, 일부는 devel 에만 있음. **둘 다 테스트해봐야 함**.

**기본 추천**: **C (양쪽 통합 · 세밀)** 또는 **B (devel 우선 + 후속 이슈로 HEAD 기능 별도 포팅)**

**결정**: [  ] · 지시 자유

---

# 그룹 3 — 신중 통합

## [11] src/renderer/composer.rs · 1 hunk

**내용**: 인라인 객체 처리 match arm
- HEAD: `Equation(eq) => Some((pos, eq.common.width as i32, i))` (한 줄)
- devel: 같은 로직 + 주석 `"HWP 저장값을 사용 — 한컴 편집기가 실제 폰트로 계산한 정확한 너비"`

**차이 성격**: **기능 동일, 포맷팅/주석만 다름**

**기본 추천**: **B (devel 우선)** — 주석이 디버깅 시 가치 있음

**결정**: [ B ]

---

## [12] src/renderer/layout/paragraph_layout.rs · 1 hunk

**내용**: 수식 레이아웃 블록
- HEAD: 줄 나누기 (rustfmt long-line)
- devel: 한 줄 압축 포맷 + **수식 baseline alignment 로직 추가** (`eq_h`, `eq_y` 계산)

**차이 성격**: devel 에 수식 baseline 정렬이 추가됨. HEAD 에는 해당 로직 없음.

**기본 추천**: **B (devel 우선)** — baseline 정렬은 필수, 수식이 텍스트 baseline 에 맞춰짐

**결정**: [ B ]

---

## [13] src/renderer/style_resolver.rs · 1 hunk

**내용**: 탭 좌표 변환
- HEAD: 여러 줄 포맷
- devel: 한 줄 포맷 + 주석 `"HWP 탭 position은 실제 좌표의 2배로 저장됨 (한컴 격자 비교로 확인)"`

**차이 성격**: **기능 동일, 주석만 다름**

**기본 추천**: **B (devel 우선)** — 한컴 격자 비교 근거 주석 보존

**결정**: [ B ]

---

## [14] src/renderer/svg.rs · 2 hunks

**내용**: SVG 렌더러 match arm
- hunk 1: **devel 에 `RawSvg` + `Placeholder` 노드 처리 추가** (PR #221 OLE/Chart 렌더링), HEAD 는 `Body` 만
- hunk 2: **devel 에 `StrokeDash` 대시 패턴 렌더링 추가** (Dash/Dot/DashDot/DashDotDot), HEAD 는 기본 stroke 만

**차이 성격**: devel 에 **신규 기능 두 개 추가**. HEAD 는 누락. 머지 안 하면 #221 의 OLE/Chart 렌더링 · 점선 대시 모두 사라짐.

**기본 추천**: **B (devel 우선)** — #221 의 핵심 신규 기능

**결정**: [ B ]

---

## [15] src/wasm_api/tests.rs · 1 hunk

**내용**: WASM API 테스트
- HEAD: 빈 바이트 썸네일 테스트만
- devel: 빈 바이트 테스트 + **#177 getValidationWarnings / reflowLinesegs WASM API 테스트 3건 추가**

**차이 성격**: devel 에 #177 validation API 테스트 추가

**기본 추천**: **B (devel 우선)** — #177 API 테스트 보존

**결정**: [ B ]

---

## [16] rhwp-studio/src/core/wasm-bridge.ts · 1 hunk

**내용**: TypeScript import 선언
- HEAD: import 에 `PageLayerTree` 추가 (layered renderer 의 핵심 타입)
- devel: import 에 `PageLayerTree` 없음 + 파일 하단에 `ValidationReport` 인터페이스 추가 (#177)

**차이 성격**: **양쪽 모두 필요**
- HEAD 의 `PageLayerTree` import: skia/canvaskit 렌더러 동작에 필수
- devel 의 `ValidationReport`: #177 기능에 필수

**기본 추천**: **C (양쪽 통합)** — import 에 PageLayerTree 유지 + ValidationReport 인터페이스 보존

**결정**: [  ] · 권장은 **C** · 자유 지시 가능

---

# 누락 점검

- `rhwp-firefox/` 폴더 — 현재 untracked 로 남아 있음. PR #165 는 #169 Firefox 포팅 이전에 분기되어 rhwp-firefox 자체가 없었음. devel merge 후 **rhwp-firefox/ 전체가 새로 추가** 됨 (자동 · 충돌 없음).

- devel 에 있던 **새 파일들** (e.g. `src/emf/*`, `src/ooxml_chart/*`, `src/parser/ole_container.rs`, `tests/svg_snapshot.rs` 등) — 모두 자동 머지 완료, 충돌 없음.

---

# 결정 후 절차

작업지시자 결정 완료 후:

1. 각 파일 해결안 반영 (16 파일)
2. `rhwp-studio/package-lock.json` 재생성 (`npm install`)
3. `cargo build` — Rust 빌드 확인
4. `cargo test --lib` — 유닛 테스트
5. `cargo test --features native-skia` — Skia 백엔드 포함
6. WASM 재빌드 (Docker)
7. `cd rhwp-studio && npx vite` — 브라우저 Canvas2D / CanvasKit 모두 확인
8. 결과 보고 → 작업지시자 승인 시 정식 절차 진입 (이슈 등록 + 수행계획서 + local/devel 머지)

실패 시 `git merge --abort` 로 언제든 롤백 가능 (local/pr165-merge-test 브랜치 자체가 실험용).
