# PR #9162 검토 · Linux Flutter SDK·Native GL 검증 경로

**검토 완료.** [통합 PR #9189](https://github.com/edwardkim/rhwp/pull/9189) 코드 head `5b3e013dfb49e82f24fc769f81170e2310adf612`의 Full CI와 적용 대상 검사가 모두 통과했다. [CI](https://github.com/edwardkim/rhwp/actions/runs/44470122043) · [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/44470122101) · [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/44470121749) · [Adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/44470122018) · [Proptest roundtrip](https://github.com/edwardkim/rhwp/actions/runs/44470122039) · [CI Impact Policy Controller](https://github.com/edwardkim/rhwp/actions/runs/44470121845)

이번 검토 기록을 반영한 head의 필수 검사를 확인한 뒤 병합한다.

[원 PR](https://github.com/edwardkim/rhwp/pull/9162) head `3be05add79e95a0c9b46e729ca1775c257d1010c`를
`c526dd402bcf55d09cb8d0993537698a25d4740a` 기준 누적 브랜치 `review/integration-8701-seo-rii-desktop`에 반영했습니다.
로컬 검증 소스는 `64ba35858d4f511bdd993f24eae9337880a7f2b2`다. 위 원격 코드 head에는 검증 증적만 추가했다.

## 원 PR 주장

실제 Linux SDK 컴파일·링크, Native GL 수용 검사와 시각 소비 진입점을 제공합니다. 실패·타임아웃 시 소유 프로세스 그룹을 정리합니다.

## 체리픽 추적

| 원 커밋 | 적용 커밋 |
| --- | --- |
| `c519d6d608ecf3d94d955003b37d8715dfc90eec` | `7bbe0b6b0d0ec0a7bfd0a4b3258023dd2e173c29` |
| `86729bd5146801ee696dcfa9f4e3ef09e099ee48` | `16452f31522b75acb4510f8d4b2210026c630b47` |
| `3be05add79e95a0c9b46e729ca1775c257d1010c` | `f8e91cef9fcc5efdc3cfa86408ee0954e0d1f5e0` |

## 실제 검증 범위

Linux runner 계약, 실제 GTK/GL 링크, single/tile compositor 검증, 실패 후 process-group 회수

Ubuntu x86_64에서 runner 계약 18건, 메모리 증적 계약 14건, Native 생산자 계약 31건, SDK 계약 26건을 통과했다. 동일 통합 SHA의 실제 Linux GTK 플러그인 compile/link와 GL Native 기능 응답을 확인했다.

Xvfb 5120×1600에서 실제 Linux SDK 검사 10단계를 모두 통과했다. 생성된 GL Native 라이브러리는 같은 통합 head에서 앞서 빌드한 바이트를 재사용했다.

[공통 보호문서 검토](../assets/issue8701-desktop-integration-20261119/protected-review.md)에서 Native·fresh WASM 전쪽 결과와 정본 TSV를 확인할 수 있다.

## Merge 후 contributor PR comment 계획

통합 PR의 병합 SHA와 최신 devel 반영을 확인한 뒤 원 PR에 링크를 남긴다. Ubuntu의 실제 GTK·GL SDK 링크와 Xvfb 환경의 10단계 결과, 겹친 세 검증 스크립트의 체리픽 해소 범위를 적는다.
