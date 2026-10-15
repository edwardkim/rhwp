# planet6897 누적 통합 리뷰 — 2026-10-15

## 최종 판정

**메인터너 보정 후 수용 가능 — 명시한 통합 범위의 로컬 검증 완료.**
검증 code head `39a634e547621c32ae6f9ba8c1694d35c827078b`, 기준 `463c48cc228170b576c832591b76826116e86a49`. [통합 PR #7923](https://github.com/edwardkim/rhwp/pull/7923)의 code candidate `cbc5815631d6c221dc6ec6e0f03565859cb1c346`에서 전체 CI·정책 체크 성공과 CLEAN을 확인했다. 문서 trailing head 체크와 실제 merge·후속 처리가 남아 있다. 사용자의 사전 승인으로 개별 archive review와 오늘할일을 이번 코드 PR에 함께 포함한다. 별도 기록 PR을 만들지 않는다.

## #7926 추가 정책 보정

사용자가 [fast-pass 판정 버그 #7926](https://github.com/edwardkim/rhwp/issues/7926) 등록·개선을 요청했다. 고정된 workflow 실행 SHA와 가변 연결 PR head를 구별하도록 helper를 보완했고, 새5회귀 수정 전 FAIL/후 PASS 및 Node354·Python186PASS와 실제 API 재생 통과를 확인했다. [단계 기록](../../working/task_m100_7926_stage1.md)을 따른다. 앞서 기록한 `cbc581563` CI는 이 추가 수정 전 결과다. 새 정책 수정 head의 전체 CI 성공을 다시 확인하기 전에는 병합하지 않는다. renderer/Rust/폰트 바이트는 그대로여서 시각·로컬 Rust 검증을 반복하지 않았다.

## 통합 범위와 원 PR별 판정

| 원 PR | source SHA | 개별 review |
| --- | --- | --- |
| [#7669](https://github.com/edwardkim/rhwp/pull/7669) | `63c609474c768fe57542a15ebcb963f2c56f11c2` | [review](pr_7669_review.md) |
| [#7672](https://github.com/edwardkim/rhwp/pull/7672) | `10b40c331fbeee2c005c61871ce80ade504c7147` | [review](pr_7672_review.md) |
| [#7692](https://github.com/edwardkim/rhwp/pull/7692) | `0c1e33d2a0f536cd516a1020ee17dffe346ec20d` | [review](pr_7692_review.md) |
| [#7701](https://github.com/edwardkim/rhwp/pull/7701) | `fd2001f801d45f191e90c99b41c14c5f7a6828dc` | [review](pr_7701_review.md) |
| [#7805](https://github.com/edwardkim/rhwp/pull/7805) | `0418825ab2e8603685964bcbd23cffb553760b0a` | [review](pr_7805_review.md) |
| [#7814](https://github.com/edwardkim/rhwp/pull/7814) | `97aa8880c61a4111a82f1539d0c83f0e9e64ae99` | [review](pr_7814_review.md) |
| [#7815](https://github.com/edwardkim/rhwp/pull/7815) | `6a59bad9e9d762db336aba7e0614085ee39ffb54` | [review](pr_7815_review.md) |
| [#7825](https://github.com/edwardkim/rhwp/pull/7825) | `9bbcd5935a19d3a4402cb89c8bef40eed423b1f3` | [review](pr_7825_review.md) |
| [#7828](https://github.com/edwardkim/rhwp/pull/7828) | `7e841a2cc093a3911cae4e068b25716793ad1fe2` | [review](pr_7828_review.md) |
| [#7836](https://github.com/edwardkim/rhwp/pull/7836) | `0718ddd1985566d526c675075febda47b12921fa` | [review](pr_7836_review.md) |
| [#7838](https://github.com/edwardkim/rhwp/pull/7838) | `63240fbbd6da9ce7cf285401c872af72c8d3e60c` | [review](pr_7838_review.md) |
| [#7859](https://github.com/edwardkim/rhwp/pull/7859) | `400e9497599f133ed8eec967af79d926e251f8b7` | [review](pr_7859_review.md) |
| [#7867](https://github.com/edwardkim/rhwp/pull/7867) | `649768e910712fb93891c9ec92db5067f2024474` | [review](pr_7867_review.md) |
| [#7869](https://github.com/edwardkim/rhwp/pull/7869) | `f8de1fa358b5604f9bb43052ae96578af656ab33` | [review](pr_7869_review.md) |
| [#7878](https://github.com/edwardkim/rhwp/pull/7878) | `24918d7db1e85084a3975f8fea4ec25489a16d3a` | [review](pr_7878_review.md) |
| [#7880](https://github.com/edwardkim/rhwp/pull/7880) | `561327eecf9dd8abdad3ad571008bdb3cd1fd467` | [review](pr_7880_review.md) |
| [#7884](https://github.com/edwardkim/rhwp/pull/7884) | `33def84c5deabccfb3dfc4537d94bfb06d666e1f` | [review](pr_7884_review.md) |
| [#7886](https://github.com/edwardkim/rhwp/pull/7886) | `58a1d3b6522138528d91a53c97b38c0f022814a4` | [review](pr_7886_review.md) |

18개 원 PR의 feature를 누적·동등 반영했다. #7669는 이미 CLOSED이며 나머지17개는 접수 시 OPEN이다. #7672 최신 삭제3건은 현재 브랜치에 동등 반영돼 있다. 원 PR의 첨부·baseline·golden을 모두 새 정상 기준으로 채택한 것은 아니며 이관한 재현 자료와 코드 적용 범위를 구별한다.

## 검증 증거

- [입력·Print·PNG SHA 및 문서별 전쪽 결과](../assets/planet6897_followup_20261011/qualified-print-d80593/README.md)
- 최초 전체 Rust:10,362실행,10,354PASS/8FAIL/45SKIP. 실물 실패6건은 사용자 지정으로 이관했다. 폰트 해시·원장 계약2건을 수정하고 경로 변경 관련55개를 모두 통과했다. 사용자 승인에 따라 초기 전체 결과와 개별 재검증을 합산해 전체 회귀 통과로 처리한다. 새 head에서10,356개를 다시 실행했다고 보고하지 않는다.
- 최종 code head의 fmt·Clippy3경로·workspace 빌드·manifest/tier·Native Skia·문서 테스트 통과. 각 명령/exit와 상세 로그는 ignored output에 보존하고 최종 요약을 PR 본문에 연결한다.
- Studio1,875PASS, Studio·Chrome·Firefox·VSCode 배포 빌드와 폰트/확장 배포 계약 통과. fresh WASM SHA-256 `839e76297ee8ca90c053f1ce46542b96b5c21b4cddde02a415b0975e4903affc`, Studio public 동기화 일치.
- Native/fresh WASM 전21문서181쪽: 전쪽90% 이상·독립 Print 쪽수 일치. 모든 체리픽 입력16종은 핵심 위치의 독립 Print ±2px와 내용·소유·순서 관계 검사를 갖춘다. 해당 고정 쪽은 양쪽98% 이상이다. 양쪽 최솟값100→99→98 우선 기준을 적용했고 중요한1쪽 매개변수/표 경계도 포함했다.
- 16문서의 새56개 Native/Canvas PNG는 이전 확대 판독본과 바이트가 같다. 새 고용4쪽은 독립 Print와 직접 판독했고 task1749·가맹사업의 새 대표 review도 열어 표 종료·뒤 본문·각주를 확인했다. 수식·미세 간격 판독을 수치로 대체하지 않는다.
- 사용자 승인한 소폭 하락: Native #7669 p1 −0.01179%p, fresh WASM ratio_steps p1 −0.05623%p. 90%/98% 조건은 유지한다.

## 추가 보정과 조판 원칙

- 배포 독립 폰트55개의 출처·전문 라이선스·SHA를 등록하고 네 배포물에 동봉했다. Noto 기반 수식 fixture와 수정 옛한글 subset은 RHWP 고유 family를 사용한다. 실제 한컴 글꼴 대체는 명시적 API에서 처리한다.
- 셀 첫 저장 띠는 측정의 `paragraph_before().occupied_before()`와 배치의 `allow_first_stored_lead`가 같은 소유 조건으로 소비한다. ColumnDef 등 배치 상자가 없는 메타데이터는 저장 띠를 제거할 근거가 아니다. 별도 원점이 있는 선형/스냅 셀과 실제 배치 상자 컨트롤은 기존 경로를 유지한다.
- 고용26번 기준선은 독립 Print112.99651px, 보정 전110.44px, 보정 후113.10667px다. 독립 기대값±2px에서 FAIL→PASS이며 잘못된 절대 위치를 허용치 확대로 가리지 않았다. 페이지·문단/표·셀 소속과 포함을 함께 보호한다.
- 새 검사21건은 독립 Print 문자 원점에서 기대값을 정했다. 7095의 공백/가시 끝 글자 대응과 따옴표 뒤 첫 글자 경계를 교정했고 glyph 원점·관계 검사를 유지했다. 임의 문서 ID 조건·숨김·clamp로 출력을 맞추지 않는다.

## 이관과 보존

- [#7774 SQL sample10](../assets/issue7774/sql-sample10-regressions-20261015/README.md): HWP5/HWPX3함수, 입력2개·PDF3분할본·원문2파일. 763쪽과489쪽 내용은 확인됐으며 실패는 단일 런의 선행 공백 검색 전제다.
- [#7922 끝 탭](../assets/issue7922/trailing-tab-regressions-20261015/README.md): 실패3함수, 입력/PDF·원문1파일. 정상 겹침·중간 탭 대조는 유지했다. 과거95.92922%와 현재98% px 완료 범위를 혼동하지 않는다.
- 두 이슈에 원본·PDF SHA, 이전/새 경로와 복원 조건을 보존했다. 활성 render 원장8행을 원문 보존 후 분리하고 정상 parser/serializer/API는 새 경로에서55개 검사로 확인했다. 이관은 새 base90% 미달 입증이나 제품 결함 해결로 기록하지 않는다.
- #7445 표 누락 #7863·#7864·#7922를 보완했다. 현재 연결93개이며7개 여유다. 이슈는 미해결 상태를 유지한다.

## PR #7923 exact-head CI

- code candidate `cbc5815631d6c221dc6ec6e0f03565859cb1c346`의 [CI](https://github.com/edwardkim/rhwp/actions/runs/38734213321)·[CodeQL](https://github.com/edwardkim/rhwp/actions/runs/38734213350)·[Render Diff](https://github.com/edwardkim/rhwp/actions/runs/38734213214)·[Adapter](https://github.com/edwardkim/rhwp/actions/runs/38734213427)·[Proptest](https://github.com/edwardkim/rhwp/actions/runs/38734213384) 및 CI Impact Policy 성공. 실패·대기 없음, MERGEABLE/CLEAN을 재조회했다.
- 네 Rust archive worker와 Build & Test 성공. 집계 로그에서 `archive_total=10162 shard_total=10162`를 확인했다. 이는 CI archive 분할 검증 수이며 로컬 최초10,362실행/집중55재검증과 구별한다.
- 제출 후 실패했던 옛한글 alias mock2곳을 현재 family 이름으로 보정했다. 기대 assert를 유지했고 로컬12단계 및 원격 Render Diff 전체가 통과했다. 제품/Rust/폰트 바이트 변경이 없어 완료한21문서181쪽 시각 증적을 재사용했다.
- 사용자 요청으로 sample10 원본91.68MiB PDF를 현재 head·최종 diff에서 제외했다. 50MiB 이상 PDF는 로컬 보관·PR 제외로 기여/검토 지침을 통일했고 현재 추가/변경 PDF47개 중 미달 없이 전부 상한 미만이다. 기존에 push한 이력은 재작성하지 않았다.
- 개별18개 review·[PR #7923 self-review](pr_7923_review.md)·오늘할일을 같은 PR의 문서-only trailing commit으로 반영한다. 해당 새 head의 실제 fast-pass·required checks 성공 후 merge한다.

## CI·Merge 후 처리

제출 직전 devel head를 확인한다. 같은 head이면 리베이스하지 않는다. 제출 뒤 exact-head CI를 모니터링하고 성공 결과/PR 번호를 같은 PR 기록에 갱신한다. 최신 required checks와 mergeability 확인 후 승인된 merge·post_merge.md 순서로 처리한다. merge SHA·duration 갱신 상태 → 자료/기록의 devel 존재 → devel 동기화 → 이슈/원 PR comment·supersede → 이번 작업의 임시 branch/worktree 정리를 확인한다. 공유 target·기본 작업공간·기여자 fork를 삭제하지 않는다.
