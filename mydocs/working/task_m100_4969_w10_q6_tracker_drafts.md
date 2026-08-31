# Task M100 #4969 W10-Q6-B — #4969·#4960 tracker 현행화 초안

- **상위 계획**: [`task_m100_4969_w10_q6.md`](../plans/task_m100_4969_w10_q6.md)
- **Q6-A checkpoint**: `8258f0284`
- **기계 판독 receipt**:
  [`w10_q6_tracker_draft_receipt.json`](../tech/investigations/issue-4969/w10_q6_tracker_draft_receipt.json)
- **작성일**: 2026-08-31 KST
- **상태**: 로컬 초안, 게시 금지
- **GitHub mutation**: 0

## 1. 게시 전 필수 치환값

아래 token이 하나라도 남으면 게시하지 않는다.

| token | 실제 값의 권위 |
| --- | --- |
| `{{Q6_PR_NUMBER}}` | `gh pr view`의 number |
| `{{Q6_PR_URL}}` | `gh pr view`의 url |
| `{{Q6_CODE_HEAD}}` | code candidate의 exact head OID |
| `{{Q6_MERGE_SHA}}` | merge 뒤 `mergeCommit.oid` |
| `{{Q6_MERGED_AT}}` | merge 뒤 `mergedAt` |
| `{{Q6_CODE_CI_SUMMARY}}` | code candidate check rollup |
| `{{Q6_REVIEW_CI_SUMMARY}}` | review-only trailing head check rollup |

게시 직전 #4969 body SHA-256 `1cbe21c09ac1c63336603cf662db42f1de40adec93beffe837af7211e11711b1`,
#4960 body SHA-256 `448847eb53209b071470027d70d7157958c0d413a05b4b48cb7913129fa7bc42`와 현재
본문을 다시 대사한다. hash가 다르면 아래 patch를 적용하지 않고 최신 본문 기준으로 초안을 다시 만든다.

## 2. #4969 최종 comment 초안

```markdown
W10-Q5/Q6 최종 자격화와 릴리즈 인계를 완료했습니다.

- 제품 통합: PR #6493, merge `3afbb066fe93724ab44309163a2e04efb954bf18`
- 최종 증적·guard PR: [#{{Q6_PR_NUMBER}}]({{Q6_PR_URL}}), code head `{{Q6_CODE_HEAD}}`, merge `{{Q6_MERGE_SHA}}` (`{{Q6_MERGED_AT}}`)
- 최종 분류: 전체 `bounded-subset`
  - Q2 horizontal old Hangul: `bounded-subset`
  - Q3 explicit variable instance: `qualified`
  - Q4 vertical HWP5 table-cell v1: `bounded-subset`
- CI: code candidate {{Q6_CODE_CI_SUMMARY}}; review-only trailing head {{Q6_REVIEW_CI_SUMMARY}}

최종 support matrix는 폰트 이름이나 한컴 버전이 아니라 exact source identity와 현재 capability를 탐지합니다.

- Q2: horizontal-tb·LTR·bidi 0의 direct old-Hangul strict/no-LineSeg atomic lane만 CanvasKit common GlyphRun으로 게시하고, 나머지는 W9 K1 또는 K0 TextRun으로 닫습니다.
- Q3: Happiness Sans VF의 canonical `wght`·`opsz` explicit default/interior/max만 자격화했습니다. no-request/default는 기존 TextRun을 보존하고, 미지원 backend·invalid instance는 문단 전체를 TextRun으로 rollback합니다.
- Q4: HWP5 `textDirection=2`, 단일 table-cell·문단·line·run·column, pure CJK upright와 exact `vhea/vmtx` source가 모두 성립할 때만 CanvasKit vertical GlyphRun을 게시합니다. 다른 tuple은 TextRun과 legacy vertical geometry를 보존합니다.

현재 merged product tree에서 native↔WASM canonical mismatch, CanvasKit replay·pixel mismatch, unsupported backend false selection, fallback disappearance, partial publication, reject 뒤 mutation과 resource multiplicity 증가는 모두 0입니다. 현재 표준 Docker WASM 절대 snapshot은 9,905,805 bytes이며 Q3·Q4의 서로 다른 causal delta는 합산하지 않았습니다.

RTL/bidi 일반화, multi-line/run batch, HWP/HWPX axis intent 추론, HWPX vertical table-cell, mixed vertical run과 다른 backend glyph replay는 현재 지원이 아니라 명시적 deferred/fallback 영역입니다. 이 범위는 실패를 숨긴 것이 아니며 별도 공개 fixture·semantic lineage·backend 증거가 준비될 때 후속 이슈 후보로 다룹니다.

이 이슈의 완료 조건인 source identity 연결, 대표 fixture의 결정적 positioning, native/WASM·portable replay, 동일 preflight/runtime fail-closed와 기존 horizontal non-target 무회귀가 충족됐습니다. 따라서 #4969를 close합니다.
```

`#4969를 close합니다`는 comment 게시이자 자동 close 명령이 아니다. comment API 검증 뒤 별도 close 승인을
사용한다.

## 3. #4960 본문 최소 patch 초안

본문 전체를 다시 쓰지 않고 다음 세 anchor만 exact replace한다.

### 3.1 W10 checklist

현재 anchor:

```markdown
- [ ] W10 — 대상 face 집합 안정화 뒤 vertical metrics·variation·shaping: #4969
  - 상태: 진행 중 — Q2-D4 bounded horizontal common shaping 병합, Q2-D5 resource reuse gate 대기
  - release gate: horizontal shaping·variation·vertical lane의 지원 subset과 fail-closed matrix 확정
```

교체 초안:

```markdown
- [x] W10 — 대상 face 집합 안정화 뒤 vertical metrics·variation·shaping: #4969
  - 상태: 완료 — Q2 horizontal `bounded-subset`, Q3 variable `qualified`, Q4 vertical `bounded-subset`; 전체 `bounded-subset`
  - 통합: PR #6493 merge `3afbb066f`, PR #{{Q6_PR_NUMBER}} merge `{{Q6_MERGE_SHA}}`
  - release handoff: capability 기반 support·fallback·unsupported matrix와 fail-closed 경계 확정
```

### 3.2 최근 실행 표 W10 행

현재 anchor:

```markdown
| W10 | 진행 중 | PR #6270, merge `1a43a507c` | Q2-D4 `qualified-bounded`; strict one-line/one-run 유지, Q2-D5 resource 재사용 gate 대기 |
```

교체 초안:

```markdown
| W10 | 완료 | PR #6270·#6493·#{{Q6_PR_NUMBER}}, final merge `{{Q6_MERGE_SHA}}` | Q2·Q4 `bounded-subset`, Q3 `qualified`; capability 기반 support·fallback·unsupported matrix와 fail-closed 릴리즈 인계 완료 |
```

### 3.3 실행 흐름도 W10 상태

현재 anchor:

```text
  └─ W9 exact kerning qualified → W10 진행 중
```

교체 초안:

```text
  └─ W9 exact kerning qualified → W10 bounded-subset 완료 → release support matrix 인계
```

각 현재 anchor는 게시 직전 최신 body에서 정확히 한 번만 나타나야 한다. 0회 또는 2회 이상이면 자동 치환하지
않는다.

## 4. #4960 최종 comment 초안

```markdown
W10 #4969의 최종 PR [#{{Q6_PR_NUMBER}}]({{Q6_PR_URL}})이 merge `{{Q6_MERGE_SHA}}`로 통합되어 W0~W10 실행 계보를 완료했습니다.

- W7.5: PR #6049, registry lifecycle·migration·evidence delta
- W8: PR #6069·#6081·#6106, rank 8·1·7 모두 no-change
- W9: PR #6214, exact-source kerning과 native/WASM·backend parity
- W10 제품 통합: PR #6270·#6493
- W10 최종 support matrix·guard·증적: PR #{{Q6_PR_NUMBER}}
- 최종 CI: code candidate {{Q6_CODE_CI_SUMMARY}}; review-only {{Q6_REVIEW_CI_SUMMARY}}

W10 최종 분류는 Q2 horizontal `bounded-subset`, Q3 variable `qualified`, Q4 vertical `bounded-subset`, 전체 `bounded-subset`입니다. exact source·table·script/language·axis·writing mode·backend capability가 모두 성립하는 tuple만 선택하고, 그 밖의 입력은 구조화된 reason과 기존 TextRun/geometry fallback으로 닫습니다.

이는 deferred surface를 일반 지원으로 계산한 완료가 아닙니다. RTL/bidi, 일반 multi-run shaping, 문서 axis intent 추론, HWPX·mixed vertical과 미지원 backend replay는 별도 증거가 준비될 때만 후속 후보로 재개합니다. 현재 v1.0.0 release 인계에는 검증된 bounded support matrix만 포함합니다.

#4960의 모든 W0~W10 실행 항목과 릴리즈 인계 근거가 실제 merge 결과로 현행화됐으므로 상위 tracker를 close합니다.
```

## 5. 게시·검증 순서

1. Q6 PR이 실제 merge됐는지 재조회한다.
2. 두 issue의 state·updatedAt·body SHA-256을 재조회한다.
3. 모든 placeholder를 실제 값으로 치환하고 남은 `{{...}}`가 0인지 검사한다.
4. #4969 comment를 UTF-8 without BOM body file로 게시하고 API로 한글·BOM·`??`를 검증한다.
5. #4960 body의 세 anchor를 exact replace하고 API로 body hash와 문자열을 검증한다.
6. #4960 comment를 게시하고 같은 문자열 검증을 수행한다.
7. 별도 close 승인과 마지막 completion audit 뒤 #4969, #4960 순서로 close한다.

이 문서는 로컬 초안이며 remote push, PR 생성, issue edit/comment, close 권한을 부여하지 않는다.
