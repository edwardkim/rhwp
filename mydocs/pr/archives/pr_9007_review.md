# PR #9007 재검토 — 채우기 무늬 인덱스와 숨김 글자 판정

## 최종 판정

**머지 보류 — 통합 담당자의 정본 시각 보호 검증 미충족.** 기여자의 기존 세 수정 요청은 해소됐다. 현재 검증 환경에서 보호 기준을 재현하지 못한 문서가 있어 승인·병합하지 않는다. 이를 기여자의 새 결함으로 단정하거나 원본 글꼴 확보 의무를 기여자에게 전가하지 않는다.

## 접수와 변경 범위

- PR: https://github.com/edwardkim/rhwp/pull/9007, 기여자 `gitprogam`, 관련 이슈 #8978.
- 검토 head: `abc4c1e82c34e4a112473c32886703b8356089d9`; 제품 보정 commit: `7de77727cdb31c9439c2582d99a7c4575f096bd6`.
- 고정한 최신 devel: `dc3d0ff15e6e7cdb39138c72a5466994dea0e0d4`.
- 2026-11-14 KST 재조회: Open, non-draft, mergeable, CLEAN. 기여자 원 commit을 변경하지 않았다.
- 기존 인덱스 해석을 공통 enum에 맞춰 무늬 없음 -1, 무늬 0~5로 통일한다. HWPX SLASH/BACK_SLASH와 모델·직렬화·Native/SVG/Canvas·편집 UI 소비를 확인했다.
- 추가 보정은 `shape_background`의 단색 판정을 `solid.hatch().is_none()`로 바꿔 pattern 0 가로 무늬를 불투명 단색 가림으로 취급하지 않게 한다. HWP/HWPX 기준 fresh WASM TSV가 추가됐고, 59043 기존 미달은 #7445 → #7778의 이관 근거와 연결됐다.

## 실행한 검증

| 검증 | 결과 |
| --- | --- |
| `issue_8978_fill_pattern_index` | 최소 글꼴 환경 6개 통과 |
| `hidden_text_contract` | 23개 통과 |
| UI 무늬 sentinel 집중 검사 | 9개 통과 |
| fresh WASM 준비 후 Frontend 전체 | 2,784개 통과, 2개 skip, 실패 0 |
| 정확한 PR head Full CI | [run 43535321673](https://github.com/edwardkim/rhwp/actions/runs/43535321673) 성공; lint·Native Skia·기본 회귀 archive·Frontend package 포함 |
| 관련 원격 검사 | CodeQL·Render Diff·Proptest·adapter 성공 |
| 최신 devel 병합 시뮬레이션 | 충돌 없음; tree `c26a81a9560e27b56d4b2dcb6435994adb56d35e` |
| 변경 문서 링크·diff 공백 검사 | 통과 |

첫 Frontend 실행은 WASM 패키지 준비 전 모듈 누락으로 실패했으며, fresh WASM 패키지를 공급한 전체 재실행 결과를 위에 기록했다. 전체 Rust 회귀는 제품 코드 변경 없이 해당 exact-head 녹색 Full CI를 재사용했다. 로컬 집중 검사와 시각 검증은 별도로 실행했다.

## 최신 devel 대비 전쪽 비교

[전쪽 정밀 비교와 정수 집계](../assets/pr_9007_recheck_20261114/base-head-comparison.tsv), [입력/PDF 해시](../assets/pr_9007_recheck_20261114/inputs.json), [요약](../assets/pr_9007_recheck_20261114/main-summary.json).
각 backend에서 무늬 HWP 1쪽·HWPX 1쪽·hwpspec 178쪽·59043 37쪽을 base/head 모두 출력했다. 96 DPI, Print, 독립 한컴 PDF, 같은 글꼴 공급, 같은 sweep 측정 경로를 사용했다. 무늬와 hwpspec은 compat 2024, 59043은 compat 2022다.

| 입력 | Native 후보 최저 raw | fresh WASM 후보 최저 raw |
| --- | ---: | ---: |
| 무늬 HWP | 99.966083% | 99.987953% |
| 무늬 HWPX | 99.966083% | 99.987953% |
| hwpspec | 95.205645% | 95.206925% |
| 59043 | 38.340558% | 38.336022% |

두 자리 표시값의 하락과 2%p 초과 하락은 모두 0쪽이다. 정밀 분수에서는 Native hwpspec 23쪽이 99.993960258501% → 99.993885288003%, 59043 9쪽이 99.967274603834% → 99.967157479884%다. 무늬 변경으로 합집합 크기가 달라졌으며 미대응 픽셀은 각각 4개와 78개로 유지됐다. 이를 정밀 하락 0쪽으로 표현하지 않는다. WASM 정밀 하락은 0쪽이다.
59043의 90% 미만 8쪽(12·14·15·16·20·25·28·37)은 같은 base에도 존재한다. #7778에 이미 연결된 기존 미해결 범위이며 이 PR에서 시각 개선 완료로 판정하지 않는다. 기존 유효한 네 회귀 검사의 보호 범위는 유지된다.

무늬 0~5의 방향·격자와 무늬 없음, hwpspec 23쪽 RECORD HEADER의 대각 무늬·본문을 독립 PDF와 직접 대조했다. 두 경로의 무늬 방향은 일치하며 본문 내용은 보존된다. 이것은 모든 보호 문서의 직접 판독 완료를 뜻하지 않는다.

## 정본 보호 검증과 해제 조건

정본은 [final.tsv](../assets/visual-protected-baseline/final.tsv), SHA-256 `ee41455cf7f05585487d92961ccb9fd8160d5d1e15c3567019ea89e2b7bf2aeb`다. 입력 18개와 PDF 18개의 SHA는 manifest와 일치했다. 공식 두 자리 Decimal 비교 결과와 실제 조건은 [검증 증거 설명](../assets/pr_9007_recheck_20261114/README.md)에 기록한다.

Native sample16 HWPX 6·22·24·25·51쪽은 보호 100.00%에 비해 99.97·99.97·99.98·99.96·99.92%다. 동일 글꼴 조건으로 최신 devel도 다섯 쪽 모두 정확히 같은 표시값을 낸다. 수식 글꼴 HYhwpEQ·HancomEQN과 공개 Latin Modern Math를 추가 공급해도 잔차는 남았다.
fresh WASM sample16 HWPX 18·35쪽의 77.57·82.43%도 같은 조건의 최신 devel에서 재현됐다. 18쪽을 직접 대조하면 서버 구성도 배경의 채움·농도 차이가 보여 단순 수식 글꼴 차이로 단정할 수 없다. 따라서 이번 PR이 만든 회귀라는 근거는 없지만, 보호 검증 통과라는 근거도 없다. Native 2022 저장본의 본문 글꼴 차이와 일부 기존 문서의 잔차도 남아 있어 실제 글꼴·출력 조건과 원인을 더 확인해야 한다.

[정본 보호 규칙](../../manual/verification/visual_verification_governance.md#정본-보호-tsv의-공식-표시값-보존)은 “한 페이지·한 backend라도 하락하면 준비 완료·PR 생성·승인·통합을 보류”하도록 정한다. 통합 담당자가 원래 공급 조건을 재현하고 3,542행 모두 기준 이상인지 확인한 다음, 해당 차이 영역의 직접 판독과 최신 head·CI 재확인을 마쳐야 해제한다. 기준값·마스크·PDF를 바꾸거나 현재 미달 출력으로 baseline을 갱신하지 않는다.

## 생산 정보와 원격 상태

- 후보 Native SHA-256: `f34b15f58bcef1874ab9c4c720cf980165fd0d9ade90cc42410aa242a49796a7`.
- 후보 fresh WASM SHA-256: `833760f0cf544d9723a220783d573938375c78874a418a8d527cb6c4f260255b`.
- devel Native SHA-256: `3b8167c693244c26520e794886cc3626108b5387e2700a514923b29ace8da798`.
- devel fresh WASM SHA-256: `69a9f565c3691b9cda79b84be31cba3bf1f269002ce001269074bf23c24249d6`.
- 승인·병합·이슈 종료는 수행하지 않았다. 이 문서와 로컬 증거는 아직 contributor branch에 push하지 않았다.

- [재검토 COMMENT review](https://github.com/edwardkim/rhwp/pull/9007#pullrequestreview-5901900092)를 원 head에 게시하고 API로 한글·상태·commit을 재조회했다. 기존 세 인라인 스레드는 모두 해결 처리했다. 기존 Request changes review를 임의로 dismiss하거나 새 Approval로 덮지 않았다.

- 보호 전쪽 출력은 양 backend 각 1,771쪽으로 완료했으나, 공식 표시값 하락은 Native 78쪽·fresh WASM 87쪽으로 미충족이다. 이 수치를 이번 PR의 신규 회귀 수로 해석하지 않는다.
