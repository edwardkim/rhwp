# PR #8376 리뷰 — fix(serializer): 비압축 HWP5의 스크립트를 파일 압축 비트대로 읽고 쓴다 (#8374)

## 최종 판정

**메인터너 보정 후 수용 가능 — 비압축 HWP5 스크립트 변환 검증 완료.** [통합 PR #8438](https://github.com/edwardkim/rhwp/pull/8438)에 반영됐으며, 검토한 원 PR head와 현재 head가 일치한다.

## 2026-11-02 통합 머지 사실과 적용 SHA 재확인

[통합 PR #8438](https://github.com/edwardkim/rhwp/pull/8438)은 `b8338ba38be288b6e9385f5372d184649323aa9f`로 머지됐다. 아래 체리픽 기록의 종전 통합 SHA는 리베이스 전 이력이다. 머지된 이력에서 원본과 커밋 제목이 일치하는 대응은 다음과 같다. 이는 변경 내용의 완전한 동등성이나 개별 시각 판정을 대신하지 않는다.

- 원본 → 머지 이력: `a4c776098e` → `7f292405de`.
- 제품 코드 `d536455b4e`의 최소 글꼴 전체 회귀는 12,368건 통과·45건 skipped였다. 이 로그에서 확인한 해당 모듈: `issue_8174_form_actions` 9건. 이 수치는 개별 PR의 독립 Print 검증과는 구분한다.
- 뒤의 중간 후보·검증 대기 문장은 당시 상태이며 최종 통합 결과가 아니다. 원 PR별 최종 시각·기능 검토와 최신 head 일치 확인은 계속한다.

## 최종 기능 검증

제품 후보 `d536455b4e`의 전체 회귀에서 `issue_8174_form_actions::uncompressed_hwp5_scripts_convert_to_hwpx_parts`가 통과했고, 같은 모듈 9건이 모두 통과했다. 원 커밋과 통합 커밋의 patch-id가 일치하며 비압축 HWP5 스크립트의 읽기·HWPX 변환 계약을 확인했다. 코드 head의 [CI](https://github.com/edwardkim/rhwp/actions/runs/41322587728)와 통합 머지를 확인했다. 이 변경의 핵심은 스크립트 스트림 저장 계약이다.

## 접수와 적용

- 원 PR: [#8376](https://github.com/edwardkim/rhwp/pull/8376), 작성자 `planet6897`, reviewer `jangster77`.
- 원 PR head 조회값: `a4c776098efa71b84e886af7d02c9048484f76cf`. 조회 시 원 PR head와 마지막 체리픽 원본이 일치한다.
- 마지막 적용 원본: `a4c776098efa71b84e886af7d02c9048484f76cf`. 검증 시작 당시 코드 후보: `abea5b028f8f135bdf000ca700e1993ebf0b1973`, 기준 `upstream/devel`: `bc9dfdd057258de59afebeb5121c4bee7398e07f`.
- 관련 이슈: [#8374](https://github.com/edwardkim/rhwp/issues/8374).
- 선행 PR: 별도 누적 선행 PR이 명시되지 않았다.

후속 `e1f03ffbb3`은 Python 테스트 준비, `af175315e7`과 `e920cba6b0`은 회귀 검사 주석, `a63bb719de`는 검증 경과를 기록했다. `5d1294811b`은 전체 그림의 잘림 오판을 보정했다. `e1b8ab0f54`, `d6fe30c3ad`, `0897582f6c`, `8c4eca265e`에 Native 진단, #8410 집중 6건 통과와 글꼴 대조를 기록했으며 새 코드의 전쪽 출력 검증이 필요하다.

## 변경 주장과 검토 관점

HWP5 스크립트 스트림을 실제 파일 압축 비트에 맞춰 읽고 대상 압축 형식으로 저장한다.
압축·비압축과 HWPX 출처 및 양식 스크립트 소비를 확인한다.

## 체리픽과 보정

- 원본 `a4c776098efa71b84e886af7d02c9048484f76cf` → 통합 `866f50582bd9fac0d737b0b21be6395e68b0c713`.

이 PR의 원본 커밋 적용 중 별도로 기록된 충돌은 없다.

기존 통합의 메인터너 보정 `3401b50256`을 새 후보의 `0e4bd3678e`로 다시 적용했다. 앞뒤 개체 여백 중복과 스크립트 압축 상태 복구는 해당 코드 경로별로 재검증한다.

## 검증과 증거

이전 통합 후보의 최소 글꼴 전체 회귀 로그에서 이 PR 관련 다음 모듈의 실행을 확인했다. 새 base 후보의 재실행 결과는 이 값으로 대신하지 않는다.
- [`issue_8174_form_actions.rs`](../../../tests/cases/issue_8174_form_actions.rs): 이전 실행 9건 통과, 0건 실패, 0건 건너뜀.

검토 후보에 포함된 입력 바이트를 Git blob과 대조했다. 아래 자료는 원 PR의 제출 증거이며 새 후보의 실행 결과를 뜻하지 않는다.
- [`hwpx_sample2.hwp`](../../../samples/hwpx_sample2.hwp) — SHA-256 `ca66d521d070ac8442e09fd3f283e17611eb9efb45eb4bf6c3a8f65e604f6b5f`.
- [`37787_regulatory_impact.hwp`](../../../samples/issue5699/37787_regulatory_impact.hwp) — SHA-256 `71496c7f481d8e7a4083fdeb9b5a1bc5dbb74f6253b8d65b448bbf12af8d7e55`.
- [`hwpers_test4_complex_table.hwp`](../../../samples/hwpers_test4_complex_table.hwp) — SHA-256 `0f09725e42a361c5695b85a4d5bcb7325b9ae709a13ef6a4e9c7b312883f6e27`.

기존 생성 목록으로 최소 글꼴 회귀 12,320건 통과·45건 건너뜀을 확인했다. 생성 목록에서 누락된 #8410 두 모듈의 6건을 목록 재작성 후 같은 환경에서 별도로 실행해 모두 통과했다. 두 실행을 구분하며 한 번의 전체 실행으로 기록하지 않는다.

2026-11-01 고정 후보의 #8389·#8397 전쪽 정밀 대조에서 2,153행 중 Native 12행, WASM 8행이 공개 기준보다 낮았다. 이 수치는 이후 GIF·Canvas 보정 전 결과이며 최종 후보의 재검증 결과가 아니다. #7905 3쪽은 GIF 오른쪽 경계 누락, sample16 35쪽 WASM은 메타파일 내부의 미대응 3픽셀이 확인됐다. #8040 두 원본은 Native·WASM 전쪽이 90% 이상이나 Native 정밀 기준 하락이 남는다. 진행과 근거는 [작업 기록](../../working/task_m100_planet6897_system_fonts_20261031.md)을 따른다. 하락 보정과 전쪽 대조, 각 PR의 독립 Print·Native/fresh WASM 및 직접 판독을 완료해야 한다. #8410·#8412의 새 base 기능도 별도로 보존 검증한다.

원 PR의 최신 head·required check와 통합 후보의 새 검증 결과를 확인하기 전에는 승인이나 merge를 진행하지 않는다.

## Merge 후 contributor PR comment 계획

통합 PR #8438의 merge `b8338ba38be288b6e9385f5372d184649323aa9f`, 코드 head CI와 해당 원 PR의 기여·메인터너 보정을 구분해 존댓말로 안내한다. 비압축 스크립트의 HWPX 변환 회귀 통과와 저장 형식 경계를 설명한다. 조판 변경이 아니므로 이미지를 사용하지 않는다. 게시 시 명시적 지시를 확인하고, 게시 뒤 API로 본문과 링크를 재조회한다.
