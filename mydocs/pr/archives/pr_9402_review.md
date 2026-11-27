# PR #9402 리뷰 — 줄 삽입 시 가로 병합 상속

## 최종 판정

메인터너 보정 후 수용 가능. 보정본은 테스트 배치 규칙, 한컴 저장·인쇄, 전체 회귀를
충족했다. 기존 보호값 미달 203행의 범위를 확인한 뒤 작업지시자가 이 후보의 제출과
PR 완료 처리를 지시했다. 동일성이 입증된 203행을 이번 건에 한해 수용한다.
통합 head의 필수 CI 성공을 확인한 뒤 병합한다.

## 메인테이너 보완

기여자 `semanticist21`은 본 저장소의 읽기 권한을 가지고 있다. 메인테이너
`edwardkim`이 원 head를 재실행했지만 `src/` 회귀 4,093개가 기준 4,092개를 넘었다.
[두 번째 Lint 실패](https://github.com/edwardkim/rhwp/actions/runs/45539271682/job/140235752629)

원 기여 커밋을 보존하고 새 혼합 병합 회귀와 헬퍼 두 개를 기존 통합 모듈로 옮겼다.
이관 커밋은 `59047307c17b57f9211ced5cce6199dd4ce87485`이고, 스타일 콤보 #9453이
들어간 `devel` `3749b19583c8d3005c3a6c8f1e310b36a697b8d9`를 충돌 없이 반영한
소스는 `0d28d2fe435b95fc267955e7f788bbd08a0f00a8`이다. 원 PR의 제품 코드와 바이트가
같다. 최신 base 상대 Rust unit tier는 4,092건, integration manifest는 48개 타깃으로
통과했다. 최소 글꼴 격리에서 표 4건과 PDF 표 테두리 34건이 모두 통과했다.

### 독립 한컴 저장본과 Native·브라우저 확인

5행 4열 공개 합성 표에서 셀 B3에 캐럿을 두고 Windows 한컴오피스 2020
11.0.0.9172의 위·아래 행 삽입 명령을 각각 실행했다. 저장한 HWPX 3개를 같은 버전으로
한 장에 한 쪽씩 직접 인쇄했다(`pdf_print_method=0`).

| 방향 | 독립 저장본의 새 행 | 기존 세로 병합 | `B3` 최종 주소 |
| --- | --- | --- | --- |
| 위 | B와 C:D, 각각 3,600·7,200 HWPUNIT | A2:A4로 확장 | B4 |
| 아래 | A, B, C:D, 각각 3,600·3,600·7,200 HWPUNIT | A2:A3 유지 | B3 |

기준 `devel`은 두 방향 모두 새 행에서 C:D를 두 칸으로 나눴다. 후보는 Native CLI와 최적화한
fresh 브라우저 WASM의 실제 편집 API에서 HWP·HWPX 각각을 두 번 저장·재열기한 뒤에도 전체 셀
주소, 병합 범위, 너비와 내용을 보존했다. 독립 한컴 Print 대비 원본 및 위·아래 삽입 결과는
Native·WASM 각 3쪽 모두 96 DPI 실루엣 raw 100.00%였다. 확대 비교에서도 새 행의 분할선과
기존 텍스트의 행 소속을 확인했다.

[한컴 위 삽입 저장본](../assets/pr9402/hancom-above.hwpx) ·
[한컴 위 삽입 Print](../../../pdf/pr9402/hancom-above-2020.pdf) ·
[인쇄 영수증](../assets/pr9402/print-receipts.tsv) ·
[전쪽 TSV](../assets/pr9402/focused-print.tsv) ·
[위 삽입 Native](../assets/pr9402/above-native-direct.png) ·
[위 삽입 브라우저](../assets/pr9402/above-wasm-direct.png) ·
[아래 삽입 Native](../assets/pr9402/below-native-direct.png) ·
[아래 삽입 브라우저](../assets/pr9402/below-wasm-direct.png)

### 최신 devel의 Studio 기능 확인

별도로 들어온 #9453의 원래 E2E를 후보 WASM을 로드한 Studio에서 실행했다. 목록을 연 뒤
스크롤과 재클릭, 키보드와 바깥 클릭, 스타일 적용과 실행 취소를 모두 확인했다.

### 전체 회귀와 정본 보호

최소 글꼴 환경의 `release-test` 회귀 14,242건이 통과했다. Native Skia는 라이브러리와
CI 통합 모듈 21개에서 총 4,227건이 통과했다. Rust fmt, Native·WASM·워크스페이스 Clippy,
워크스페이스 빌드도 통과했다. 기여자 공통 보호 문서 4종은 양쪽 경로에서 통과했고,
편람 HWPX Native 383쪽은 최저 99.97%로 쪽별 하한을 모두 충족했다.

보호 정본 `final.tsv` SHA-256은
`8141acb297ce2f39148f9014e463986431d266120629e8f368665e5917138f9d`이다.
23개 원본·인쇄 PDF 및 글꼴 공급 조건을 확인했다. 비교 기준은 최신 devel과 Rust 제품 소스가
같은 직전 #9444 검증 소스 `d5c4c39006d4ea00c110eba7e99a5dc4556c981f`의 PNG와
표시값이다. 최신 devel의 Native에서 개요 번호, 중첩 표, 편람 HWPX와 sample10도
같은 조건으로 다시 출력했다.

| 경로 | 검증 쪽수 | 직전 검증과 PNG·점수 일치 | 공식 하한 미달 |
| --- | ---: | ---: | ---: |
| Native | 2,023 | 2,023 | 180 |
| fresh WASM | 2,023 | 2,023 | 23 |

[공식 하한 미달 203행](../assets/pr9402/protected-existing-below.tsv)은 #9444에서 확인한
키·쪽·하한·표시값과 동일하다. 정본 하한과 허용치는 유지한다.

## 병합 및 후속 기록

수용 [PR #9457](https://github.com/edwardkim/rhwp/pull/9457)의 제출 head
`3daba3871d0cfa508ddaca4cefd8f7ec6d370a5b`에서 [Build & Test](https://github.com/edwardkim/rhwp/actions/runs/46007155639),
[Render Diff](https://github.com/edwardkim/rhwp/actions/runs/46007155407),
[CodeQL](https://github.com/edwardkim/rhwp/actions/runs/46007155315),
[Adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/46007155512),
[Proptest roundtrip](https://github.com/edwardkim/rhwp/actions/runs/46007155389)과
[CI Impact Policy](https://github.com/edwardkim/rhwp/actions/runs/46012340294)가 모두 성공했다.
[셀프 리뷰](https://github.com/edwardkim/rhwp/pull/9457#pullrequestreview-6089310802) 뒤 일반 merge commit으로 병합했다.

병합 커밋 `dda4affdb01d776e2dd2634930849a9d30009947`의 트리는 제출본과 같다.
원 기여 커밋은 병합 이력에 포함됐고 [원 PR #9402](https://github.com/edwardkim/rhwp/pull/9402)는
GitHub에서 MERGED로 반영됐다. 두 PR의 종료 대상 이슈 목록에는 연결된 이슈가 없다.
[병합 후 nextest 갱신 실행](https://github.com/edwardkim/rhwp/actions/runs/46012584763)

## Merge 후 contributor PR comment 계획

원 PR #9402에 원 기여 수용, 테스트 이관 이유, 확인한 CI·로컬 회귀 수치를 한국어 존댓말로 남긴다.
한컴 2020의 동일 표본에서 위·아래 삽입 결과를 검증한 Native·브라우저 확대 비교 4장을
병합 SHA `dda4affdb01d776e2dd2634930849a9d30009947`의 raw URL로 고정한다. 원본을 포함한 양쪽 각 3쪽의 raw 100.00%를
[전쪽 TSV](../assets/pr9402/focused-print.tsv)에 연결한다. 기준 보호 4,046쪽은 직전과 같았고,
기존 하한 미달 203쪽을 이번 건에 한해 수용한 사실을 검토 기록에 보존한다.

## 최초 검토

## 접수 정보

| 항목 | 값 |
| --- | --- |
| 검토일 | 2026-11-27 KST |
| 원 PR | [#9402](https://github.com/edwardkim/rhwp/pull/9402), `semanticist21`, `devel` 대상, Open |
| 검토한 원 head | `dce71b1a65009de6b1b6a575ed4c76947d2b9151` |
| 실패한 CI의 PR base | `0fbee2c04ecee63cf526a9a7e61b38b32608bc3c` |
| 최신 `devel` 대조 | `3cf4ceb5f1ebca8612906a7a9324ea82a3f724fc`와 `git merge-tree --write-tree` 통과 |
| 변경과 접수 상태 | 1커밋, 4파일, +149/−16. 관련 이슈와 지정된 리뷰어 없음 |

## 변경과 검토 범위

`Table::insert_row_inheriting`은 기준 행을 덮는 가로 병합의 시작 열과 폭을 셀 이동 전에 기록한다. 새 행에서는 가로 병합 셀을 한 번 생성하며 삽입 위치를 걸치는 세로 병합만 연장한다. 편집 후 새 셀의 너비와 경계선이 달라지는 변경이다.

## 검증 입력과 결과

| 검증 | 원 head | 테스트를 옮긴 로컬 대조 |
| --- | --- | --- |
| 실제 PR base를 사용한 `rust-unit-test-tiers --check` | 실패. 전체 4,093 / 기준 4,092, 표 모듈 81 / 기준 80 | 전체 4,092, 표 모듈 80으로 통과 |
| Rust integration manifest | GitHub에서 48/48 통합 타깃 통과 | `--prepare`와 `--check --base-ref` 통과, 48/48 |
| `cargo fmt --all -- --check` | GitHub 성공 | 로컬 성공 |
| `table_merged_only_column_width` | 새 혼합 병합 검사는 모델 소스에 위치 | 기존 3건과 옮긴 혼합 병합 1건 통과 (`regression_suite_027`) |

`src/model/table/tests.rs`의 기존 서식 상속 검사는 유지했다. 새 검사 하나와 전용 헬퍼 둘을 `tests/cases/table_merged_only_column_width.rs`로 옮겨 대조했다. 5행 4열의 행별 시작 셀 수 `[4, 4, 2, 4, 4]`와 공개 `rebuild_grid()`를 사용했다. 제품 소스와 정책 파일은 원 head를 사용했다.

집중 검사는 `bwrap`에서 OS·사용자 설치 글꼴 경로를 격리하고 `RHWP_FONT_PATH`를 해제한 뒤 `release-test`로 실행했다. `fc-list`에서 보인 파일은 저장소 `ttfs/opensource/`의 `NotoSansKR-ExtraLight.ttf`, `NotoSansKR-Regular.ttf`, `NotoSansSymbols2-Regular.ttf`, `RHWPOldHangul-Regular.otf`, `RHWPPublicBatang-Regular.ttf` 다섯 개다. 전체 Rust 회귀의 로컬 재실행은 이번 CI 오류 대조 범위에서 제외했다.

최초 실패는 Lint 내부 `Validate Rust test suite manifest`의 unit-tier 검사다. 이로 인해 `Build & Test`도 실패했고 정확한 head의 `CI Impact Policy`는 `failure`였다. [Lint 원본 로그](https://github.com/edwardkim/rhwp/actions/runs/45539271682/job/138953694304) · [집계 로그](https://github.com/edwardkim/rhwp/actions/runs/45539271682/job/138964822012)

## 시각 증적과 남은 차이

작성자는 한글 12.30(Mac)에서 아래쪽으로 삽입한 뒤 저장한 HWPX의 `cellSpan`을 확인했다. 위쪽 삽입 시 시작 주소와 가로·세로 병합, 인쇄한 셀 폭·경계선은 독립 출력으로 확인해야 한다. 한컴 저장 HWPX와 Print PDF, 같은 입력의 Native·fresh WASM 쪽별 TSV 및 비교 PNG를 요청했다.

## 기여자 후속 조치

기여자가 수정 사항과 위쪽 삽입 근거를 원 PR에 제출하면 해당 커밋의 필수 CI와 실제 출력으로 재검토한다. [원 PR에 게시한 피드백](https://github.com/edwardkim/rhwp/pull/9402#issuecomment-7204592674)
