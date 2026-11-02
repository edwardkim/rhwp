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

4쪽의 표 윗선은 RHWP 135–136px, Print 148–149px다. 원본의 직전 표 호스트 저장 줄은 `vpos=72484`, `lineHeight=62183`, `lineSpacing=552`이고 다음 표 호스트는 `vpos=136219`다. 따라서 저장 줄 끝과 다음 줄 사이 `136219-(72484+62183+552)=1000 HWPUNIT`이 남으며, 다음 표 문단의 앞 간격에 해당하는 13.33px와 Print의 이동량이 맞는다. 현재 공통 `typeset/table/host_spacing.rs`는 단 상단의 문단 앞 간격을 제외하고, `table/block/entry.rs`도 표가 새 단 상단에서 열릴 때 `column_top_before`로 제한한다. 이는 원인 후보이며, 다른 저장 사다리와 [이슈 #8149](https://github.com/edwardkim/rhwp/issues/8149) 대조군을 함께 확인하기 전 전역 완화로 결론 내리지 않는다. 5·6·9쪽은 단순 13px 이동과 달라 별도 경로도 조사한다.

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

[PR #8309](https://github.com/edwardkim/rhwp/pull/8309)가 반영된 base
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

표 호스트 문단 12의 저장 `vpos=136219`는 직전 표 줄의
`72484 + 62183 + 552`에 현재 문단의 앞 간격 `1000 HWPUNIT`을 더한 값이다.
`stored_tac`의 저장 사다리 판정은 직전 쪽에서 소비한 줄과 이 간격을 확인하지만,
새 단의 `saved_origin`을 0으로 다시 정해 간격을 버린다. 후보는 이 확인을 통과한
연속 줄에서만 미소비 앞 간격을 공통 원점에 반영한다. 해당 원점은 개체의 fit과
`InlineBoxPlacement`의 실제 위치·흐름 끝이 함께 사용한다. 저장 사다리가 끊기거나
새 축으로 다시 시작하는 경로는 기존대로 둔다. 아직 후보의 실행·대조 결과는 없다.

9쪽 표 앞 공백 글줄은 문단 51의 첫 저장 줄이며, 이전 표 줄 끝과 후행 간격의 합
`403295 + 52868 + 552`가 그 줄의 `vpos=456715`와 같다. 한컴 Print는 이 표를
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

추가 후보 실행에서 6쪽의 다음 문단 진입 원점은 29.39→22.03px, 표의 원점은
51.41→44.05px가 됐다. 기존 한컴 PDF와 Native 96 DPI 직접 대조 결과 4·5·6·9쪽은
각각 98.75·99.37·99.19·99.47%다. 총 12쪽은 유지되며 9쪽의 공백 선행 줄은
8쪽, 표는 9쪽에 속한다. [PR #8396](https://github.com/edwardkim/rhwp/pull/8396)의
4개 표 분할 대조군은 전쪽 문단·표 소속과 총 쪽수가 유지됐다. 이는 Native 후보의
중간 확인이며 전체 시각·회귀 검증 완료가 아니다. 2·11·12쪽은 여전히 미달이다.

11쪽의 문단 54는 표 다음에 글줄이 이어지고, 12쪽 문단 55는 표 줄의 후행
간격이 음수다. 둘 다 직전 쪽에서 끝난 저장 줄 다음에 문단 앞 간격 1000
HWPUNIT이 정확히 이어진다. 단일 저장 줄 계획이 수용하지 않는 일반 경로에도
동일한 저장 사다리 확인을 적용하는 후보를 마련했다. 표 첫 줄의 소유·외곽
높이와 실측 높이가 일치하는 경우에만 대상이 되며, 실제 배치와 대조군은
후보 실행 뒤 확인한다.
