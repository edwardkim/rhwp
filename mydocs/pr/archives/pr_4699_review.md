---
kind: pr-review
status: local-review-complete
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-13
---

# PR #4699 검토 - 문서 전체 HTML / Word(.doc) 내보내기

## 접수

| 항목 | 기록 |
| --- | --- |
| PR | [#4699](https://github.com/edwardkim/rhwp/pull/4699) |
| 작성자 / source | @keepYaoung / `feat/export-html-doc` |
| 최초 기여 여부 | 이전 merged PR이 없는 첫 기여자. 같은 날 열린 후속 PR #4714는 #4699의 접수 시점에 영향을 주지 않는다. |
| 원 source head | `e58720af754d576bbef472093f2f72f5054a263a` |
| 메인터너 보정 head | `199caf545bc8f86048c2a4b126145eb131967fad` |
| 기준 devel | `3c7b89356eef4f69cc2101d8e07507ba2ecf2425` |
| 가시성 검토 branch | `review/keepYaoung-4699-20260813` |
| reviewer | @jangster77 지정 완료 |

기여자 변경은 Studio 파일 메뉴에 문서 전체 HTML 내보내기와 Word 호환 `.doc` 내보내기를 추가한다.
HTML은 문서의 각 section을 이어 붙이고, Word 출력은 같은 HTML을 Word HTML MIME 형식 Blob으로 내려받는다.
두 명령은 일반 Studio chrome에서만 제공돼야 하며, host가 문서 수명주기를 소유하는 `?chrome=embed`
환경에는 나타나면 안 된다.

## 메인터너 보정

원 변경을 최신 `devel`과 병합해 Studio build와 테스트를 실행했을 때 텍스트 충돌과 기능 호환 문제는
없었다. 다만 코드 검토에서 아래 세 계약 누락을 확인해, 기여자 원 commit을 변경하지 않고 같은 source
branch 위에 메인터너 보정 `199caf545`를 추가했다.

1. `file:export-html`, `file:export-doc`가 embed hidden command 집합에 없었다. embed host가 download와
   문서 수명주기를 소유해야 하는 경계에서 로컬 export command가 노출되는 회귀다.
2. 마지막 문단 범위를 `getTextRange(..., 1_000_000).length`로 추정했다. 100만 글자를 넘는 문단은
   조용히 잘리고 JavaScript UTF-16 길이도 engine의 char 단위와 다를 수 있다. engine의
   `getParagraphLength`를 사용하도록 바꿨다.
3. section 하나의 변환 오류를 건너뛰고 나머지만 내려받으면, 사용자에게 성공처럼 보이는 부분 문서가
   생성된다. section 오류를 export 전체 오류로 전파하도록 변경했다.

보정은 export 형식이나 정상 문서의 출력 내용을 넓히지 않는다. host 경계, 정확한 document range,
실패 원자성을 기존 Studio 계약에 맞게 고정한다. 세부 변경과 회귀 추가는
[메인터너 보정 구현 기록](pr_4699_review_impl.md)에 남겼다.

## 완료한 검증

| 범위 | 명령 또는 근거 | 결과 |
| --- | --- | --- |
| export / embed 집중 회귀 | `node --test rhwp-studio/tests/export-html.test.ts rhwp-studio/tests/chrome-mode.test.ts` | 22건 통과 |
| Studio 전체 단위 테스트 | `npm --prefix rhwp-studio test` | 886건 통과 |
| Studio production build | `npm --prefix rhwp-studio run build` | 통과 |
| 최신 devel 호환 | 최신 `upstream/devel` merge simulation 뒤 Studio build | 충돌 없이 통과 |
| 브라우저 실동작 | foreground Vite에서 headless Chromium으로 일반 chrome과 `?chrome=embed` 확인 | HTML·Word Blob 다운로드, 오류 시 다운로드 0건, embed 명령·메뉴 0건 |
| GitHub Actions | code candidate `199caf545`의 Build & Test, Frontend package gates, CodeQL, Canvas visual diff | 모두 성공; Rust 전용 lane은 frontend-only 변경 정책으로 skipped |
| 병합 전 정합 | `git merge-tree --write-tree upstream/devel HEAD`, `git diff --check upstream/devel...HEAD` | merge tree 생성 및 공백 오류 없음 |

브라우저 실동작에서는 일반 chrome에서 `새 문서.html`, `새 문서.doc` Blob이 생성됐고 HTML은 단일
`<html>` 문서와 본문을 포함했다. 변환 함수를 의도적으로 실패시킨 경우 `구역 1 HTML 변환 실패`가
표시되고 Blob과 download가 모두 생성되지 않았다. `?chrome=embed`에서는 두 export command와 메뉴 항목이
모두 보이지 않았다.

renderer/layout 출력의 fidelity를 바꾸는 변경은 아니므로 HWP/HWPX fixture PDF sweep은 적용하지 않았다.
Canvas visual diff는 최신 code candidate에서 성공했으며, 이 PR의 핵심은 Studio command·download 경계와
HTML range 계약이다.

## 판단

**통합 수용 권고.** code/test 보정이 포함됐으므로 `199caf545`에서 GitHub Actions를 새로 확인했고 모두
통과했다. 이 검토 기록과 오늘할일은 같은 source branch의 단일 trailing docs-only commit으로만 추가한다.
그 head의 review-only fast-pass aggregate, mergeable 상태와 작업지시자 승인을 재확인한 뒤 merge한다.

첫 기여자 처리로 merge 후 원 PR에는 환영과 감사, HTML/Word 내보내기라는 기여의 구체적 가치, 실제 검증
결과, 그리고 메인터너 보정이 contributor의 구현 방향을 바꾸지 않고 embed 경계·대형 문단·오류 원자성을
보완한 이유를 남긴다.
