---
kind: pr_review
status: active
pr: 4773
issue: 3931
last_verified: 2026-08-14
---

# PR #4773 검토: 저장 RowBreak 표의 물리 fragment 경계

## 메타데이터

| 항목 | 값 |
| --- | --- |
| PR | [#4773](https://github.com/edwardkim/rhwp/pull/4773) |
| 관련 이슈 | [#3931](https://github.com/edwardkim/rhwp/issues/3931) |
| 작성자 | `edwardkim` |
| reviewer | `jangster77` 요청 완료 |
| base / head | `devel` / `task_m100_3931` |
| code candidate | `39372acee66d4497d054ef4e46cd525e3ee6f50c` |
| 작성 시점 merge 상태 | `MERGEABLE`, `BLOCKED` |
| 규모 | 16 files, +1,647 / -28, 11 commits |

base route: maintainer_general
modifiers: intake_and_review.md, local_validation.md, visual_fixture_evidence.md, rework_and_exceptions.md
loaded documents: pr_review_workflow.md, pr_review/README.md, maintainer_general.md,
intake_and_review.md, local_validation.md, visual_fixture_evidence.md, rework_and_exceptions.md
initial review-record code candidate: `d81d11e545883ce8817b665ee64bd9262874da58`
CHANGES_REQUESTED remote head: `010c005890d41b5cd2467276025d6a613f154c04`
current code candidate after requested changes: `39372acee66d4497d054ef4e46cd525e3ee6f50c`

## 변경 범위와 코드 판정

- 빈 host 문단의 다행 RowBreak 표에서 저장 내부 reset과 flow anchor가 어긋날 때, 완결된 구조
  증거가 있는 경우에만 기존 fragment scanner가 현재 쪽 조각을 만들도록 한다.
- 질문 11의 24.5px pitch와 16줄 전체 높이는 유지하고 물리 p287의 12줄과 p288의 4줄 소유권을
  고정한다. 특정 파일명·쪽수·문단 인덱스 예외나 전역 tolerance는 없다.
- #4763 terminal response tail은 가시 텍스트가 있는 control-free 문단 경계에서만 억제한다.
  control-only local reset과 질문 14 중첩 도형은 기존 source-frame 계약을 유지한다.
- 변경 요청 뒤 `native_multirow_saved_reset_trailing_trim`에도 이전·다음 문단의 control-free 가드를
  추가했다. 따라서 `text -> control-only paragraph(vpos=0)`인 로컬 reset에서는 마지막 줄의
  line/paragraph spacing을 빼지 않는다.
- partial table host를 확인하지 못하면 margin을 회수하지 않는 fail-closed 경로를 사용한다.
- 1,000줄을 넘는 PR이므로 즉시 admin merge 대상이 아니다. 코드 검토, simulation, 시각 판정과
  최신 trailing head CI를 분리해 확인한다. 줄 수의 상당 부분은 단계 문서와 실제 fixture 래칫이지만
  renderer 다섯 파일의 위험 경계를 축소해서 보지 않는다.

## 로컬 검증 결과

- code candidate는 `upstream/devel` `99f6c9312` 위로 충돌 없이 rebase한 계보를 유지한다. 변경 요청
  보완 중 원격 `devel`이 #4772 `a5a92ca3b`까지 한 커밋 전진했으며,
  `git merge-tree --write-tree --messages upstream/devel HEAD`로 만든 merge tree
  `1dbff2825a26519962597d1adeb2c7559b35df11`은 충돌 없이 생성됐다. 아래 전체 로컬 검증은 고정
  code candidate tree 기준이고, 최신 base까지 포함한 merge-ref 판정은 push 뒤 GitHub CI에서 다시 받는다.
- 변경 요청의 control-only 형상을 unit test로 먼저 실행해 수정 전 `8.0px` trim과 기대값 `0.0px`의
  RED를 확인했다. 두 문단을 모두 control-free로 제한한 뒤 plain-text 양성 경로, `text -> control-only`,
  control-bearing 이전 문단의 세 계약이 통과했다.
- `issue_3930_hwpx_hwp_save_layout` 3/3, `issue_3931_declared_rowbreak` 5/5와
  `issue_3738_rowbreak_table_footnote_fragment` 33/33이 통과했다.
- `cargo nextest run --cargo-profile release-test --target-dir target/pr-review --tests
  --test-threads 12 --no-fail-fast`는 새 회귀를 포함해 6,028/6,028 통과, 38 skip이다.
- `cargo clippy --target-dir target/pr-review --all-targets -- -D warnings`, `cargo fmt --check`,
  `git diff --check`가 통과했다.
- Native Skia 공식 세 축은 58/58, 2/2, 4/4 통과했다.
- `docker compose --env-file .env.docker run --rm wasm` 공식 최적화 빌드가 통과했다.
  `pkg/rhwp.js` SHA-256은 `a226ed9d30ba724addbdd6b3539c407e9909e8ae7d8a195f2632bf56b9419c9b`,
  `pkg/rhwp_bg.wasm` SHA-256은 `a6f21e19f81137851cee2bd49b62461668744f47af899f4037becd6948aa5cd5`다.
- Studio `npm test`는 923건 중 922 통과, 1 skip, 실패 0이다.
- Windows Chrome 150 CDP에서 Vite HTTP fixture로 HWP/HWPX 각각 383쪽, source format,
  지정 page owner와 Canvas2D 경로를 확인했다. WSL 절대 경로의 host file input은 Windows Chrome이
  읽을 수 없으므로 기존 `loadHwpFile` HTTP 계약을 사용했다.

## 비공개 10k 무회귀 검증

변경 요청 보완 code candidate `39372acee66d4497d054ef4e46cd525e3ee6f50c`의 release-test CLI를
비공개 HWP/HWPX 10,000건에 적용했다. 원본, 파일명, 개별 경로와 원시 NDJSON은 저장소·PR에
포함하지 않고 로컬 임시 경로에만 보존한다. 하니스는 `<private-corpus-root>`의 HWP/HWPX를 같은
filesystem 순서로 `rhwp batch info --json --threads 12`에 전달한다. 보존된 직전 최종 후보를 같은
명령으로 재실행한 결과 10,000행 NDJSON이 byte-identical해 입력 순서와 실행 계약을 먼저 확인했다.

| 항목 | #4763 기준선 | `39372acee` 후보 | 변화 |
| --- | ---: | ---: | ---: |
| 전체 입력 | 10,000 | 10,000 | 0 |
| 성공 / 오류 | 9,948 / 52 | 9,948 / 52 | 전이 0 |
| 성공 집합 공통 | 9,948 | 9,948 | 누락 0 |
| 쪽수 동일 | - | 9,943 | - |
| 쪽수 변화 | - | 5 | 증가 0, 감소 5 |

쪽수 변화 5건은 기존 최종 후보와 동일한 HWP5 집합이다. 네 건은 1쪽, 한 건은 5쪽 감소하며
기준선 합계 331쪽에서 후보 322쪽이 된다. `39372acee`와 변경 요청 전 최종 후보의 성공 문서
9,948건은 쪽수가 전부 동일하므로 이번 control-only 가드가 새 쪽수 변화를 만들지 않았다.
변화 5건은 `39372acee`로 322쪽 전부 SVG export를 완료했고 `overflowCellLines=0`이며, 직전 후보와
쪽수·overflow 집계가 동일하다.

## 시각 검증

- 원본 HWP: `samples/2025 행정업무운영 편람(최종).hwp`, SHA-256
  `40d6d05eac4d55bdc4b0c62c42d93af104d5123b447581246f36fd15de7bd46f`
- 원본 HWPX: `samples/2025 행정업무운영 편람(최종).hwpx`, SHA-256
  `c6dd7e847a99f219681afc5a29c80a9665c04df9cda4d820a3350d739664fdf6`
- 한컴 2020 HWP PDF: `pdf/2025 행정업무운영 편람(최종)-hwp-kopub-2020.pdf`, SHA-256
  `6c7be7602cb92bb9b5e6a0b66e9cd80700fceabeade89fabd1a0fcd32adc4413`
- 한컴 2020 HWPX PDF: `pdf/2025 행정업무운영 편람(최종)-hwpx-kopub-2020.pdf`, SHA-256
  `5c11205cb43ba3a1ca3e607e4019b69a937332526a1b740d3dda754dcc4e3f0a`
- 임시 근거: `output/3931/visual/current/issue3931-current/`의 compare·overlay·review 4쪽.
  검토 페이지는 물리 p284, p285, p287, p288이며 자동 accuracy 보조값은 62.59162%,
  61.11530%, 67.22229%, 46.20458%다.
- 대표 asset: `mydocs/pr/assets/pr_4773_issue3931_rowbreak_review.png`, SHA-256
  `9896cd128a5aebd625aa59122aeab19f3075653e7353263602b86ce40a838180`
- 작업지시자가 네 동일 물리 쪽과 7702 Studio 렌더링을 확인하고 시각 판정 통과를 선언했다.

![PR 4773 visual review](../assets/pr_4773_issue3931_rowbreak_review.png)

## CI와 위험

변경 요청 전 원격 head `010c005890d41b5cd2467276025d6a613f154c04`에서는 CI, CodeQL,
Render Diff, Native Skia와 전체 test shard가 모두 성공했다. 다만 `39372acee`는 source와 test를
바꾼 새 code candidate이므로 이전 head의 성공을 재사용하지 않는다. 이 문서 보정 commit을 더한 최신
head를 push한 뒤 GitHub Actions 전건 성공을 다시 확인해야 한다.

로컬 headless Chrome UI 전체 초기화는 WASM 383쪽과 `CanvasView.loadDocument` 완료 뒤 진행 표시
갱신 구간에서 장시간 정체됐다. Windows Chrome CDP의 직접 WASM·CanvasView 계약은 두 형식 모두
통과했으므로 #3931 renderer 차단 결함으로 분류하지 않되, Studio 대형 문서 초기화 성능 관측은
후속 이슈 후보로 남긴다.

## 결론

변경 요청 세 항목인 control-only trim 가드·최신 후보 10k 무회귀 근거·maintainer 검토 경로와 실제
확인일 정정을 로컬에서 완료했다. focused·전체·Native Skia·Docker WASM과 10k 검증에서 #3931 변경을
막는 새 결함은 발견하지 못해 재검토 후보로 권고한다. 다만 대형 PR 규칙에 따라 즉시 admin merge하지
않는다. 최신 head를 push한 뒤 reviewer 재검토, GitHub Actions 전건 성공, `MERGEABLE`/`CLEAN`과
작업지시자 승인을 다시 확인한 뒤 merge를 판단한다. merge 후에는 #3931 종료와 원격·로컬 branch
정리를 수행한다.
