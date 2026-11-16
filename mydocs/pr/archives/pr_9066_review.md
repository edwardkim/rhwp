# PR #9066 셀프 리뷰 — #9042 제목·본문 표 공동 앵커 배치 수용

## 최종 판정

**승인** — 수용 완료. 작성자 edwardkim의 self-review 기록이며 GitHub 자기 승인 review가 아니다. code head `4cf0b66548`의 전체 검사와 문서 head `4d6274f8ad`의 실제 fast-pass·최신 check·MERGEABLE/CLEAN·최신 base 시뮬레이션을 확인한 뒤 사용자 승인 범위에서 merge commit `5f78bb8d0297b499043dc4c45d76996549c0b2ef`로 통합했다(2026-11-14 08:41:04 KST).

## 문제와 변경 결과

원 PR #9042의 같은 문단에 앵커된 제목 표와 본문 표가 가용 영역에 들어가도 3쪽으로 밀리던 문제를 수용한다. 최종 후보는 제목·본문 표를 첫 쪽에 순서대로 놓고, 명시적 쪽 나누기 뒤의 다음 표를 2쪽에 보존한다. Header/Footer·실제 한컴 저장 중첩 TAC·원 합성본의 저장 줄·BodyX 같은 줄 편집과 HWP/HWPX 반복 저장 경계도 함께 보완했다.

## 소스와 승인 범위

- [원 PR #9042](https://github.com/edwardkim/rhwp/pull/9042), puckqook의 첫 rhwp 기여. 원 SHA `3ae4eeb332c3b0edbb3a8fbe334daf2b0a48115b`를 조상으로 보존했다. 원 기여를 이미 적용한 뒤 이력을 연결한 bookkeeping merge 전후 tree가 같음을 증명했다.
- 로컬 필수 검증 제품 `d66d5f05f9efa4d182228494cd970614d479fb7a`; exact 제출 code head `4cf0b665488cb3570af964b23f6c09ab5e92138a`. 그 사이에는 검토 증적·기록만 바뀌었고 source/test/fixture는 같다.
- 최신 확인 base `9e2794796787b46815462fcecd1536ab74ac8528`. 후보에 base를 반복 merge/rebase하지 않고 실제 병합 결과를 추가 검증했다. [추가 병합 검증](../assets/pr9042/final-validation/postapproval-base-merge/README.md).
- fork repository의 push=false만으로 maintainer_can_modify=true인 branch의 쓰기 가능성을 단정하지 않았다. 최신 devel 이력을 fork에 섞지 않는 upstream 수용 PR 경로를 사용자에게 제시해 승인받았다. 원 fork branch는 유지한다.
- 기본 maintainer_general, 보조 first_time_contributor·intake_and_review·local_validation·visual_fixture_evidence·collaborator_self_merge·review_only_fast_pass·post_merge를 적용한다. CI 실행 정의 변경이 없으므로 동일 PR의 문서 trailing 경로다.

## 조판 원칙·코드 재검토

| 항목 | 확인 결과 |
| --- | --- |
| 입력·독립 근거 | 공개 RHWP 합성 입력과 실제 한컴 2020 저장/Print, 원 macOS 한컴 Print를 구분했다. 검증 입력 100개의 commit 바이트 감사와 생성 경로를 보존했다. |
| 원 해결 PR 선행 분석 | #8187 공통 tail 계약과 #8682 보호, 최신 #9054 저장 자원/캐럿 계약을 추적했다. 기대값부터 완화하지 않았다. |
| 측정·배치·paint | 저장 host의 실제 inline 원점과 float 바깥 여백에서 같은 점유 끝점을 계산한다. 닫힌 끝 행의 fit 결과를 format·예약·paint·저장 writeback이 소비한다. |
| 중첩·분할 경계 | 실제 저장 한 줄/줄 사다리·완전한 자식 상자와 여백으로 내용 하한을 증명한다. span·rewind·폭 미입증·실제 수직 재조판은 기존 경로를 유지한다. |
| 저장 정보 수용 | 실제 BodyText digest가 맞는 RHWP 관리 캐시의 단순 한 줄 셀만 현재 글자 크기와 문단 전진을 함께 대조한다. digest는 소유 표지이며 신뢰성 인증으로 간주하지 않는다. 정상 큰 strut와 의도적 빈 문단을 보존한다. |
| 편집·저장·원본 IR | readonly 렌더·저장은 사본을 사용한다. 유효한 셀 편집 대상 확인 뒤 같은 메트릭을 인계하며 cache/dirty 상태를 갱신한다. 실제 저장 예상 IR도 같은 변환·자원 pruning 사본을 소비하고 엄격 diff를 유지한다. |
| 일반성·보안 | 문서 ID·목표 px·임의 상수·좌표 clamp·출력 숨김이 없다. 새 소유 digest 비교는 길이·형식을 검사하고 할당 없이 수행한다. Storage 경로는 같은 budgeted 복제 API를 사용한다. |
| 회귀·직접 판독 | 원 사례/반례의 최종 출력 소속·순서·보존·포함/겹침을 검증했다. 같은 범위의 원본·독립 PDF·Native/fresh WASM과 실제 384 DPI를 직접 대조했다. baseline 완화·예외·검사 이관은 없다. |

실제 소비 위치와 적용·비적용 경계는 [원 PR 보완 기록](pr_9042_review_impl.md), 최종 수용 판단은 [원 PR 검토](pr_9042_review.md)에 연결한다. 이 재검토에서 추가 제품 보정이 필요한 새 결함은 발견하지 않았다.

## 완료된 로컬 검증

- 최소 글꼴 전체 13,473 PASS / 0 FAIL / 45 설정상 skip, Native 공식 4,073 PASS, 집중 48 PASS와 필수 format·Native/WASM32/workspace Clippy·workspace build·suite 검사를 통과했다. 중복 검사 수는 합산하지 않는다.
- 목표 8종 전 16쪽의 Native/fresh 최적화 WASM 각각 100.00%. 대조군 포함 22종 각 40쪽은 Native 최저 90.01%·WASM 97.08%, 정본 보호 3,542쪽의 공식 표시값 하락 0건과 별도 Native 편람 383쪽 gate 통과다.
- CLI/브라우저 반복 저장·실제 BodyX 편집·엔진 스냅샷 복원, 저장본 전쪽 재검증과 보안 90건을 완료했다. Studio UI Undo/Redo나 임의 장문 편집 전체 개선으로 확대하지 않는다.
- 승인 후 전진한 base의 실제 병합 결과에서도 Rust 집중 49건·호스트 글꼴 40건·Native 6쪽 100.00%를 추가 확인했다. 이를 전체·fresh WASM 재실행으로 부르지 않는다. 원본/BodyX 4쪽 RGB는 기존 직접 판독 출력과 같다.
- [최종 검증 원장](../assets/pr9042/final-validation/README.md). 초기 잘못된 Noto 대체 캡처와 WASM 감독 도구 outer exit 143은 완료 증거와 구분했고, 실제 HCR 프로그램 공급·해시 및 개별 완료 결과를 별도 감사했다.

## exact code head GitHub 결과

[CI 영수증](../assets/pr9042/final-validation/pr9066-code-ci.json): code head `4cf0b66548`의 check 34 SUCCESS / 정상 SKIP 5건, 실패·대기 0건이며 mergeable/CLEAN을 확인했다. 수동 재실행·workflow 보정·보안 분류 예외는 사용하지 않았다.

- [CI](https://github.com/edwardkim/rhwp/actions/runs/43523106842): success. Archive A–E 실제 로그는 13,314 PASS / 0 FAIL / 45 skipped이며 로컬 전체 수와 구분한다. Lint·Frontend·Native·Build & Test도 성공했다.
- [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/43523106808), [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/43523106818), [Adapter](https://github.com/edwardkim/rhwp/actions/runs/43523106889), [Proptest](https://github.com/edwardkim/rhwp/actions/runs/43523106955): 모두 success.
- 최종 CI Impact Policy도 success다. [실제 PR 화면](../assets/pr9042/final-validation/pr9066-browser-display.json)에서 SHA 고정 대표 이미지 4개의 로드를 확인했고 raw PNG 해시도 commit과 같았다.

## 병합 확인과 후속 처리

제품·테스트가 같은 문서 head `4d6274f8ad`는 CI·CodeQL·Render Diff의 preflight가 모두 `fast_pass=true`, candidate `4cf0b66548`을 실제 채택했다. 최신 head 13 SUCCESS·정상 skip 24건, 실패·대기 0건이며 최종 정책 감사도 성공했다. [실제 재사용·집계](../assets/pr9042/final-validation/pr9066-trailing-ci.json), [최종 병합·사전 tree 일치](../assets/pr9042/final-validation/pr9066-merge.json).

merge 뒤 #9042의 간접 MERGED 반영을 확인했다. 직접 연결된 해결 issue는 없다. [duration 갱신](../assets/pr9042/final-validation/pr9066-duration-refresh.json)은 ready:true·published:true다. maintainer 운영 기록을 devel에 반영·동기화한 뒤 원 PR의 첫 기여 감사 댓글과 branch·output·유휴 debug 정리를 수행한다. 원 fork·다른 작업공간·사용자 글꼴·local crates 소비 환경은 보존한다. 최종 로컬 종료 로그는 `local/pr-cleanup/pr9066.json`에 기록한다.
