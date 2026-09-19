---
kind: snapshot
status: historical
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-19
---

# PR #7277 검토

## 최종 판정

**승인.** 본문 넘침 허용치 두 항목을 1에서 0으로 낮추는 변경은 현재 실측과 CI에 부합한다.
렌더링 동작을 바꾸거나 기존 실패를 허용하는 변경이 아니다. 비차단 주석 경로 오류 1건은 아래에 기록한다.
전체 PDF 일치를 승인한 것이 아니며, GitHub approve·merge는 수행하지 않았다. 사용자 지시에 따라 아래 기록을 원 PR에 직접 push하는 trailing 문서 commit으로 준비했다.

## 접수와 검토 범위

- [원 PR #7277](https://github.com/edwardkim/rhwp/pull/7277), 작성자 planet6897, base devel, nondraft.
- 원 head `7ef460076e9e4e3415e524e142c1c4092a1aca16`, 원 branch `fix/7234-scaffold-tall-table-ratchet`.
- reviewer jangster77 지정. maintainerCanModify=true. 마지막 조회 OPEN / MERGEABLE / CLEAN.
- base route: collaborator_external_pr. modifiers: intake_and_review, local_validation, visual_fixture_evidence.
  해당 자식 문서와 pr_review_workflow·선택표·시각 검증 거버넌스·Visual Sweep 가이드를 읽었다.
- 최신 base `2a662a54b81ad4ba4626dd1759d6f66fb003f86a` 위 로컬 `review/pr7277-20260919`.
  원 commit을 -x로 체리픽한 검토 head `fc577ca9ffde1a0b4e71deaf79774f648ea6d9ff`.
- 원 PR 직접 push 단계에서는 같은 로컬 branch를 원 source head `7ef460076e9e4e3415e524e142c1c4092a1aca16` 위로 정렬했다.
  앞선 `fc577ca9f`는 로컬 통합 검증 대상이며 원 PR에 push하지 않는다. code candidate 뒤에는 검토·증적·오늘할일만 추가한다.
- source head와 최신 base의 merge tree `e6a4c198f4e1c0a7f3cb6fb7483af4d2a705fd61`, 충돌 없음.
- 실제 diff: `tests/fixtures/body_overflow_baseline.tsv` 한 파일, +6/-4. 숫자 변경은 아래 두 항목뿐이다.

| 입력 | 기존 상한 | 제안 상한 | 로컬 실측 |
| --- | ---: | ---: | ---: |
| samples/issue7216/tall_table_before.hwpx | 1 | 0 | 0 |
| samples/issue7216/tall_table_after.hwpx | 1 | 0 | 0 |

## 비차단 검토 의견

**[P3] baseline 주석의 테스트 경로를 실제 파일로 정정.**
`tests/fixtures/body_overflow_baseline.tsv:227`이 가리키는
`tests/cases/issue_7234_scaffold_tall_table_row_split.rs`는 source head와 최신 devel 모두에 없다.
실제 파일은 [issue_7234_scaffold_table_cell_height.rs](../../../tests/cases/issue_7234_scaffold_table_cell_height.rs)다.
PR 본문에는 올바른 파일명이 있다. 주석이므로 테스트 실행·래칫 효과에는 영향이 없으나,
향후 컷 위치 계약을 추적할 때 잘못된 경로로 안내한다. 이번 검토에서 원 코드는 수정하지 않았다.

## 검증 결과

원 code head의 [Full CI](https://github.com/edwardkim/rhwp/actions/runs/35439874907)는 SUCCESS이며
run headSha가 `7ef460076e9e4e3415e524e142c1c4092a1aca16`과 일치한다.
실제 로그에서 `body_overflow_does_not_grow_partition_0`~`15`의 **16 PASS**와
`issue_7234_scaffold_table_cell_height`의 **4 PASS**를 확인했다.
Lint 및 Native Skia, [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/35439874913),
[Adapter](https://github.com/edwardkim/rhwp/actions/runs/35439874878),
[Proptest](https://github.com/edwardkim/rhwp/actions/runs/35439874863)도 성공했다.
원 head의 CI와 최신 base 위 로컬 검증은 별도 증거로 기록한다.

로컬에서는 Cargo 전체 회귀·Clippy·WASM 빌드를 다시 실행하지 않았다. Rust source/test helper 변경이
없고, 현재 검토 head의 실행 코드·Cargo 입력은 직전 검증 빌드 source
`e93d7f23684396163876c2a10733f912472fe932`와 Git diff가 없다.
출처와 아래 해시가 일치하는 Native/WASM 빌드를 재사용해 **이번 입력을 새로 실행·캡처**했다.
이번 PR에서 새로 빌드한 fresh WASM이라고 보고하지 않는다.

- Native SHA256: `ded73871ffae61ed1161a26767d4cd843c4bf52d7d4d51b5fc93363a2da60d54`
- rhwp.js SHA256: `a7353a7603b7e07db2d33ff93fff6b213ea79e01da91c190cbb607e752c6b5a7`
- rhwp_bg.wasm SHA256: `ec21e0b957c22a17170a4448dc6a87e3ffc843ffb82b96a849b85804444bd1d3`

실행 명령(두 HWPX에 각각 실행):

```sh
<verified-rhwp> layout-anomaly samples/issue7216/tall_table_before.hwpx --json
<verified-rhwp> dump-pages samples/issue7216/tall_table_before.hwpx
<verified-rhwp> layout-anomaly samples/issue7216/tall_table_after.hwpx --json
<verified-rhwp> dump-pages samples/issue7216/tall_table_after.hwpx
```

네 명령 모두 exit 0. 두 입력 모두 pageCount=2, overflowCount=0, offCanvasCount=0,
textOverlapCount=0. CLI 기본 공차 1px에서도 0이며, baseline 소비자는 `over_bottom > 2px`만 센다.
두 입력 모두 첫 조각 rows=0..47, 이어받은 조각 rows=47..61이고 두 페이지 RenderTree도 입력 간 같다.
Native와 WASM의 첫 열을 별도로 읽어 1쪽 1~46, 2쪽 47~60을 확인했다. 제목행을 포함해 47+14=61행이며
중복·누락이 없고, 2쪽의 표 다음 문단이 보존된다. 기존 회귀 검사도 이 의미를 직접 검사한다.
이번 TSV 변경은 같은 실제 출력 0에 대해 허용 범위를 줄인다. 코드 결함 수정의 red/green으로 포장하지 않는다.

## 입력·기준 PDF 출처와 독립 검증

다음 파일은 모두 기존 tracked 파일을 재사용했다. 원 PR commit의 blob과 실행 파일의 바이트 일치를 확인했다.
중복 HWPX/PDF를 추가하지 않았다.

| 파일 | SHA256 |
| --- | --- |
| [tall_table_before.hwpx](../../../samples/issue7216/tall_table_before.hwpx) | `82386dde2f8bc8b47a212fda9c623cf7094a6a1fb385be426e334f6990aa9df4` |
| [tall_table_after.hwpx](../../../samples/issue7216/tall_table_after.hwpx) | `ba2ec290b1406476431c9be2421147d7c500f614138f94b5cde78a4158409f9f` |
| [tall_table_after-2020.pdf](../../../samples/issue7216/tall_table_after-2020.pdf) | `9610f80615236c298575c34db74819a4407e330be4322aef0c504fb8d1913ca7` |

PDF SHA1 `3c597022f9646d7126dacac26a366fbd28cade70`, 2쪽, 595×841pt,
Creator `Hwp 2020 0.0.0.0`, Producer `Hancom PDF 1.3.0.550`, PDF 1.4.
기존 #7216/#7234 기준 자료다. 메타데이터를 이유로 재변환하지 않았다.
PyMuPDF get_drawings로 너비 400px 초과·높이 1.5px 미만의 가로선 y를 중복 제거해 독립적으로 확인했다.

| 쪽 | 기준 PDF 가로선 | 행 | PDF 윗변/아랫변(px, 96dpi) | rhwp 표 윗변/아랫변(px) |
| --- | ---: | ---: | --- | --- |
| 1 | 48 | 47 | 199.621 / 1002.104 | 194.0 / 997.6 |
| 2 | 15 | 14 | 136.011 / 374.949 | 132.3 / 371.8 |

본문 바닥은 두 쪽 모두 약 1009.1px다. 표 바닥은 그 안에 있다.
PDF 페이지 크기 보정 없이 실제 96dpi 좌표를 표기했다. PR 본문의 척도 보정 수치와 혼합하지 않는다.
`18a9fa85e`는 선행 통합 변경을 포함하지만, 이번 검토에서 이등분 탐색으로 원인 commit을 새로 확정하지 않았다.

## Visual Sweep 직접 판독

[Visual Sweep 정본](../../manual/verification/visual_sweep_guide.md#github-merge-comment)에 따라
Native와 실제 Chrome WASM 각각 1~2쪽 compare·standalone overlay·review를 새로 산출했다.

```sh
venv/bin/python scripts/visual_sweep.py   --file-target after samples/issue7216/tall_table_after.hwpx   samples/issue7216/tall_table_after-2020.pdf   --rhwp-bin <verified-rhwp> --pages 1,2 --dpi 96 --out <scratch>/native
# WASM은 위 명령에 --wasm-pkg <verified-wasm4>를 추가하고 out을 <scratch>/wasm으로 지정
```

스크래치: `/private/tmp/rhwp-pr7277-review-20260919`.
Native 바이너리와 WASM package는 `/private/tmp/rhwp-planet-7272-7274-20260919`의
`rhwp-maintainer4`, `wasm4`이며 SHA256과 코드 입력 동등성을 위에서 고정했다.
두 sweep 모두 exit 0, complete, flagged 0/2. Native/WASM raw PNG는 **2/2 바이트 동일**이다.
Native review 패널 두 장을 직접 열어 PDF와 표 경계·마지막 행·다음 문단을 판독했다.
동일한 WASM 출력은 해시로 대조했으며, 재사용한 이전 캡처를 이번 증적으로 세지 않는다.

| 쪽 | pixel match | 내용 픽셀 자동 일치율 보조값 | 직접 판독 |
| --- | ---: | ---: | --- |
| 1 | 88.84185% | 8.01058% | 제목행+1~46, 본문 바닥 안. 표가 PDF보다 약 5.6px 위이며 글꼴/선 차이 잔존 |
| 2 | 96.63763% | 7.66838% | 47~60과 다음 문단 보존. 표가 PDF보다 약 3.7px 위 |

높을수록 좋음: 기준 PDF와 rhwp PNG가 더 비슷함.
낮을수록 나쁨/검토 필요: 잉크 위치나 형태 차이가 큼.
단, 사람 판정 정확도가 아니라 내용 픽셀 중심 자동 일치율 보조값이다.
낮은 잉크 일치율과 flagged 0은 양립한다. 이 결과를 전체 시각 일치나 원점 문제 해결로 승인하지 않는다.
이번 수용 범위는 기존 넘침 감소를 래칫에 반영하는 것뿐이다.

## 공통 조판 원칙 심사

| 항목 | 판정 | 근거 |
| --- | --- | --- |
| 구현 근거와 범위 | 충족 | 1→0으로 허용 범위를 축소. 렌더러 분기·clamp 추가 없음 |
| 측정과 배치의 공통 결과 | 비해당 | 이번 diff는 TSV와 주석뿐. 기존 의미는 focused CI 및 실제 좌표로 대조 |
| 분할·이어받기 계약 | 충족 | 기존 47/14행 분할, 첫 열 1~60 보존, 후속 문단 직접 확인. 새 분할 구현 없음 |
| 줄 소속과 점유 높이 | 비해당 | 구현 변경 없음 |
| 사례와 증거의 독립성 | 충족 | 기존 한컴 PDF 가로선을 별도 추출하고 실제 패널 확인 |
| 기준값 변경 | 충족 | 두 입력 0건, 최신 source CI 16 partition PASS. 상향·신규 허용 없음 |
| 주장과 검증 범위 | 충족 | 원 head CI와 통합 검토 head·빌드 재사용을 구분. 전체 PDF 일치 주장 없음 |
| 검증 입력 커밋 | 충족 | 기존 HWPX/PDF commit blob과 바이트 동일 |

## Merge 후 contributor PR comment 계획

실제 merge가 승인되어 완료된 뒤에만 merge SHA·최종 CI URL과 감사 인사를 한국어로 게시한다.
원 PR에 직접 반영했는지 체리픽 통합했는지 실제 경로를 명시하고, 허용치 축소와 남은 원점 차이를 구분한다.
아래 Native/WASM 1~2쪽 review 4개와 standalone overlay 4개를 실제 이미지로 표시하고 compare 4개를 링크한다.
raw URL은 `https://raw.githubusercontent.com/edwardkim/rhwp/<actual-merge-sha>/mydocs/pr/assets/pr7277_review/<filename>`이다.
위 페이지별 지표·4줄 의미 설명과 Visual Sweep 정본을 연결한다. UTF-8 --body-file 게시 뒤 API 본문·한글·SHA·이미지 URL을 재확인한다.

관련 [#7234](https://github.com/edwardkim/rhwp/issues/7234)는 조회 당시 이미 CLOSED이며,
작성자가 넘침/분할 해소를 설명하고 닫은 상태다. PR 본문의 "이슈를 닫지 않는다"는 현 상태와 시점이 다르다.
이번 review는 이슈 상태를 변경하지 않았다. 후속 comment에는 기존 closed 상태와 잔여 원점 차이를 정확히 설명한다.

- [native_compare_001.png](../assets/pr7277_review/native_compare_001.png)
- [native_overlay_001.png](../assets/pr7277_review/native_overlay_001.png)
- [native_review_001.png](../assets/pr7277_review/native_review_001.png)
- [native_compare_002.png](../assets/pr7277_review/native_compare_002.png)
- [native_overlay_002.png](../assets/pr7277_review/native_overlay_002.png)
- [native_review_002.png](../assets/pr7277_review/native_review_002.png)
- [wasm_compare_001.png](../assets/pr7277_review/wasm_compare_001.png)
- [wasm_overlay_001.png](../assets/pr7277_review/wasm_overlay_001.png)
- [wasm_review_001.png](../assets/pr7277_review/wasm_review_001.png)
- [wasm_compare_002.png](../assets/pr7277_review/wasm_compare_002.png)
- [wasm_overlay_002.png](../assets/pr7277_review/wasm_overlay_002.png)
- [wasm_review_002.png](../assets/pr7277_review/wasm_review_002.png)
