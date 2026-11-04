# PR #8477 리뷰 — 그림 칸의 저장 프레임·정렬과 분할 소유 보정

## 최종 판정

**메인터너 보정 후 수용 가능**

원 PR의 그림·앵커 소유 보정을 유지하고, 첫 분할 프레임과 각 그림 셀의 가운데 정렬 소비를 메인터너가 보정했습니다. 아래 두 쪽의 Native/fresh WASM 출력과 직접 판독을 완료했습니다. 통합 전체 회귀·lint·새 base 보존 검증은 진행 중이므로 통합 PR 준비 완료나 원 PR 직접 병합을 선언하지 않습니다. PR 생성은 작업지시자 승인 대기입니다.

## 접수·검토 코드

| 항목 | 값 |
| --- | --- |
| 원 PR | [PR #8477](https://github.com/edwardkim/rhwp/pull/8477) · `planet6897` |
| 원 head | `d53da96d75daf912d3e53d9137eadc7770732bbf` |
| 리베이스 후 원 PR 적용 | `c3320fc829, cb04b95295, 3df53c930f, cff3d2c355` |
| 추가 제품 보정 | `9c5720da81`, `40778b67d6` |
| 제품 소스 | `40778b67d6f69ccd3ee86cde84818dc2deea2cff` |
| 기준 devel | `1924d371d6757392d063b8dc2e9bbe9a298be817` |
| 관련 이슈 | [이슈 #8472](https://github.com/edwardkim/rhwp/issues/8472) |
| reviewer 상태 | `jangster77` 요청 · 2026-11-04 원 head 불변·Open·non-Draft 재확인 |
| 기본 경로 | `maintainer_general` |
| 보조 경로 | `intake_and_review`, `local_validation`, `visual_fixture_evidence`, `multi_pr_update_branch`, `post_merge` |

## 검증 입력과 독립 Print

입력은 [cell_picture_row_split.hwp](../../samples/issue8472/cell_picture_row_split.hwp), SHA-256 `d0b00c7c54154b5aed971ba28b5dbc3389b0f972c33bc2c5fb2a8c1b2b469c40`입니다. 기존 저장소의 동일 원본을 사용했습니다.

기준은 [issue8472_cell_picture_row_split-2020.pdf](../../pdf/issue8472_cell_picture_row_split-2020.pdf), SHA-256 `34da9c3790e26a66f61b6f2fbadd438c8a97fea266536e91a6a46742be070848`입니다. 파일명과 달리 선행 작성자가 한컴 2022(12.0.0.4547) Print로 기록한 기존 두 쪽 PDF이며 이번에 재산출하지 않았습니다. 원 보고서 전체를 검증했다고 확대하지 않고 원 PR의 정식 재현 입력 두 쪽을 판정합니다.

## 전쪽 측정과 주장별 증적

2026-11-04, 96 DPI, Print profile, compat 2022, 색상 경계 보정 전 2px raw입니다. 전체 쪽수는 원본·PDF·두 출력 경로 모두 2쪽으로 일치하고 누락·90% 미만 쪽은 없습니다.

| 페이지·주장 | Native raw·review / overlay | fresh WASM raw·review / overlay |
| --- | --- | --- |
| 1쪽 · 온전한 앞 차트 표와 뒤 표 첫 조각·라벨 정렬 | 100.00% · [review](assets/planet6897_20261103/pr_8477/pr8477-claim-p001-cell_picture_row_s-native-review.png) / [overlay](assets/planet6897_20261103/pr_8477/pr8477-claim-p001-cell_picture_row_s-native-overlay.png) | 99.99% · [review](assets/planet6897_20261103/pr_8477/pr8477-claim-p001-cell_picture_row_s-wasm-review.png) / [overlay](assets/planet6897_20261103/pr_8477/pr8477-claim-p001-cell_picture_row_s-wasm-overlay.png) |
| 2쪽 · 그림·빈 앵커를 함께 이어받고 각 셀의 높이로 가운데 정렬 | 100.00% · [review](assets/planet6897_20261103/pr_8477/pr8477-claim-p002-cell_picture_row_s-native-review.png) / [overlay](assets/planet6897_20261103/pr_8477/pr8477-claim-p002-cell_picture_row_s-native-overlay.png) | 100.00% · [review](assets/planet6897_20261103/pr_8477/pr8477-claim-p002-cell_picture_row_s-wasm-review.png) / [overlay](assets/planet6897_20261103/pr_8477/pr8477-claim-p002-cell_picture_row_s-wasm-overlay.png) |

전쪽 원본 TSV: [Native](assets/planet6897_20261103/pr_8477/pr8477-cell_picture_row_split-f5c3c377-native.tsv) · [fresh WASM](assets/planet6897_20261103/pr_8477/pr8477-cell_picture_row_split-f5c3c377-wasm.tsv). 전쪽 최저는 각각 **100.00% / 99.99%**입니다.

실제 시각 판독에서는 첫 쪽 표의 높이·라벨 세 줄씩·앞 그림 두 개의 정렬, 두 번째 쪽의 그림 두 개와 표 내부 포함·라벨 중복 없음·그림 위변 차이를 확인했습니다. 2px raw 100%에서도 남았던 약 1px 위변 차이는 별도로 보정했습니다.

## 조판 원칙과 공통 소비 경로

[분석 문서](../working/task_m100_8472_declared_opening_frame.md)에 원본 저장값·독립 PDF·선행 PR #8059와 생산·소비 지점을 기록했습니다.

- 공통 첫 조각 생산자 `declared_first_frame.rs`가 원본 선언·실제 컷·이월된 그림 앵커를 인증합니다. scan의 예약 높이와 부분 표 paint가 같은 물리 프레임을 소비합니다.
- 가운데 정렬은 공통 `CellCutFrame::content_height`의 안여백 제외 내용을 소비합니다. 온전한 그림 셀도 기존 저장 그림 프레임 생산자의 셀별 내용을 사용합니다.
- 이어받기는 그림과 빈 앵커를 이미 함께 소유한 유닛의 내용 높이를 재사용합니다. 빈 앵커 줄을 다시 더하거나 출력 좌표를 clamp하지 않습니다.
- 편집·재조판, 중첩 표, 원본 vpos 리셋, 독립 최소를 가진 행의 기존 계약은 유지합니다. PR #7518·#7870·#8060 집중 대조군을 함께 검사했습니다.

## 회귀 검사와 독립 좌표

[정식 회귀 파일](../../tests/cases/issue_8472_cell_picture_anchor.rs)은 원본 선언과 남은 본문, 라벨 중심·내용 보존, 셀별 저장 앵커와 내용 높이의 상대 차이, 그림·앵커의 같은 조각 소유와 셀 내부 포함을 검사합니다.

양쪽 raw가 100.00%인 2쪽만 독립 PDF 이미지 사각형의 네 경계를 ±1 CSSpx로 추가 보호합니다. 관계 검사는 유지하며 특정 설치 글꼴·OTF를 요구하지 않습니다. 실제 최소 글꼴 노출 목록과 같은 좌표를 확인했습니다. [독립 PDF·최소 글꼴 경계 기록](assets/planet6897_20261103/pr_8477/pr8477-p2-independent-image-bounds.json)을 근거로 사용합니다.

수정 전 새 관계 검사는 실제 첫 프레임 60.26667px / 원본 선언 198.62667px와 이어받기 그림 상대차 0 / 원본 상대차 −1.27333px로 실패했습니다. 제품 보정 후 최소 글꼴 집중 40건이 모두 통과했습니다. 최종 좌표 검사를 포함한 이 파일 6건도 최소 글꼴 환경에서 모두 통과했습니다. 이는 최종 전체 회귀의 대체가 아닙니다.

## 빌드·측정의 출처

Native 바이너리 SHA-256은 `6d556ea3b231bd1976303978848045d05141b4d3d4404c689764ea72fcc69b7d`, fresh WASM은 `09c68917e431c2baf5ab7cf9612a6802d4557cb6229f23911e74aff0c1f9f6da`입니다. JS는 `bb9be9d806756697eddee5d185e80e9256bd0d97947d78a94777222100f12393`입니다.

Native는 보정 소스를 미커밋 상태로 빌드한 뒤 같은 내용으로 커밋했습니다. 실행 당시 호출 worktree의 Git SHA와 제품 소스를 혼동하지 않도록 [제품 소유 확인](assets/planet6897_20261103/pr_8477/owned-frame-build-primary-ownership.json)에 1,727개 Rust 파일의 동일성을 기록했습니다. 기존 manifest의 실행 SHA는 정정하지 않고 보존합니다. 실제 WASM의 새 base paint-font API와 글상자 입력의 열기 성공도 확인했습니다.

[Native manifest](assets/planet6897_20261103/pr_8477/owned-frame-native-manifest.json) · [WASM manifest](assets/planet6897_20261103/pr_8477/owned-frame-wasm-manifest.json) · [증적 SHA-256](assets/planet6897_20261103/pr_8477/owned-frame-asset-hashes.json). 설치 글꼴은 측정에만 사용하고 글꼴 파일은 커밋하지 않습니다.

재현은 저장소 루트에서 `venv/bin/python scripts/visual_sweep.py --silhouette-only --hwp samples/issue8472/cell_picture_row_split.hwp --pdf pdf/issue8472_cell_picture_row_split-2020.pdf --key pr8477-owned-frame --rhwp-bin <검증한 Native 바이너리> --compat 2022 --dpi 96 --page-workers 1 --out <검증 폴더>`로 수행합니다. WASM은 같은 명령에 `--wasm-pkg <검증한 fresh pkg>`를 추가합니다. 정확한 원 명령·입력·환경·해시는 위 manifest와 분석 기록에 연결합니다.

## 통합 전 남은 조건

현재 통합의 전체 Rust·Skia 3종·lint, 필수 편람 Native 383쪽 gate, 새 devel 대비 0.1%p 초과 하락 금지, PR #8389·#8397의 원 TSV 표시값 무하락과 각 원 PR의 대응 증적 갱신을 완료해야 합니다. GitHub CI·원격 게시·통합 승인 조건은 별도이며 PR 생성은 승인 대기합니다.

## Merge 후 contributor PR comment 계획

통합 완료가 승인되고 실제 merge SHA가 확정되면 위 동일 입력의 1·2쪽 Native/fresh WASM review·overlay와 전체 TSV를 첫 comment부터 그 merge SHA의 raw URL로 표시합니다. 다른 PR의 대표 이미지로 대체하지 않습니다. 게시 전에 이미지 응답·형식·해시를 위 자산과 대조하고 게시 후 API 본문과 화면을 확인합니다. 원 PR 닫기는 통합 SHA의 devel 포함과 모든 후속 조건을 확인한 뒤 승인 범위에 따라 처리합니다.
