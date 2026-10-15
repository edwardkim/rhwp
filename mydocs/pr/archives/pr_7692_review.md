# PR #7692 리뷰 — 병합 칸 행 높이·칸 줄 상자

## 최종 판정

**메인터너 보정 후 수용 가능 — 아래 통합 적용 범위의 로컬 검증 완료.**
검토 대상은 원 head `0c1e33d2a0f536cd516a1020ee17dffe346ec20d`를 누적·보정한 통합 code `39a634e547621c32ae6f9ba8c1694d35c827078b`다. 원 PR의 모든 첨부·golden이나 문서 전체 피델리티를 자동 수용한 판정은 아니다. [통합 PR #7923](https://github.com/edwardkim/rhwp/pull/7923)의 code candidate `cbc5815631d6c221dc6ec6e0f03565859cb1c346`에서 전체 CI·정책 체크 성공과 CLEAN을 확인했다. 문서 trailing head의 최신 체크·mergeability 확인이 병합 전 조건이다.

## 접수 정보

| 항목 | 값 |
| --- | --- |
| 원 PR | [#7692](https://github.com/edwardkim/rhwp/pull/7692) / planet6897 / base `devel` |
| source head | `0c1e33d2a0f536cd516a1020ee17dffe346ec20d` |
| 접수 시 상태 | OPEN, draft=True; draft와 무관하게 현재 통합을 진행하라는 사용자 지시 |
| 기준 devel | `463c48cc228170b576c832591b76826116e86a49` |
| 검토 역할 | collaborator의 체리픽 통합·보정 및 제출 PR self-review |

## 변경과 실제 소비 경로

병합 칸 선언 초과분을 배분한 뒤 미지 행을 풀고 같은 칸의 실제 줄 상자와 요구 높이를 소비한다. 그림 칸·줄 나눔 경계의 원 기여와 통합 보정을 구분한다. 원 기여의 미달 신규 후보는 기존 assets에 보존한다.

- `src/renderer/layout/table_layout.rs`
- `src/renderer/layout/table_partial.rs`

현재 통합 계보의 관련 적용·보정 commit:

- `3eb7ff36287973f7569440fe5581dbc0b7f7bb9a`
- `ece68e79f67b04e135d57b97f17d065d76c31259`

## 검증과 원칙 대조

- [통합 검증·원칙·이관 기록](planet6897_followup_20261015_review.md) 및 [입력/PDF/PNG SHA 증적](../assets/planet6897_followup_20261011/qualified-print-d80593/README.md)을 함께 적용한다.
- 최초 전체10,362건에서10,354PASS/8FAIL/45SKIP. 사용자 지정으로 실물6건 이관 후 폰트 계약 누락2건을 수정했고, 경로 변경을 포함한 관련55건을 모두 통과했다. 사용자 승인에 따라 이 결과를 합산해 전체 회귀 통과로 처리했다.
- 최종 code head의 fmt·native/WASM/workspace Clippy, workspace 빌드, suite/tier 검사, Native Skia와 문서 테스트를 완료했다. Studio1,875건과 네 배포 빌드/폰트 계약도 통과했다.
- 전쪽 시각 수용 범위는 명시한21문서·181쪽이다. 원본·독립 Print의 커밋 바이트와 SHA를 확인했고 새 Native/fresh WASM 출력은 모든 쪽90% 이상이다. 핵심 위치의 ±2px 검사는 양쪽98% 이상과 직접 판독을 조건으로 관계 검사와 함께 유지한다.
- 원 PR에서 넘어온 미달/미검증 신규 실물 검사·golden은 기존 이관 기록과 원문을 보존하며 새 정상 기준으로 등록하지 않는다. 구체적인 이관은 #7445와 자식 이슈에 남고 이번 PR에서 종료하지 않는다.

### 이 PR에 연결한 실제 문서 출력

| 문서 키 | 전쪽 | Native 최저% | fresh WASM 최저% | 직접 비교·핵심 쪽 |
| --- | ---: | ---: | ---: | --- |
| `remaining-employment` | 4 | 98.81410 | 96.17493 | [4] |
| `pr7669-reduced` | 2 | 99.73672 | 99.83858 | [2, 1] |

![remaining-employment fresh WASM review](../assets/planet6897_followup_20261011/qualified-print-d80593/remaining-employment-wasm-p004-review.png)

## Merge 후 contributor PR comment 계획

통합 PR 최신 CI와 실제 merge SHA를 확인한 뒤 원 PR에 한국어 존댓말로 원 기여·추가 보정·수용 범위를 설명한다. 해당 merge에 존재하는 PNG를 merge SHA 고정 raw URL로 표시하고 Visual Sweep 정본·실제 페이지/수치·남은 #7445 범위를 연결한다. 원 fork branch는 보존하며 supersede 처리와 게시 본문은 API로 재확인한다.
