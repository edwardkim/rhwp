# [PR #8316](https://github.com/edwardkim/rhwp/pull/8316) 리뷰 — fix(text): 한/글이 기호·영문 슬롯으로 그리는 글자를 그 슬롯으로 분류한다 — 그리스·키릴·…·※·₩·€·℃·№·Latin-1 기호 ([이슈 #8136](https://github.com/edwardkim/rhwp/issues/8136))

## 최종 판정

**통합 PR에 반영됨 — 원 PR별 최종 검토 진행 중.** [통합 PR #8438](https://github.com/edwardkim/rhwp/pull/8438)은 머지됐다. 이 원 PR의 주장·변경 범위와 독립 증거를 대조한 뒤 원 PR 종료 여부를 판정한다.

## 2026-11-02 통합 머지 사실과 적용 SHA 재확인

[통합 PR #8438](https://github.com/edwardkim/rhwp/pull/8438)은 `b8338ba38be288b6e9385f5372d184649323aa9f`로 머지됐다. 아래 체리픽 기록의 종전 통합 SHA는 리베이스 전 이력이다. 머지된 이력에서 원본과 커밋 제목이 일치하는 대응은 다음과 같다. 이는 변경 내용의 완전한 동등성이나 개별 시각 판정을 대신하지 않는다.

- 원본 → 머지 이력: `1ee303ef68` → `c96ff031f6`, `20306cbfc9` → `4417378bed`, `e7fd96c718` → `fdefe7dd66`, `8f53775fa5` → `952465174c`.
- 제품 코드 `d536455b4e`의 최소 글꼴 전체 회귀는 12,368건 통과·45건 skipped였다. 이 로그에서 확인한 해당 모듈: `issue_7092_curly_quote_latin_slot` 3건, `issue_8136_lang_slot_ranges` 3건. 이 수치는 개별 PR의 독립 Print 검증과는 구분한다.
- 뒤의 중간 후보·검증 대기 문장은 당시 상태이며 최종 통합 결과가 아니다. 원 PR별 최종 시각·기능 검토와 최신 head 일치 확인은 계속한다.

## 최종 코드의 기호 슬롯·곡선 따옴표 대조

### [PR #8340](https://github.com/edwardkim/rhwp/pull/8340) 머지 전후의 추가 원본 비교

원 PR에 제시됐으나 당시 Print 비교가 없던 `samples/issue5701/1530000-200800002_slice_p139_tac_reset_tail.hwp`를 기존 한컴 2020 Print PDF와 96 DPI Native로 대조했다. [PR #8340](https://github.com/edwardkim/rhwp/pull/8340) 머지 전 `1a7ae8f5bd`, 직후 `c88657d9c3`, [PR #8438](https://github.com/edwardkim/rhwp/pull/8438) 제품 코드 `d536455b4e` 순서의 전쪽 결과는 각각 [TSV](../assets/issue_8136/final-integration-8438/native-slice-before-8340.tsv), [TSV](../assets/issue_8136/final-integration-8438/native-slice-after-8340.tsv), [TSV](../assets/issue_8136/final-integration-8438/native-slice-after-8438.tsv)다. 12쪽 모두 [PR #8340](https://github.com/edwardkim/rhwp/pull/8340) 전후 Native PNG의 SHA-256이 같으며, 4쪽 raw 표시값은 25.01%, 25.01%, 25.07%다. 이 문서의 큰 위치 차이는 [PR #8340](https://github.com/edwardkim/rhwp/pull/8340) 이전에도 존재했다. [PR #8438](https://github.com/edwardkim/rhwp/pull/8438) 이후에는 1·4·5쪽의 PNG가 달라져 세부 차이와 원인 확인이 필요하다. [PR #8340](https://github.com/edwardkim/rhwp/pull/8340) 기능 자체의 머지 후 보존 여부는 이 비교만으로 판정하지 않는다.

4쪽의 표 윗선은 RHWP 135–136px, Print 148–149px다. 원본 레코드에서 직전 표 호스트의 첫 줄 `vpos`는 3180, 다음 표 호스트는 1000 HWPUNIT이다. 로드 과정에서 본문 축을 재구성한 뒤에는 각각 72484와 136219가 되며, 이 재구성 값 사이의 차이는 문단 앞 간격 1000 HWPUNIT에 해당한다. 재구성 값을 원본 저장 좌표의 연속성 증거로 사용할 수 없다. Print의 이동량과 앞 간격은 96 DPI에서 약 13.33px로 비슷하므로 실제 원점과 쪽 경계 처리를 계속 대조한다. 공통 `typeset/table/host_spacing.rs`와 `table/block/entry.rs`의 단 상단 처리도 함께 확인한다. 다른 입력과 [이슈 #8149](https://github.com/edwardkim/rhwp/issues/8149) 대조군 없이 전역 완화로 결론 내리지 않는다.

제품 코드 `d536455b4e`의 Native와 fresh WASM으로 기호 슬롯 2쪽과 독립 한컴 2024 Print, 곡선 따옴표 7쪽과 한컴 2020 Print를 96 DPI로 전쪽 대조했다. 기호 슬롯은 Native 최저 98.96%, WASM 98.37%, 곡선 따옴표는 각각 96.96%, 97.73%이며 쪽수가 일치한다. 기호 슬롯 1·2쪽과 곡선 따옴표 3·4쪽의 RHWP·PDF PNG는 앞서 직접 판독한 자료와 SHA-256이 같아 해당 review·overlay를 재사용했다.

| 입력 | Native | WASM | 대표 페이지 |
| --- | --- | --- | --- |
| 기호 슬롯 | [전쪽 TSV](../assets/issue_8136/final-integration-8438/native-lang-slot.tsv) | [전쪽 TSV](../assets/issue_8136/final-integration-8438/wasm-lang-slot.tsv) | [2쪽 review](../assets/issue_8136/integration-planet6897/native-review-002.png) · [overlay](../assets/issue_8136/integration-planet6897/native-overlay-002.png) |
| 곡선 따옴표 | [전쪽 TSV](../assets/issue_8136/final-integration-8438/native-treatise.tsv) | [전쪽 TSV](../assets/issue_8136/final-integration-8438/wasm-treatise.tsv) | [3쪽 review](../assets/issue_7092/integration-planet6897/native-review-003.png) · [overlay](../assets/issue_7092/integration-planet6897/native-overlay-003.png) |

이 결과는 두 입력에 관한 것이며 원 PR에 제시된 나머지 문서의 최종 후보 검증을 대신하지 않는다.

## 속기록·HWP3 변환본 전쪽 재확인

최종 제품 코드의 기존 Native 실행 파일과 fresh WASM으로 원 PR에 제시된 속기록 36쪽, `hwp3-sample-hwp5.hwp` 16쪽을 같은 독립 한컴 2020 Print와 96 DPI, 호환 모드 2022로 다시 비교했다. 속기록 전쪽 최저는 Native 95.49%, WASM 95.59%, HWP3 변환본 전쪽 최저는 각각 97.33%, 99.90%다. 양쪽 전체 쪽수는 PDF와 같고 90% 미만은 없다.

| 원본 | Native TSV | WASM TSV | 대표 확인 쪽 |
| --- | --- | --- | --- |
| 속기록 | [36쪽](../assets/issue_8136/final-integration-8438/native-briefing.tsv) | [36쪽](../assets/issue_8136/final-integration-8438/wasm-briefing.tsv) | [36쪽 review](../assets/issue_8136/final-integration-8438/native-briefing-p036-review.png) · [overlay](../assets/issue_8136/final-integration-8438/native-briefing-p036-overlay.png) |
| HWP3 변환본 | [16쪽](../assets/issue_8136/final-integration-8438/native-h3s5.tsv) | [16쪽](../assets/issue_8136/final-integration-8438/wasm-h3s5.tsv) | [5쪽 review](../assets/issue_8136/final-integration-8438/native-h3s5-p005-review.png) · [overlay](../assets/issue_8136/final-integration-8438/native-h3s5-p005-overlay.png) |

원 PR 당시 head TSV와 비교하면 속기록 Native 36쪽은 97.32%에서 95.49%, HWP3 변환본 Native 5쪽은 99.47%에서 97.33%다. 원 PR은 별도의 글꼴 디렉터리를 명시했으나 이번에는 시스템 글꼴을 사용했다. HWP3 5쪽 제목에서 Native와 PDF의 글자 폭 차이를 직접 확인했으므로 글꼴 선택·기준 조건을 조사한다. 이 수치만으로 동일 글꼴 조건의 회귀 여부를 단정하지 않는다.

### HWP3 변환본 5쪽의 제목 차이 원인 대조

동일 시스템 글꼴 조건에서 [PR #8438](https://github.com/edwardkim/rhwp/pull/8438) 이전 이력 `99a176b7cb`([PR #8397](https://github.com/edwardkim/rhwp/pull/8397))의 Native 실행 파일로 5쪽을 다시 출력했다. 현재 제품 코드와 해당 이전 출력 PNG의 SHA-256은 모두 `a52876c78f720d9e6838a6d92ed9ca1c70e6eeffd4f91aec29cd8f5a94c5def9`로 일치한다. 이 기준에서는 [PR #8438](https://github.com/edwardkim/rhwp/pull/8438)로 새로 생긴 시각 하락이 아니다. 원 PR 당시 별도 글꼴 디렉터리의 TSV와는 다른 환경이다.

Native 기본 조건에서 글머리 `•`의 줄 상자 폭은 8.1 CSSpx, 같은 시스템에 설치된 한양신명조의 별도 완성 글리프 공급 폰트를 명시한 실험에서는 15.4 CSSpx였고 PDF의 제목 원점과 맞아 해당 쪽 raw가 100.00%였다. 실제 renderer snapshot 추적에서 기본 조건의 제목 한글은 `HY Sinmyeongjo`, 글머리는 `한양신명조`로 선택됐다. 시스템에는 동일 family에 여러 프로그램이 있으므로 단순 이름만으로 파일을 단정하지 않는다. 글꼴 파일은 증적이나 배포물에 포함하지 않았다. 기본 Native의 눈에 보이는 차이는 후속 검토 대상으로 남기며, WASM 동일 쪽은 99.95%다. [WASM review](../assets/issue_8136/final-integration-8438/wasm-h3s5-p005-review.png) · [WASM overlay](../assets/issue_8136/final-integration-8438/wasm-h3s5-p005-overlay.png).

## 접수와 적용

- 원 PR: [PR #8316](https://github.com/edwardkim/rhwp/pull/8316), 작성자 `planet6897`, reviewer `jangster77`.
- 원 PR head 조회값: `8f53775fa506fc35e820dfe5d8632d0c6b8aafb6`. 조회 시 원 PR head와 마지막 체리픽 원본이 일치한다.
- 마지막 적용 원본: `8f53775fa506fc35e820dfe5d8632d0c6b8aafb6`. 검증 시작 당시 코드 후보: `abea5b028f8f135bdf000ca700e1993ebf0b1973`, 기준 `upstream/devel`: `bc9dfdd057258de59afebeb5121c4bee7398e07f`.
- 관련 이슈: [이슈 #8136](https://github.com/edwardkim/rhwp/issues/8136).
- 선행 PR: 별도 누적 선행 PR이 명시되지 않았다.

후속 `e1f03ffbb3`은 Python 테스트 준비, `af175315e7`과 `e920cba6b0`은 회귀 검사 주석, `a63bb719de`는 검증 경과를 기록했다. `5d1294811b`은 전체 그림의 잘림 오판을 보정했다. `e1b8ab0f54`, `d6fe30c3ad`, `0897582f6c`, `8c4eca265e`에 Native 진단, [PR #8410](https://github.com/edwardkim/rhwp/pull/8410) 집중 6건 통과와 글꼴 대조를 기록했으며 새 코드의 전쪽 출력 검증이 필요하다.

## 변경 주장과 검토 관점

독립 Print에서 구분된 기호와 Latin-1 문자를 해당 언어 슬롯으로 분류한다.
기호가 없는 글꼴의 fallback과 기존 [PR #8397](https://github.com/edwardkim/rhwp/pull/8397) 정밀 기준을 함께 확인한다.

## 체리픽과 보정

- 원본 `1ee303ef68242582d6613e8d9f368830f1418606` → 통합 `3679eafcee028eb56e515f91e8fccec3291a542a`.
- 원본 `20306cbfc905478284b1d95b4e1e95bbe2382fd3` → 통합 `f87c21358a800af05c5caeaa05863b4a6e72145e`.
- 원본 `e7fd96c718ed80aa0f4fe28ac0b8ecbab0dffd29` → 통합 `2ce7fc47d0f758d464a7a9ce6148b2761d9f457a`.
- 원본 `8f53775fa506fc35e820dfe5d8632d0c6b8aafb6` → 통합 `d750d480ffd855b0311150a0129e43037b120dbb`.

이 PR의 원본 커밋 적용 중 별도로 기록된 충돌은 없다.

## 검증과 증거

이전 통합 후보의 최소 글꼴 전체 회귀 로그에서 이 PR 관련 다음 모듈의 실행을 확인했다. 새 base 후보의 재실행 결과는 이 값으로 대신하지 않는다.
- [`issue_7092_curly_quote_latin_slot.rs`](../../../tests/cases/issue_7092_curly_quote_latin_slot.rs): 이전 실행 3건 통과, 0건 실패, 0건 건너뜀.
- [`issue_8136_lang_slot_ranges.rs`](../../../tests/cases/issue_8136_lang_slot_ranges.rs): 이전 실행 3건 통과, 0건 실패, 0건 건너뜀.

검토 후보에 포함된 입력 바이트를 Git blob과 대조했다. 아래 자료는 원 PR의 제출 증거이며 새 후보의 실행 결과를 뜻하지 않는다.
- [`lang-slot-ranges-2024.pdf`](../../../pdf/issue8136/lang-slot-ranges-2024.pdf) — SHA-256 `27a3d1916d078fa064be171c777807d4f278cd62059c810decaf7a0d233f327d`.
- [`lang-slot-ranges.hwpx`](../../../samples/issue8136/lang-slot-ranges.hwpx) — SHA-256 `575de965af86769779a8533b7eea73f79d4ab76737b9daacc514728c92d8e0eb`.

기여자가 제출한 기준 자료: [`ab_changed_pages_native.tsv`](../../../mydocs/pr/assets/issue_8136/ab_changed_pages_native.tsv). 해당 자료만으로 새 통합 후보의 시각 검증을 통과 처리하지 않는다.

2026-11-01 중간 Native 후보 실행 파일 SHA-256 `f29ec78b0d2a1f8b18a172ea4e481436f85d75873ba530f4427debe078ec93b3`로 같은 HWPX와 독립 2024 Print PDF를 96 DPI에서 다시 비교했다. 전 2쪽 raw는 각각 `99.48%`, `98.95%`다. [TSV](../../../output/pr-review/planet6897-ordered-20261031/stored-guard-issue8136-native/issue8136/silhouette.tsv), 변경 기호가 있는 [1쪽 review](../../../output/pr-review/planet6897-ordered-20261031/stored-guard-issue8136-p001-review/issue8136/review/review_001.png)와 [overlay](../../../output/pr-review/planet6897-ordered-20261031/stored-guard-issue8136-p001-review/issue8136/overlay/overlay_001.png)를 대조했다. 이후 저장 줄 출처 판정이 변경돼 최종 후보 Native·fresh WASM 재검증을 진행한다. `output/` 자료는 임시이며 PR 제출 증적은 별도 정식 경로에 고정해야 한다.

확대 판독에서 `™` 등 기호의 윤곽 차이가 보인다. Print의 내장 `Gungsuh` 프로그램을 설치 글꼴의 같은 코드포인트 윤곽과 대조하면 `™`는 설치 `궁서`가 아닌 `궁서체`와 같고, `€`는 반대로 `궁서`와 같다. `㉠`은 둘 다와 같다. 글꼴 명칭 또는 총점만으로 시각적 일치를 선언하지 않는다. 시스템에서 UPM 1024 글꼴 137개를 대조한 `™` 결과는 임시 `issue8136-print-trademark-fonts.log`에 있다. 글꼴 버전과 실제 Native/WASM 공급자의 원인 확인이 남는다.

Windows 검증 호스트의 `C:\Windows\Fonts\batang.ttc`를 임시 검증 경로로 복사해 대조한 결과 그 안의 `Gungsuh`는 세 글자 모두 Print와 윤곽이 같다. 해당 TTC의 SHA-256은 `84b0ba79a5d1c6012bbccf8d364ddc04a0c8b135d227a159196c558225cf1d89`이다. 나머지 시스템 글꼴도 노출한 상태에서 TTC를 명시적으로 추가 공급한 Native 진단의 1쪽 raw는 `99.48% → 99.90%`, 2쪽은 `98.95% → 98.95%`다. 이 결과는 글꼴 버전 차이를 확인하는 진단이며 자동 선택이나 fresh WASM 검증을 대신하지 않는다. 글꼴 파일은 `output/`에만 두고 저장소나 배포물에 포함하지 않는다.

시스템 글꼴 전체에 TTC 한 파일을 추가한 임시 Fontconfig에서 `fc-match 궁서`는 해당 TTC를 반환했지만, 명시 경로 없이 실행한 Native 1쪽은 여전히 `99.48%`다. 원본 HWPX는 `궁서`를 TTF로 선언한다. 같은 임시 환경에서 별도 기본 2022 프로필 글꼴 결정 추적은 `™`·`€`를 CFF `Gungseo`, `㉠`을 `Gungsuh`로 보고했다(`issue8136-font-trace-auto-summary.log`). 추적 helper의 PNG는 위 2024 프로필 측정 PNG와 해시가 달라 동일 실행의 paint 증거로 취급하지 않는다. Native 후보 사다리가 source의 TTF/HFT 유형을 소비하는지와 2024 실제 paint 선택을 추가 확인한다.

같은 helper에 2024 프로필을 명시해 재실행하자 추적 PNG와 임시 Fontconfig Native 측정 PNG의 SHA-256이 모두 `b3f9e10e69cde4fadbd553287dcbb19490190167f99fb25e7b0a256d0ea92724`로 일치했다. `™`·`€`는 `Gungseo`, `㉠`은 `Gungsuh`로 관측됐다(`issue8136-font-trace-auto-2024.json`). 원본 TTF 선언에도 CFF 후보가 먼저 사용되는 조건을 공통 경로에서 분석한다.

TTF 선언을 글꼴 선택에 전달한 새 Native 후보 SHA-256 `91d4e37673a8c796c1e499c76a30d4a7924ca9084c107490e8b0141d94f3e5f5`의 전 2쪽 raw는 각각 `99.90447%`, `98.95550%`다. 시스템 글꼴 전체와 같은 독립 Print PDF를 사용했으며 [TSV](../../../output/pr-review/planet6897-ordered-20261031/font-source-native-sweep/issue8136/issue8136/silhouette.tsv), [1쪽 review](../../../output/pr-review/planet6897-ordered-20261031/font-source-issue8136-review/issue8136/review/review_001.png) 및 [overlay](../../../output/pr-review/planet6897-ordered-20261031/font-source-issue8136-review/issue8136/overlay/overlay_001.png)를 남겼다. 이는 Native 중간 결과로, 최종 fresh WASM 및 다른 원본의 무하락 검증은 남아 있다.

이후 HWP5의 미확정 글꼴 속성 비트를 TTF 선언으로 확정하지 않도록 변경한 후보 `b38728ae6740b44deb2a6b5c574110d07c5b44535db34b593284a45276b2a3c7`에서 원 HWPX는 같은 두 수치를 유지했다. 그러나 그 HWPX를 rhwp로 HWP에 저장하고 다시 연 1쪽은 `99.48%`로 낮아졌다([임시 TSV](../../../output/pr-review/planet6897-ordered-20261031/issue8136-roundtrip-current-native/issue8136-roundtrip/silhouette.tsv)). 저장본에는 `RhwpHwpxOrigin` 출처가 있으며 rhwp의 직렬화기는 HWPX의 명시적 글꼴 유형을 HWP 속성에 보존한다. 이 확정 출처까지 일반 HWP5의 미확정 비트로 취급하면 안 되므로 출처를 구분해 보정하고 같은 두 경로를 다시 검증한다.

출처 구분을 보완한 Native 후보 `e27de3655ca5806043ba63110f2823c6f17db663e80ea55d93477c208cef2200`에서는 원 HWPX와 위 HWP 저장본의 raw가 각각 1쪽 `99.904468740049%`, 2쪽 `98.955495004541%`로 정확히 일치한다([정밀 비교](../../../output/pr-review/planet6897-ordered-20261031/hwp5-hwpx-lineage-focus-compare.log)). 두 경로의 fresh WASM과 최종 직접 판독은 별도로 확인해야 한다.

현재 코드에서 [이슈 #8136](https://github.com/edwardkim/rhwp/issues/8136)의 슬롯 분류 3건을 최소 글꼴 환경으로 다시 실행해 모두 통과했다. [이슈 #8079](https://github.com/edwardkim/rhwp/issues/8079)의 4건과 [이슈 #8315](https://github.com/edwardkim/rhwp/issues/8315)의 3건도 같은 실행에서 통과했다([집중 검사 로그](../../../output/pr-review/planet6897-ordered-20261031/font-slot-focused-minfont-after-commit.log)). 이는 해당 검사 10건의 결과이며 전체 회귀나 시각 검증을 대신하지 않는다.

기존 생성 목록으로 최소 글꼴 회귀 12,320건 통과·45건 건너뜀을 확인했다. 생성 목록에서 누락된 [PR #8410](https://github.com/edwardkim/rhwp/pull/8410) 두 모듈의 6건을 목록 재작성 후 같은 환경에서 별도로 실행해 모두 통과했다. 두 실행을 구분하며 한 번의 전체 실행으로 기록하지 않는다.

2026-11-01 고정 후보의 [PR #8389](https://github.com/edwardkim/rhwp/pull/8389)·[PR #8397](https://github.com/edwardkim/rhwp/pull/8397) 전쪽 정밀 대조에서 2,153행 중 Native 12행, WASM 8행이 공개 기준보다 낮았다. 이 수치는 이후 GIF·Canvas 보정 전 결과이며 최종 후보의 재검증 결과가 아니다. [PR #7905](https://github.com/edwardkim/rhwp/pull/7905) 3쪽은 GIF 오른쪽 경계 누락, sample16 35쪽 WASM은 메타파일 내부의 미대응 3픽셀이 확인됐다. [PR #8040](https://github.com/edwardkim/rhwp/pull/8040) 두 원본은 Native·WASM 전쪽이 90% 이상이나 Native 정밀 기준 하락이 남는다. 진행과 근거는 [작업 기록](../../working/task_m100_planet6897_system_fonts_20261031.md)을 따른다. 하락 보정과 전쪽 대조, 각 PR의 독립 Print·Native/fresh WASM 및 직접 판독을 완료해야 한다. [PR #8410](https://github.com/edwardkim/rhwp/pull/8410)·[PR #8412](https://github.com/edwardkim/rhwp/pull/8412)의 새 base 기능도 별도로 보존 검증한다.

원 PR의 최신 head·required check와 통합 후보의 새 검증 결과를 확인하기 전에는 승인이나 merge를 진행하지 않는다.

### 2026-11-02 새 base 최소 글꼴 집중 회귀

검증 기준 base
`d3f7209bbb0fdb1711c5f40a70ad22f0ca0be66d`의 실행 파일에서 `issue_8136_lang_slot_ranges`
3건이 최소 글꼴 환경에서 모두 통과했다. 이 검사는 위에서 확인한 문서의 전체
페이지 시각 검증을 대체하지 않는다.

### 4쪽 표 이동의 분리 진단

동일한 기존 Native·Print 래스터에서 Native 전체 이미지를 세로로만 이동해
재계산했다. 원본 raw 25.07%에 비해 아래로 12·13·14·15px 이동한 진단값은
각각 98.40·98.75·98.87·96.71%다. 전역 수직 이동이 4쪽 차이의 대부분을
설명한다는 근거이며, 수정된 엔진의 결과나 픽셀 고정값이 아니다. 앞서 확인한
저장 줄 사다리와 문단 앞 간격의 소비 경로를 기준으로 보정해야 한다. 다른
미달 쪽은 각각 따로 확인한다.

같은 방식으로 5·6·9쪽을 -14·0·+14·+28px 이동해 대조했을 때 최고 raw는
각각 47.71·47.04·51.34%였다. 이 네 이동량 범위에서는 전체 쪽의 단일 수직
이동으로 설명되지 않으므로 4쪽의 간격 원인만으로 전쪽 개선을 주장하지 않는다.

추가 확대 판독으로 위 진단의 이동 범위가 부족함을 확인했다. 더 넓은 범위에서
5쪽은 +43px일 때 97.33%, 6쪽은 +44px일 때 99.19%, 9쪽은 -22px일 때
99.47%였다. 따라서 세 쪽도 표 전체의 수직 원점 차이가 대부분을 설명한다.
위 수치는 래스터 진단값이며, 서로 다른 이동량을 문서별 보정값으로 쓰지 않는다.

저장 문단과 페이지 항목을 추가 대조했다. 현재 배치의 4쪽 본문 가용 높이는
876.84px인데 사용 높이는 880.53px이고, 끝의 빈 문단 23은 앞 간격을 포함해
42.4px를 점유한다. 5쪽의 두 빈 문단 26·27은 각각 22.03px다. 5·6쪽에서
각각 확인한 약 43·44px 수직 차이는 이 빈 문단들의 점유 높이와 근접한다.
이는 페이지 경계의 빈 줄 수용과 이월을 조사할 근거이지, 아직 빈 문단의 한컴
쪽 소속을 직접 확정하거나 구현 원인을 확정한 것은 아니다.

### 공통 원점 보정 후보

로드 후 문단 12의 재구성 `vpos=136219`는 직전 줄의 재구성 값
`72484 + 62183 + 552`에 앞 간격 1000 HWPUNIT을 더한 값이다. 원본 레코드의
좌표는 각각 3180과 1000이므로 위 등식은 원본 저장 사다리의 증거가 아니다.
후보는 새 단에서 앞 간격을 원점에 반영하며, 그 원점을 개체 fit과
`InlineBoxPlacement`의 실제 위치·흐름 끝에 함께 사용한다. 다만 로드 중 재구성된
좌표가 저장 좌표로 오인되지 않는지와 적용 조건의 일반성은 재검토 중이다.
선행 [PR #8184](https://github.com/edwardkim/rhwp/pull/8184)의
[이슈 #8149](https://github.com/edwardkim/rhwp/issues/8149)는 빈 호스트의
문단 기준 자리차지 표가 쪽 아래에서 분할·이월되는 사례다. 이 원본의 11쪽
표는 글자처럼 취급되는 표이므로 두 경우의 단 상단 간격을 같은 규칙으로
가정하지 않고 실제 배치와 독립 출력을 함께 대조한다.

본문 축 재구성의 선행 근거인 [이슈 #5871](https://github.com/edwardkim/rhwp/issues/5871)은
기존 한컴 2020 Print PDF로 다시 대조했다. 후보 전과 후보 3의 Native 전 3쪽 TSV는
각각 [보정 전](../assets/issue_8136/final-integration-8438/native-issue5871-before-spacing.tsv),
[후보 3](../assets/issue_8136/final-integration-8438/native-issue5871-candidate3.tsv)이며
표시값이 모두 99.98·99.96·99.98%로 같다. 이는 해당 대조군의 Native 결과이고
후보의 일반성이나 fresh WASM 검증 전체를 대신하지 않는다.

9쪽 표 앞 공백 글줄은 문단 51의 첫 줄이다. `403295 + 52868 + 552 = 456715`는
로드 후 재구성 좌표의 관계다. 원본 문단 51의 두 줄 좌표는 62332와 0으로, 둘째
줄에서 원본 쪽 좌표가 다시 시작한다. 한컴 Print는 이 표를
쪽 상단에 두지만 현재 배치는 공백 줄과 표를 함께 새 쪽에 둔다. 표 문단 진입의
`controls/tac_fit.rs`와 실제 표 이월의 `typeset.rs`가 선행 줄의 수용 가능성을
후행 간격까지 포함한 전진량으로 각각 판단하는 경로를 확인했다. 후보는 두 곳이
기존 `FormattedParagraph::line_content_height`로 실제 점유 끝을 판정하고, 수용된
줄의 전진량과 표의 새 쪽 원점은 기존 배치 경로가 소비하도록 한다. 9쪽의 실제
소속과 정상 대조군은 후보 실행 뒤에 판정한다.

첫 후보의 Native 직접 대조에서 4·5·9쪽은 각각 98.75·99.37·99.47%였으나
6쪽은 39.33%였다. 6쪽 첫 빈 문단의 수용된 높이는 22.03px인데 다음 문단 진입
커서는 29.39px로, 저장 줄간격 7.36px가 한 번 더해졌다. 새 쪽에서 분할 경로로
통째 수용한 `FullParagraph`는 점유 끝이 기록되지 않았고, `HeightCursor`는 실제
줄간격 소비를 확인하는 기존 경로를 HWPX에만 적용했다. 후보는 수용된 줄의 내용
끝을 기록하고, 저장 원점·줄 사다리·실제 소비 끝이 일치하는 경우 같은 확인을
공통 적용한다. 두 경로의 측정과 paint가 같은 근거를 소비하는지는 재실행한다.

원본 한컴 2020 PDF의 2쪽은 제목만 있는 희소 쪽이다. 제목은 겹치지만 PDF의
용지 네 모서리 인쇄 표식이 RHWP 출력에는 없어 raw가 64.80%다. PDF 콘텐츠
스트림에서 용지 경계에 그린 선을 확인했다. 같은 입력 SHA-256의 기존
[다른 한컴 PDF](../assets/pr_6159/baseline/issue5700/1530000-200800002_slice_p139_tac_reset_tail-hwp2020.pdf)는
6쪽 가로 2-up이므로 12쪽 1-up 기준으로 혼용하지 않는다. 현재 2쪽은 90% 게이트를
통과한 것으로 처리하지 않는다.

추가로 이미 보관된 같은 축약 입력의 한컴 2020·2024 1-up PDF를 확인했다.
두 파일도 각각 12쪽이며 2쪽 네 모서리에 같은 선이 있다. 2024 PDF의
MediaBox와 CropBox는 모두 595×841pt이고, 콘텐츠 스트림의 0.12pt 선은
용지 경계에서 약 28.3pt 양쪽으로 뻗는다. 따라서 한 파일의 손상으로
단정하지 않는다. 이 관측만으로 선의 원인이 원본 인쇄 설정인지 PDF 출력
장치인지 구분할 수 없으며, 실제 원인을 확인하기 전에는 선을 가리거나
90% 미달을 통과로 처리하지 않는다.
기존 PDF는 [2020](../../../pdf/semanticist21-20261019/null-metadata-print-1379/engine-2020/1530000-200800002_slice_p139_tac_reset_tail-hwp/1530000-200800002_slice_p139_tac_reset_tail-hwp-2020.pdf),
[2024](../../../pdf/semanticist21-20261019/null-metadata-print-1379/engine-2024/1530000-200800002_slice_p139_tac_reset_tail-hwp/1530000-200800002_slice_p139_tac_reset_tail-hwp-2024.pdf)다.
원본 `Section0`의 첫 `PAGE_BORDER_FILL`은 종이 기준, 간격 0, 테두리 ID 28이다.
`DocInfo`의 해당 `BORDER_FILL` 원본 네 변 종류는 모두 0(없음)이므로 일반 쪽
테두리 네 변을 그대로 그려서 이 선을 재현할 근거도 없다. 별도 한컴 Print인
`hwpspec` 2020·2024 PDF의 2쪽에는 같은 용지 경계 선이 없다. 이것만으로
인쇄 설정의 구체적인 키나 선의 생성 계층을 확정하지는 않는다.

같은 원본을 설정 진단용으로 한컴 2020 MCP에서 HWPX로 변환한 결과
`settings.xml`의 `PrintInfo`에 `PrintCropMark=1`이 명시돼 있다. 한컴 제품은
11.0.0.9136이며 변환 과정의 `input_preprocess`는 `scripts_disabled`다.
이 HWPX는 인쇄 설정을 확인하기 위한 임시 산출물로, PDF 기준이나 원본의
바이트 동일한 대체물로 취급하지 않는다. HWP 원본 `DOC_DATA`에는
`0x0006401a=1`이 있고 경계선이 없는 `hwpspec.hwp`에는 같은 키가 0이다.
두 사실은 해당 키가 재단 표시와 관련될 가능성을 보여 주지만, 키의 정확한
매핑과 값별 동작은 다른 설정·입력의 대조로 확인해야 한다. 사용자 문서의
인쇄 설정을 무시하거나 시각 비교에서 선을 삭제하지 않고 공통 출력 경로를
추적한다. 진단 HWPX SHA-256은
`ad66be93a37ed544036d3178a5dbd9d8a0bbebd087e6468e4965496a114adbbc`다.
추가로 기존 HWP·HWPX 쌍 15개는 HWP 키 `0x0006401a=0`과 HWPX
`PrintCropMark=0`이 일치한다. 재단 표시 1인 한컴 변환본의 설정과 함께
키의 0·1 대응을 뒷받침한다. 그 밖의 값과 눈금·중심선 같은 다른 인쇄
표시의 동작까지 입증하는 것은 아니다. 적용을 검토할 공통 경로는
`DocInfo` 원본 설정 → `PageRenderTree`의 인쇄 메타데이터 →
`PaintBuilder`의 물리 용지 좌표 → Native·WASM 출력이며, 화면 표시와
인쇄 표시가 구분돼야 한다. 두 출력 backend에 문서별 선을 따로 추가하지 않는다.
기존 2020 PDF의 1·2·4·6·9·11·12쪽 콘텐츠를 확인한 결과, 각 쪽에
같은 0.12pt 재단선 여덟 개가 있다. 용지 네 꼭짓점을 중심으로 한 가로·세로
선이며 각 선의 절반만 용지 안에 들어간다. 선택 쪽에서 검증한 뒤에는 전체
쪽과 재단 표시가 없는 대조군도 확인해야 한다.

추가 후보 실행에서 6쪽의 다음 문단 진입 원점은 29.39→22.03px, 표의 원점은
51.41→44.05px가 됐다. 기존 한컴 PDF와 Native 96 DPI 직접 대조 결과 4·5·6·9쪽은
각각 98.75·99.37·99.19·99.47%다. 총 12쪽은 유지되며 9쪽의 공백 선행 줄은
8쪽, 표는 9쪽에 속한다. [PR #8396](https://github.com/edwardkim/rhwp/pull/8396)의
4개 표 분할 대조군은 전쪽 문단·표 소속과 총 쪽수가 유지됐다. 이는 Native 후보의
중간 확인이며 전체 시각·회귀 검증 완료가 아니다. 2·11·12쪽은 여전히 미달이다.

11쪽의 문단 54는 표 다음에 글줄이 이어지고, 12쪽 문단 55는 표 줄의 후행
간격이 음수다. 원본 레코드에서는 각각 앞 줄 끝에 1000 HWPUNIT을 더한 위치가
다음 줄의 저장 좌표와 맞지만, 로드 후에는 쪽 축이 재구성되고 일부 문단의 문자
분할이 무효화된다. 일반 경로에 앞 간격을 전달하는 후보는 이 유효성 검사를
통과하지 못했으므로 원본 좌표의 관계만으로 적용하지 않는다.

일반 경로에 같은 앞 간격을 전달한 후보는 Native 선택 쪽 대조에서 11·12쪽이
각각 33.82·37.72%로 바뀌지 않았다. 4쪽은 98.75%를 유지했다. 진단 결과
해당 문단들의 문자 분할이 무효화되어 후보 조건이 수용되지 않았다. 원본
좌표를 그대로 복원하지 않고 공통 재조판 경로의 원점을 계속 조사한다.
현재 공통 inline 줄 계획은 저장 줄이 없거나 합성 줄인 표 문단을 대상으로 하며,
저장 줄이 남아 있지만 문자 분할만 무효화된 문단은 같은 진입 조건을 통과하지
못한다. 11쪽의 문단이 이 상태다. 무효화된 텍스트의 공통 줄 계획 참여가 맞는지
표 분할·쪽 예산·실제 paint 소비와 정상 저장 줄 대조군을 함께 확인해야 한다.
정상 저장 줄을 유지해야 하는 대조군으로
[`issue_7622_inline_table_font_slots.rs`](../../../tests/cases/issue_7622_inline_table_font_slots.rs)의
문자와 TAC 표가 섞인 한컴 저장 문단도 확인한다. 해당 검사는 저장 문자 분할이
유효하다는 전제와 글꼴 슬롯을 함께 확인하므로 무효화된 문단의 결과로 대체하지 않는다.

무효화된 문자 분할을 공통 inline 줄 계획에 포함한 Native 후보
(실행 파일 SHA-256 `44d48322f370fe91dc20c507ec3a4d6af55161dd5f0f37777fbbc5e11c280aec`)의
기존 Print 대조에서 11쪽 raw는 33.82%에서 98.65%로 올랐다. 4·5·6·9쪽의
표시값은 각각 98.75·99.37·99.19·99.47%로 유지됐고, 12쪽은 37.72%로
여전히 미달이다. [PR #8396](https://github.com/edwardkim/rhwp/pull/8396)의
4개 대조군은 쪽수와 페이지별 문단·표 소속이 같았다. 이는 후보의 Native 선택 쪽과
구조 비교로, 전쪽·fresh WASM·직접 확대 판독·회귀 검증 완료를 의미하지 않는다.
11쪽 표 뒤 `Appendix 2` 줄은 overlay에서 글자와 간격이 완전히 겹치지 않아
표 원점의 높은 점수만으로 미세 간격까지 통과했다고 판단하지 않는다.
후보 코드 `a415f7ba04602b9ff8be6f3b6e5da267804b0152`에 해당하는 실행에서
최소 글꼴 집중 회귀 16건이 모두 통과했다. 유효한 저장 글줄, 단 정의 부재,
기존 표 분할과 되감긴 꼬리의 소유 검사이며 로그 SHA-256은
`ee45378d516bcb440a786e853045237ff9437a3405337f8ea984d1cc50356a2c`다.
11쪽은 무효화된 문자와 표가 공통 줄 계획으로 옮겨진 반면 12쪽의 표 전용
문단은 문자 분할 무효화 표시가 없어 기존 저장 표 경로에 남는다. 두 문단 모두
단 정의 부재 처리로 본문 세로 축은 재구성됐으므로, 문자 분할의 유효성과
본문 축의 출처를 같은 상태로 취급하지 않고 공통 계획의 적용 근거를 추적한다.
12쪽은 텍스트가 없는 단일 TAC 표이며 원본 줄 높이가 표 높이와 위아래 바깥
여백의 합과 같다. 따라서 이 줄의 물리 상자는 보존하되, 재구성된 본문 축을
원본 쪽 좌표로 인증하지 않는 출처 정보가 필요하다. 한 줄에 여러 개체나 명시적
글줄이 있는 문단은 이 단일 개체 계획으로 단축하지 않는다.

재구성된 축을 별도로 표시한 미커밋 후보의 Native 실행 파일 SHA-256은
`ac7e12988d39c9aa7565a4c0c5452efd0629170cafecfc3974b65c08055e079f`다.
같은 원본과 기존 2020 Print로 4·5·6·9·11·12쪽을 비교한 결과 각각
98.75·99.37·99.19·99.47·98.65·98.48%다. 12쪽은 직전 후보의 37.72%에서
개선됐고 나머지 다섯 쪽의 수치는 변하지 않았다. 12쪽의 표와 뒤 문단 순서를
직접 대조했다. 이 결과는 전체 쪽과 fresh WASM 검증을 대체하지 않는다.
11쪽 제목 공백의 차이와 PDF의 재단 표시로 인한 2쪽 미달도 남아 있다.
축 출처를 가진 문단을 편집으로 분리할 때 새 줄에서 출처를 초기화한 뒤에도
최소 글꼴 집중 회귀 16건이 모두 통과했다. 실행 로그 SHA-256은
`9546b8b2626f820a2ea17ce7bdd2a1fc4f33209104d2fb52c6de66f801942def`다.
11쪽의 원본 문단 54는 TAC 표 뒤에 공백 세 개가 있고, 원본 두 번째
LineSeg의 시작은 제어 8단위 뒤의 `text_start=10`이다. 따라서 원본에서는
공백 두 개가 표를 소유한 줄에 속한다. 현재 후보는 공백 세 개를 다음 줄의
TextRun에 포함하며, 그 결과 한컴 Print와 제목 원점이 다르다. 단 정의가 없어
본문 축과 문자 분할을 재구성한 상황이므로 저장 줄 시작값만으로 모든 재조판의
줄 소속을 강제하지 않는다. 현재 표 폭이 원본 저장 줄 폭을 넘는 조건과 공백의
줄 넘김 규칙을 함께 확인해야 한다.

기존 재조판의 공백 경계 함수는 줄보다 넓은 첫 개체와 같은 위치의 공백을
개체가 소유하게 하고, 다음 넘친 공백을 앞줄에서 소비한다. 공통 inline 계획도
같은 함수를 사용한 후보에서 표 뒤 제목의 앞 공백 한 개가 한컴 Print와 같은
줄에 남는다. Native 11쪽 raw는 98.65%에서 99.34%로 올라갔고, 4·5·6·9·12쪽은
직전 후보와 같다. 독립 확대 판독에서도 제목 원점이 가까워진 것을 확인했다.
렌더 트리에서도 공백 두 개와 ` 〔Appendix 2〕...`가 서로 다른 줄에 속하고
원문 공백 세 개가 모두 보존된다.
실행 파일 SHA-256은 `44f6f7634f20d6aba886d92741a12bd287a58450fb1c6dd397dcdbc2075e399e`다.
서버 재시작으로 중단된 비교는 동일 입력·실행 파일·head를 확인해 체크포인트를
재사용했으며 6쪽 모두 완료했다. 별도 표 분할 대조군 네 문서의 쪽수와 내용
소유는 이전 후보와 같았다(로그 SHA-256
`03d055b81a9df348552bb23acdf110e29086e7626c5d18b686cd7fb17b99775b`).
이 부분 비교는 전쪽·fresh WASM·최종 회귀를 대신하지 않는다.

이 축약본의 선행 [이슈 #5700](https://github.com/edwardkim/rhwp/issues/5700)과
[PR #6125](https://github.com/edwardkim/rhwp/pull/6125)도 확인했다. 원 PR은 표 뒤
되감긴 꼬리 문단이 표 위로 그려지는 역전을 paint 흐름 하한으로 막았으며,
쪽 나눔 자체는 바꾸지 않았다. 이번에 관찰한 쪽 상단 원점 차이는 그 해결을
되돌린다는 증거가 아니므로, 기존 표·꼬리의 순서 보존을 유지하면서 별도로 보정한다.

### 의존성 업데이트 반영 후 Native 재확인

`de2295d430b588f0741021f66fae086eb4ff4009`로 리베이스한 뒤 새 Native Skia
실행 파일(SHA-256 `585f455e83f3e1e7ba4fe91ce24fb313dc0347dc281a265fe8b178a49a97a9d0`)로
같은 원본과 기존 한컴 Print를 다시 비교했다. [12쪽 TSV](../assets/issue_8136/final-integration-8438/native-slice-rebase-de2295.tsv)의
4·5·6·9쪽은 각각 98.75·99.37·99.19·99.47%이고, 2·11·12쪽은 각각
64.80·33.82·37.72%로 미달이다. 전쪽 90%와 fresh WASM 검증은 아직 완료되지 않았다.
[이슈 #5871](https://github.com/edwardkim/rhwp/issues/5871) 대조군 3쪽의 Native TSV는
앞서 기록한 후보 3의 표시값과 모두 동일하다.

### 2026-11-02 의존성 리베이스 후 전체 Rust 회귀

최소 글꼴 환경에서 전체 선택 12,369건을 두 실행으로 완료해 모두 통과했다.
첫 실행의 중단과 나머지 307건의 재실행, 정확한 목록 대조는
[공통 실행 근거](../assets/review_8438_followup/rebase-de2295-rust.md)에 기록했다.
이는 이 PR의 시각 검증이나 최종 판정을 대신하지 않는다.
