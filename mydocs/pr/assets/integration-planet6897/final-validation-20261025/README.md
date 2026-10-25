# 기존 체리픽 PR 통합 후보 검증

> 기존 체리픽 PR 10건의 통합 검증입니다. 원격 제출·CI·병합은 로컬 검증과 구분합니다.

## 범위와 생산 소스

- 브랜치: `review/planet6897-20261024`
- 적용 원 PR: #8111, #8141, #8152, #8157, #8159, #8163, #8170, #8176, #8185, #8193. #8130은 미병합 종료한 중복 구현입니다.
- 검증 code head: `673c1d5f7cdcb5573b793ba2d2abaaedd2602134`. 렌더러 보정 `38ad63a903`, MCP fixture 기대값 보정 `673c1d5f7c`입니다. D86의 부모 a24a5eed82 + diff SHA `08abebbcaa42fd7924e649c4e3ecb6941e0496b7df537c438b68e0cbd2fada1b`가 첫 commit과 정확히 같고, 다음 commit은 MCP 테스트 한 파일만 바꿉니다.
- 고정 devel: `9418ed0c7cf289c5c4ac2b3bb8cdc18a1b637a9a`. 새 문서·별도 개선은 추가하지 않았습니다.
- sample16 전체와 편람 HWP/HWPX는 사용자 지시로 제외하고 기존 회귀 삭제를 유지했습니다.

## 시각 조건과 전체 TSV

동일 원본·독립 한컴 Print PDF·글꼴 공급, canonical Visual Sweep, PDFium, Print profile, 96 CSSpx, 2px 관용을 사용했습니다. 각 조건의 전체 페이지가 90% 이상이며 비교 가능한 같은 쪽에서 고정 devel 대비 raw 0.1%p 이상 하락은 없습니다. Mac fresh WASM은 locked wrapper `--target web --out-dir pkg --no-opt` 로컬 대체 빌드입니다.

| 기존 조건 | 쪽수 | Native 최저·전쪽 TSV | fresh WASM 최저·전쪽 TSV | 고정 base |
| --- | ---: | --- | --- | --- |
| `field-reset` | 2 | [100.00000%](head/native/field-reset.tsv) | [100.00000%](head/wasm/field-reset.tsv) | [Native](base/native/field-reset.tsv) · [WASM](base/wasm/field-reset.tsv) |
| `field-plain` | 2 | [100.00000%](head/native/field-plain.tsv) | [100.00000%](head/wasm/field-plain.tsv) | [Native](base/native/field-plain.tsv) · [WASM](base/wasm/field-plain.tsv) |
| `symbol-slot-2020` | 1 | [100.00000%](head/native/symbol-slot-2020.tsv) | [100.00000%](head/wasm/symbol-slot-2020.tsv) | [Native](base/native/symbol-slot-2020.tsv) · [WASM](base/wasm/symbol-slot-2020.tsv) |
| `symbol-slot-2024` | 1 | [100.00000%](head/native/symbol-slot-2024.tsv) | [100.00000%](head/wasm/symbol-slot-2024.tsv) | [Native](base/native/symbol-slot-2024.tsv) · [WASM](base/wasm/symbol-slot-2024.tsv) |
| `titled-host` | 4 | [94.04017%](head/native/titled-host.tsv) | [94.04017%](head/wasm/titled-host.tsv) | [Native](base/native/titled-host.tsv) · [WASM](base/wasm/titled-host.tsv) |
| `saved-host` | 4 | [100.00000%](head/native/saved-host.tsv) | [100.00000%](head/wasm/saved-host.tsv) | [Native](base/native/saved-host.tsv) · [WASM](base/wasm/saved-host.tsv) |
| `omitted-after` | 2 | [100.00000%](head/native/omitted-after.tsv) | [100.00000%](head/wasm/omitted-after.tsv) | [Native](base/native/omitted-after.tsv) · [WASM](base/wasm/omitted-after.tsv) |
| `hwp5-control` | 24 | [98.01407%](head/native/hwp5-control.tsv) | [98.01624%](head/wasm/hwp5-control.tsv) | [Native](base/native/hwp5-control.tsv) · [WASM](base/wasm/hwp5-control.tsv) |
| `onsaemiro-control` | 46 | [92.38021%](head/native/onsaemiro-control.tsv) | [92.38142%](head/wasm/onsaemiro-control.tsv) | [Native](base/native/onsaemiro-control.tsv) · [WASM](base/wasm/onsaemiro-control.tsv) |
| `offset-min` | 2 | [98.90710%](head/native/offset-min.tsv) | [98.85533%](head/wasm/offset-min.tsv) | 3→2쪽 · 본문 소유·순서 보존 |
| `rowspan-header` | 4 | [96.03764%](head/native/rowspan-header.tsv) | [95.48353%](head/wasm/rowspan-header.tsv) | [Native](base/native/rowspan-header.tsv) · [WASM](base/wasm/rowspan-header.tsv) |
| `tac-successor` | 4 | [99.45515%](head/native/tac-successor.tsv) | [99.44577%](head/wasm/tac-successor.tsv) | [Native](base/native/tac-successor.tsv) · [WASM](base/wasm/tac-successor.tsv) |
| `policy1790387-actual` | 64 | [92.18753%](head/native/policy1790387-actual.tsv) | [92.12692%](head/wasm/policy1790387-actual.tsv) | 65→64쪽 · 본문 소유·순서 보존 |
| `briefing1487` | 6 | [95.74283%](head/native/briefing1487.tsv) | [95.76159%](head/wasm/briefing1487.tsv) | [Native](base/native/briefing1487.tsv) · [WASM](base/wasm/briefing1487.tsv) |
| `franchise` | 142 | [93.81069%](head/native/franchise.tsv) | [93.80829%](head/wasm/franchise.tsv) | [Native](base/native/franchise.tsv) · [WASM](base/wasm/franchise.tsv) |
| `regulatory86712` | 64 | [98.73269%](head/native/regulatory86712.tsv) | [98.73139%](head/wasm/regulatory86712.tsv) | [Native](base/native/regulatory86712.tsv) · [WASM](base/wasm/regulatory86712.tsv) |
| `cbta1613` | 204 | [91.87422%](head/native/cbta1613.tsv) | [91.82753%](head/wasm/cbta1613.tsv) | [Native](base/native/cbta1613.tsv) · [WASM](base/wasm/cbta1613.tsv) |
| `task1749x` | 5 | [99.99836%](head/native/task1749x.tsv) | [99.99835%](head/wasm/task1749x.tsv) | [Native](base/native/task1749x.tsv) · [WASM](base/wasm/task1749x.tsv) |
| `task1749h` | 5 | [99.99836%](head/native/task1749h.tsv) | [99.99835%](head/wasm/task1749h.tsv) | [Native](base/native/task1749h.tsv) · [WASM](base/wasm/task1749h.tsv) |
| `near-top-reset` | 205 | [98.42761%](head/native/near-top-reset.tsv) | [98.42748%](head/wasm/near-top-reset.tsv) | [Native](base/native/near-top-reset.tsv) · [WASM](base/wasm/near-top-reset.tsv) |
| `curriculum-control` | 415 | [95.79585%](head/native/curriculum-control.tsv) | [95.79585%](head/wasm/curriculum-control.tsv) | [Native](base/native/curriculum-control.tsv) · [WASM](base/wasm/curriculum-control.tsv) |
| `math1342000-actual` | 49 | [92.90997%](head/native/math1342000-actual.tsv) | [92.76172%](head/wasm/math1342000-actual.tsv) | [Native](base/native/math1342000-actual.tsv) · [WASM](base/wasm/math1342000-actual.tsv) |
| `roadmap-control` | 302 | [94.97284%](head/native/roadmap-control.tsv) | [94.80964%](head/wasm/roadmap-control.tsv) | [Native](base/native/roadmap-control.tsv) · [WASM](base/wasm/roadmap-control.tsv) |
| `onsaemiro-converted-control` | 46 | [94.65583%](head/native/onsaemiro-converted-control.tsv) | [94.55536%](head/wasm/onsaemiro-converted-control.tsv) | 48→46쪽 · 본문 소유·순서 보존 |
| `source8152-d2176` | 6 | [95.84490%](head/native/source8152-d2176.tsv) | [95.96239%](head/wasm/source8152-d2176.tsv) | [Native](base/native/source8152-d2176.tsv) · [WASM](base/wasm/source8152-d2176.tsv) |
| `source8152-d17253` | 5 | [99.49419%](head/native/source8152-d17253.tsv) | [99.49419%](head/wasm/source8152-d17253.tsv) | [Native](base/native/source8152-d17253.tsv) · [WASM](base/wasm/source8152-d17253.tsv) |
| `source8152-d80168` | 157 | [95.44852%](head/native/source8152-d80168.tsv) | [95.44144%](head/wasm/source8152-d80168.tsv) | [Native](base/native/source8152-d80168.tsv) · [WASM](base/wasm/source8152-d80168.tsv) |
| `source8152-d1853` | 52 | [95.71982%](head/native/source8152-d1853.tsv) | [95.70173%](head/wasm/source8152-d1853.tsv) | [Native](base/native/source8152-d1853.tsv) · [WASM](base/wasm/source8152-d1853.tsv) |
| `source8152-d1880` | 10 | [98.40056%](head/native/source8152-d1880.tsv) | [98.40002%](head/wasm/source8152-d1880.tsv) | [Native](base/native/source8152-d1880.tsv) · [WASM](base/wasm/source8152-d1880.tsv) |
| `source8152-d5169` | 6 | [99.84756%](head/native/source8152-d5169.tsv) | [99.84705%](head/wasm/source8152-d5169.tsv) | [Native](base/native/source8152-d5169.tsv) · [WASM](base/wasm/source8152-d5169.tsv) |
| `source8152-hfine` | 2 | [100.00000%](head/native/source8152-hfine.tsv) | [100.00000%](head/wasm/source8152-hfine.tsv) | [Native](base/native/source8152-hfine.tsv) · [WASM](base/wasm/source8152-hfine.tsv) |
| `source8152-ha10m` | 3 | [100.00000%](head/native/source8152-ha10m.tsv) | [100.00000%](head/wasm/source8152-ha10m.tsv) | [Native](base/native/source8152-ha10m.tsv) · [WASM](base/wasm/source8152-ha10m.tsv) |
| `source8152-dband` | 12 | [99.96527%](head/native/source8152-dband.tsv) | [99.96529%](head/wasm/source8152-dband.tsv) | [Native](base/native/source8152-dband.tsv) · [WASM](base/wasm/source8152-dband.tsv) |
| `source8152-hs2` | 29 | [95.83018%](head/native/source8152-hs2.tsv) | [95.82665%](head/wasm/source8152-hs2.tsv) | [Native](base/native/source8152-hs2.tsv) · [WASM](base/wasm/source8152-hs2.tsv) |
| `source8152-d80168x` | 156 | [94.79496%](head/native/source8152-d80168x.tsv) | [94.79506%](head/wasm/source8152-d80168x.tsv) | [Native](base/native/source8152-d80168x.tsv) · [WASM](base/wasm/source8152-d80168x.tsv) |

Native/fresh WASM 각각 35개 조건·2,041쪽입니다. 쪽수 개선 3개 조건은 같은 쪽 번호의 raw 하락으로 대체하지 않았습니다. 오프셋 최소 입력 3→2쪽, 정책연구 입력 65→64쪽, 온새미로 변환본 48→46쪽은 렌더 트리의 본문 문자·문단 소유와 순서·보존을 별도로 확인했습니다. 온새미로의 본문 29,868글자·480개 문단 소유키가 양쪽에서 동일합니다.

## 대표 review와 overlay

| 기존 대상·쪽 | Native review | Native overlay | WASM review | WASM overlay |
| --- | --- | --- | --- | --- |
| `curriculum-control` 48쪽 | [review](images/native/curriculum-control-p048-review.png) | [overlay](images/native/curriculum-control-p048-overlay.png) | [review](images/wasm/curriculum-control-p048-review.png) | [overlay](images/wasm/curriculum-control-p048-overlay.png) |
| `curriculum-control` 87쪽 | [review](images/native/curriculum-control-p087-review.png) | [overlay](images/native/curriculum-control-p087-overlay.png) | [review](images/wasm/curriculum-control-p087-review.png) | [overlay](images/wasm/curriculum-control-p087-overlay.png) |
| `curriculum-control` 178쪽 | [review](images/native/curriculum-control-p178-review.png) | [overlay](images/native/curriculum-control-p178-overlay.png) | [review](images/wasm/curriculum-control-p178-review.png) | [overlay](images/wasm/curriculum-control-p178-overlay.png) |
| `regulatory86712` 27쪽 | [review](images/native/regulatory86712-p027-review.png) | [overlay](images/native/regulatory86712-p027-overlay.png) | [review](images/wasm/regulatory86712-p027-review.png) | [overlay](images/wasm/regulatory86712-p027-overlay.png) |
| `cbta1613` 48쪽 | [review](images/native/cbta1613-p048-review.png) | [overlay](images/native/cbta1613-p048-overlay.png) | [review](images/wasm/cbta1613-p048-review.png) | [overlay](images/wasm/cbta1613-p048-overlay.png) |

교육과정표의 셀 정렬·이어받기 및 규제분석의 닫는 선을 현재 양쪽 출력에서 직접 판독했습니다. CBTA 48쪽은 그림 안 작은 글자의 농도와 본문 자형 차이가 남습니다. raw는 고정 base의 Native 91.65747% / WASM 91.57028%보다 현재 91.87422% / 91.82753%가 높으며, 별도 이미지 개선을 이 범위에 추가하지 않았습니다.

## 실제 소비 경로

Native Print 15개는 동일 producer의 전쪽 PNG에서 해시를 확인하여 재사용했습니다. Studio Canvas2D 15개와 지원 CanvasKit 12개는 현재 fresh WASM으로 실제 화면을 새로 캡처했습니다. 요청 backend·readiness·초기 JS/WASM 응답을 확인했고 Canvas2D fallback을 CanvasKit 통과로 계산하지 않았습니다. Native Print와 Studio screen profile을 구분합니다.

## 필수 검사

현재 head 게이트: **complete**.      Summary [1920.075s] 11481 tests run: 11481 passed (46 slow), 45 skipped

- `prepare`: exit 0
- `fmt-check`: exit 0
- `clippy-native`: exit 0
- `clippy-wasm`: exit 0
- `workspace-build`: exit 0
- `clippy-all-targets`: exit 0
- `clippy-native-skia`: exit 0
- `full-nextest`: exit 0
- `skia-lib`: exit 0
- `skia-placeholder`: exit 0
- `skia-direct-pdf`: exit 0
- `manifest-check`: exit 0
- `unit-tier-check`: exit 0
- `manifest-contracts`: exit 0

[개별 PR별 이미지·입력·페이지·해시 대응](EVIDENCE.md). 로그와 중간 JSON은 ignored output에만 보존합니다.

## Producer와 현재 code head 대응

- Native 실행 파일 SHA-256: `e1389384bfb2fbd9e22f350979123af4a590b6116887d7ea1c4d4c79ba7a5e45`.
- fresh WASM 원본 SHA-256: `0a81d25f3e910d91d9b89fa5a06dbcb5cd6f38d126cb2caf3b1abb27bffe118d`.
- JS 원본 SHA-256: `ba97ed3f9cb04605f9fee14363eb76bf572eca3722ece29fd4181fe0076e7da0`.
- root pkg·Studio public·실제 WASM 초기화 HTTP200 응답 hash가 일치합니다. Vite가 변환한 실제 JS 응답 hash와 정규화한 원본 source hash를 구분했습니다.
- D86는 commit 전에 빌드했으며, 부모 a24a5eed82와 원 diff가 runtime commit38ad63a903의 전체 diff와 byte 단위로 일치합니다. 이후673c1d5f7c는 MCP test 한 파일만 바꾸었습니다. 새로 빌드한 것으로 재표기하지 않습니다.

| 검증한 소스 | SHA-256 |
| --- | --- |
| `src/renderer/height_measurer.rs` | `4a84790aa36b6b54b4d74f05f816d7bc24a0b021d2a2e3239928a8baaa5c19ce` |
| `src/renderer/layout/table_layout.rs` | `b2aaed7d828b27792447c3c48245980564da75ed78cb15ff3cc59f2f0220c280` |
| `src/renderer/layout/table_layout/plain_cell_flow.rs` | `4dd094e6890bbac7071391c6b3a2aea40c15c7ce10067c60a7a04476c63ff35f` |
| `src/renderer/layout/table_layout/stored_body_closed_frame.rs` | `d61876ac14e5f973f5daaedf47ee1de7178f78b737a42ddd07f9596eb1bbb32a` |
| `src/renderer/layout/table_layout/stored_span_content.rs` | `634c9f4464e82e1c27ed21bf0199419c89f9692ef0db729f4496dc90d0395d5e` |
| `src/renderer/layout/table_partial.rs` | `58908e9e6baeef754134e446999c8625e11e3d56d9f6960dcb20fb512b62e72e` |
| `src/renderer/layout/table_partial/tail_alignment.rs` | `0dda26f31964ecd68e2507db67b5cfcc4e88c42bad298d6e8bd9ff3b790c1066` |
| `src/renderer/render_normalization.rs` | `1aa0ca0b01bfd9cfd7d5b61e0ba640212f41248b3265a40c01e565637bf3a4d7` |
| `tests/cases/issue_8152_completed_curriculum_primary_program.rs` | `8d18164410dea804e2ec1ab63320605a48a7470f1e4549d803bd046787055a95` |

## 고정 base 생산 출처와 쪽수 개선 대조

- base Native SHA-256: `45871f79033755de0bbe2402e27e14b48972f4e74a258eff78e73fff93009a20`.
- base rhwp.js SHA-256: `f90e8e9655657ea0f23b467ccb89b37b6af24f8f1b387dab9055dae747aea5b9`.
- base rhwp_bg.wasm SHA-256: `cd4068e4e9e73e7d3b8da2e9e856ddd0ce9552a71e494090327fba0c216719ae`.

입력·PDF·글꼴·renderer hash를 고정한 본문 소유 대조 원장 SHA-256: `d1071685a288d0eb4edd85a9960f51dc04f00ac130f6faee314837754a84050b`. 원장 원본은 ignored output에 보존합니다. 같은 쪽수의 지표 비교와 다른 계약입니다.

| 조건 | backend | base → 현재 쪽수 | 대응 |
| --- | --- | --- | --- |
| `offset-min` | native | 3 → 2 | 본문 문자·소유·순서 보존 |
| `offset-min` | wasm | 3 → 2 | 본문 문자·소유·순서 보존 |
| `policy1790387-actual` | native | 65 → 64 | 본문 문자·소유·순서 보존 |
| `policy1790387-actual` | wasm | 65 → 64 | 본문 문자·소유·순서 보존 |
| `onsaemiro-converted-control` | native | 48 → 46 | 본문 문자·소유·순서 보존 |
| `onsaemiro-converted-control` | wasm | 48 → 46 | 본문 문자·소유·순서 보존 |
