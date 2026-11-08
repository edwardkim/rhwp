# #7445 리베이스 보정 64단계: 중첩 컷과 원자 표 호스트의 물리 간격

## 기존 대상과 선행 분석

기존 입력 목록 40번 `samples/issue6776/78494-virtual-convergence-industry-decree.hwpx`를
검증했다. 확장자와 달리 실제 바이트는 한컴 2018 HWP5다. 같은 입력 SHA와 독립 한컴 2020
Print PDF 74쪽을 유지하며 compat 2022를 적용했다. 보정 대상을 확대하지 않았다.

원래의 #7418 처리 `c75bbee725d`, Latin 분리 `2f0f6ca3`, 통합 `731de9e1b4`,
로컬 C449 `e1db8d063e2e0b7f7d31ed776566e3a8137f2637` 및 이전 27단계의 실제 컷을
현재 코드와 대조했다. 리베이스 과정의 `4a7` 변경으로 셀별 일반 줄 판별이 행 전체 판별로
바뀌었고, 이어받은 중첩 컷의 예약 높이 차이를 양수로만 남기고 있었다. 기존 #7870의 TAC
원자성 및 측정·부분 paint의 호스트 종료 간격 계약도 함께 확인했다.

## 공통 원인과 수정

`src/renderer/layout/table_layout.rs`의 공통 세 경로를 보정했다. 문서명·쪽 번호·독립 PDF의
좌표를 생산 코드의 분기로 넣지 않았다.

1. `reflow_recursive_run_extra`는 선택한 원본 유닛의 전진과 실제 자식 RowCut의 물리 높이를
   비교한다. 예약 합에는 부모가 따로 더하는 continuation 시작 띠를 한 번 포함하고,
   차이는 음수도 보존한다. 이전의 양수 제한은 마지막 글줄 뒤 전진을 실제 상자에 남겨
   37쪽 부모를 6.906667 CSS px 키우고 뒤 행을 내렸다. 같은 컷이 Capacity/Occupied 및
   부분 paint에서 소비되도록 기존 질의 경로를 유지했다.
2. `reflow_cut_trailing_spacing`은 현재 셀의 일반 글줄 유닛만 판별한다. 병렬 셀의 중첩 표가
   있다는 이유로 텍스트 셀의 종료 간격까지 중첩 유닛으로 취급하지 않는다. 행 전체의
   API는 모든 소유 셀을 같은 셀별 함수로 확인한다. `시행령`은 36쪽에서 닫히고 다음 쪽에는
   `제4조, [별표2]`만 이어지는 원본 소속을 복원했다.
3. 저장 줄이 없는 원자 TAC 표의 유닛은 마지막 문단이 아닐 때 기존
   `no_ls_tac_table_host_trailing_spacing_px`를 소비한다. 행별 투영과 측정·부분 paint가 이미
   쓰는 같은 간격이다. p193 부모의 원자 11×3 표 호스트는 원본 13pt·120%에서
   3.466667 CSS px를 소유하지만 일반 atom 유닛은 이를 빠뜨려 후속 제목을 앞당겼다.
   컷 끝의 보이지 않는 간격 이송은 기존 `nested_atom_invisible_tail` 계약을 유지한다.

첫 시도인 시작 띠 보정만으로는 37쪽이 개선되지 않았다. 부호 있는 차이만 복구한 다음
후보도 셀 종료 글줄 소속과 21쪽의 원자 호스트 간격이 남아 있었다. 중간 후보를 최종
통과로 인용하지 않고 위 세 경로가 반영된 v4를 새로 빌드·검증했다.

## 실제 회귀 검사

기존 기대값을 제거하거나 완화하지 않았다. 원래 실패하던 6776 종료 높이·기준선과 7418
일반 셀 닫는 줄 검사가 통과했다. 기존 #7870 원자 표 및 #7062 호스트 높이 대조도 통과했다.

원자 호스트와 후속 제목을 보호하는 두 검사를 기존 6776 모듈에 추가했다.

- 같은 원본의 호스트 문단 스타일만 120%→100%로 복제해 후속 제목 위치의 차이가
  원본 글자 크기와 줄간격의 물리 차이와 일치하는지 검사한다. 원본 74쪽 및 같은 쪽 소속도 확인한다.
- 원본 제목의 내용·순서·단일 출현, 부모 종료 뒤 배치 및 Native Print paint의 실제 글리프
  원점을 확인한다. 독립 PDF 21쪽 text matrix `(56.609317779541016, 354.212890625)`pt를
  CSS 96으로 변환해 ±2px로 검사한다. 후보 bbox에서 기대값을 복사하지 않았다.

보존한 v3 라이브러리에 같은 두 계약을 연결한 실행은 2 FAIL, v4는 2 PASS다.
보정 전 실제 제목 기준선은 469.173333 CSS px, 독립 PDF는 472.283854 CSS px이며
보정 후는 472.64 CSS px다. 전 74쪽 양쪽 raw 90% 이상과 해당 21쪽 양쪽 98% 이상 및
review·overlay 판독을 완료한 뒤 정식 회귀를 추가했다.

최종 파일에서 fmt → manifest prepare → 실제 suite 파생 → 6776 모듈 재실행은
5 PASS / 0 FAIL, 테스트 0.875초다. manifest 검사는 1,736개 source / 7,609개 정적 test 속성,
28개 suite와 20개 예외의 48/48 target을 확인했다. unit tier는 4,101 tests / 296 modules로 통과했다.
초기 잘못된 `--base` 옵션 실행은 증거에서 제외하고 정식 `--base-ref`로 다시 확인했다.

집중 66건은 **56 PASS / 10 FAIL**, 테스트 8.203초다. 전체 nextest 결과가 아니다.
남은 실패는 5734, 4068, task1725, 1937, overlap partition 12/7, biotoxin,
roadmap Finland, 7379, 7470이다. 필수 해결·재검증을 계속한다.

## Native와 fresh WASM 전 74쪽

96 DPI, PDFium 153.0.7999.0, `pdfium_smooth_rgb_v1`, 2px 관용 raw 90%를 적용했다.
비교 전 후보는 Stage63이며 고정 devel 대비 최종 검증을 대신하지 않는다.

| 범위 | Native | fresh WASM |
| --- | ---: | ---: |
| 전 74쪽 최저 / 미달 | 95.21322% / 0쪽 | 95.21405% / 0쪽 |
| 21쪽 | 100% | 100% |
| 36쪽 | 97.21908% | 97.21106% |
| 37쪽 | 99.98294% | 99.98291% |
| 38쪽 | 100% | 100% |

보정 전 21쪽은 Native 74.77564%, 37쪽은 Native 69.68017% / WASM 69.70580%였다.
양쪽 모두 보정 전 미달은 21/37쪽이며, 보정 후 미달은 없다. 같은 backend 전후 74쪽 중
68쪽 PNG는 byte 동일하고 변경된 6쪽도 raw 하락이 없다. 다른 쪽 개선으로 하락을 상쇄하지 않았다.

전쪽 RenderTree의 텍스트 소유 경로 1,686개를 각각 비교했다. 공백·제어문자를 정규화한
내용·순서·누락·중복 차이는
없고 이미지·수식·차트·OLE의 소유 수도 같다. 페이지 소속 변화는 Table331/row2/col1의
`시행령` 문단이 36쪽에서 닫히는 것뿐이다. 입력과 독립 PDF는 바이트가 바뀌지 않았다.

보정 전 21/37쪽 및 보정 후 18/19/20/21/36/37쪽의 review와 overlay를 Native/WASM에서
직접 판독했다. 후속 제목·연도별 숫자·닫는 글줄·다음 행이 보존됐다. 최저인 19쪽의 작은
잉크·테두리 차이와 36쪽 하단의 미세 테두리 차이는 남으며, RGB 완전 일치라고 보고하지 않는다.
36쪽은 98% 미만이므로 새 픽셀 핀을 추가하지 않았다.

## 생산 출처와 증적

- 부모 HEAD: `1cd5b41b85d6cc423e630db36ca7b1ce557e7e7f`
- 고정 base: `605ccfd92f2f12f18e14182548d729f9929fda00`
- 변경된 생산 파일 SHA-256: `f75e9d4ec1ed9c5f665e48da4961cc00c3003d530f25cdbac317ec1ead4c5987`
- Native SHA-256: `d377e430f6e23604ab371e8fdd383fc5312201609a95fa831ccfbeb2c7320a95`
- fresh WASM 빌드는 246.642초, exit 0. Mac locked web/pkg/no-opt 로컬 대체 빌드다.
  Docker 최적화 통과가 아니다. root pkg/Studio public/frozen JS·WASM hash가 같다.
- 정확한 입력·PDF SHA, 전후 Native/WASM TSV 원본 4개 및 review·overlay는
  `mydocs/pr/assets/issue7445/completed-20261009/rebase-605ccfd9-20261022/recursive-cut-flow/`에 있다.
- 전체 로그·중간 후보·생산 manifest·내용 소유 감사 및 최종 JSON은 ignored
  `output/pr-review/7445-completed-20261009/rebase-8155-20261022/stage64-recursive-opening-reservation/v4/`다.
  완료한 생성 SVG만 hash manifest 후 정리했고 PNG·TSV·원본·PDF·로그는 보존했다.

## PR까지 남은 항목

이 단계는 로컬 보정 커밋이며 PR 준비 완료가 아니다. 남은 10개 집중 실패, 최신 전체 nextest,
변경 범위의 필수 lint/build/Skia/manifest/unit tiers, 승인된 문서 전체 Native/fresh WASM 비교와
실제 Studio 초기화 응답·소비 경로 검증을 완료한다. 최신 upstream/devel 확인과 필요한 리베이스
검증도 PR 전에 수행한다. 과거 PR 래칫의 자동 강제 연결은 복구하지 않는다.
