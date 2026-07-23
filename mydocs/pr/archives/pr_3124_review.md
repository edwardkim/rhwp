# PR #3124 검토 기록 — #2428 종료 검증 및 수동 close 근거

## 1. 메타

| 항목 | 값 |
| --- | --- |
| PR | [#3124](https://github.com/edwardkim/rhwp/pull/3124) |
| 작성자 | `postmelee` |
| base | `devel` |
| head | `postmelee:issue-2428-closure-verification-pr` |
| 관련 이슈 | [#2428](https://github.com/edwardkim/rhwp/issues/2428) |
| 성격 | collaborator self-merge 후보, 문서-only 종료 검증 |
| 기준 | `upstream/devel@29b5547e256a3d6a1f8c94c9434c14a351b5543a` |
| 작성일 | 2026-07-23 |

`draft`, `mergeable`, head SHA와 CI 상태는 변동값이다. 최종 merge 조건은 PR 최신 head의
GitHub Actions 통과와 작업지시자 승인이다.

## 2. 목적과 범위

#2428의 구현은 원 기여 PR #2471이 아니라 누적 통합 PR #2521을 통해 이미 `devel`에 들어갔다.
저장소 기본 브랜치는 `main`이고 실제 통합 대상은 `devel`이라 자동 close 대신 수동 상태 정리가
필요했지만, 구현 포함 여부 재검증과 이슈 close 후속이 누락됐다.

이 PR은 코드를 다시 수정하지 않고 다음 운영 근거만 반영한다.

- 종료 검증 계획서
- 환경·픽스처 해시·실제 pointer 계측을 담은 working 기록
- 완료 조건 판정, 미종료 이유와 이슈 코멘트 초안을 담은 최종 보고서
- 2026-07-22 오늘할일 기록
- 이 collaborator self-merge 검토 기록

## 3. 최신 devel 정렬

최초 검증 기준 `12f8a820` 뒤 `devel`이 46커밋 전진해 PR 생성 전에
`29b5547e`로 rebase했다.

- #2428 fast-reject가 있는 `cursor_rect.rs`, `wasm_api.rs`, `wasm-bridge.ts`,
  `input-handler-mouse.ts`에는 변경이 없었다.
- `tests/issue_2428_footnote_fast_reject.rs`와 HWP/HWPX 거대 표·각주 픽스처 4개도
  변경되지 않았다.
- renderer/layout 통합, per-page 각주 조사, svg2pdf 공급원 이관의 간접 영향 가능성을
  고려해 production build와 실제 pointer 검증을 전부 재실행했다.
- #2428은 PR 준비 시점에도 OPEN, 코멘트 0건이고 중복 종료 검증 PR은 없었다.

## 4. 종료 검증

로컬 검증:

```text
env CARGO_TARGET_DIR=/Users/melee/Documents/projects/forks/rhwp/target \
  CARGO_BUILD_JOBS=1 cargo test --test issue_2428_footnote_fast_reject -- --nocapture
env CARGO_BUILD_JOBS=1 wasm-pack build --target web --out-dir pkg
npm --prefix rhwp-studio run build
git diff --check
```

결과:

- focused Rust: 1 passed, 0 failed
- production WASM build: PASS
- Studio `tsc` + Vite/PWA: 169 modules, PASS
- HWP/HWPX UI 114쪽 실제 pointer 클릭: 포맷별 12회
  - native `hitTestFootnote`: 두 포맷 모두 0/12회
  - 캐럿: 24/24회 cell paragraph 2499, offset 77/78, page 113
  - 표 객체 선택·각주 오진입: 0회
  - HWP handler: p50 258.1→2.8ms, p95 268.6→9.225ms
- 실제 각주 HWP/HWPX:
  - 본문 marker 진입, 각주 영역 유지, 본문 복귀 모두 PASS

## 5. 영향과 리스크

최종 diff는 `mydocs/**` 문서만 추가·수정한다. 소스, 테스트, workflow, sample,
golden/baseline과 렌더 산출물을 바꾸지 않으므로 visual sweep 대상이 아니다.

거대 표 마지막 상태 갱신에서 기존 `getCursorRectInCell` 오류가 `hitTest` cursor rect로
fallback되는 경고가 포맷별 1회 있었지만 최종 캐럿 rect와 offset은 정확했다. 이는
`hitTestFootnote` fast-reject 회귀가 아니며 #2428 merge blocker로 보지 않는다.

## 6. merge 권고와 후속

문서-only fast-pass PR로 merge를 권고한다. 최종 조건은 다음과 같다.

1. 이 review 문서를 포함한 최신 PR head 기준 preflight와 required check가 성공한다.
2. PR이 `MERGEABLE`이고 branch protection의 pending/failing check가 없다.
3. 작업지시자의 merge 승인을 확인한다.

merge 후에는 merge SHA와 이 PR의 검증 근거를 #2428에 남긴 뒤 수동 close한다. 별도 후속
문서 PR은 만들지 않는다. 현재 #2428 범위에서 알려진 잔여 구현 과제는 없다.
