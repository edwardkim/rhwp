# PR #8671 검토 — fix(native): fall back per glyph in positioned equations (P79)

## 최종 판정

**머지 보류 — 접수·의존성·충돌 조사는 완료했고 필수 검증을 진행하기 전이다.** 이 기록은 거절이나 GitHub review를 뜻하지 않는다. 현 source head와 통합 검증 head, 실제 실행 결과를 구분해 검증 후 갱신한다. 공개 댓글·push·merge는 아직 승인받거나 수행하지 않았다.

## 접수와 범위

| 항목 | 관측값 |
| --- | --- |
| 원 PR | [#8671](https://github.com/edwardkim/rhwp/pull/8671), seo-rii, Open/Ready |
| source repository / branch | `seo-rii/rhwp` / `feat/issue-536-p79-equation-glyph-fallback` |
| source head | `a4e839d1c504d9adc122c1ef381ac4a49c11bfcb` |
| reviewer | edwardkim; 2026-11-07T03:00:34.131852+00:00 API 재확인 |
| 검토 base | `b8dd67c5001dfb0410e9e192528676396cdd6a4e` |
| 단계별 비교 시작 | `5847f6f25627faf09e4c72f19c2311ec89798d21`; 누적 devel diff와 이번 단계 diff를 구분 |
| merge simulation | clean; tree `7ecb18ddbd459c51c745a9091f3b9826aa8fc732` |
| source CI | {"SKIPPED": 6, "SUCCESS": 31, "CANCELLED": 2, "FAILURE": 2}; 접수 시점 스냅샷 |

원 기여의 이 단계 source commit·경로와 원 head의 CI는 [접수 원장](assets/seo-rii-20261107/intake.json)에 보존한다. 모든 source head가 최종 #8728의 조상이다. 원 PR을 따로 덮어쓰거나 ancestor commit을 반복 체리픽하지 않는다. Draft #8646의 선행 P73 두 commit도 이 원 head 안에 포함돼 있으며, 그 PR 자체의 상태 변경·merge는 별도 판단 대상이다.

## 검토·검증 계획

[공통 수행 계획](seo_rii_20261107_review_impl.md)의 해당 단계와 공통/개별 소비 경로를 따른다. 기존 이슈 [#536](https://github.com/edwardkim/rhwp/issues/536)·[#8701](https://github.com/edwardkim/rhwp/issues/8701), 선행 P69–P72 통합과 현재 base를 대조한다. 조판 원칙 적용 여부·실제 입력 commit·독립 Print·Native/fresh WASM·확대 판독은 소스의 실제 소비 경로로 판정하며 아직 통과로 기록하지 않는다. GPU/host/SDK 컴파일·실행의 플랫폼별 근거도 서로 대용하지 않는다.

수용 전 조건은 해당 단계의 필수 계약·실제 실행·시각 검증과 최신 head CI, 작업지시자의 원격 처리 승인이다. CI 성공·원 기여자의 자기 검토만으로 메인테이너 검증 완료를 주장하지 않는다.

## Merge 후 contributor PR comment 계획

아직 미실행인 시각 결과·이미지 URL은 만들지 않는다. 해당 PR의 실제 검증 범위·결과와 최종 통합 head가 확정되면 [Visual Sweep comment 정본](../manual/verification/visual_sweep_guide.md#github-merge-comment)에 따른 대응 review/overlay·지표·잔여 한계를 이 절에 보완한다. devel 반영·merge SHA·API 및 실제 표시 확인 뒤 승인 범위에서 게시한다.

## 실제 원 사례에서 발견한 미해결 경계

#8337의 원 파일과 같은 기존 공개 입력에서 AB는 복원됐지만, 원 수식 글꼴이 없으면 전용 표시 코드 U+E037·U+E036이 원 숫자 4·3으로 대체되지 않는다. 본문 글꼴을 동일하게 공급한 대조의 raw가 99%대여도 직접 판독상 의미가 달라 **머지 보류**다. [실행 증거·제한 보정 계획](assets/seo-rii-20261107/positioned-equation-fallback-finding.md)을 따르며 2026-11-07 승인된 메인테이너 보정을 진행한다.

2026-11-07 작업지시자가 위 원인에 대한 메인테이너 보정과 필수 검증을 승인했다. 현재 판정은 보정·재검증 완료 전까지 보류이며 원 source SHA를 보존한다.

## 승인한 보정의 실제 Native 대조

`cda7c8cb33`에서 producer의 사설 코드 매핑을 재사용해 원 숫자 의미를 복원했다. 같은 최소 글꼴·본문 subset 조건의 원 사례 전 1쪽 raw는 99.63%이며, 수정 전 다른 문자였던 4·3을 직접 확대 판독했다. 원 수식 face 공급 대조는 수정 전후 PNG가 완전히 같다. [실행·이미지·TSV](assets/seo-rii-20261107/positioned-equation-fallback-finding.md#승인한-보정의-native-실행-결과)를 따른다. fresh WASM·정식 producer 회귀·공통 필수 검증 완료 전에는 최종 수용으로 바꾸지 않는다.
