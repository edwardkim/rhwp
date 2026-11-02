# PR #8309 리뷰 — 글상자 HWP 세로 정렬 저장

## 최종 판정

**승인·원 PR 정상 merge 완료.** 원 기여 commit과 메인터너 보정을 함께 통합했다.

보정 전 검증 base에서는 기본 Top·편집 Bottom·재편집 Top이 HWP 저장 후 Center로 바뀌었다.
공통 writer를 보정하고 수정 전 3/3 FAIL → 수정 후 3/3 PASS, 실제 저장·재열기·독립 Print·
Native/fresh WASM·Studio 검증을 완료했다. 새 제출 head의 GitHub CI·정책 감사·최종 충돌 확인 뒤 정상 merge를
완료했다. 원 head의 과거 녹색 CI를 보정 후보의 성공으로 표시하지 않는다.

## 접수와 승인 범위

| 항목 | 값 |
| --- | --- |
| 원 PR | [#8309](https://github.com/edwardkim/rhwp/pull/8309), `physwkim`, CONTRIBUTOR, Merged (접수 시 Open·non-draft) |
| 원 source | `physwkim/rhwp:fix/textbox-list-attr-valign` |
| 원 head | `a9836a959015e9f4709d1b659994b7316f812289` |
| 실제 빌드·검증 base | `0c1e5a6fc20f4ac3e950b7377f831928ad52f5e0` |
| 최초 검증 base / 공통 조상 | `c88657d9c30b57d8481de40f74fd76fd3e2a0af7` / `faa885e8b54ffc7cdf4196cd43a3103f362499a2` |
| 제품 검증 source | `c4686b913cb13ed7a161f941b8e6eadb2be37171` |
| 접수 reviewer | `edwardkim`, requested-reviewers API에서 확인 |
| 검토 branch | `review/pr-8309-textbox-vertical-align` |
| worktree | `/home/edward/mygithub/rhwp-worktrees/pr-8309-textbox-vertical-align` |
| 연결 이슈 | 원 PR은 관련 이슈 없음으로 제출됨. 동일 저장 증상의 별도 공개 해결 PR은 확인하지 못함 |
| 원 head CI | [CI](https://github.com/edwardkim/rhwp/actions/runs/40461458933)·[CodeQL](https://github.com/edwardkim/rhwp/actions/runs/40461459296)·[Adapter](https://github.com/edwardkim/rhwp/actions/runs/40461459210)·[Proptest](https://github.com/edwardkim/rhwp/actions/runs/40461459120)·정책 감사 success |

기본 경로는 `maintainer_general`, 보조 경로는 `intake_and_review`, `local_validation`,
`visual_fixture_evidence`, `post_merge`다. 기존 CONTRIBUTOR이므로 첫 기여자 경로는 추가하지
않았다. 작업지시자가 승인한 범위는 충돌 해소·메인터너 보정·필수 검증의 로컬 단위다.
이후 원격 처리·머지·후속 처리까지 별도 승인받았다. 실제 원격 수행 결과는 후속 절에 기록한다. 기본 작업공간의
기존 devel·사용자 Studio 7700·배포용 WASM과 다른 작업의 worktree는 보존했다.

## 원인과 선행 해결 경로

생성 경로 `src/document_core/commands/object_ops/shape.rs`는 raw `list_attr=0x20`과 semantic
`vertical_align=Top`을 동시에 만들었다. 속성 setter는 semantic 값을 바꾸지만 HWP writer는
오래된 raw를 그대로 썼다. 파서는 그 비트에서 정렬을 복원하고 `shape_layout`은 복원한
semantic 정렬로 실제 자식 텍스트를 배치한다. HWPX는 semantic 값을 저장하므로 보존됐다.

| 최신 base의 실제 WASM 저장 action | 편집 중 | HWP 재열기 | HWPX 재열기 |
| --- | --- | --- | --- |
| 새 글상자 | Top | Center | Top |
| Center로 편집 | Center | Center | Center |
| Bottom으로 편집 | Bottom | Center | Bottom |
| Top으로 재편집 | Top | Center | Top |

[최신 base probe](../assets/pr8309-latestbase-20261102/base-probe.json)는 최적화 WASM
`4ac5f4022146819136965192e81491ca86f42c6360e2f3d8b35c1c70f0699e70`의 실제 API·바이트 재열기다.
빈 글상자 진단을 실제 글자의 시각 증거로 대신하지 않았다.

[PR #8134](https://github.com/edwardkim/rhwp/pull/8134) → 통합
[#8393](https://github.com/edwardkim/rhwp/pull/8393), merge
`16797c520cb46fd3b1c1ea9072cb7fb5f0aa79b6`는 저장 LineSeg가 없는 글상자의 내용 높이·정렬
보정이다. [선행 검토](pr_8134_review.md)·원본·독립 Print를 확인했다. 저장 raw 비트
생성과 다른 계층이며 이번 최신 base에서도 문제가 남아 있었다. #8122의 관계형 회귀와 전쪽
Native/WASM 출력으로 그 기능 보존을 확인했다.

[PR #8360](https://github.com/edwardkim/rhwp/pull/8360) / #8174, merge
`f125c735c7d2d44eaa44b78c70465bc89af45fb1`은 저장을 Storage·공통 payload writer로 분리했다.
원 PR의 충돌은 `src/serializer/control.rs` 한 곳이었다. 실제 경로
`try_serialize_text_box_if_present` → `ControlPayload::TextBox` → `write_text_box_header`를
유지하며 원 기여의 비트 생성을 이식했다. raw의 비정렬 비트·추가 바이트·측정과 쓰기의 같은
writer·Storage 한도·I/O `?` 전달을 보존했다. 글상자의 내부 offset 4 bit 5–6과 셀/캡션의
u32 offset 2 bit 21–22를 구분한다. 새 사양 정오표를 주장하지 않는다. HML의 raw뿐 아니라
semantic 정렬·추가 바이트 preflight도 그대로다.

## commit과 최신 base 기능 보존

| 단계 | commit·내용 |
| --- | --- |
| 최초 호환 merge | `dda384eadc4148ff4c939efdf1a184aacd0e59fa`, 최신 Storage writer 보존 |
| semantic writer 보정 | `01e9f4ff012acceaa477c7f4f0a085b86747ef42`, 정렬 두 비트만 생성 |
| 이전 base 증적 동결 | `67501ea78bd34c7f6b1f0dd9c4bd99bdaae9282c`, c886 기준 검증만 기록 |
| 새 devel 호환 merge | `c4686b913cb13ed7a161f941b8e6eadb2be37171`, #8438/#8445와 clean merge |
| 최신 증적 head | `458ad11cea2a0e89cf26554d62557da37335aa3e` |

검증 도중 #8438과 #8445가 반영됐다. 최초 base 확인 assertion 실패 후 shell이 commit을
계속한 점을 바로 확인하고 후속 명령은 실패 즉시 중단하도록 보정했다. 해당 commit은 실제
c886 검증 결과의 동결이며 최신 base 결과로 표시하지 않았다. 이후 기준/후보를 모두 새로
빌드하고 전체 필수 검증을 재실행했다. 원 기여 commit은 rebase·amend하지 않았다.

새 base와 제품 차이는 두 파일의 9줄 추가·2줄 삭제다. 새 `0a2e306754` (#8399)의 도형 밴드
기하·저장 줄 갱신은 생성/속성 setter에서 유지했다. `1903cdf1d2` (#8335)의 번호 범주
bit 26–27은 별도 CommonObjAttr payload이며 그 enum/mask를 되돌리지 않았다. #8302 HWPX
lineage·grouped CTRL_DATA·기존 Storage 계약도 유지했다. 직접 맞닿는 새 기능은 Native 집중
15건과 A·D·E·P HWP/HWPX 전쪽 출력으로 재검증했다. 이름이 같은 helper나 컴파일 성공만으로
보존을 판정하지 않았다.

이 변경은 저장 경계의 원인 보정이며 측정/paint/페이지 분할·이어받기 알고리즘을 변경하지
않는다. 문서 ID·수치 휴리스틱·좌표 clamp·backend별 보정을 추가하지 않았다. 원 회귀
assertion/helper는 그대로이며 첫머리에 API/저장 계약과 생성 방식·시각 측정 비해당 이유·
별도 문서 시각 증거를 기록했다. 절대 화면 px 핀을 추가하지 않았고 #8122의 관계 검사도 유지했다.

## 실제 검증 결과

[최신 정식 증적](../assets/pr8309-latestbase-20261102/README.md)에 입력·PDF·원본 TSV·PNG·
일반 pixel/proxy·직접 판독·실행 파일/글꼴 해시와 재현 명령을 보존했다. 이전 c886 결과는
[이전 증적](../assets/pr8309-maintainer-20261102/README.md)으로 분리했다.

| 입력 (각 1/1쪽) | Native base → 후보 | WASM base → 후보 |
| --- | ---: | ---: |
| 기준 HWPX Top·Center·Bottom, 3개 | 각 100.00% → 100.00% | 각 100.00% → 100.00% |
| 실제 저장 HWP Top·Center·Bottom, 3개 | 각 100.00% → 100.00% | 각 100.00% → 100.00% |
| 기존 #8122, 1개 | 99.55% → 99.55% | 99.42% → 99.42% |
| #8399 A·D·E HWP/HWPX, 6개 | 각 100.00% → 100.00% | 각 100.00% → 100.00% |
| #8399 P HWP/HWPX, 2개 | 각 99.98% → 99.98% | 각 99.98% → 99.98% |

96dpi·print·raw 2px 실루엣이며 기존 #8122만 compat 2022, 나머지는 compat 2024다.
고유 15개 입력·4조합의 전쪽 TSV 60개, 미달·누락·제외 쪽 0, 독립 PDF와 쪽수 일치다.
폰트 예외를 쓰지 않았다. 동일 입력 전후 PNG는 Native 15쌍·WASM 15쌍 모두 바이트가 같다.
저장 개선은 수정 전 FAIL/수정 후 PASS와 실제 저장 HWP 자체의 Print로 별도 확인했다.

새 합성 입력은 한컴 생성 원본이라고 표시하지 않았다. 기준 HWPX와 후보 HWP의 바이트가
이전 독립 Print 입력과 같아 해시 검증 후 원 Print를 재사용했다. 새 Print 실행으로 표시하지
않고 원 job·13.0.0.3901·PrintMethod 0·one-up 방식·font scope·입력/PDF 해시를 연결했다.
마지막 편집 Top은 기본 Top과 두 형식 모두 바이트가 같다. 재현 스크립트 별도 재실행도 저장
입력 8개와 바이트가 같았다.

같은 2배 배율로 정렬 3종의 여백·기준선·문자 간격·테두리·내용 보존과 A·D·E·P의 줄 안
그림 앞뒤 글자·어울림 외곽·줄바꿈·뒤 문단을 직접 판독했다. 잘림·누락·중복·새 겹침은 없다.
수식이 없는 입력이다. 기존 #8122의 획 굵기·래스터 차이는 base부터 같으며 이번 수정으로
늘지 않았다. raw 100%를 완전 pixel 일치로 표시하지 않고 일반 pixel/proxy는 별도로 기록했다.

최적화 fresh 후보 WASM SHA-256은
`4734ed402f46493c92ccc107a6a29d78b2e3b66cee22a39b0b237ef38832f255`다. 실제 Studio 7706의
초기 로딩 응답 바이트도 CDP에서 같은 해시임을 확인했다. 실제 속성 변경·bridge export·파일
입력 재열기 8회에서 글꼴 창 응답·로딩/busy 종료·Canvas·글자·정렬·1쪽 상태를 확인했다.
page error는 0이다. 해당 대체 글꼴 UI 스크린샷을 Print 시각 측정으로 사용하지 않았다.

| 최신 로컬 필수 gate | 결과 |
| --- | --- |
| fmt·Native/WASM32/workspace all-target Clippy·workspace build·base 지정 manifest | 모두 PASS |
| 전체 nextest release-test | 12,369 PASS, 기존 skip 45개, slow 8개, 실패 0 |
| Native Skia lib | 4,040 PASS (본체 3,858 + workspace 182), 기존 ignored 12개 |
| Native missing picture·direct PDF export | 2 + 22 PASS |
| 원 정렬·기존 보존 집중 검사 | 18 PASS |
| 새 base #8335·#8399·#8302 보존 집중 검사 | 4 + 8 + 3 PASS, Native 집중 총 57개 |
| TypeScript·Studio npm·제품 build | PASS, npm 2,716 PASS·기존 skip 2개 |

Rust 회귀는 시스템/개인/이전·현재 fidelity 폰트 경로를 bubblewrap로 숨기고 공개 fallback
5개만 노출했다. 시각 비교는 명시적 fidelity 폰트를 쓰며 두 환경을 구분했다. nextest 0.9.137의
권고 버전·미사용 CI JUnit profile 경고는 기록하되 실제 실행은 정상 완료됐다. 추가 ignore·
timeout·기대값·source assertion 변경으로 실패를 회피하지 않았다. 파생 suite는 제출에서 제외한다.

## 승인 범위와 통합 절차

작업지시자는 2026-11-02에 원 fork push·본문 증적 반영·새 head CI·self-review·원 PR merge와
감사 comment·운영 기록·정리를 승인했다. 제출 직전 #8447의 문서·신규 기준 자료만 담은 devel
`04338dc242ff1bd9fab01290b0d90bafcddc5aab`을 `f72fba39d5651673a6da74a10bc07406a358d269`로 병합했다. 제품·테스트·빌드·workflow와 실측 입력은 같아
기존 전체 PASS를 유지했다. [최종 base 보존 기록](../assets/pr8309-latestbase-20261102/final-base-sync.json)을 남겼다.

소형 공통 writer 보정이므로 별도 implementation 문서 대신 이 문서의 commit 순서와 다음
절차를 사용한다. 로컬 보정·검증은 완료했다. 원 fork의 원 branch fast-forward push·본문 증적 보강 → 새 제출 head CI → self-review → 원 PR
merge를 승인받아 수행했다. 아래 최종 통합 결과에 exact head와 실제 성공·merge SHA를 기록했다.
보정 시작 시점에는 원격 head를 보존했으며 승인 뒤 추가 commit을 fast-forward로 제출했다.

merge 뒤에는 archive review·오늘할일에 실제 merge SHA/CI를 기록하는 문서 전용 운영 반영,
devel 동기화·기여자 comment·검토 전용 branch/worktree·root output·사용 종료 debug 정리를
순서대로 수행한다. 다른 작업과 contributor fork branch는 보존한다. 원격 승인과 실제 수행 결과는 아래 최종 통합 결과에 기록했다.

## Merge 후 contributor PR comment 계획

정본은 [Visual Sweep](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/verification/visual_sweep_guide.md#github-merge-comment)이다.
comment에는 고유 입력 15개(각 1쪽)의 두 경로·최저 raw 99.55%/99.42%, 정렬 저장본 6개 raw
100%, 어울림 문서 8개 최저 99.98%, 미달·누락 0과 직접 판독 결과를 적는다. raw와 일반
pixel/proxy를 구분한다. 대표 `saved_bottom` 1쪽은 pixel Native 99.88517%·WASM 99.91458%,
proxy Native 59.35638%·WASM 68.37074%다. 이는 사람의 판정 정확도가 아니다.

대표 이미지 경로는 `mydocs/pr/assets/pr8309-latestbase-20261102/`의
`saved_bottom-p001-native-review.png`, `saved_bottom-p001-wasm-review.png`다.
실제 merge 뒤 `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/`로 고정하고
asset이 devel에 있는지 확인한 뒤 승인된 범위에서 UTF-8 `--body-file`로 게시한다. API에서
한글·URL·수치·게시 결과를 재확인한다. PR 본문에는 그보다 앞서 최종 **fork head SHA**의
Native/WASM review·standalone overlay와 전쪽 TSV 원본 링크를 넣는다.

## 승인 뒤 제출

원격 처리 범위는 2026-11-02에 승인받았다. 최종 제출 후보는 `ffe44575528367fc9df864abe03f21785a255015`다.
#8447의 증적 전용 base를 포함했고 제품·테스트는 기존 전체 검증과 같다. 원 contributor head를
조상으로 보존한 fast-forward push와 본문 증적 보강을 진행했다. 이후 새 head CI·self-review·
최종 충돌 확인을 완료하고 정상 merge했다.

## 원격 반영과 self-review 진행

2026-11-02에 contributor 원 branch가 `ffe44575528367fc9df864abe03f21785a255015`로
fast-forward됐고 API에서 확인했다. 원 PR 본문은 기존 내용을 유지한 채 해당 fork SHA의
TSV 60개·이미지 24개·입력/PDF와 해시 증적을 보강했다. UTF-8 본문 일치를 API로 재확인했다.
첫 전송은 LFS 잠금 확인에서, HTTPS 전송은 workflow OAuth scope에서 거부됐으며 원격
head가 그대로임을 확인했다. LFS 사전 판독 결과 변경 14,166개 경로에 LFS 추적은 0개였고
신규 object도 없었다. 정본 9.3.0에 따라 LFS 전송 단계만 생략한 SSH 전송을 완료했다.
전역 hook·권한·인증 scope 설정은 바꾸지 않았다.

제품 두 파일과 원 3개 assertion을 다시 대조했다. 정렬의 semantic/raw 불일치를 공통 writer에서
해결하고 나머지 raw·저장 payload 크기·한도·추가 바이트를 보존한다. 추가 할당·순회 없이
같은 4바이트를 쓰므로 이번 변경으로 문서 크기에 비례하는 새 비용은 없다.
코드·회귀·시각 증적에서 추가 차단 사항은 없었으며 새 head CI의 성공은 아래 최종 통합 결과에 기록했다.

## CI 인프라 지연 — lint checkout

동일 head `ffe44575528367fc9df864abe03f21785a255015`의 [lint 작업](https://github.com/edwardkim/rhwp/actions/runs/41382006599/job/125075980884)은
2026-11-02T03:20:04Z부터 저장소 fetch에서 머무르다 03:40:01Z에 20분 job timeout으로
취소됐다. 실제 lint·Rust/WASM 검사는 시작하지 못했다. workflow의 기존 timeout은 20분이며
수동 취소나 workflow·기대값 변경을 하지 않았다. 최종 로그의 fetch 대상 merge는
`18c39e7acf1620339658a4465c642867b24173bf`다.

Frontend·Adapter·Proptest·Archive A/C/D는 성공했으며 나머지 기존 실행은 계속 관찰한다.
전체 실행이 끝난 뒤 동일 head의 lint와 종속 집계만 한 번 재실행하는 범위를 별도로 요청했고 승인받았다.
기존 성공한 검사는 유지하며 이전에 승인된 merge·후속 처리 범위는 변하지 않는다.

작업지시자는 lint와 종속 집계의 동일 head 1회 재실행을 승인했다. 첫 실행의 Native·Archive B까지
성공한 뒤 2026-11-02T03:49:27Z에 정확한 job 재실행 endpoint를 한 번 호출했다. attempt 2의
새 lint job은 `125084196825`다. attempt 1/2의 실제 시작·완료 시각을 대조해 Frontend·Native·
Archive A–E의 성공 실행은 재실행되지 않았음을 확인했다. skipped job의 복사 metadata에 새
시각이 생긴 것은 실제 실행과 구분했다. 새 checkout과 실제 lint·최종 집계는 모두 성공했다.

## 최종 통합 결과

- 최종 head: `ffe44575528367fc9df864abe03f21785a255015`.
- 원 PR merge: `b4d2d8f7d80836d5046f0ab3ea03d63f477f631d`, 2026-11-02T04:09:43Z.
- [Self-review](https://github.com/edwardkim/rhwp/pull/8309#pullrequestreview-5731999600)는 정확한 제출 head를 승인했다.
- [CI](https://github.com/edwardkim/rhwp/actions/runs/41382006599): success, attempt 2.
- [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/41382006596): success, attempt 1.
- [Adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/41382006661): success, attempt 1.
- [Proptest roundtrip](https://github.com/edwardkim/rhwp/actions/runs/41382006689): success, attempt 1.
- CI Impact Policy 최종 감사 success와 최신 base 충돌 simulation을 확인했다.
- lint checkout timeout 뒤 승인받은 부분 재실행 한 번으로 실제 lint와 최종 Build & Test가 통과했다.
  다른 Frontend·Native·Archive A–E의 성공 실행은 실제 시각 대조로 보존됨을 확인했다.
- Duration metadata: B/C/D/E의 원 attempt 1 실측을 검증해 duration 데이터 branch 갱신 성공 (latest CI attempt 2, ready=true).
- 연결 issue는 없어 close 대상이 없다. contributor fork branch는 보존한다.
- 후속 문서는 maintainer 운영 기록으로 직접 반영한다. 그 뒤 devel sync·기여자 comment·검토
  worktree/branch와 root output·사용 종료 debug 정리를 수행한다. 정리 로그는 output 밖에 남긴다.

[원격 통합 증적](../assets/pr8309-latestbase-20261102/remote-integration.json)에 실행 결과와 review·merge·부분 재실행 보존 근거를 기록했다.
