# JinHo-von-Choi PR 3건 검토 계획

2026-11-14 KST. 작업지시자는 리뷰어가 없는 PR에 메인테이너를 지정한 뒤 검토를 시작하도록 지시했다.
#9070·#9071·#9072 모두 기존 사용자/팀 reviewer와 제출 review가 없었고, 각 원 PR에 `edwardkim`을 요청한 뒤 API로 재확인했다. 이전 #9052의 실제 MERGED도 확인했다. GitHub association의 FIRST_TIME_CONTRIBUTOR 표시는 과거 merge가 없다는 증거로 사용하지 않는다.

## 기준과 대상

| PR | 원 head | 원 branch | 범위 |
| --- | --- | --- | --- |
| #9070 | `8669ec6dc4b91a57d6af8e6b82dd391dff0832af` | `task/8912-hwp3-parser-fixes` | HWP3 괘선/기호 디코딩과 잘린 교차참조 가드 |
| #9071 | `a1bd3fa4edb8bb79c0ae312166358b6be299e5d0` | `task/7631-hwpx-hwp-attr-fidelity` | 위치 속성·색 없음·바탕쪽 슬롯·각주/미주 왕복 |
| #9072 | `d268b792eda1aa678ac5d67d285455f6760a2754` | `task/7381-cli-field-export-fixes` | replace-text 도움말·루트 필드 인덱스·독립 TextRun 수집 |

원 fork는 `JinHo-von-Choi/rhwp`, 원 PR base metadata는 `a210861488468868b5992231c55b78798e622e98`다.
검토 기준은 최신 `upstream/devel` `88170fd75f9d0ce156fe49d79aeabea9bb3a4bcc`로 고정했다.
각 원 head와 최신 devel의 독립 merge-tree는 모두 exit 0이다. 원 head를 변경하거나 fork에 push하지 않는다.
공통 대표 이슈가 없는 세 변경 축이므로 날짜 기반 로컬 누적 검토 branch를 사용한다. 원 기여 commit을 보존하고 판단·검증·원격 조치는 PR별로 구분한다.

## 수행 범위

1. 각 이슈의 전체 논의와 기존 해결 PR·통합 보정·기준 출력을 먼저 추적한다. 최신 devel에서 원 증상과 입력 계약이 남는지 대조한다.
2. 원 head의 실제 diff·commit·CI와 본문의 검증 SHA를 구분한다. stale 본문이나 작성자 자체 결과를 메인테이너의 독립 검증으로 표시하지 않는다.
3. HWP3 문자 의미와 경계 검사, HWP/HWPX의 필드·색·슬롯·각주 길이의 입력→IR→출력, CLI 실제 명령·필드 순서·텍스트 누락/중복을 검증한다. 관련 사양과 정상·반례를 대조한다.
4. 실제 화면/배치에 영향을 주는 경로와 기존 렌더 회귀 변경은 독립 Print·Native/fresh WASM 증거를 확인한다. 필요한 검증을 생략한 상태를 수용으로 판정하지 않는다.
5. 최신 devel에 들어온 #9007의 채우기 무늬 해석과 보안 판정, 기존 #9052의 CLI 편집 계약이 유지되는지도 해당 변경 소비 경로에 맞춰 확인한다.
6. 검증은 기본 작업공간의 공용 `target/pr-review`를 사용해 Cargo/Git 변경을 직렬 수행한다. 원격 CI 재사용 조건과 변경별 필수 게이트는 현행 local_validation 4.3을 따른다.
7. 각 PR의 `pr_N_review.md`에 판정·근거·필요한 보완 범위를 기록한다. source 보정이나 conflict 해소가 필요하면 실제 근거·제한 보정안을 준비해 담당/범위 승인을 확인한다.

현재 원격 승인은 reviewer 지정까지다. 댓글·승인 review·push·새 PR·merge·issue close·workflow 조작은 수행하지 않는다. 로컬 조사·검증과 기록을 완료한 뒤 구체적인 후속 범위만 별도로 요청한다.


## 재개 후 기준과 실제 검증

최신 devel `53da4d2da28ae0cacf8feac3a989cc6a69fb2622`를 `a1975f149e`에 통합했다. 2026-11-15 00:30 KST 원격을 다시 읽어 base·원 head 3개·reviewer가 그대로이고 본인 열린 PR이 0개임을 확인했다. fork의 실제 push 권한은 false다. 아직 원격 수용 경로를 실행하지 않았다.

기본 경로는 `maintainer_general`, 보조는 `intake_and_review`, `local_validation`, `multi_pr_update_branch`, `visual_fixture_evidence`, `post_merge`다. 모 router·선택표와 해당 자식 문서 및 개발 환경·시각 정본을 로드했다. [개별 #9070](pr_9070_review.md) · [#9071](pr_9071_review.md) · [#9072](pr_9072_review.md)의 판정과 원격 승인을 분리한다.

승인된 공통 보완을 제품 source `eeaecff59e`로 확정했고 회귀 검증 head `3b5e750d66`와 제품 경로가 같다. 전체 회귀 13,574개와 lint·편람 gate·Native/fresh WASM 각 134쪽 비교는 통과했다. Native 공식 3종 4,073개 통과와 남은 과거 특별 보호값 107행 재현 조건을 [최종 원장](../assets/jinho-von-choi-20261114/README.md)에 기록한다. 필수 판정 미완료를 원격 CI나 미검증 표시로 대신하지 않는다.


## 2026-11-15 최종 완료 확인

승인된 로컬 보완·필수 검증을 예외 없이 완료했다. 제품 source `8ad1c5c08a`·최신 base `2d83fa4c8a`, 전체 회귀 13,608건·Native 공식 4,073건과 보호 3,542행 하락 0건을 [최종 검증](../assets/jinho-von-choi-20261114/final-source-8ad/README.md)에 기록했다. 상승 19행은 정본에 반영했다. 원격 승인 요청은 실제 최종 제출안으로 갱신하며 앞선 미재현 예외 제안을 재사용하지 않는다.
