# PR #8866 검토 — 바탕쪽 편집 API와 한컴 적용 범위

## 최종 판정

**메인터너 보정 후 수용 가능 — #8020 해결 범위의 로컬 검증 완료.**

메인터너 보정을 포함한 [통합 PR #8906](https://github.com/edwardkim/rhwp/pull/8906)의 최신 head `8beab3f5677d985f07866ffbc61fde2fca6b7cfe` CI를 확인한 뒤 병합했다. merge `d7873c65bd5b5f3c55b842ee462afe8fcce51861`. 아래 완료 기록에서 기존 로컬 실측과 새 CI를 구분한다.

## 접수와 통합

- 작성자: `semanticist21`, 관련 이슈: #8020.
- 원 head: `b3dd82f130e057bff9a9d213c0b9dfe946955aa7`.
- 최초 검증 devel: `5f4e95c187f64116f48f710c3c048162786c1c14`.
- reviewer `jangster77` 지정을 API로 확인한 뒤 원 head를 fetch했다. 원 contributor 브랜치는 수정하지 않았다.
- 적용 순서: `e66f254b2b → 541c545066`, `78ad6de758 → 6c682d2a96`,
  `eb65e8946a → 6d4f8ee231`, `b3dd82f130 → a2fdc92f15`. 각 commit에 `-x` 출처를 보존했다.

## 변경과 검토 범위

`createMasterPage`·`deleteMasterPage`·조회·편집 대상 API, 바탕쪽 글의 머리말/꼬리말 편집 경로,
홀짝·마지막 쪽·임의 쪽·겹침·앞 레이어 선택, 저장 목록, 화면 전용 흐림을 확인한다.
새 Studio 도구 모음 UI를 구현한 PR로 해석하지 않는다.

선행 PR #1682의 실제 통합 commit은 `e0035ba5846ced85c324ddab71c0ece49b0ac292`이다.
해당 PR은 plane·z-order·paper-origin과 구역 간 홀짝 이어받기를 다뤘다. 새 PR이 제거한 것은
홀짝과 이어받기 모두에 맞지 않을 때 단일 바탕쪽을 강제로 고르던 마지막 fallback이다.
앞 구역의 홀짝 이어받기 경로는 유지한다. 한컴 12.30의 짝수 바탕쪽 독립 Print와 대조한다.

## 메인터너 보정

`f94a074958409ed774a1760f18746831d819723f`는 HWPX 바탕쪽 파트를 붙인 뒤,
저장기가 읽는 첫 문단의 구역 정의 사본에도 목록을 반영한다. 공개 Native 직접 저장 경로에서
HWPX를 거쳐 재저장하면 기본 바탕쪽이 3개 중 1개 사라지는 문제를 통합 검사로 발견했다.
메모 모양과 바탕쪽을 먼저 적용하는 순서를 바꾸고 HWP/HWPX 및 재저장을 검사했다.
보정 전 2건 중 1실패, 보정 후 2건 모두 통과다. 이 결과를 모든 Studio 저장 경로의 실패로 확대하지 않는다.

## 독립 시각 근거

입력은 contributor의 API 생성 HWP이고, 기준 PDF는 별도로 한컴 12.30.0(6544, macOS)의
파일 > 인쇄 > PDF로 저장한 3쪽씩이다. 입력 생성과 독립 Print의 역할을 구분한다.
기존 `mydocs/pr/assets/issue_8020/README.md`의 원본/PDF 해시를 유지한다.

- `samples/issue8020/master_even_only.hwp`: 1·3쪽 바탕쪽 없음, 2쪽에만 `짝수바탕`.
- `samples/issue8020/master_last_optional_overlap.hwp`: 1쪽 `가양쪽`, 2쪽 `가양쪽`+`다둘째`,
  3쪽 `나끝쪽`. 같은 쪽의 `라셋째`보다 마지막 쪽 바탕쪽이 우선한다.
- 고정 base Native/fixed WASM의 실패를 같은 원본·PDF·96 DPI/compat 2022/2px raw 기준으로 재현했다.
- 통합 후보 Native 6쪽과 fresh WASM 6쪽은 모두 100.00%. 두 입력 모두 flagged 후보는 0이다.
  Native의 엄격 pixel_match 최저 99.97465%, fresh WASM 최저 99.97566%이며,
  visual_accuracy_proxy 최저는 각각 61.62988%·63.15789%다. 작은 글자·래스터 윤곽 차이가 있으므로
  2px 실루엣 100%를 RGB 픽셀 완전 동일성으로 해석하지 않는다.
  같은 페이지의 review·overlay에서 홀수 쪽 미적용, 겹친 바탕쪽과 마지막 쪽 우선순위, 본문을 직접 대조했다.

## 검증

- 통합 집중: 최소 글꼴 환경 22건 모두 통과. 그중 원 PR 검사는 9건이다.
- 실제 fresh WASM 브라우저: 화면 흐림·해제 복원, 인쇄 불변, 겹친 쪽의 편집 대상·preview,
  두 형식 저장·재저장과 메모 보존을 확인했다. 실제 초기화 JS/WASM 응답 해시도 기록했다.
- Studio TypeScript 통과, tests 2,784 PASS / 0 FAIL / 2 skip.
- 최소 글꼴 전체 Rust 회귀 13,258 PASS / 0 FAIL / 45 skip(629.281초).
- Native/WASM/all-target Clippy·workspace build 통과. Native Skia 3종과 보호 fresh WASM 전쪽 비교까지 통과했다.
- 보호 Native 18문서 1,771쪽 공식 두 자리 표시값 하락 0. 필수 편람 Native 383쪽 gate 통과.
- 이후 들어온 upstream #8899의 리베이스 후 기능 보존 검사 35건도 통과했다. 아래 재사용 절에 실측 head를 구분한다.

## 조판 원칙 준수

| 항목 | 근거 | 현재 판정 |
| --- | --- | --- |
| 형식/독립 기대값 | 한컴 12.30 저장 LIST_HEADER·구역 비트·Print, 기존 #1682 범위 대조 | 충족 |
| 생산과 소비 | 파서의 종류/속성 → MasterPage 소유/우선순위 → PageContent 선택 → 공통 paint/backend | 충족 |
| 화면/인쇄 계약 | dim 옵션은 실제 화면 Canvas에서만 적용, 인쇄 PNG 전후 동일 | 충족 |
| 저장 보존 | HWPX 늦은 파트 결합과 inline 구역 정의 동기화, 두 형식 재저장 | 충족 |
| 최종 전쪽/회귀/게이트 | 전쪽 시각·전체 회귀·Skia·lint/build와 리베이스 후35건, 재사용 출처 기록 | 충족 |

## Merge 후 contributor PR comment 계획

[Visual Sweep merge comment 정본](../../manual/verification/visual_sweep_guide.md#github-merge-comment)을 따른다.
통합 PR이 실제 병합되고 아래 asset이 devel에 포함된 뒤, 병합 SHA를 고정한 다음 URL을 사용한다.

- 원 PR 주장 페이지: `master_even_only.hwp` 1·2·3쪽, `master_last_optional_overlap.hwp` 1·2·3쪽.
- 대표 raw: `https://raw.githubusercontent.com/edwardkim/rhwp/d7873c65bd5b5f3c55b842ee462afe8fcce51861/mydocs/pr/assets/semanticist21_20261110/master-pages/candidate/native/master_even_only/rhwp_png/rhwp_002.png`
- 대표 review·overlay: 같은 폴더 아래 `review/review_002.png`·`overlay/overlay_002.png`.
- 마지막 쪽·지정 쪽 우선순위: `master_last_optional_overlap`의 같은 페이지 2·3쪽 raw/review/overlay.
- 게시 내용은 원 기여의 바탕쪽 API와 적용 범위 수정, 메인터너의 HWPX 직접 저장 보정,
  전쪽 지표·직접 판독·RGB 차이의 한계를 구분해 한국어 존댓말로 설명한다.
- 최신 head CI와 병합을 확인한 뒤 승인 범위에서 `--body-file`로 게시하고 API로 본문·이미지를 재확인한다.

## 충돌 없는 리베이스와 검증 자료 재사용

검증 base `5f4e95c187f64116f48f710c3c048162786c1c14`에서 새 devel
`411d4a91fd692f88fdeebd6b323eca0b295318b1`로 리베이스했다. 현재 통합 code head는
`6eccab18f3458090bb8c037c8040f49c28b102f7`이며 이후 commit은 지침·증적·리뷰 문서다.
11개 패치가 모두 동일하게 재적용됐다. [전후 commit 대응](../assets/semanticist21_20261110/rebase-commit-map.tsv).
원 contributor head와 원격 브랜치는 변경하지 않았다.

사용자의 **충돌이 없으면 리베이스 후 현재 자료 재활용** 지시에 따라 기존 전쪽 실측을 보존했다.
기본 전체 회귀는 리베이스 전 `8b4e09c054`의 **13,258 PASS / 0 FAIL / 45 skip**이며,
새 code head에서는 최소 글꼴 환경 집중 **35 PASS / 0 FAIL**을 실제 실행했다.
기존 통합 22건·raw 무효화 위임 5건과 새 #8863의 삭제·치환·왕복 8건을 포함한다.
[새 head 집중 실행](../assets/semanticist21_20261110/post-rebase-state.json).

runtime 3,398개 파일 중 새로 달라진 것은 `src/model/paragraph.rs` 하나이며 새 devel의 내용과 동일하다.
삭제 전에 중복 글자 모양의 유효 값을 보존하는 #8899/#8863 변경이다. 읽기 경로의 호출도 확인했다.
보호 18개 문서와 바탕쪽·메모 입력 6개에는 미주가 없고, HWP3 두 문서는 중복 위치를 미리 정리한 뒤
outline marker를 삭제한다. [호출 경로·입력 감사](../assets/semanticist21_20261110/rebase-read-path-audit.json).
[실제 입력49개 Git 일치](../assets/semanticist21_20261110/post-rebase-committed-input-audit.json).

전쪽 시각 producer는 `32c6023fb5f333bb9c464c75258f922559176047`의 제품 소스다.
Native 실행 파일과 Mac locked wrapper의 web/no-opt WASM package를 해시로 고정해 사용했다.
최신 head에서 전쪽을 다시 출력한 결과로 표기하지 않으며, 위 영향 감사와 사용자 승인으로 재사용한다.
[이전 실측·정식 게이트·재사용 출처](../assets/semanticist21_20261110/pre-rebase-validation.json).

## 공통 최종 검증

- 최소 글꼴 격리: CoreText 528 faces 모두 OS 기본 경로, 개인·상용 글꼴 0개.
  일반 환경의 검증 전후 1,875 faces 목록도 동일했다. 실제 시각 비교의 명시적 글꼴 공급과 구분한다.
- fmt·정책 base 고정 manifest·unit tiers 통과. Native/WASM/workspace all-target Clippy와 workspace build 통과.
- Native Skia lib: 루트 3,852 PASS / 0 FAIL / 12 ignored, 나머지 라이브러리 182 PASS.
  placeholder 2 PASS, 직접 PDF 출력 35 PASS. 실행 명령과 producer는 위 정식 게이트 기록에 보존했다.
- Studio TypeScript 통과, tests 2,784 PASS / 0 FAIL / 2 skip.
- 실제 fresh WASM 브라우저 8경로 통과. Root pkg·Studio public·초기화 응답 JS/WASM 해시가 일치했다.
  [브라우저 실제 결과](../assets/semanticist21_20261110/actual-browser/result.json).
- 보호 18개 문서의 Native 1,771쪽·WASM 1,771쪽, 공식 두 자리 표시값 하락 0.
  최저는 각각 98.36%·98.37%. 필수 편람 Native 383쪽 gate도 통과했다.
  [Native 전쪽 TSV](../assets/semanticist21_20261110/protected/native/)
  · [fresh WASM 전쪽 TSV](../assets/semanticist21_20261110/protected/wasm/).

## 실행 설정 보정과 추가 요청

기존 승인된 글꼴 환경과 원본 글꼴 1,219개 파일의 해시를 유지했다. 최초 이관 실행에서 빠뜨린
명시적 글꼴 조건은 복원해 재실행했으며, 그 미설정 결과를 최종 통과 증거에 사용하지 않았다.
또한 sample10의 연결 그림 3개 공급 누락으로 1쪽이 31.60%였던 실행은 실패 증거로 보존했다.
원본과 동일한 `oracle.gif`·`rdb02.gif`·`s1.jpg`를 Mac에서 `--linked-image`로 공급하여
1쪽 99.96%를 확인했다. 전 763쪽 트리에서 이 그림들이 1쪽에만 쓰임을 확인하고 나머지 762쪽 PNG를
재활용해 정본으로 전쪽 TSV를 다시 계산했다. [재사용 감사](../assets/semanticist21_20261110/sample10-linked-image-reuse-audit.json).
그림 제외 옵션이나 판정 문턱 변경은 적용하지 않았다.

사용자 요청으로 기여자 공통 절차·PR 템플릿에 실제 편집·저장·재열기·재저장 검증을 추가했다.
Mac 연결 그림 공급 지침은 사용자명 없는 `$PWD` 예시로 기록했다.
Adapter inter-diff 사전 검증은 실제 3분 job 취소를 근거로 10분, 전체 이력 checkout은 8분으로 조정했다.
worker 25분·권한·실패 시 본 검증 실행 계약은 유지했다. actionlint·관련 Python 79건·Node 334건 통과.
[CI 변경 검증](../assets/semanticist21_20261110/ci-timeout-validation.json).
통합 PR의 최신 required check 성공과 승인된 병합을 확인했다. CI 변경의 병합 후 기록은 아래 옵션 2 절로 정리한다.

## 통합 PR #8906 CI·병합 완료 및 옵션 2 기록

- 통합 PR: [#8906](https://github.com/edwardkim/rhwp/pull/8906).
- 최종 검증·병합 head: `8beab3f5677d985f07866ffbc61fde2fca6b7cfe`.
- squash merge: `d7873c65bd5b5f3c55b842ee462afe8fcce51861`, 2026-11-10 18:56:59 KST. `CLEAN / MERGEABLE`과 최신 head의 전체 check 성공을 다시 확인한 뒤 일반 병합했다. admin 우회는 사용하지 않았다.
- [CI](https://github.com/edwardkim/rhwp/actions/runs/42790512773), [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/42790512527), [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/42790512513), [Adapter](https://github.com/edwardkim/rhwp/actions/runs/42790512548), [Proptest](https://github.com/edwardkim/rhwp/actions/runs/42790512536)와 exact head의 `CI Impact Policy`가 성공했다. CI 아카이브 A~E의 실제 합계는 12,998 PASS / 0 FAIL / 45 skip이다. Native Skia·프런트엔드 게이트도 성공했다.
- 이전 head `5dd73f65`의 아카이브 C 실패는 새 통합 검사의 시간대 가정 오류였다. 선행 #8269의 FILETIME→현지 시각 계약과 한컴 HWPX의 리터럴 Z를 보존하면서, 같은 형식의 무편집 저장 대조군과 날짜를 포함한 메모 목록 전체를 비교하도록 테스트 1개 파일만 보정했다. 제품·직렬화·시간대 정책은 변경하지 않았다.
- 최소 글꼴 환경 `TZ=UTC` 27 PASS와 `TZ=Asia/Seoul` 27 PASS, fmt·suite manifest·unit tier 검사를 통과했다. 새 Linux CI에서도 실패했던 검사와 아카이브 C 2,600건 모두 PASS를 확인했다. 실패한 이전 CI를 최종 통과 근거로 재사용하지 않았다.
- 기존 로컬 전체 회귀·전쪽 시각 검증의 실제 producer와 리베이스 후 재사용 근거는 위 기록을 유지한다. 테스트 보정 뒤 제품 파일은 바뀌지 않았으며, 이전 실측을 새 head의 재출력으로 표시하지 않는다.
- [duration 갱신](https://github.com/edwardkim/rhwp/actions/runs/42794973755)이 성공했고 44 target을 게시했다. [최종 CI·병합·duration 원장](../assets/semanticist21_20261110/ci-8906-complete.json)과 [시간대 보정 근거](../assets/semanticist21_20261110/memo-timezone-validation.json)를 보존한다.
- CI workflow 변경이 포함되어 이 완료 기록·오늘할일·최종 CI 증적은 원 PR의 trailing commit 대신 옵션 2 별도 기록 PR로 반영한다. 기록 PR의 실제 fast-pass와 병합을 확인한 뒤 원 PR별 결과 댓글·종료와 #8020 API 완료 처리를 진행한다. #8017의 기존 CLOSED 상태는 유지하며 새로운 Studio 댓글 풍선 UI 완료를 주장하지 않는다.
