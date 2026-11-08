# PR #8356 리뷰 — fix(cli): export-hwpx --verify가 HWPX 저장의 의도된 정규화를 손실로 세지 않는다 (#8306)

## 최종 판정

**메인터너 보정 후 수용 가능 — 로컬 검증 완료.** 원 PR은 통합 후보에 순서대로 적용했으며 제품 코드 `d536455b4e`의 최소 글꼴 전체 회귀 12,368건과 Native Skia 필수 검사를 통과했다. #8389·#8397의 기존 기준 2,153행은 Native와 fresh WASM 모두 표시값 하락이 없다. [Native 대조표](../assets/planet6897_integration_final/pr8389_8397/native-display-comparison.tsv) · [WASM 대조표](../assets/planet6897_integration_final/pr8389_8397/wasm-display-comparison.tsv). 통합 PR의 최종 head CI와 merge 가능 상태는 게시 후 별도로 확인한다.

## 접수와 적용

- 리베이스 후 코드 후보: `1c375f616a16166343ad0425b8e841ed999b990a`, 기준 `upstream/devel`: `2adaa0859f94f30f7dc3b994ee27c34a872906ba`. 체리픽 대응표에는 새 SHA와 리베이스 전 SHA를 함께 기록했다.
- 원 PR: [#8356](https://github.com/edwardkim/rhwp/pull/8356), 작성자 `planet6897`, reviewer `jangster77`.
- 원 PR head 조회값: `ceb9167e3a6f9bdc564013971039130faf153677`. 조회 시 원 PR head와 마지막 체리픽 원본이 일치한다.
- 마지막 적용 원본: `ceb9167e3a6f9bdc564013971039130faf153677`. 검증 시작 당시 코드 후보: `abea5b028f8f135bdf000ca700e1993ebf0b1973`, 기준 `upstream/devel`: `bc9dfdd057258de59afebeb5121c4bee7398e07f`.
- 관련 이슈: [#8306](https://github.com/edwardkim/rhwp/issues/8306).
- 선행 PR: 별도 누적 선행 PR이 명시되지 않았다.

후속 `e1f03ffbb3`은 Python 테스트 준비, `af175315e7`과 `e920cba6b0`은 회귀 검사 주석, `a63bb719de`는 검증 경과를 기록했다. `5d1294811b`은 전체 그림의 잘림 오판을 보정했다. `e1b8ab0f54`, `d6fe30c3ad`, `0897582f6c`, `8c4eca265e`에 Native 진단, #8410 집중 6건 통과와 글꼴 대조를 기록했으며 새 코드의 전쪽 출력 검증이 필요하다.

## 2026-11-01 리베이스 후 전체 회귀

HWPX 정규화 관련 `issue_8306` 검사 5건은 모두 통과했다. 최소 시스템 글꼴 환경에서 라이선스가 확인된 저장소 테스트 글꼴을 명시적으로 제공해 전체 12,349건을 실행했다. 12,347건이 통과했고 편람 261쪽 JPEG 채움 색상 HWP·HWPX 검사 2건이 실패했다. 두 실패는 오래된 JPEG 인코더 캐시를 재컴파일한 뒤 집중 검사에서 모두 통과했다 ([재검사 로그](../../../output/pr-review/planet6897-ordered-20261031/jpeg-clean-cache-focused.log)). 충돌한 PDF 세션의 HWP·HWPX 메타데이터 예산 검사는 모두 통과했다. [전체 로그](../../../output/pr-review/planet6897-ordered-20261031/rebase-full-minfont-verified-nextest.log).

제품 코드 커밋 `d536455b4e`에서 최소 글꼴 환경의 전체 회귀 12,368건은 모두 통과했다. 이후 변경은 테스트 이동과 서식 정리이며 이동한 검사만 재실행한다. #8389·#8397 기준 1,770쪽, 원 기준 TSV 2,153행의 Native·fresh WASM 전쪽 대조에서 표시값 하락은 없었다.

## 원 PR 주장과 시각 증거 적용 여부

이 PR은 `export-hwpx --verify`의 비교 기준을 정규화하며 저장 출력이나 조판을 변경하지 않는다. 따라서 이 PR의 주장에 대응하는 페이지 비교는 적용하지 않고, 실제 손실 검출 반례와 CLI 종료 코드 검사를 사용한다.

## 변경 주장과 검토 관점

HWPX 내보내기 검증에서 저장기의 의도된 정규화를 손실로 잘못 세지 않도록 비교한다.
쪽 번호 접기·합성 줄 정규화와 실제 내용 손실 검출을 함께 확인한다.

## 체리픽과 보정

- 원본 `ceb9167e3a6f9bdc564013971039130faf153677` → 통합 `53e2692b86a9e8505e7c71b20dd3e4e312e4b373` (이전 통합 `6db76bcdb9dbfd341534f2ecf142a4f82ecff653`, 그 이전 `9f098c382a8a1fbe45d50da3bc623abacef87702`).

`src/cli/commands/conversion.rs` 충돌은 이전 보정 당시와 양쪽 Git blob이 동일해 기존 해결본을 재사용했다.

## 검증과 증거

이전 통합 후보의 최소 글꼴 전체 회귀 로그에서 이 PR 관련 다음 모듈의 실행을 확인했다. 새 base 후보의 재실행 결과는 이 값으로 대신하지 않는다.
- [`issue_8306_hwpx_verify_normalization.rs`](../../../tests/cases/issue_8306_hwpx_verify_normalization.rs): 이전 실행 5건 통과, 0건 실패, 0건 건너뜀.

기존 생성 목록으로 최소 글꼴 회귀 12,320건 통과·45건 건너뜀을 확인했다. 생성 목록에서 누락된 #8410 두 모듈의 6건을 목록 재작성 후 같은 환경에서 별도로 실행해 모두 통과했다. 두 실행을 구분하며 한 번의 전체 실행으로 기록하지 않는다.

2026-11-01 고정 후보의 #8389·#8397 전쪽 정밀 대조에서 2,153행 중 Native 12행, WASM 8행이 공개 기준보다 낮았다. 이 수치는 이후 GIF·Canvas 보정 전 결과이며 최종 후보의 재검증 결과가 아니다. #7905 3쪽은 GIF 오른쪽 경계 누락, sample16 35쪽 WASM은 메타파일 내부의 미대응 3픽셀이 확인됐다. #8040 두 원본은 Native·WASM 전쪽이 90% 이상이다. Native 2쪽의 정밀값은 미세하게 낮지만, 사용자 지정 표시값 기준으로는 기존과 현재 모두 100.00%여서 하락이 아니다. 진행과 근거는 [작업 기록](../../working/task_m100_planet6897_system_fonts_20261031.md)을 따른다. 하락 보정과 전쪽 대조, 각 PR의 독립 Print·Native/fresh WASM 및 직접 판독을 완료해야 한다. #8410·#8412의 새 base 기능도 별도로 보존 검증한다.

원 PR의 최신 head·required check와 통합 후보의 새 검증 결과를 확인하기 전에는 승인이나 merge를 진행하지 않는다.

## Merge 후 contributor PR comment 계획

[통합 PR #8438](https://github.com/edwardkim/rhwp/pull/8438), merge `b8338ba38be288b6e9385f5372d184649323aa9f`와 [코드 head CI](https://github.com/edwardkim/rhwp/actions/runs/41322587728)를 연결한다. 저장 정규화와 실제 손실 검출을 함께 검증했다. 조판 출력 변경이 아니므로 시각 이미지를 첨부하지 않는다. 감사와 원 기여, 메인터너 보정 범위를 존댓말로 설명하고 게시 뒤 API로 확인한다.
