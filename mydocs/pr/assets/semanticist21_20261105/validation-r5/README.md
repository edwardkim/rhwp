# 통합 R5 검증 기록 — 진행 중

제품 source는 `71e5e0ae2a72a6b8f51682d131077bd4df44994f`, 고정 base는 `170b886f167b705b6e630c5b34bb638d92fae0a9`다. 원 PR 18개와 새 base의 기능, 기존 PR #8389·#8397의 보호 범위를 함께 검증한다. 이 문서는 중간 수용 판정이 아니며, 모든 필수 항목을 실제로 완료한 뒤 최종 판정을 갱신한다.

| 경로 | 완료 입력 / 전체 | 완료 쪽수 | 완료 범위의 최저 raw | 상태 |
| --- | ---: | ---: | ---: | --- |
| Native | 128 / 128 | 301 | 91.81401% | 전쪽 수치 완료 |
| fresh WASM | 21 / 128 | 59 | 99.45768% | 나머지 진행 중 |

각 문서는 독립 한컴 Print PDF와 전체 쪽수를 대조한다. 96 DPI PDFium·Print 프로필·색상 경계 보정 전 2px raw를 사용한다. [입력 256행](inputs.json), [실제 측정·정밀 페이지 값](measurements.json), [원본 TSV](tsv/), [현재 수치 상태](numeric-progress.json)를 보존한다. 정확히 90%는 통과하며, 일부 높은 페이지로 다른 쪽을 상쇄하지 않는다.

원 PR의 주장 입력·페이지에 대응하는 대표 review·overlay는 [이미지 대응표](representative-images.json)와 각 `pr_N_review.md` 본문에 넣었다. raw 100%는 RGB까지 같은 픽셀 완전 일치를 뜻하지 않는다. 직접 확대 판독과 점수는 별도로 판단한다.

[최소 글꼴 집중 회귀](focused-regression.txt)는 184건 PASS다. [실제 공개 API](api/)는 fresh WASM 12그룹과 Native API 저장본 20개의 동일성을 기록한다. 이 결과가 전체 Rust·lint·Native Skia·새 base 대조·보호 문서·편람·브라우저의 완료를 뜻하지는 않는다.

원 PR head와 reviewer는 2026-11-06 재조회에서 18건 모두 같았다. 실제 upstream/devel도 고정 base와 같다. 후보·base의 실제 source와 산출물 해시는 각 producer JSON으로 구분한다.

Native·fresh WASM 검증에는 설치된 시스템 글꼴과 승인된 대체 설정을 공급하고, 회귀 실행은 저장소의 공개 글꼴 5개만 노출한 별도 최소 글꼴 환경을 사용한다. 글꼴 프로그램 파일을 이 자료에 포함하지 않는다.

갱신: 2026-11-06T04:33:44+09:00.

전체 Rust는 [목록 대조](full-rust-inventory-audit.json)에서 12,735건 PASS·45건 ignored·누락 0을 확인했다. [실제 API 전후 비교](api/README.md)와 [브라우저 검증](ui/README.md)을 별도로 보존한다. 나머지 시각·lint 게이트는 진행 중이다.
