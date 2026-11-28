# PR #9480 검토

고정한 기준 devel은 `3a6b7d2b318287361f443b0814702783eb973533`입니다. 원 PR 6건의 고유 커밋 10개를 체리픽했고, #9473과 #9472에 포함된 선행 14개 커밋은 devel에 이미 반영된 SHA를 확인했습니다. 각 원본 head와 적용 SHA는 개별 검토 문서에 기록했습니다. [통합 PR](https://github.com/edwardkim/rhwp/pull/9480)

[#9472](pr_9472_review.md) · [#9474](pr_9474_review.md) · [#9475](pr_9475_review.md) · [#9476](pr_9476_review.md) · [#9477](pr_9477_review.md) · [#9478](pr_9478_review.md)

## 로컬 검증

Mac에서 집중 회귀 16건, 전체 nextest 14,267건, Native Skia 라이브러리 4,032건과 통합 모듈 21개의 193건이 통과했습니다. Rust lint와 workspace 빌드를 완료했습니다. 정본 보호 23개 문서의 Native·fresh WASM 4,046행이 공식 하한을 충족했고, devel과 페이지별 PNG SHA-256이 모두 같았습니다. [로컬 검증 결과](../assets/integration-8506-20261128/local-verification.json) · [정본 보호 대조표](../assets/integration-8506-20261128/protected-page-comparison.tsv)

win10-ted의 Windows 10 x64에서 .NET 10.0.400과 후보 MSVC DLL로 C# 편집·미저장 Print PDF·저장 스모크를 통과했습니다. NETSDK1005를 재현해 README 명령을 보정하고 같은 환경에서 새 명령을 다시 실행했습니다. HWP5 캐시 식별자는 SHA-256으로 보정했고, 기준·후보의 암호 파일 SHA-256이 일치했습니다. 동일 WASM 반복 저장은 이번 측정에서 20.114초에서 0.033초로 줄었습니다. [#9472 검토](pr_9472_review.md) · [#9478 검토](pr_9478_review.md)

## #8506 시각 대조

한/글 2020의 2쪽 Print PDF를 96 DPI로 비교했습니다. 1쪽에서 두 제목 줄과 상하 괘선의 배치를 직접 확인했습니다. 2쪽의 후보 PNG는 경로별로 devel과 같았습니다. 양쪽 2쪽에서 별도 검토 대상으로 표시된 페이지는 0개입니다. [원 PR 대비 전쪽 TSV](../assets/issue_8506/integration/source-page-comparison.tsv) · [#9477 검토](pr_9477_review.md)

| 출력 | 쪽 | 픽셀 일치율 | 시각 정확도 보조값 | raw 실루엣 |
| --- | ---: | ---: | ---: | ---: |
| Native | 1 | 98.87144% | 74.55521% | 100.00% |
| Native | 2 | 95.78086% | 64.37555% | 99.96% |
| fresh WASM | 1 | 98.66366% | 70.63177% | 100.00% |
| fresh WASM | 2 | 95.94564% | 65.77274% | 99.97% |

## GitHub Actions

제출 head `a031f92461e59ddac52f5ed439e5ed0cee2b31c3`의 Full CI와 최종 정책 판정이 통과했습니다. [Build & Test](https://github.com/edwardkim/rhwp/actions/runs/46083366418) · [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/46083366410) · [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/46083366393) · [Proptest roundtrip](https://github.com/edwardkim/rhwp/actions/runs/46083366431) · [Adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/46083366399) · [CI Impact Policy Controller](https://github.com/edwardkim/rhwp/actions/runs/46083366402)

이 기록을 포함한 새 head의 CI와 병합 가능 상태를 확인한 뒤 병합합니다. 원 PR 6건의 검토 결과와 #8506의 종료를 확인합니다.
