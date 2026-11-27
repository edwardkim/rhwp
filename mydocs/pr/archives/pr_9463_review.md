# PR #9463 검토

[postmelee 님의 원 PR](https://github.com/edwardkim/rhwp/pull/9463)

## 검토 버전

- 원 PR head `a33ce75223a1ee7ef3754db81096278fdcf404f3`
- 기준 devel `77e4c3d1369623818e8cf3213e3fdc21d8e6502e`
- 통합 코드 후보 `b25bbad4f477a5f6c9bfc212f58b6f03e2a090dc`
- 검토 브랜치 `review/integration-8678-chart-layout-native-editing`

| 원본 커밋 | 체리픽 커밋 |
| --- | --- |
| `d88a1c4bb519dd2a54434e4ed7bac331987de115` | `715436994bb8ac28837107f9a64e781c8527abb4` |
| `8cfab0bd0282804da2d8adfa9a07db7583243f63` | `24c20151d2542f96fad179c32c0efc0a022f7a65` |

## 검토 근거

원 PR에서 구형 수식 OTF로 sample16의 세 저장본에서 Native·fresh WASM 6쪽을 각각 100.00%로 확인했습니다. 메인터너 커밋 `9a365c3d27`에서 이 OTF와 `HYHWPEQ.TTF` 1.10을 SHA-256으로 제외하고 여섯 행을 1.13 공급 기준으로 복원했습니다.

[원 PR 측정 조건](../assets/visual-protected-baseline/updates/sample16-legacy-equation-20261127/conditions.json) · [현행 공급 지침](../../verification/protected-font-supply/README.md)

## 승인된 글꼴 재측정

세 저장본의 Native·fresh WASM 각 64쪽이 모두 보호 하한을 충족했습니다. `HYHWPEQ.TTF` 1.13을 공급한 6쪽의 값은 다음과 같습니다.

| 저장본 | Native | fresh WASM |
| --- | ---: | ---: |
| HWPX | 99.97% · [비교](../assets/issue_9463/sample16-equation-113-20261128/native/sample16-hwpx/review_006.png) · [겹침](../assets/issue_9463/sample16-equation-113-20261128/native/sample16-hwpx/overlay_006.png) | 99.97% · [비교](../assets/issue_9463/sample16-equation-113-20261128/wasm/sample16-hwpx/review_006.png) · [겹침](../assets/issue_9463/sample16-equation-113-20261128/wasm/sample16-hwpx/overlay_006.png) |
| HWP 2022 | 99.96% · [비교](../assets/issue_9463/sample16-equation-113-20261128/native/sample16-hwp2022/review_006.png) · [겹침](../assets/issue_9463/sample16-equation-113-20261128/native/sample16-hwp2022/overlay_006.png) | 99.97% · [비교](../assets/issue_9463/sample16-equation-113-20261128/wasm/sample16-hwp2022/review_006.png) · [겹침](../assets/issue_9463/sample16-equation-113-20261128/wasm/sample16-hwp2022/overlay_006.png) |
| HWP 2024 | 99.97% · [비교](../assets/issue_9463/sample16-equation-113-20261128/native/sample16-hwp2024/review_006.png) · [겹침](../assets/issue_9463/sample16-equation-113-20261128/native/sample16-hwp2024/overlay_006.png) | 99.97% · [비교](../assets/issue_9463/sample16-equation-113-20261128/wasm/sample16-hwp2024/review_006.png) · [겹침](../assets/issue_9463/sample16-equation-113-20261128/wasm/sample16-hwp2024/overlay_006.png) |

검토 이미지를 구성한 두 입력 PNG는 전쪽 정본 보호 검증의 6쪽 PNG와 각각 해시가 같습니다.

[현행 보호 하한](../assets/visual-protected-baseline/final.tsv) · [보호 검증 환경 기준](../../manual/verification/visual_verification_governance.md#정본-보호-tsv의-공식-표시값-보존)

## Merge 후 contributor PR comment 계획

원 PR의 구형 수식 글꼴 발견과 실측에 감사드립니다. 현행 공급의 버전 기준과 금지 SHA-256에 맞춰 1.13으로 확인한 sample16 6쪽 세 형식의 결과를 설명합니다. 승인한 글꼴로 생성한 양 출력 경로의 review·overlay와 공식 공급표를 연결합니다.

통합 PR의 병합 SHA와 최신 head의 실제 CI를 고정합니다. 연결할 증적은 병합 SHA의 raw PNG와 대조하고, 한국어 존댓말로 원 PR에 검토와 반영 결과를 남깁니다.
