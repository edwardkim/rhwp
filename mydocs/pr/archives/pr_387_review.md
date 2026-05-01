# PR #387 검토 — 그림 밝기/대비 효과 SVG 반영 (#150)

## PR 정보

| 항목 | 값 |
|------|-----|
| PR 번호 | [#387](https://github.com/edwardkim/rhwp/pull/387) |
| 작성자 | [@oksure](https://github.com/oksure) (Hyunwoo Park) |
| base / head | **`main`** ⚠️ ← `contrib/image-brightness-contrast` |
| state | OPEN |
| mergeable | MERGEABLE |
| mergeStateStatus | **BLOCKED** (main 분기 보호 — REVIEW_REQUIRED) |
| isDraft | false |
| 이슈 | [#150](https://github.com/edwardkim/rhwp/issues/150) |
| 변경 통계 | +61 / -0, 7 files |
| CI | 전부 SUCCESS (Build & Test, CodeQL Rust/JS/Python) |

## 작성자 정보

신뢰 컨트리뷰터:
- 머지 이력: PR #334 (replaceOne API), #335 (이미지 base64 임베딩)
- 다른 OPEN PR: #388 (수식 렌더링 개선)

## 결함 요약

이슈 #150 — 그림 효과 중 **밝기 / 대비** 가 파싱은 되지만 SVG 출력에 미반영. PR #149 에서 그레이스케일/흑백만 구현됨.

## 변경 내용

### 1. `ImageNode` 필드 추가 (`src/renderer/render_tree.rs`)

```diff
 pub struct ImageNode {
     ...
+    pub brightness: i8,  // -100 ~ +100
+    pub contrast: i8,    // -100 ~ +100
 }
```

### 2. SVG 필터 수식 (`src/renderer/svg.rs`)

```rust
slope = (100 + contrast) / 100
intercept = (0.5 - 0.5 × slope) + brightness / 100
```

`<feComponentTransfer>` 로 밝기/대비 조정. 기존 effect filter (그레이스케일/흑백) 와 **중첩 `<g filter>` wrapping** 으로 조합.

### 3. ImageNode 생성 지점 8곳에 brightness/contrast 전달

`src/renderer/layout.rs`, `paragraph_layout.rs`, `picture_footnote.rs`, `shape_layout.rs`, `table_cell_content.rs` 등.

## ⚠️ 핵심 정황 — base=main

본 PR 의 **base 가 main** 입니다 (다른 모든 PR 은 base=devel).

### 영향
- main 분기 보호 정책: REVIEW_REQUIRED → mergeStateStatus = BLOCKED
- 머지 시 main 직접 변경 → release 절차 (devel → main PR + 태그) 와 충돌
- v0.7.7 의 정상 절차: devel 에 머지 → 다음 minor/patch 릴리즈 시 main 으로 PR

### 처리 방향
- **base 를 devel 로 변경** 요청 (작성자에게)
- 또는 메인테이너가 cherry-pick 으로 devel 에 흡수 (작성자 attribution 보존)

## 메인테이너 작업과의 충돌 분석

### svg.rs
- devel 에 **PR #373** 의 형광펜 배경 (`draw_text` 함수) 변경 있음
- PR #387 의 변경은 **이미지 렌더링 함수** (다른 영역) → **자동 머지 가능성 높음**

### render_tree.rs
- devel 에 **PR #385** 의 `Serialize` derive 추가 있음 (ImageNode 도 포함)
- PR #387 의 변경은 ImageNode 에 **brightness / contrast 필드 추가** (다른 줄)
- → **자동 머지 가능성 높음**, 충돌 시 수동 통합 (Serialize + 신규 필드)

### 신규 메서드 영향
- ImageNode 의 컨스트럭터 (`new`) 변경 시 PR #385 의 신규 호출 (있다면) 영향
- 다른 ImageNode 생성 지점 8곳 변경 시 충돌 가능성 (한 곳만 신규 필드 추가, 다른 7곳도 같은 패턴)

## 변경 평가

### 강점
1. **명확한 결함 정정** — 파싱은 되지만 SVG 미반영된 속성을 표준 SVG 필터로 적용
2. **검증된 수식** — `<feComponentTransfer>` 의 표준 brightness/contrast 공식
3. **기존 효과와 조합 가능** — `<g filter>` 중첩 wrapping
4. **변경 범위 작음** (+61 / -0)
5. **CI 통과**: cargo test 1008 + clippy + svg_snapshot 6 통과 (작성자 검증)
6. **신뢰 컨트리뷰터** (PR #334/#335 머지 이력)

### 약점 / 점검 필요
1. **base=main** — devel 로 변경 필요
2. **이슈 #150 의 워터마크 부분 미구현** — PR 본문에 명시 안 함 (별도 task 후보)
3. **HTML / Canvas 렌더러 미적용** — SVG 만 적용 (다른 렌더러는 별도 PR 필요할 수 있음)
4. **devel 의 PR #385 (Serialize derive) 와 가까운 영역** — cherry-pick 시 머지 충돌 가능성 (자동 해결 가능성 높지만 점검)

## 처리 방향 후보

### 옵션 A: base=devel 로 변경 요청 후 정상 머지

작성자에게 base 변경 요청. 작성자가 PR 의 base 를 devel 로 변경하면 BLOCKED 해소 + 정상 진행.

### 옵션 B: cherry-pick 으로 devel 에 흡수

PR #366, #371, #373, #385 와 같은 방식 — 작성자 attribution 보존하며 메인테이너가 직접 devel 에 cherry-pick.

장점: 작성자 base 수정 부담 없음, 작업지시자 일관 절차.
단점: 작성자 측에서 PR 처리 결과 명확히 보임 (close 후 흡수 commit 으로 attribution).

### 옵션 C: PR 보류 + 작성자 응답 대기

작업지시자가 base 변경 요청 댓글로 작성자 직접 정정 유도.

## 권장

**옵션 B (cherry-pick)** — 다음 이유:
1. 다른 PR 들 (#366, #371, #373, #385) 과 일관 처리 절차
2. 작성자에게 base 수정 / re-PR 부담 안 줌
3. 메인테이너가 직접 devel 에 흡수 + 검증 → 빠른 진행

cherry-pick 시 devel 의 PR #385 (Serialize derive) 와 같은 ImageNode 영역 변경 → 자동 머지 가능성 높지만 충돌 시 수동 통합.

## 다음 단계 — 작업지시자 결정

옵션 A / B / C 중 결정 부탁드립니다.

옵션 B 진행 시:
1. 구현계획서 + 승인
2. cherry-pick → 충돌 해결 (svg.rs / render_tree.rs)
3. 검증 (cargo test 1016+ 유지, svg_snapshot 6/6, clippy)
4. devel merge + push
5. PR close + 이슈 #150 close + 보고서

## 검토 항목 (Claude 점검 완료)

- [x] 작성자 신뢰도 (PR #334/#335 머지 이력) ✅
- [x] 코드 품질 (표준 SVG 필터 + 검증된 수식) ✅
- [x] CI 통과 ✅
- [x] 메인테이너 동시 정정 없음 ✅
- [x] base=main 정황 — 정상 머지 절차 위반 ⚠️
- [x] devel 의 PR #385 와의 충돌 가능성 (자동 해결 가능성 높음)

## 참고

- 이슈: [#150](https://github.com/edwardkim/rhwp/issues/150) (OPEN)
- PR: [#387](https://github.com/edwardkim/rhwp/pull/387) (OPEN, BLOCKED)
- 동일 작성자 다른 OPEN PR: [#388](https://github.com/edwardkim/rhwp/pull/388) (수식 렌더링 개선)
- 선행 PR: [#149](https://github.com/edwardkim/rhwp/pull/149) (그레이스케일/흑백 구현)
