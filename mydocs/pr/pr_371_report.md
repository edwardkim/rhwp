# PR #371 처리 보고서 — 정상 머지 (cherry-pick 방식)

## PR 정보

| 항목 | 값 |
|------|-----|
| PR 번호 | [#371](https://github.com/edwardkim/rhwp/pull/371) |
| 작성자 | [@planet6897](https://github.com/planet6897) (Jaeuk Ryu) |
| base / head | `devel` ← `local/task370` |
| 이슈 | [#370](https://github.com/edwardkim/rhwp/issues/370) |
| 처리 결정 | **정상 머지 (cherry-pick 방식)** |
| 처리 일자 | 2026-04-27 |

## 결함 요약

`form-002.hwpx` 10쪽 등 한글 볼드 문구 (예: 연구개발기간, 정부지원연구개발비) 가 SVG 출력에서 볼드로 표시 안 됨.

원인: SVG 의 `font-weight="bold"` 속성은 정상 적용되지만 폰트 폴백 체인의 환경별 한계:
- **Linux**: `'Batang','바탕','AppleMyungjo','Noto Serif KR'` 모두 표준 환경에 미설치
- **macOS Chrome**: `AppleMyungjo` 매칭 후 Regular variant 만 보유 → Bold 합성 실패

## 변경 내용

`src/renderer/mod.rs::generic_fallback()` 의 한글/영문 세리프 분기 두 곳:

```diff
- 'Batang','바탕','AppleMyungjo','Noto Serif KR',serif
+ 'Batang','바탕','Nanum Myeongjo','AppleMyungjo','Noto Serif KR','Noto Serif CJK KR',serif
```

| 추가 폰트 | 위치 | 효과 |
|---|---|---|
| `'Nanum Myeongjo'` | AppleMyungjo 앞 | macOS 10.9+ 기본 설치, Bold variant 보유 → macOS Chrome 매칭 |
| `'Noto Serif CJK KR'` | Noto Serif KR 뒤 | Linux noto-cjk 패키지 |

## 처리 절차

### Stage 1: cherry-pick
- `local/pr371` 브랜치 (`local/devel` 분기)
- PR #371 의 두 commit (`e826510`, `9d84838`) cherry-pick
- 작성자 attribution 보존 (Jaeook Ryu)

### Stage 2: 충돌 해결
- `mod.rs`: 자동 머지 (PR #366 의 `pub mod page_number;` 와 다른 위치)
- `mydocs/orders/20260427.md`: 수동 통합 (Task #361, #362 + #370 세 항목 보존)

### Stage 3: 자동 회귀 검증

| 항목 | 결과 |
|------|------|
| `cargo test --lib` | **1014 passed, 0 failed** |
| `cargo test --test svg_snapshot` | 6/6 통과 (골든 SVG 3건 갱신 반영) |
| `cargo test --test issue_301` | 1/1 통과 |
| `cargo test --test page_number_propagation` | 2/2 통과 (PR #366 효과 유지) |
| `cargo clippy --lib -- -D warnings` | 통과 |
| `cargo check --target wasm32-unknown-unknown --lib` | 통과 |

### Stage 4: 샘플 회귀 + form-002 검증

| 샘플 | 페이지 | LAYOUT_OVERFLOW |
|------|------|------|
| form-01 | 1 | 0 |
| aift | 77 | 3 |
| KTX | 27 | 1 |
| k-water-rfp | 27 | 0 |
| exam_eng | 11 | 0 |
| kps-ai | 79 | 5 |
| hwp-multi-001 | 10 | 0 |

→ 모든 샘플 페이지 수 + LAYOUT_OVERFLOW 무변화 (Task #361, #362, PR #366 효과 모두 유지).

**form-002.hwpx 10쪽 SVG 폰트 체인 검증**:
```
font-family="바탕,'Batang','바탕','Nanum Myeongjo','AppleMyungjo','Noto Serif KR','Noto Serif CJK KR',serif"
font-family="함초롬바탕,'Batang','바탕','Nanum Myeongjo','AppleMyungjo','Noto Serif KR','Noto Serif CJK KR',serif"
```

→ 변경 정상 적용 확인.

## 메모리 점검 (`feedback_font_alias_sync`)

본 PR 의 변경은 CSS `font-family` 폴백 체인 (브라우저 OS 폰트 매칭용). rhwp 내부의 metric lookup 경로 (`resolve_metric_alias`) 와 분리되어 있어 메모리의 동기화 의무 적용되지 않음.

확인:
- `font_metrics_data.rs:91`: `"나눔명조" => "NanumMyeongjo"` 매핑 존재
- `font_metrics_data.rs:140`: `"Noto Serif CJK KR" => "Noto Serif KR"` 매핑 존재
- 영문 `"Nanum Myeongjo"` (공백 포함) 별칭은 없으나 본 PR 의 변경은 SVG 출력 시 fallback 체인 (rhwp 가 입력으로 받지 않음) 이라 메트릭 매핑 추가 불필요

## 흡수 commit 목록

```
1a50f4c docs(pr): PR #371 처리 — 정상 머지 결정 (@planet6897)
2d7a4af Task #370 v2: macOS Chrome 볼드 매칭을 위해 Nanum Myeongjo 삽입 [planet6897]
585d969 Task #370: 한글 세리프 폴백 체인에 Noto Serif CJK KR 추가 [planet6897]
```

## 작성자 기여

@planet6897 (Jaeuk Ryu) — 결함 진단 + 두 단계 변경 + 골든 SVG 갱신 + 시각 검증.
v2 commit 에서 macOS Chrome bold variant 매칭을 위한 Nanum Myeongjo 추가는 PR 의 정밀화 가치.

## 다음 단계

- local/devel merge → devel push
- PR #371 close (PR 댓글 후)
- 이슈 #370 close

## 참고

- 검토 문서: `mydocs/pr/pr_371_review.md`
- 구현계획서: `mydocs/pr/pr_371_review_impl.md`
- 메모리: `feedback_font_alias_sync.md` (본 PR 에 적용 안 됨)
