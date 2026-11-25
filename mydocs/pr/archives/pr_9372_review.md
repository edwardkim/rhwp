# PR #9372 저장 조판·개체 소속 개선 통합 검토

planet6897 님의 원 PR 8건을 통합했다. 검증 제품 커밋은 `e70dc6231a22ec0bd569537c4e5cb0ff930fd606`, 보호 기준 해시를 갱신한 코드 검증 head는 `901d624358dba59105b0a1bcb90c0966431e5c89`이다. [통합 PR #9372](https://github.com/edwardkim/rhwp/pull/9372)

| 원 PR | 검증한 변경 | 개별 검토 |
| --- | --- | --- |
| [#9289](https://github.com/edwardkim/rhwp/pull/9289) | 자리차지 표와 같은 문단의 그림·치수선 원점 | [검토 기록](pr_9289_review.md) |
| [#9305](https://github.com/edwardkim/rhwp/pull/9305) | 표로 시작하는 쪽의 저장 줄·문단 간격·후속 내용 | [검토 기록](pr_9305_review.md) |
| [#9311](https://github.com/edwardkim/rhwp/pull/9311) | 용지 기준 자리차지 표 앞 빈 줄의 쪽 소속 | [검토 기록](pr_9311_review.md) |
| [#9317](https://github.com/edwardkim/rhwp/pull/9317) | 분할 표의 첫 조각에 용지 기준 도형을 한 번 배치 | [검토 기록](pr_9317_review.md) |
| [#9324](https://github.com/edwardkim/rhwp/pull/9324) | HWP·HWPX 반복 저장에서 원점 표지와 저장 줄 보존 | [검토 기록](pr_9324_review.md) |
| [#9327](https://github.com/edwardkim/rhwp/pull/9327) | 캡션·자리차지 그림과 식량안보 보고서의 쪽 소속 | [검토 기록](pr_9327_review.md) |
| [#9363](https://github.com/edwardkim/rhwp/pull/9363) | 쪽 첫 빈 줄의 저장 전진 및 숨겨진 빈 줄 반례 | [검토 기록](pr_9363_review.md) |
| [#9364](https://github.com/edwardkim/rhwp/pull/9364) | 음수 줄 간격과 쪽 되감김의 구분 | [검토 기록](pr_9364_review.md) |

## 로컬 검증

- 최소 글꼴 환경 전체 nextest 14,028건 통과, 45건 제외. Native Skia, Native·WASM lint, 워크스페이스 빌드, suite manifest 및 단위 검사도 통과했다.
- #9364 의왕시 문서의 반각 낫표가 쓰인 기준 PDF와 맞도록, 명시한 글꼴 슬롯에서만 자연 전진으로 줄 경계를 계산했다. 전쪽 Native·fresh WASM 3쪽이 각각 100.00%였다. 전각 낫표 인쇄 조건의 3쪽도 두 경로에서 각각 기존 픽셀을 보존했다. [전후 TSV·인쇄 영수증](../assets/planet6897_stored_layout_20261123/pr-9364/maintainer-wrap/validation.json) · [3쪽 Native review](../assets/planet6897_stored_layout_20261123/pr-9364/maintainer-wrap/native/hcr/review-003.png) · [overlay](../assets/planet6897_stored_layout_20261123/pr-9364/maintainer-wrap/native/hcr/overlay-003.png)
- 최종 글꼴 슬롯 보정이 활성화되는 정본 보호 조건은 0건이었다. 영향 범위를 확인해 기존 46조건·4,046쪽 증적을 재사용했다. [보호 검증 이력](../assets/planet6897_stored_layout_20261123/protected-validation.json)
- #9365 배치 검증의 기준·후보 24개 PNG 및 #9317의 17쪽 Native·fresh WASM PNG는 쌍별로 동일했다. 나머지 주장 쪽의 인쇄 기준과 원본 출처는 각 개별 검토에 기록했다. [재사용 판정](../assets/planet6897_stored_layout_20261123/affected-and-reused-pages.json) · [시각 증적 목록](../assets/planet6897_stored_layout_20261123/README.md)

## 제출 코드의 GitHub CI

초기 제출 head `7ad8d9604c17da6afec88bb89d2e814dcadcd022`의 Lint에서 정본 TSV와 편람·기여자 공통 검사의 해시 불일치로 271건 중 15건이 실패했다. 후행 커밋 `901d624358dba59105b0a1bcb90c0966431e5c89`에서 두 검사 참조만 갱신했다. 정본 4,046행을 대조해 7행 상향·하락 0행을 확인했고 동일한 의존성 버전으로 Visual Sweep 계약 검사 271건이 로컬에서 통과했다. [초기 Lint 로그](https://github.com/edwardkim/rhwp/actions/runs/45211636936/job/137822693375)

| 실행 | 결과 |
| --- | --- |
| CI | [성공 · 시도 1](https://github.com/edwardkim/rhwp/actions/runs/45213886272) |
| CodeQL | [성공 · 시도 1](https://github.com/edwardkim/rhwp/actions/runs/45213886085) |
| Render Diff | [성공 · 시도 1](https://github.com/edwardkim/rhwp/actions/runs/45213885776) |
| Adapter inter-diff | [성공 · 시도 1](https://github.com/edwardkim/rhwp/actions/runs/45213885893) |
| Proptest roundtrip | [성공 · 시도 1](https://github.com/edwardkim/rhwp/actions/runs/45213886374) |
| 최종 정책 감사 | [성공](https://github.com/edwardkim/rhwp/actions/runs/45220246055) |

## 병합 후 처리

원 PR 8건은 각 주장 쪽에 해당하는 비교·오버레이를 연결하고 통합 병합을 안내한 뒤 닫는다. 연결된 해결 이슈 7건의 종료 상태를 확인한다. 식량안보 보고서 90~93쪽의 흐름은 [#8997](https://github.com/edwardkim/rhwp/issues/8997)에서 계속 추적한다.
