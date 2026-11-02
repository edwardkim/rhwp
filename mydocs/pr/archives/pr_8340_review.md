# PR #8340 리뷰 — 중첩 표 문단 정렬과 Undo/Redo

## #8438 통합 이후 기능 보존 확인 진행

#8340은 `c88657d9c3`으로 먼저 머지됐고, #8438의 검증 head는 이를 포함하지 않은 상태에서 후속 머지됐다. 따라서 #8438 머지 이전의 전체 테스트를 #8340 기능 보존의 증거로 사용하지 않는다.

머지 후 소스로 새로 만든 개발용 WASM(`rhwp_bg.wasm` SHA-256 `4b8bfd61315f0bca6a1ebaad56b70742249c7daee6f059119b1e32effb3d11c8`)과 Studio의 실제 중첩 셀 정렬·Undo/Redo·저장 왕복 집중 검사 21건은 모두 통과했다. 종전 로컬 `pkg`는 #8340 API가 없는 이전 산출물이었으며, 이를 사용한 첫 실행의 실패와 새 WASM의 결과를 구분한다.

기존 한컴 Print와 SHA-256 `d85550717552101699122adc5b948e8129881f39ffcf802cafe62aac3b1ced40`의 함초롬바탕을 사용해 위 20개 입력의 전체 쪽을 재측정했다. Native와 개발용 WASM은 각각 20건 모두 raw 100.00%이며 입력별 원본 TSV는 [Native](../assets/pr8340-post-8438/native/)와 [WASM](../assets/pr8340-post-8438/wasm/)에 있다. Native 실행 파일은 #8438 제품 코드 `d536455b4e`의 SHA-256 `d53e4dac28839e01f4c6bf19761c4bbc181ef70a7a38eb65abf6f6c4050b96fc`이며, 머지 결과와 렌더러·파서·모델 소스가 같다. 이 저장 입력 비교와 실제 편집 검사를 구분한다.

최종 통합 소스의 최적화 fresh WASM(`rhwp_bg.wasm` SHA-256 `14c7a52f1e0f7461222126d26b9358072545fb77d4ea76593d52882fdb2183c6`)으로도 같은 실제 편집 집중 검사 21건이 통과했다. 20개 입력의 전쪽 raw는 모두 100.00%였고, WASM TSV 20개는 위 개발용 WASM 결과와 바이트 단위로 동일했다. 이 범위에서는 #8438 통합 후 #8340의 검증된 기능과 출력이 유지된다.

최적화 WASM 집중 검사는 시스템·사용자 글꼴과 `/opt`를 숨기는 `bwrap` 환경에서도 21건 모두 통과했다. Node 24 실행 파일만 별도로 노출했고, `fc-list`에는 저장소 `ttfs/opensource`의 Noto Sans KR 2종, Noto Sans Symbols 2, RHWP 옛한글·공개 바탕 등 5개 파일만 표시됐다. 첫 격리 실행은 Node 20의 TypeScript 확장자 오류였으며 Node 24로 재실행해 확인했다. 이 회귀 환경과 위 Print용 글꼴 환경은 별개다.

## 최종 판정

**메인터너 보정 후 수용 가능 — #7637의 경로 기반 문단 서식과 Studio 연결 범위의 로컬 검증 완료.**
원 기여의 Rust·Studio 구현을 보존하고 누락된 public TypeScript 선언과 편집 저장본의 독립 Print
증거를 보완했다. 두 공개 문서의 20개 상태 모두 Native/fresh WASM 전쪽 raw 실루엣 100.00%,
쪽수 일치와 확대 판독을 확인했다.

로컬 제출 후보는 `d073ed921b5dc7e8e9b7f8449b1ef58ac7daf52d`다. **이 code head의 전체 CI와 별도 검사·정책 감사가 모두 성공했다**.
2026-11-02 작업지시자가 push·새 head CI·self-review·merge·후속 댓글을 승인했다.
원 PR의 원격 head와 fork ref가 제출 후보와 같음을 다시 확인했다.

## 접수와 검토 경로

| 항목 | 값 |
| --- | --- |
| PR·작성자·base | [#8340](https://github.com/edwardkim/rhwp/pull/8340) / `ashcastle`, 첫 기여 / `devel` |
| 관련 이슈 | [#7637](https://github.com/edwardkim/rhwp/issues/7637), Refs이며 자동 종료 요청 없음 |
| 원 source head | `806e151776e6fae7a482bccad1aa4af8fa8dda23` |
| 기여자의 검증 source | `9bf6137e30a9a7153feb7a38cb506af413c754f2`, 이후 원 head까지 증적만 추가 |
| 최신 검증 base | `1a7ae8f5bd17ebc22b281fd1915973e18e4e641f` |
| 검증 merge ref·tree | `bc4da7875991725d7bdcef78e0e11e2ddf2929ad` / `a42db2adae181a31788805cd4e5852744a8a2744` |
| 원 PR 제출용 선언 보정 | `3fa96a6e8da8558d8a7a9cdaa32adaf4644e202d`, 원 source 위 9줄 추가 |
| 최초 로컬 통합 검증 보정 | `f19b334e66f4d70632669daa91ca4b403682cd5e`, 원격 push 대상 아님 |
| 검증 증적 commit | `d073ed921b5dc7e8e9b7f8449b1ef58ac7daf52d` |
| reviewer | `edwardkim`, requested-reviewers API로 지정 확인 |
| branch·worktree | `review/pr-8340-source-correction` / `/home/edward/mygithub/rhwp-worktrees/pr-8340-nested-paragraph-format` |

기본 경로는 `maintainer_general.md`다. 접수·로컬 검증·첫 기여자·시각 증적·review template과
후행 기록의 `review_only_fast_pass.md`를 읽었다. 저장소 진입 문서, PR 라우터·선택표,
문서·Git 및 GitHub 운영 규칙, 개발 환경과 시각 검증 거버넌스·Visual Sweep 가이드를 적용했다.
기본 작업공간은 clean한 `devel`로 보존했고 별도 검토 branch에서 최신 base의 통합 결과를 검사했다.
기여자의 원 commit을 rebase·amend하지 않았으며 source/fork ref와 base SHA도 재조회했다.

## 기존 해결 여부와 원 기여

#7637의 본문·댓글·연결 이력을 확인했다. 공개 연결 PR은 #8340이며 이슈는 열려 있다.
최신 base의 실제 WASM과 Studio InputHandler에서는 깊이 2·3 모두 서식 대상 0개, 정렬 operation
0개이며 새 경로 API 세 개도 없다. 최신 devel에서도 필요한 수정이다.

기여자는 전체 `cellPath`로 최내곽 문단의 모양을 조회·적용·ID 복원하는 API와 Studio 브리지,
셀 블록·같은 셀 문단 범위·캐럿 대상, 툴바와 CommandHistory를 연결했다. 모양 ID를 사용한
Undo/Redo, 여러 대상의 배치 처리, 무효 경로의 변경 전 거절과 캡션 호환 검사도 포함한다.

#8300의 칸 안 표 만들기는 base/candidate 실제 WASM에서 2×2 표 생성·원문 보존으로 확인했다.
후보에서는 새로 만든 안쪽 표의 문단 정렬도 동작했다. #8138의 구조 명령, 중첩 셀 스타일 전체
지원과 다른 셀을 가로지르는 일반 텍스트 선택은 이번 PR의 해결 범위가 아니다.

## 메인터너 보정과 조판 원칙

public JS에 존재하는 새 API가 선언 파일 두 개에서 빠져, public 경로를 직접 import한 소비자는
세 메서드 모두 TS2551로 실패했다. 검증한 fresh package에서 클래스 메서드
3개·InitOutput 3개·raw WASM export 3개를 가져왔다. 최초 통합 producer는 네 산출물의 해시가
일치한다. 원 PR 제출용 보정은 contributor source 바로 위에 같은 9개 선언만 추가하고 기존
순서를 보존했다. 별도 제출 branch의 직접 소비자 컴파일도 통과했다. 최신 base와 clean merge한
결과는 검증 후보와 모든 runtime/test blob이 같으며, 선언의 서명 집합도 같은 것을 확인했다.
최신 devel 이력이나 다른 PR 변경을 contributor fork에 push하지 않는다. [동기화 기록](../assets/pr8340-maintainer-20261101/binding-sync.json).

| 생산 → 소비 | 검토 결과 |
| --- | --- |
| 깊은 문단 조회 → 공통 ParaShapeMods·새 모양 ID | 경로를 먼저 검증하며 바깥 셀·미선택 셀을 보존한다. |
| 공통 원점 helper → 저장 LineSeg·indent | flat도 공통 결과를 소비하며 길이 1은 기존 캡션 특수 인덱스를 유지한다. |
| 공통 flow 판정 → 최내곽 셀 폭 reflow·vpos flush | 정렬만 바꾸면 재래핑을 하지 않고 간격·여백 변경은 기존 공통 흐름을 소비한다. |
| 바깥 문단 dirty → 측정 캐시·행 높이·중첩 배치 | 바깥 소유 revision으로 측정 subtree를 무효화하며 backend별 좌표 보정을 추가하지 않는다. |

변경된 ParaShape·LineSeg는 공통 `HeightMeasurer::measure_table_impl_with_storage`,
`layout/table_layout.rs`의 행 높이와 `table_cell_content.rs`의 실제 배치가 소비한다.
컷·rowspan 소유 유닛을 결정하는 알고리즘은 바꾸지 않았다. 기존 RowBreak 원점을 다루는
Native 집중 검사와 같은 줄/폭 부족에 따른 다음 줄 관계를 확인했다. 시각 판정 범위는 두 원본의
20개 1쪽 상태이며 다페이지 분할 개선을 주장하지 않는다.

원본 경로와 공용 참조도 추적했다. `issue_7419_no_ls_tac_row_growth.rs`의 관련 검사는 이미
소유·포함·순서 관계와 독립 Print 기반 ±2px 검사다. 현재 두 원본은 양쪽 raw 100%와 직접 판독을
충족하므로 정확한 기존 기대값을 유지했다. 테스트·baseline·조판 상수를 바꾸지 않았다.

## 검증 결과

| 검사 | 실제 결과 |
| --- | --- |
| 보정 code head Full CI | [41345743886](https://github.com/edwardkim/rhwp/actions/runs/41345743886), 정확한 `d073ed921b5d…`, success |
| 보정 head 별도 검사 | [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/41345743856)·[Render Diff](https://github.com/edwardkim/rhwp/actions/runs/41345743870)·[Adapter](https://github.com/edwardkim/rhwp/actions/runs/41345743857)·[Proptest](https://github.com/edwardkim/rhwp/actions/runs/41345743897), 모두 success |
| 보정 head 정책 감사 | [41348084055](https://github.com/edwardkim/rhwp/actions/runs/41348084055), `CI Impact Policy` success, check-runs 39건의 실패·미완료 0 |
| 원 head Full CI | [40631025065](https://github.com/edwardkim/rhwp/actions/runs/40631025065), attempt 2, 정확한 `806e151776e6…`, success |
| 같은 원 head의 별도 검사 | CodeQL·Render Diff·Adapter·Proptest·정책 감사 success |
| 최신 base 병합·producer | clean merge, fresh Native Skia·release-test와 최적화 WASM 빌드 성공 |
| Native 집중 회귀 | 최소 글꼴 환경 10 PASS, 선택 밖 319건 filtered |
| 실제 WASM 명령 | 깊이 2·3, 셀 블록·같은 셀 범위·캐럿, 제외 셀·무효 경로·캡션, Undo/Redo·HWP/HWPX PASS |
| Studio 전체 | 일반 및 최소 글꼴 환경 각각 2,713 PASS·기존 2 skip·실패 0 |
| 추가 집중 검사 | 셀 블록 20 PASS, 라우팅·히스토리 32 PASS |
| TypeScript·소비자·production build | 모두 PASS |
| 실제 브라우저 | 정식 파일 열기, 실제 안쪽 깊이 2, 도구 모음 정렬·키보드 Undo/Redo·F5 전체 두 칸 정렬·미선택 표 보존 PASS |
| 새 독립 Print | 편집을 유지한 API 저장본 14개·실제 Studio 저장본 2개, 한컴 2024 PrintMethod 0 |
| 전쪽 시각 검증 | 20개 상태 × Native/fresh WASM, 전쪽 최저 raw 각각 100.00%, 모두 1/1쪽, 90% 미만·누락 0 |
| base 대비 | 2%p 초과 하락 0, 같은 입력의 base/candidate PNG 바이트도 모든 경로에서 동일 |
| 편집 중 → 정확한 저장본 | 16개 상태의 글 순서·수량·쪽수와 TextRun bbox 변화 0 |

검토 재사용 규칙 3.2.2/4.3.0에 따라 이미 성공한 원 Rust code head의 전체 Rust·Native 광범위
회귀는 중복 실행하지 않았다. 메인터너 보정은 생성된 TypeScript 선언에 한정되며 Rust source/test,
fixture 기대값·workflow·baseline을 바꾸지 않았다. Runtime source는 [최종 제출 후보의 merge-tree 대조](../assets/pr8340-maintainer-20261101/submission-candidate.json)에서 검증한 merge ref와 동일하다.
로컬 결과와 원 head CI를 새 후보의 성공으로 대신하지 않고, 보정 head를 실제 push해 새 Full CI의 성공을 확인했다.

회귀 환경에는 저장소의 라이선스 확인 기본 fallback 5개만 노출했다. 개인·시스템 글꼴과 fidelity용
HCRBatang 프로그램을 격리한 [실제 노출 목록](../assets/pr8340-maintainer-20261101/font-isolation.json)을 보존했다.
인쇄 비교는 별도로 HCRBatang 1.980, compat 2024, Print, 96 DPI를 사용했다.

## 시각 판독과 증적

[정식 입력·PDF·전쪽 TSV·producer·명령·해시](../assets/pr8340-maintainer-20261101/README.md)에
실행 파일과 commit의 대응을 기록했다. 원 PR의 원상 복원본 두 개도 다시 비교했다. 16개 새 Print는
실제 편집 저장본의 출력이며 원본 PDF만으로 편집 상태를 통과시키지 않았다.

[오른쪽 정렬](../assets/pr8340-maintainer-20261101/images/zoom-nested-same-line-right-hwpx.png),
[앞 간격·줄간격](../assets/pr8340-maintainer-20261101/images/zoom-nested-width-wrap-before-line-hwpx.png),
[여백·내어쓰기](../assets/pr8340-maintainer-20261101/images/zoom-nested-width-wrap-indent-hwpx.png)를
같은 좌표·배율로 확대해 독립 PDF와 Native/WASM의 줄 시작·끝, 줄바꿈·기준선·간격, 두 중첩 표의
포함·순서와 주변 문단을 직접 읽었다. 수식은 없고 내용 누락·중복·추가 겹침은 관찰하지 않았다.
RGB/안티앨리어싱 차이는 남으며 2px 실루엣 100%를 RGB 전체 동일로 해석하지 않는다.

일반 RGB pixel match는 대표 쪽의 Native 98.61955–98.76188%, WASM 98.90534–98.92869%다.
잉크 영역 proxy는 각각 27.20923–34.83013%, 39.37962–42.50781%이며, raw 실루엣 gate와
구분한다. [일반 시각 보조값 원본](../assets/pr8340-maintainer-20261101/normal-review-metrics.json)과
같은 좌표의 직접 판독을 함께 기록했다.

## 제출 전 점검과 원격 후속 처리

최종 제출 후보는 원 source의 자손이며 메인터너 선언·증적 두 commit만 추가했다. 다섯 원 기여
commit을 모두 보존했다. 대상 전체의 LFS filter와 pending object는 없고, hook을 유지한
`GIT_LFS_SKIP_PUSH=1 git push --dry-run`은 원 fork branch의 fast-forward 가능을 확인했다.
원격 ref를 변경하지 않은 사전 점검이다.

원 PR source branch에 검증 후보를 push하고 해당 새 head의 Full CI·필수 검사를 확인한다.
그 성공 뒤에만 이 review·오늘할일을 별도 trailing commit으로 기록하고 최신 상태를 재확인한다.
작업지시자 승인 범위에서 정상 merge와 contributor 감사·검증 결과 댓글을 진행한다.
원 기여 commit과 contributor fork branch를 보존한다. #7637은 Refs이며 #8138은 별도 이슈다.


## Merge 후 contributor PR comment 계획

최종 head의 required CI·self-review와 정상 merge를 확인한 후 한국어 존댓말로 게시한다.
실제 merge SHA와 code/trailing CI URL, devel에 존재하는 review·증적 direct link를 포함한다.
대표는 same-line-right-hwpx 1쪽 Native/WASM 각각 후보 1쪽, flagged 0/1이다. raw 실루엣은
양쪽 100.00%, pixel match는 98.76188%/98.92588%, ink proxy는 34.83013%/42.50781%다.
같은 좌표 확대에서 문단 포함·순서·정렬·간격을 확인했고 내용 누락·중복·추가 겹침은 없었다.
이 보조값은 사람의 판정 정확도가 아니라는 설명과 Native/WASM review PNG를 표시한다.
문서 비교 방법은 Visual Sweep 정본에 연결하고 #7637·#8138의 자동 종료를 주장하지 않는다.

### 댓글 초안

CI·merge 확인 후 실제 최종 head와 merge SHA를 붙여 게시한다.

> rhwp 첫 기여를 보내 주셔서 감사합니다. 중첩 표 안 문단의 정렬·간격과 Undo/Redo를
> 전체 셀 경로로 연결해 주신 구현을 검토했습니다.
>
> 메인터너 쪽에서는 공개 TypeScript 선언에 빠진 세 API를 9줄로 보완하고 편집 저장본의
> 독립 한컴 Print 검증 자료를 추가했습니다. 원 기여 commit은 모두 보존했습니다.
> Studio 전체 2,713건, 실제 브라우저 정렬·Undo/Redo·셀 블록 조작을 확인했고, 20개 상태의
> Native/fresh WASM 전쪽 raw 실루엣 최저값은 각각 100.00%였습니다.
>
> #7637의 문단 서식 범위로 수용했으며 #8138의 행·열 등 구조 편집은 별도 범위로 남습니다.
> contributor fork branch는 유지합니다. 첫 기여를 환영하며 다음 기여도 기다리겠습니다.


## Self-review

- 새 public 선언 9개를 fresh binding의 class·InitOutput·raw export와 대조했다. 원 PR의 JS에도
  실제 세 메서드가 존재하며 기존 서명을 삭제하지 않는다.
- 전체 셀 경로가 조회·적용·ID 복원과 Studio toolbar·셀 블록·범위·캐럿까지 이어진다.
  길이 1의 캡션 특수 인덱스와 기존 평면 셀 처리를 보존한다.
- 경로 검증을 모양 표 변경 전에 수행한다. 공통 LineSeg 원점·최내곽 reflow·바깥 revision
  무효화와 저장 raw_stream 무효화를 확인했다. passthrough 예외는 실제 무효화 함수 위임에
  한정돼 있으며 실제 WASM HWP/HWPX 재열기 증거가 있다.
- 합성 WASM driver는 속성·소유·히스토리·저장 계약 검사다. 화면 픽셀 일치 증거와 구분했으며
  실제 시각 판정은 별도의 20개 공개 문서 상태와 독립 Print 전쪽 비교·확대 판독으로 확인했다.
- 추가 차단 결함은 발견하지 않았다. 보정 code head의 실제 Full CI·CodeQL·Render Diff·Adapter·
  Proptest와 정책 감사를 확인했다. 문서 trailing head의 필수 검사·실제 재사용 판정은 push 후
  별도로 확인하며, 모두 성공한 뒤 해당 head에 승인 리뷰와 정상 merge를 진행한다.
  테스트 기대값·workflow·baseline 보정은 없다.


## 승인과 기록 시점

작업지시자는 2026-11-02에 원 PR push → 새 head CI → self-review·정상 merge → 감사 댓글을
명시 승인했다. 원 fork의 첫 기여자 실행 승인은 `/approve`로 처리했으며 수동 `/rerun`·취소·
workflow 변경은 하지 않았다. 제품 run의 attempt 2는 실행 승인 후의 실제 실행으로 구분한다.
`CI preflight`의 실제 분류는 `full`, 사유는 `fail-closed:wasm-contract`였다.
Native Skia, lint, Frontend package gates, 아카이브 A–E와 각 회귀 실행·Build & Test가 모두 성공했다.

이 기록과 [오늘할일](../../orders/20261102.md)은 code head의 CI 성공 뒤 원 source에 잇는 문서
전용 trailing commit이다. 병합 전 검증 시점의 스냅샷이며 실제 merge SHA·최종 head 검사 URL·
후속 처리 확정값은 원 PR의 메인터너 댓글에 남긴다. #7637·#8138의 close는 수행하지 않는다.
