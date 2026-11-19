# PR #9166 검토 · Native 설치 글꼴로 누락 한자 복원

**검토 완료.** [통합 PR #9189](https://github.com/edwardkim/rhwp/pull/9189) 코드 head `5b3e013dfb49e82f24fc769f81170e2310adf612`의 Full CI와 적용 대상 검사가 모두 통과했다. [CI](https://github.com/edwardkim/rhwp/actions/runs/44470122043) · [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/44470122101) · [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/44470121749) · [Adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/44470122018) · [Proptest roundtrip](https://github.com/edwardkim/rhwp/actions/runs/44470122039) · [CI Impact Policy Controller](https://github.com/edwardkim/rhwp/actions/runs/44470121845)

이번 검토 기록을 반영한 head의 필수 검사를 확인한 뒤 병합한다.

[원 PR](https://github.com/edwardkim/rhwp/pull/9166) head `9e8b05f62b985d245ea1143e62c6cdfb27e1baec`를
`c526dd402bcf55d09cb8d0993537698a25d4740a` 기준 누적 브랜치 `review/integration-8701-seo-rii-desktop`에 반영했습니다.
로컬 검증 소스는 `64ba35858d4f511bdd993f24eae9337880a7f2b2`다. 위 원격 코드 head에는 검증 증적만 추가했다.

## 원 PR 주장

명시 공급 후보의 원래 순서를 유지하고, 실제 글리프가 없을 때에만 플랫폼 문자 fallback으로 탐색합니다. 렌더링과 글꼴 진단이 같은 resolver를 사용합니다.

## 체리픽 추적

| 원 커밋 | 적용 커밋 |
| --- | --- |
| `2c04978e957aeddb6030a16e9f1a11ffec3b6c08` | `7b1c7f21c3e5470144df164d3d2e34b5822c2781` |
| `739f9589c74d4b65013b5284c50676c7c913c17c` | `b5b668f6c844a18a46e1d8cde0d7ca5520d98cb2` |
| `8ab05cd1f64fc204c38f2ade798584a8e4fc23f1` | `64ba35858d4f511bdd993f24eae9337880a7f2b2` |

## 실제 검증 범위

Linux 전용 Fontconfig 공개 fixture 양성·음성 대조, 고정 base/후보 Native 및 fresh WASM 두 쪽 독립 한컴 Print, 보호 문서 검증

## 독립 한컴 Print

같은 두 쪽 HWP와 Hancom PDF가 생산한 한컴 2022 데스크톱 Print를 재사용했다. 96 DPI에서 Native 99.66%·100.00%, fresh WASM 99.67%·100.00%였고, 첫 쪽의 한자 여섯 글자와 양쪽 표 경계를 확인했다. 설치 Serif만 노출한 원 작성자의 Linux 재현 조건과 Mac 전체 문서 검증은 구분한다.

통합 fresh WASM의 JS와 바이너리 두 SHA가 보호 검증에서 수용된 WASM과 같았다. 23문서·2,023쪽의 기존 WASM TSV 해시와 현 정본의 각 하한을 모두 재대조했다. 편람 HWPX Native 공개 웹폰트 경로는 KoPub 바탕·돋움의 Light·Medium·Bold 여섯 TTF와 함초롬바탕 공급으로 383쪽을 검증했다. 정본 하락은 0쪽, 최저 99.97%였다.

Ubuntu x86_64의 통합 head에서 suite 생성 결과에 따라 `regression_suite_014`로 실제 Fontconfig 설치 Serif 검사를 실행했다. 공개 글꼴 fixture의 양성·음성 대조 1건이 통과했다.

정본 보호 Native 23종·2,023쪽에서 하락·상승이 모두 0쪽이었다. 수용 조건으로 다시 측정한 sample10의 474쪽은 100.00%였고, 전 763쪽 TSV는 기존 저장 원본과 바이트가 같았다. [Native 전쪽 검증 증적](../assets/issue8701-desktop-integration-20261119/protected-native-validation.json)

고정 devel과 같은 입력·글꼴 조건으로 다시 빌드해 2쪽을 비교했다. Native와 fresh WASM 각각의 PNG·TSV가 통합 후보와 바이트 단위로 일치했다. Linux Fontconfig 격리 환경에서는 명시 후보 글꼴에 없는 글리프를 설치 Serif에서 찾는 동작을 확인했다. [고정 devel 대조](../assets/issue8701-desktop-integration-20261119/p136-devel-comparison.json).

[공통 보호문서 검토](../assets/issue8701-desktop-integration-20261119/protected-review.md)에서 Native·fresh WASM 전쪽 결과와 정본 TSV를 확인할 수 있다.

## Merge 후 contributor PR comment 계획

통합 PR의 병합 SHA와 최신 devel 반영을 확인한 뒤 원 PR에 링크를 남긴다. 원 PR의 P136 가족관계 등록부 HWP 1쪽 양쪽 표의 한자 6글자를 대조한다. 통합 결과의 Native(99.66%)·fresh WASM(99.67%) 같은 1쪽 review와 overlay 4개를 넣고, 저장소의 2쪽 전쪽 TSV와 데스크톱 한컴 2022 Hancom PDF Print 출처를 연결한다. 체리픽 충돌을 해소하며 #9177의 폰트 테스트와 P136 회귀를 함께 보존한 근거를 적는다.
