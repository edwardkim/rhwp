# semanticist21 PR 10건 누적 검토

## 현재 판정과 기준

**통합과 승인된 후속 처리 완료. 통합 PR #8609와 원 PR 10건 모두 MERGED이며, CI·self-review·감사 댓글·devel 동기화·종료 정리를 확인했다.** [선정·수행 계획](semanticist21_20261105_review_plan.md)의 대상 10건을 유지한다. 최초 선정 뒤 reviewer `edwardkim`을 지정했으므로 현재 reviewer 필터로 다음 10건을 다시 고르지 않는다.

- 비교 base: `22947f6a90c998e95fa51238d5b907a0ebd37162`.
- 마지막 확인한 원격 devel: `e96b500e3ea259c821f05440f0388355916a9e9b`. 이후 추가된 파일은 이전 PR 검토 기록 3개뿐이며 제품 코드·테스트·설정의 변경은 없다. 현재 후보와의 merge-tree도 충돌 없이 성공했다. [차이 근거](../assets/semanticist21-20261105/latest-base-refresh.json).
- 로컬 branch: `review/integration-20261105-semanticist21-batch`.
- 작업공간: `/home/edward/mygithub/rhwp-worktrees/review-integration-20261105-semanticist21`.
- 시각 검증 producer: `2f44d7d0ab19c743089ae309a8ca64eee57efae2`. 원 기여 10개 head를 merge ancestry로 보존한다. 기본 작업공간의 배포용 pkg는 덮어쓰지 않았다.

## 기능 PR 8건의 최종 Native/fresh WASM

| PR | 문서 / 전체 쪽수(경로별) | Native 최저 | fresh WASM 최저 | 상태 |
| --- | --- | --- | --- | --- |
| #8326 | 4개 / 40쪽 | 98.43% | 98.43% | 검증 후 통합 완료, 원 PR MERGED |
| #8347 | 1개 / 7쪽 | 99.32% | 99.98% | 검증 후 통합 완료, 원 PR MERGED |
| #8349 | 20개 / 20쪽 | 98.83% | 90.63% | 검증 후 통합 완료, 원 PR MERGED |
| #8358 | 1개 / 4쪽 | 100.00% | 100.00% | 검증 후 통합 완료, 원 PR MERGED |
| #8359 | 1개 / 1쪽 | 100.00% | 100.00% | 검증 후 통합 완료, 원 PR MERGED |
| #8363 | 1개 / 1쪽 | 100.00% | 100.00% | 검증 후 통합 완료, 원 PR MERGED |
| #8366 | 1개 / 1쪽 | 95.61% | 96.79% | 검증 후 통합 완료, 원 PR MERGED |
| #8367 | 1개 / 1쪽 | 94.92% | 94.81% | 검증 후 통합 완료, 원 PR MERGED |

저장 재열기를 포함해 경로별 **30개 문서 75쪽 모두 90% 이상**이다. 원본·저장본을 합친 base/후보 228쪽 비교에서 2%p 초과 하락은 없었다. [쪽별 차이](../assets/semanticist21-20261105/base-candidate-page-deltas.json). 최종 runtime에서 모든 PNG를 다시 출력했으며, 독립 Print와 직접 확대 판독한 사전 출력과 75쪽씩 모두 바이트가 같다. [최종 전쪽 결과·PNG 해시](../assets/semanticist21-20261105/functional-final-visual-results.json). 새 측정을 직접 판독 대신 해시만으로 처음 승인한 것이 아니라, 이미 판독한 출력과 동일함을 확인했다.

- #8326: #8517의 HWP 그림 글머리표 참조·오류·Storage 계약을 보존하고 HWPX header 그림 ID 정규화를 보완했다. 실제 내보내기·재열기까지 확인했다.
- #8347: 행 높이의 선언/내용 제약을 공통으로 풀고 측정·배치에 공급한다. 표 끝·뒤 안내 표·본문·다음 쪽 이어받기와 누락/중복을 정식 관계 검사로 보호했다.
- #8349: 아래 일점쇄선의 공통 기하와 원본 run 위상을 Native/SVG/Canvas2D/CanvasKit에서 확인했다. 목표 shape 3과 실제 browser SVG는 100.00%다. 최종 WASM 장식 32개를 실제 CanvasKit consumer에서 실행했다.
- #8359: 첫 구역 값이 다른 구역으로 전파되는 원 PR의 회귀를 보정했다. 구역별 scope와 캐시 namespace를 공유하고 setter의 불필요한 전체 스타일 재해소를 제거했다. 최종 WASM에서도 구역 교대 조회·편집·다른 구역 보존·HWPX 재열기가 통과했다.
- 나머지 4건의 시작 쪽 번호·대각선·입체선·머리말 번호는 원본 속성부터 공통 생산·소비 지점을 추적하고 독립 Print와 대조했다. 개별 review에 범위와 기존 차이를 구분했다.

## 성능 PR 2건의 판단 대기

#8331의 JPEG 픽셀 인계는 실제 최적화 WASM에서 첫 그리기 중앙값 1772.8 → 1118.6ms(-36.9%)와 linear memory 감소를 관측했다. 메인터너는 거대 이미지 헤더의 예산 곱셈 overflow를 보정했으며 실제 production 함수 음성 대조는 수정 전 FAIL / 수정 후 PASS다. #8332의 핵심 캐시는 #8348에서 이미 구현돼 중복 캐시를 제거하고 tokenizer 개선과 편집 계약 검사를 보존했다. 옛 base의 15초 → 2.7초를 현재 개선량으로 인용하지 않는다.

두 성능 문서 39쪽은 최초 비교에서 base/후보의 Native/fresh WASM PNG가 모두 같았지만 독립 Print 대비 기존 조판 미달이 남았다. reference PDF의 누락 글꼴을 보완한 비교에서도 미달을 확인해 최종 Native/fresh WASM 전쪽 재측정도 완료했다. **작업지시자가 기존 미달 중 base/후보 출력이 같은 범위에 한정해 수용 예외를 승인했다.** 평균이나 다른 문서 점수로 미달을 상쇄하지 않는다. [실측과 결정 범위](../assets/semanticist21-20261105/performance-control-review.md).

## 완료된 공통 검증

- 최종 Native·최적화 fresh WASM 빌드 성공. [source·명령·artifact 해시](../assets/semanticist21-20261105/final-build-runs.json).
- 최종 Native 편람 **383쪽 통과**, 최저 **99.96%**, 370쪽 100.00%. 기준 KoPub/HCR 글꼴 환경을 적용했으며 baseline·허용값을 바꾸지 않았다. 다른 사례용 글꼴 환경으로 실행했던 첫 실패와 바로잡은 근거를 [함께 보존](../assets/semanticist21-20261105/handbook-font-profile-correction.json)했다.
- 최소 글꼴 집중 회귀 **21개 통과**. 나머지 2,589개는 집중 filter 범위 밖이며 전체 통과로 세지 않는다. suite/unit 정책은 source unit **4,099개·296모듈**을 유지한다.
- Studio 전체 **2,722개 중 2,720개 통과·기존 skip 2개·실패 0**, TypeScript·production build 성공. 실제 CanvasKit 집중 21개 통과.
- 원본·PDF·생성 저장본 **65개 Git blob**의 바이트 일치 확인. 생성 HWPX 5개에 글꼴 프로그램을 포함하지 않았으며 모든 검증 PDF는 50 MiB 미만이다. [입력 검증](../assets/semanticist21-20261105/input-commit-verification.json). `samples/` 아래 새 문서는 없다. 다른 경로의 신규/변경 문서 31개는 실제 CLI의 세 탐지기(은닉 글자·injection low/fields 포함·unicode)로 총 93회 실행해 모두 clean을 확인했다. [실제 결과](../assets/semanticist21-20261105/security-cli-sweep.json). 최초 전체 회귀에는 `samples/` 전용 환경변수에 다른 경로를 넘겨 호출 guard 1건이 실패했다. 탐지기·테스트·allowlist를 바꾸지 않고 올바른 입력으로 전체 12,499개를 다시 실행해 모두 통과했다. [호출 보정 근거](../assets/semanticist21-20261105/security-caller-correction.json).

전체 최소 글꼴 nextest **12,499개 통과·기존 제외 45개**를 확인했고 Native 공식 3종도 합계 **4,064개 통과**를 확인했다. 기존 lib ignore 12개와 집중 filter 밖 항목은 통과 수에 포함하지 않는다. [Native 실제 결과](../assets/semanticist21-20261105/native-official-results.json). 최종 Rust lint 묶음·문서 예제·suite/unit 정책도 모두 통과했다. 설명 주석의 길이 변화로 자동 suite 배정 갱신이 필요했지만, 2,018개 소스의 실행 코드와 전체 수집 대상이 그대로임을 확인했고 최종 배정의 all-target Clippy·정책 검사도 통과했다. [소스 동등성](../assets/semanticist21-20261105/generated-harness-refresh.json), [마지막 검사](../assets/semanticist21-20261105/final-metadata-check-runs.json). 기존 all-target lint의 테스트 생성자 6곳 오류는 같은 파일 10개 생성자의 기본 context 초기화로 보정한 뒤 통과했으며, 기대값·검사 수를 바꾸지 않았다.

## CI 실패 처리 판단과 원격 범위

원 PR 4건의 실패는 실제 log·annotation을 확인한 checkout/compile 시간 초과였다. assertion·컴파일 오류를 발견하지 않았다. 최신 base의 Lint 45분·builder 60분 정책과 메인터너 새 후보 검증으로 처리하며 기여자 코드 재작업 댓글을 택하지 않았다. 기존 run의 취소·재실행이나 CI 명령·예산 변경은 수행하지 않았고, 미실행 원 검사를 성공으로 기록하지 않는다.

로컬 필수 gate는 모두 완료했다. 예외와 원격 범위를 승인받았으므로 exact 제출 SHA·본문·원 PR별 후속 처리안을 확정해 원격 단계로 진행한다. GitHub CI·merge의 실제 완료는 별도로 확인한다.

## 로컬 self-review 결과와 남은 결정

1. #8331의 거대 헤더 예산 곱셈 overflow는 `u128`으로 보정하고 production 함수 음성 대조와 정식 unit 실행으로 확인했다.
2. #8359의 첫 구역 값 전역 적용은 두 구역 반례에서 실제 회귀를 재현해 공통 scope·캐시 namespace로 보정했다. setter의 불필요한 전체 스타일 재해소도 제거했다.
3. #8332는 #8348과 중복인 캐시만 제거하고 기여의 tokenizer 개선·공개 편집 계약을 유지했다.
4. 원 CI 실패 4건은 source 오류나 assertion 실패가 아닌 checkout/build 시간 초과로 판단했다. 현재 정책과 새 후보 검증으로 처리하며 재작업 댓글을 선택하지 않았다. [실제 job·annotation](../assets/semanticist21-20261105/original-ci-failures.json).
5. 성능 문서의 기존 미달은 예외 결정 전까지 수용에 적용하지 않는다. #8368의 다른 테두리 유형 차이 등 개별 기록에 명시한 기존 범위는 이 작업의 해결 성과나 자동 issue 종료 대상으로 삼지 않는다.

[95개 최종 대표 PNG 목록](../assets/semanticist21-20261105/representative-final-pngs.json), [전쪽 TSV](../assets/semanticist21-20261105/visual-manifest.json), [원격 처리 제안](../assets/semanticist21-20261105/remote-action-plan.md). 검토와 후속 처리를 종료했고 후보 branch/worktree를 제거했다. merge·close 등 검토 종료 시 canonical 절차에 따라 output 전체와 검토 debug를 정리한다.

## 원격 처리 승인

2026-11-05 작업지시자가 #8331·#8332의 변경 전부터 있던 동일 출력 차이에 한정한 수용 예외와 10건의 통합 push·Open PR·해당 head CI·self-review·merge·원 PR 완료/감사 댓글·동기화·정리를 승인했다. 새 회귀·검사 실패·쪽수 차이·누락·90% 미만의 새 렌더링 회귀 추가는 예외 대상이 아니다. [원문 답변과 한정 범위](../assets/semanticist21-20261105/maintainer-approval.json). 원격 실행 결과와 CI는 실제 완료 후 별도로 기록한다.

## 제출 후 보안 진단 보정

[통합 PR #8609](https://github.com/edwardkim/rhwp/pull/8609)의 첫 head Actions 검사들은 성공했지만 별도 GHAS CodeQL에서 기존 조판 진단 sink 두 개가 High로 판정됐다. 앞선 #8548의 정책을 따라 진단 채널만 제거하고 실제 암호 fixture의 stderr 계약을 보강한다. 계산·렌더 출력의 검증 증거는 같은 producer로 구분해 재사용한다. [정확한 flow·수정 전 실제 실패·검증 범위](../assets/semanticist21-20261105/ci-security-followup.md). 새 head의 필수 로컬 검사·독립 CodeQL 성공 전에는 merge를 보류한다.

보안 후속 source `d75809517f`에서 전체 12,499개, Native CI 22개 명령 192개와 별도 Native 라이브러리 4040개가 통과했다. 실제 암호 Native CLI의 해당 진단 출력은 0회이며 SVG 24쪽 전부가 기존 producer와 같다. [보안 보정 후 실제 검증](../assets/semanticist21-20261105/security-local-validation.json). 새 code head `da0074f41e`의 독립 GHAS CodeQL과 필수 CI까지 모두 성공했다.

## GitHub CI와 self-review

[통합 PR #8609](https://github.com/edwardkim/rhwp/pull/8609)의 code head `da0074f41ee01d7fa79e6bd4ffd5cadf7c69665d`에서 [Full CI](https://github.com/edwardkim/rhwp/actions/runs/41937077126)·[CodeQL](https://github.com/edwardkim/rhwp/actions/runs/41937077195)·[Render Diff](https://github.com/edwardkim/rhwp/actions/runs/41937077068)·Adapter·Proptest와 최종 CI 정책 집계의 성공을 확인했다. self-review의 추가 차단 사항은 없으며 통합 후보는 승인이다. [실제 check·job 결과](../assets/semanticist21-20261105/github-ci-code-head.json), [self-review 근거](../assets/semanticist21-20261105/self-review.json).

이 기록은 녹색 code head 뒤의 문서 전용 single-parent trailing commit이다. 새 trailing head의 실제 fast-pass·필수 집계·최신 base 충돌 상태를 확인한 뒤 승인된 merge를 진행한다.

독립 CodeQL의 두 차단 경고는 현재 PR ref에서 fixed이며 Rust 분석 결과는 126→122개, open ID는 12→10개다. [코드 보정과 독립 분석의 최종 결과](../assets/semanticist21-20261105/ci-security-followup.md). source·검증 기준·보안 분류를 우회하지 않고 해당 code head의 모든 검사를 확인했다.

## 확정된 원격 결과

[통합 PR #8609](https://github.com/edwardkim/rhwp/pull/8609)을 merge commit `e9d4a23be4523a97c29ce6f5e407aba6c4cfb26c`로 통합했다. 원 PR 10건 모두 간접 MERGED를 확인했으며 원 기여 commit·fork branch를 보존했다. [code CI](https://github.com/edwardkim/rhwp/actions/runs/41937077126)·[문서 head CI](https://github.com/edwardkim/rhwp/actions/runs/41941236661)와 실제 fast-pass·최종 정책 status·fresh-base merge-tree를 확인했고, [COMMENT self-review](https://github.com/edwardkim/rhwp/pull/8609#pullrequestreview-5778015173)를 최종 head `1557f70b03b92f0af88fb0e6f4ef280df4b8683a`에 게시했다.

병합 후 [duration 갱신](https://github.com/edwardkim/rhwp/actions/runs/41941963866)은 `ready:true`·`published:true`이며 code run `41937077126`의 B/C/D/E 실제 측정 artifact를 검증해 metadata commit `8bcf7906f2d923d19e8b65621dbb307ee7c0f117`으로 게시했다. 병합 후 제품 검증 CI를 새로 실행하지 않았다. 별도 PR #8368과 이슈 #8236은 열린 상태로 보존했고 관련 신규 issue close는 없다. [확정 결과와 원 PR별 상태](../assets/semanticist21-20261105/remote-completion.json), [실제 metadata 결과](../assets/semanticist21-20261105/postmerge-automation.json).

후속 기록은 maintainer 직접 반영(Option M)으로 source·test·workflow·baseline을 포함하지 않는 운영 문서 commit에 담는다. 승인된 원 PR별 감사 댓글·devel 동기화·소유 worktree/branch·output 전체·검토 debug 정리를 마친 실행 실적은 원 PR 댓글과 로컬 종료 기록에서 확인한다.

## 후속 처리 종료

원 PR 10건의 감사 댓글을 모두 게시했고 API 본문과 실제 GitHub 화면의 review/overlay 56개를 확인했다. 검토 전용 local/remote branch와 worktree, 저장소 root output 전체 및 target/pr-review/debug를 제거했다. 확보 공간은 24.75 GiB, Linux 여유 공간은 204.33 GiB다. 다른 worktree·target profile과 배포 WASM은 보존했다. [댓글 URL·검증·종료 상태](../assets/semanticist21-20261105/remote-completion.json).
