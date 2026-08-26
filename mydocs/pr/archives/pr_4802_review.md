---
kind: review
status: self-review-complete
pr: 4802
issue: 4741
author: edwardkim
base: devel
---

# PR #4802 자체검토 — Local Font Access 부분 열거 보완

## 절차 라우팅

```text
base route: collaborator_self_merge.md
modifiers: intake_and_review.md, local_validation.md,
  visual_fixture_evidence.md, rework_and_exceptions.md
loaded documents: pr_review_workflow.md, pr_review/README.md,
  collaborator_self_merge.md, intake_and_review.md, local_validation.md,
  visual_fixture_evidence.md, rework_and_exceptions.md
current code candidate: 4084c024d0f888e02ffd0334d8548031dc939c3a
```

## PR 메타데이터

| 항목 | 값 |
| --- | --- |
| PR | [#4802](https://github.com/edwardkim/rhwp/pull/4802) |
| 작성자 | `edwardkim` (collaborator self PR) |
| base / head | `devel` ← `task/4741-local-font-access-gap-probe` |
| code candidate | `4084c024d0f888e02ffd0334d8548031dc939c3a` |
| 작성 시점 최신 devel | `bcb65ed6814d7c7867c05f4bcf9e8d6af9ede0d2` |
| merge base | `93805ebb0548a48704a0046262044295aade4bcc` |
| 규모 | 17 files, +1,225/-67, 6 commits |
| 관련 이슈 | `Closes #4741`; #1328, #2217, #4739 회귀; #3931 별도 조판 축 |
| 작성 시점 상태 | code candidate `MERGEABLE` / `CLEAN`, Actions 통과 참고값 |
| 검토 | collaborator 자체검토; 외부 reviewer 미지정 |

1,000줄을 넘는 대형 PR이므로 즉시 admin merge하지 않고 코드 검토, 최신 base simulation,
focused·시각 검증과 trailing 기록 CI를 별도 cycle로 판정한다. 증분의 절반 이상은 계획·조사·canonical
manual·검증 기록이며, 제품 변경은 Studio의 local-font 감지와 Canvas2D 접합면에 한정된다.

## 변경 검토

### 부분 열거와 상태 모델

기존 구현은 `queryLocalFonts()`가 존재하면 그 결과를 완전한 목록으로 간주해 문서 후보 probe를
건너뛰었다. 이 PR은 열거 record와 현재 문서 후보를 대조하고, family/full/PostScript/style alias로
해소되지 않은 후보만 raw Canvas probe에 보낸다. snapshot v3은 열거 face와 probe-only face의
`detectionSource`, `checkedFamilies`, `probedFamilies`, `unresolvedFamilies`를 보존한다. v1/v2 저장값은
과거 Local Font Access 결과를 문서 후보 기준 완료 상태로 과장하지 않고 보수적으로 승격한다.

`complete`를 항상 false로 두되 이미 판정한 후보는 `checkedFamilies`로 구분하므로, 새 문서 후보는 다시
확인할 수 있고 같은 후보는 반복 prompt·probe하지 않는다. probe 폭이 fallback과 구분되지 않는 경우는
설치 face로 승격하지 않고 unresolved에 남는다. 같은 감지 세대의 결과는 저장 snapshot과
`detectedAt` 기반 repaint generation으로 재사용되며 글자 측정·그리기 hot path에는 probe 호출이 없다.

### raw Canvas와 backend 경계

`canvas-font-raw.ts`는 전역 substitution patch 설치 직전 native `font` descriptor를 한 번 보존한다.
presence probe는 이를 직접 호출해 확인 대상이 먼저 fallback chain으로 치환되는 순환을 피한다. descriptor가
없는 비표준 Canvas/mock만 기존 property setter로 fail-soft한다.

probe-only record는 문서가 선언한 정확한 face 이름만 갖고 PostScript name이나 SFNT bytes를 추정하지
않는다. 따라서 Canvas2D chain은 exact face를 첫 항목으로 사용할 수 있지만 CanvasKit의
`loadLocalFontBytesFor()`는 이 record를 local Typeface로 등록하지 않는다. Local Font Access에서 실제
열거된 face만 기존 SFNT blob 경로를 사용한다.

개발 모드의 `window.__localFonts` 진단 표면은 제품 singleton과 실제 전역 patch를 E2E에서 관찰하기
위한 것이며 production build에는 노출되지 않는다. 옵션·권한 모달 문구도 전체 열거와 문서별 누락 후보
확인을 구분하고, 결과를 로컬 저장소 밖으로 전송하지 않는 경계를 유지한다.

### 범위와 관련 이슈

- #1328은 API 지원/미지원 2분할의 선행 설계로 유지하며 부분 열거를 세 번째 상태로 보완한다.
- #2217의 localized alias와 SFNT byte 조달 경계를 회귀로 유지한다.
- #4739의 KoPub serif/style, 첫 paint 이전 snapshot과 단일 repaint를 회귀로 유지한다.
- #3931의 Rust 페이지 분할 문턱과 전역 metric은 변경하지 않는다.
- KoPub·정부상징·ROKG 폰트 바이너리는 저장소와 PR에 포함하지 않았다.

작성 시점 재조회에서 #1328, #2217, #3931, #4739는 CLOSED이고 #4741은 `edwardkim` 담당 OPEN이다.
PR의 `Closes #4741`에 따른 실제 종료와 잔여 조건은 merge 후 다시 확인한다.

## 최신 base simulation

PR 생성 기준 `devel` 뒤에 #3950 IME commit `bcb65ed6814d7c7867c05f4bcf9e8d6af9ede0d2`가
추가됐다. 두 변경의 파일 교집합은 없고 다음 simulation이 충돌 없이 tree를 만들었다.

```text
git merge-tree --write-tree upstream/devel 4084c024d0f888e02ffd0334d8548031dc939c3a
fdb0732a5af3f772682a3ce9f73ffdae12dcb0a6
```

`git diff --check upstream/devel...HEAD`도 통과했다. review·오늘할일 trailing commit에는 #4802 기록만
추가하며, 최신 devel의 #4805 오늘할일을 source branch에 복사하지 않는다. trailing head를 만들고 다시
simulation해 실제 merge tree가 양쪽 기록을 모두 보존하는지 확인한다.

## 완료한 검증

| 검증 | 결과 |
| --- | --- |
| 자체검토 focused | local-font, 문서 상태, substitution, modal copy, 초기화 순서 5개 suite 통과 |
| `npx tsc --noEmit` | 통과 |
| Stage 5 focused | 세부 TypeScript 계약 37/37 통과 |
| Stage 5 `npm test` | 931건 중 930 통과, 1 skip, 실패 0 |
| Stage 5 production build | `npm run build` 통과 |
| E2E manifest | tracked 102개 / manifest 102행 일치 |
| Chrome 151 CDP | 부분 열거 강제, LFA 1회, raw/patched KoPub 폭 delta 0px 통과 |
| 실제 HWP | `samples/2025 행정업무운영 편람(최종).hwp` 383쪽과 물리 11쪽 exact KoPub face 확인 |
| code candidate Actions | CI, CodeQL, Render Diff 성공; required `Build & Test` 통과 |
| Markdown / diff | 상대 링크·metadata 검사와 `git diff --check` 통과 |

Rust/renderer/WASM 소스, fixture, golden, baseline을 바꾸지 않았으므로 Cargo·Native Skia·wasm-pack 전체
재빌드는 범위에서 제외했다. 동일 code candidate에서 로컬 Studio 전체 검증과 GitHub의 frontend package
gate가 성공했으므로 자체검토에서는 이를 재사용하고 focused 계약과 TypeScript 검사를 반복했다.

시각 판단은 pixel visual sweep이 아니라 실제 Windows Chrome의 Canvas2D face·폭·383쪽 경계와
메인테이너의 화면 판정으로 수행했다. 따라서 임시 compare/overlay/review PNG나 별도 PR asset은 없다.
Render Diff의 Canvas visual diff도 같은 code candidate에서 성공했다.

## 위험과 판정

- Canvas 폭 probe는 여러 text/fallback에서도 exact face와 fallback이 구분되지 않으면 false negative가
  될 수 있다. 이 경우 설치를 오탐하지 않고 unresolved로 남겨 portable fallback을 유지한다.
- probe-only face는 CanvasKit SFNT 조달 능력을 뜻하지 않는다. backend별 상태를 합치지 않은 현재 경계를
  유지해야 한다.
- snapshot의 checked 후보는 사용자 승인 감지 시점의 결과다. 명시적 rescan은 새 generation을 만들고,
  같은 generation은 hot path에서 재측정하지 않는다.

코드·테스트·문서 검토에서 merge를 막는 finding은 발견하지 못했다. review·오늘할일 trailing head의 최신
required checks가 모두 통과하고 `MERGEABLE` / `CLEAN` 및 exact head SHA를 다시 확인한 뒤, 메인테이너가
merge를 별도로 승인하면 squash merge를 권고한다.
