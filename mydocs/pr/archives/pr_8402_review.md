# PR #8402 리뷰 — fix(layout): 저장 줄 없는 문단의 어울림 그림·도형이 자기 문단과 뒤 문단의 줄을 한/글처럼 줄인다 (#8282)

## 최종 판정

**메인터너 보정 후 수용 가능 — #8282 어울림 그림·글상자 검증 완료.** [통합 PR #8438](https://github.com/edwardkim/rhwp/pull/8438)에 반영됐으며 원 PR head가 검토한 head와 일치한다.

## 2026-11-02 통합 머지 사실과 적용 SHA 재확인

[통합 PR #8438](https://github.com/edwardkim/rhwp/pull/8438)은 `b8338ba38be288b6e9385f5372d184649323aa9f`로 머지됐다. 아래 체리픽 기록의 종전 통합 SHA는 리베이스 전 이력이다. 머지된 이력에서 원본과 커밋 제목이 일치하는 대응은 다음과 같다. 이는 변경 내용의 완전한 동등성이나 개별 시각 판정을 대신하지 않는다.

- 원본 → 머지 이력: `2ba2e962ef` → `a903a2e8f4`, `625e9ab960` → `03970056a2`, `b2069a8656` → `5b48ef78cf`, `1990cf5db3` → `7eaa894507`.
- 제품 코드 `d536455b4e`의 최소 글꼴 전체 회귀는 12,368건 통과·45건 skipped였다. 이 로그에서 확인한 해당 모듈: `issue_8282_hwpx_reopen_tac_picture_square` 8건. 이 수치는 개별 PR의 독립 Print 검증과는 구분한다.
- 뒤의 중간 후보·검증 대기 문장은 당시 상태이며 최종 통합 결과가 아니다. 원 PR별 최종 시각·기능 검토와 최신 head 일치 확인은 계속한다.

## 최종 Native·fresh WASM Print 대조

제품 코드 `d536455b4e`에서 빌드한 Native 실행 파일 SHA-256 `d53e4dac28839e01f4c6bf19761c4bbc181ef70a7a38eb65abf6f6c4050b96fc`와 fresh WASM SHA-256 `3afec6e53b9374a8a6bc6f9ea23bc7c66831f8c496d3aad265120f5dfd38c0f6`으로 네 HWPX와 같은 입력의 한컴 2020 Print PDF를 96 DPI, 호환 모드 2022로 전쪽 비교했다. 네 입력은 각각 1쪽이고 양쪽 모두 raw 100.00%이며 전체 쪽수도 일치했다. Native·WASM 각 입력의 RHWP와 PDF raster는 앞서 직접 판독한 PNG와 각각 SHA-256이 일치해 기존 review·overlay를 재사용한다. 입력과 PDF의 SHA-256 및 직접 판독 근거는 [기존 증거 설명](../assets/issue_8282/integration-planet6897/README.md)에 있다.

| 입력 | Native 전쪽 TSV | fresh WASM 전쪽 TSV |
| --- | --- | --- |
| picture_overlap | [Native](../assets/issue_8282/final-integration-8438/native-picture_overlap.tsv) | [WASM](../assets/issue_8282/final-integration-8438/wasm-picture_overlap.tsv) |
| textbox | [Native](../assets/issue_8282/final-integration-8438/native-textbox.tsv) | [WASM](../assets/issue_8282/final-integration-8438/wasm-textbox.tsv) |
| textbox_left_overlap | [Native](../assets/issue_8282/final-integration-8438/native-textbox_left_overlap.tsv) | [WASM](../assets/issue_8282/final-integration-8438/wasm-textbox_left_overlap.tsv) |
| textbox_overlap | [Native](../assets/issue_8282/final-integration-8438/native-textbox_overlap.tsv) | [WASM](../assets/issue_8282/final-integration-8438/wasm-textbox_overlap.tsv) |

동일한 대표 쪽의 [Native review](../assets/issue_8282/integration-planet6897/native-review-textbox-overlap.png) · [Native overlay](../assets/issue_8282/integration-planet6897/native-overlay-textbox-overlap.png) · [WASM review](../assets/issue_8282/integration-planet6897/wasm-review-textbox-overlap.png) · [WASM overlay](../assets/issue_8282/integration-planet6897/wasm-overlay-textbox-overlap.png)에서 그림·글상자와 주변 글줄을 확인했다. 관련 회귀 8건과 [코드 head CI](https://github.com/edwardkim/rhwp/actions/runs/41322587728)가 통과했다.

## 접수와 적용

- 원 PR: [#8402](https://github.com/edwardkim/rhwp/pull/8402), 작성자 `planet6897`, reviewer `jangster77`.
- 원 PR head 조회값: `1990cf5db31d8ecf7ea512000076c5c119e8aa64`. 조회 시 원 PR head와 마지막 체리픽 원본이 일치한다.
- 마지막 적용 원본: `1990cf5db31d8ecf7ea512000076c5c119e8aa64`. 검증 시작 당시 코드 후보: `abea5b028f8f135bdf000ca700e1993ebf0b1973`, 기준 `upstream/devel`: `bc9dfdd057258de59afebeb5121c4bee7398e07f`.
- 관련 이슈: [#8282](https://github.com/edwardkim/rhwp/issues/8282).
- 선행 PR: 별도 누적 선행 PR이 명시되지 않았다.

후속 `e1f03ffbb3`은 Python 테스트 준비, `af175315e7`과 `e920cba6b0`은 회귀 검사 주석, `a63bb719de`는 검증 경과를 기록했다. `5d1294811b`은 전체 그림의 잘림 오판을 보정했다. `e1b8ab0f54`, `d6fe30c3ad`, `0897582f6c`, `8c4eca265e`에 Native 진단, #8410 집중 6건 통과와 글꼴 대조를 기록했으며 새 코드의 전쪽 출력 검증이 필요하다.

## 변경 주장과 검토 관점

저장 줄 없는 문단에서 어울림 그림·도형의 바깥 상자로 호스트와 후속 줄을 나눈다.
자기 문단·뒤 문단, 기존 저장 줄과 그림/도형 대조군의 공통 소유를 확인한다.

## 체리픽과 보정

- 원본 `2ba2e962ef3bac72d779edcf5bef188ed312ad2f` → 통합 `f55d72464d15e5fc3323b32ae6e597ff52c7569d`.
- 원본 `625e9ab96051b1711224c556e2d0cfa20faa5bc0` → 통합 `f2f12adfdfb8828c4242ae13dd92ecb82348331d`.
- 원본 `b2069a86561974d775e3b3aebbdfa434402a284c` → 통합 `0e61cbf5095f7374a22abf5ad70304fa7f5035f1`.
- 원본 `1990cf5db31d8ecf7ea512000076c5c119e8aa64` → 통합 `a32b78567da679c850d55687f1863d4d1cebd3b8`.

이 PR의 원본 커밋 적용 중 별도로 기록된 충돌은 없다.

기존 통합의 메인터너 보정 `3401b50256`을 새 후보의 `0e4bd3678e`로 다시 적용했다. 앞뒤 개체 여백 중복과 스크립트 압축 상태 복구는 해당 코드 경로별로 재검증한다.

## 검증과 증거

이전 통합 후보의 최소 글꼴 전체 회귀 로그에서 이 PR 관련 다음 모듈의 실행을 확인했다. 새 base 후보의 재실행 결과는 이 값으로 대신하지 않는다.
- [`issue_8282_hwpx_reopen_tac_picture_square.rs`](../../../tests/cases/issue_8282_hwpx_reopen_tac_picture_square.rs): 이전 실행 8건 통과, 0건 실패, 0건 건너뜀.

검토 후보에 포함된 입력 바이트를 Git blob과 대조했다. 아래 자료는 원 PR의 제출 증거이며 새 후보의 실행 결과를 뜻하지 않는다.
- [`tac_picture_square_picture_overlap-2020.pdf`](../../../pdf/issue8282/tac_picture_square_picture_overlap-2020.pdf) — SHA-256 `2ff48072e4f122e266d552396699b692f24a9d01e6a95e312ede1dcbbcd3fb9e`.
- [`tac_picture_square_textbox-2020.pdf`](../../../pdf/issue8282/tac_picture_square_textbox-2020.pdf) — SHA-256 `b0cb291e344891951cb95a1643974059203525c04ad19bd110e2f5a792776346`.
- [`tac_picture_square_textbox_left_overlap-2020.pdf`](../../../pdf/issue8282/tac_picture_square_textbox_left_overlap-2020.pdf) — SHA-256 `bfee35fd2b2a1579537442fb30a12236c094ab9b0dd5a12ef48fb66de514120a`.

기여자가 제출한 기준 자료: [`picture_overlap_silhouette.tsv`](../../../mydocs/pr/assets/issue_8282/base/native/picture_overlap_silhouette.tsv). 해당 자료만으로 새 통합 후보의 시각 검증을 통과 처리하지 않는다.

기존 생성 목록으로 최소 글꼴 회귀 12,320건 통과·45건 건너뜀을 확인했다. 생성 목록에서 누락된 #8410 두 모듈의 6건을 목록 재작성 후 같은 환경에서 별도로 실행해 모두 통과했다. 두 실행을 구분하며 한 번의 전체 실행으로 기록하지 않는다.

2026-11-01 고정 후보의 #8389·#8397 전쪽 정밀 대조에서 2,153행 중 Native 12행, WASM 8행이 공개 기준보다 낮았다. 이 수치는 이후 GIF·Canvas 보정 전 결과이며 최종 후보의 재검증 결과가 아니다. #7905 3쪽은 GIF 오른쪽 경계 누락, sample16 35쪽 WASM은 메타파일 내부의 미대응 3픽셀이 확인됐다. #8040 두 원본은 Native·WASM 전쪽이 90% 이상이나 Native 정밀 기준 하락이 남는다. 진행과 근거는 [작업 기록](../../working/task_m100_planet6897_system_fonts_20261031.md)을 따른다. 하락 보정과 전쪽 대조, 각 PR의 독립 Print·Native/fresh WASM 및 직접 판독을 완료해야 한다. #8410·#8412의 새 base 기능도 별도로 보존 검증한다.

원 PR의 최신 head·required check와 통합 후보의 새 검증 결과를 확인하기 전에는 승인이나 merge를 진행하지 않는다.

## Merge 후 contributor PR comment 계획

통합 PR #8438의 merge `b8338ba38be288b6e9385f5372d184649323aa9f`, 코드 head CI와 해당 원 PR의 기여·메인터너 보정을 구분해 존댓말로 안내한다. 어울림 그림·글상자 네 입력의 한컴 2020 Print 대비 Native/fresh WASM 전쪽 100.00%, 대표 페이지 review·overlay를 연결한다. 게시 시 명시적 지시를 확인하고, 게시 뒤 API로 본문과 링크를 재조회한다.
