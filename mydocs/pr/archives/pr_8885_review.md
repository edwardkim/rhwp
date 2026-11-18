# PR #8885 검토 — 메모 모양과 댓글 관계 API

## 최종 판정

**메인터너 보정 후 수용 가능 — #8017의 메모 모양·댓글 API 범위의 로컬 검증 완료.**

메인터너 보정을 포함한 [통합 PR #8906](https://github.com/edwardkim/rhwp/pull/8906)의 최신 head `8beab3f5677d985f07866ffbc61fde2fca6b7cfe` CI를 확인한 뒤 병합했다. merge `d7873c65bd5b5f3c55b842ee462afe8fcce51861`. 아래 완료 기록에서 기존 로컬 실측과 새 CI를 구분한다.

## 접수와 통합

- 작성자 `semanticist21`, 관련 이슈 #8017의 후속 API.
- 원 head `d49cedb7297c92927bebc0897fec4603a49f04b0` → 통합 commit `5f3a058fadc135b643a1c90607c7e198af4d395b`.
- 고정 devel `5f4e95c187f64116f48f710c3c048162786c1c14` 위에 #8866 뒤로 `-x` 적용.
- reviewer `jangster77` 지정을 API로 확인한 뒤 상세 검토·fetch했다. 원 PR head는 변경하지 않았다.
- 선행 메모 API PR #8269의 통합 commit은 `ccc2453261f777a1c12662d8d5ff6dafda174071`이다.

## 변경과 독립 근거

한컴 12.30 Mac 실제 저장본 `tests/fixtures/memo_hancom/shape-new-section.hwp/.hwpx`,
`reply.hwp/.hwpx`의 메모 모양 레코드·구역 참조·댓글 부모 필드 번호를 대조한다.
`getMemoShape`·`setMemoShape`, `getMemoList`의 `parentId`, `insertMemoReply`와 부모 삭제 시
댓글 삭제가 대상이다. Studio의 새로운 댓글 풍선 UI를 제공한 것으로 해석하지 않는다.

## 충돌 해소

`src/serializer/control.rs::write_section_def`에서 #8866과 충돌했다.

| 저장 영역 | 보존할 의미 | 통합 결과 |
| --- | --- | --- |
| `raw_ctrl_extra[2..4]` | 구역 끝 바탕쪽 목록 수 | 구역을 다 쓴 뒤 실제 목록 수로 patch하는 #8866 계약 유지 |
| `raw_ctrl_extra[6..8]` | 메모 모양 참조 번호 | #8885의 nonzero 참조 쓰기와 나머지 원본 byte 보존 유지 |

두 필드를 독립적으로 유지하는 writer로 결합했다. 생성 순서를 바꾸고 두 형식 저장·재저장에서
바탕쪽 수/종류와 메모 모양·댓글 내용/부모 관계를 함께 검사한다.

## 메인터너 보정

`32c6023fb5f333bb9c464c75258f922559176047`:
손상된 HWPX의 댓글 부모가 순환하면 `removeMemo`가 재귀 호출을 반복해 스택 오류로 종료됐다.
실제 저장·로드 후 삭제를 호출한 음성 대조에서 SIGABRT/stack overflow를 확인했다.

삭제 전에 자손 계획과 순환을 확인하고, 반복 순회로 자식부터 삭제한다. 댓글 추가도 실제 최상위
부모를 찾아 순환을 거절한다. 순환 시 내용·관계 불변, 정상 다단계 댓글의 최상위 부모 선택과
다른 대화 보존을 검사해 보정 후 3건 모두 통과했다. 원본 한컴 댓글 동작도 원 PR 8건으로 유지했다.

## 검증

- 최소 글꼴 통합 집중 22 PASS / 0 FAIL. 원 PR 8건, #8866 9건, 통합 저장 2건, 댓글 경계 3건.
- 실제 fresh WASM에서 메모 모양 설정·조회, 댓글 추가/삭제와 바탕쪽 공존·두 형식 저장/재저장 확인.
- Studio TypeScript 통과, 2,784 PASS / 0 FAIL / 2 skip.
- 최소 글꼴 전체 Rust 회귀 13,258 PASS / 0 FAIL / 45 skip(629.281초).
- Native/WASM/all-target Clippy·workspace build 통과. Native Skia 3종과 보호 fresh WASM 전쪽 비교까지 통과했다.
- 메모 삭제의 실제 raw 무효화를 유지하면서 공용 passthrough guard의 `DelegatesTo(remove_memo_only)`
  분류를 갱신했다. 기존 위임 검사 5건이 실제 helper의 무효화 도달을 확인해 통과했다.
- 이후 들어온 upstream #8899의 리베이스 후 기능 보존 검사 35건도 통과했다. 아래 재사용 절에 실측 head를 구분한다.

## 조판 원칙과 증적 분류

이 PR의 메모 모양·댓글 관계 API는 저장 메타데이터와 편집 계약이며 본문 조판/paint를 바꾸지 않는다.
한컴 저장본의 byte/XML와 실제 API·저장 후 재로드 결과를 근거로 삼는다.
통합 브랜치의 조판 검증은 #8866의 영향을 포함해 별도로 완료하며, 메모 API 결과를 픽셀 피델리티
검증으로 표시하지 않는다. 모든 입력의 commit 포함·실행 byte 일치도 감사했다.

## 증적과 contributor 안내 계획

원 PR의 실제 한컴 저장본 4개의 HWP raw 필드·HWPX XML과 테스트 assertion을 연결한다.
바탕쪽 review 이미지는 이 메모 API 자체의 검증 그림으로 사용하지 않는다.
통합 API 동작은 `mydocs/pr/assets/semanticist21_20261110/actual-browser/result.json`의
`both-format-roundtrip`·`repeat-save`·`root-reply-and-cascade-delete` 결과로 연결한다.
최소 글꼴 Rust 검사는 상용 원본 글꼴 설치 없이 수행했다.

통합 PR 병합 후 원 PR에는 메모 모양·부모 필드 구현의 기여와, 순환 참조 삭제 시 프로세스가 종료되어
메인터너 보정이 필요했던 이유·검증 범위를 구분해 한국어 존댓말로 안내한다. 실제 병합 SHA·CI·이 검토
문서를 연결하고 승인된 시점에 `--body-file`로 게시한 뒤 API로 본문을 확인한다. 이 API 범위로 #8017 전체나
새로운 Studio 댓글 UI 구현 완료를 주장하지 않는다.

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
