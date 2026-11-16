# 정본 보호값 재현 조사

## 동일 환경의 실제 devel·후보 대조

검증 base는 최신 `53da4d2da28ae0cacf8feac3a989cc6a69fb2622`, 후보 제품 source는 `eeaecff59e834f28437c98b509089bbb36a7682d`다. 두 Native 바이너리를 각각 lib/bin 함께 빌드해 같은 입력·독립 PDF·현재 승인 font context·실제 프로그램 공급으로 정본 18 dataset·1,771쪽씩을 전수 실행했다.

현재 정본 `final.tsv` SHA-256은 `1063d78078b704a2bfb712ca3929ab251596dd391b41bc1a9a44e9935e4990e5`다. 과거 공식 두 자리 값보다 낮은 행은 양쪽 모두 **51건**이고, base·후보의 공식 측정값 차이는 **0건**이다. 이번 수정이 만든 Native 하락으로 분류하지 않았지만 정본 gate 통과로 처리하지도 않았다. [base TSV](protected-base-native.tsv) · [후보 TSV](protected-native.tsv).

| 문서 key | 과거 공식값 미재현 행 |
| --- | --- |
| nested | 6 |
| sample16-hwp2022 | 38 |
| sample16-hwp2024 | 2 |
| sample16-hwpx | 2 |
| seo-body-list-typed | 1 |
| seo-helvetica-styles | 1 |
| yeongwol | 1 |

최저는 양쪽 모두 97.99%다. 90% 이상은 특별 보호값 하락 금지 조건의 대체가 아니다. 편람 HWPX Native 383쪽 `--handbook-gate`는 별도로 통과했다.

PNG는 1,771쌍 중 1,762쌍이 바이트 단위로 같다. 다른 9쪽은 HWP3 sample10의 괘선 영역이며, 원본 PDF·base·후보를 같은 배율로 직접 비교했다. 그 문서의 기존 끝 괘선 표시 차이를 이번 수정이 해결했다고 주장하지 않는다. 공식값과 본문 위치는 동일하다. [전쪽 PNG 비교 집계](protected-base-native-png.json). 새 devel 기능 보존 41쪽씩은 Native/WASM 82쌍 모두 바이트 단위로 같다. [보존 결과](incoming-preservation.json).

## 기준 생산 source와 실제 글꼴

최근 기준 생산 기록은 `planet6897_integration_20261114/protected-validation.json`의 runtime `5fa2645186fc8bec4e53074aa6ff0af7ef67f0ca`와 “normal installed fonts; no regression sandbox”다. GitHub의 실제 Git tree/blob을 읽어 현재 base와 비교했다. Cargo.toml·Cargo.lock·vendor는 같고 src의 차이는 `shape_layout.rs` 한 파일이다. 내용은 뒤에 통합한 #9021의 공통 control 줄 소속·TAC 공유 기준선 보완이다. [실제 blob SHA](protected-producer-src-differences.json) · [차이](protected-producer-shape-diff.patch). 따라서 source와 환경이 모두 같은 생산 결과라고 가정하지 않는다.

현재 실행은 해당 기록의 문서별 font environment를 그대로 사용했다. 이전에 커밋된 프로그램 공급 목록에 맞춰 준비한 파일들과 선택한 CFF 프로그램 ID·해시를 확인했고, Noto와 수식 글꼴은 해당 문서에만 공급했다. [실제 실행별 프로그램 해시](actual-font-program-receipts.json). 상용 글꼴 파일 자체는 포함하지 않는다.

선행 글꼴 목록의 HBATANG은 32,483,900바이트·SHA-256 `bef6371830a14b7e275087f86517074392a54a9147819506a2c07637c88b4068`이다. 사용자가 안내한 `/home/edward/mygithub/ttfs/hwp/HBATANG.TTF`는 재확인 결과 32,361,044바이트·`35f84328500fc2c3ee0b148aa75de0ed384bf9eee8dee9148c01b0a11a27fe05`다. 한컴 2020 설치본도 현재 파일과 같다. 최신 기준 생산자의 전체 실공급 목록은 확보하지 못했다. 다른 해시의 존재만으로 이 차이의 원인을 HBATANG으로 확정하지 않는다.

## 진행 상태와 다음 확인

최종 fresh WASM 보호 1,771행은 최저 94.65%이며 과거 공식값 미재현은 56행이다. Native와 합계 3,542행·107건이고 실제 실행에서 누락·쪽수 불일치·90% 미달은 없다. 특별 보호 gate는 통과하지 않았다. [최종 집계](protected-results.json). 미재현 56행을 exact base source의 Docker·wasm-opt 완료 프로그램으로 같은 조건에서 재실행했다. **56행 모두 후보와 값이 같고 PNG도 56쌍 모두 같다.** [실제 base/candidate 값·프로그램 SHA·PNG 대조](protected-base-wasm-selected.json) · [base 원본 TSV](base-wasm-selected/). 이는 미재현 56행의 대조이며 base WASM 전 1,771쪽을 실행했다고 확대하지 않는다.

현재 조판 보완 #9070·#9071은 특별 보호값 재현 조건 때문에 제출 준비 완료로 판정하지 않는다. 기준값·기대값·font mapping·비교 영역을 임의로 바꾸거나 기존 차이 예외를 적용하지 않았다. 특별 보호 gate를 통과하려면 기준 생산자의 실제 프로그램·환경을 확보해 재현하거나 확인한 기존 차이를 보완·재검증해야 한다. 별도 방법은 이번 PR의 기존 차이에 한정한 작업지시자의 명시적 수용 결정이며 아직 승인되지 않았다. 정확한 base와의 동일성만으로 정본 gate 통과를 선언하지 않는다. 최종 전체 회귀 13,574개와 모든 Rust lint는 통과했고 Native 공식 3종도 4,073개 통과했다.


## 사용자 안내 파일 재확인

2026-11-15 KST 사용자가 다시 확인한 `/home/edward/mygithub/ttfs/hwp/HBATANG.TTF`가 실제 공급 파일이다. 현재 크기·해시는 위 `35f843…` 기록과 같으며 이 파일을 찾지 못했거나 잘못된 파일로 판정한 것이 아니다. 같은 파일은 #9070 독립 Print의 hmtx·전진 대조에서도 확인했다. 과거 기록과 해시가 다른 사실을 실제 보호 차이의 인과관계로 확대하지 않는다.


## 미재현 문서의 직접 판독

현재 최종 프로그램의 독립 PDF·Native·fresh WASM을 같은 페이지·배율로 나란히 읽었다. 아래는 미재현 범위의 대표 판독이며 107행의 원인을 모두 글꼴로 확정한 것이 아니다.

| 원본·쪽 | 직접 확인한 차이와 보존 범위 |
| --- | --- |
| [seo-helvetica-styles 1쪽](protected-review/seo-helvetica-styles.png) | STYLE 1의 Native 글꼴 폭·굵기와 WASM 획 굵기가 독립 Print와 다르다. 기울임/굵기 조합의 문자 자체와 순서는 남는다. HBATANG 한 파일로 설명할 수 없는 별도 영문 face 선택 범위다. |
| [seo-pic-crop-01 1쪽](protected-review/seo-pic-crop-01.png) | 두 그림 내용·자르기 위치는 같고 독립 Print의 외곽 검은 테두리가 제품에서는 더 가늘거나 일부 희미하다. 실제 차이를 높은 점수로 면제하지 않는다. |
| [nested 16쪽](protected-review/nested.png) | 문단·제목의 줄 소속과 내용 순서를 대조했다. 획·영문 메트릭 차이가 남으며 source와 환경의 인과관계를 추가로 구분해야 한다. |
| [sample16-hwp2022 30쪽](protected-review/sample16-hwp2022.png) | 표 외곽·셀 내용 소속과 후속 내용은 보존된다. 글꼴 획·영문 형태의 차이를 확인했다. 38행 전체의 단일 원인으로 확대하지 않는다. |

Native의 정확한 base 비교는 전쪽 동일 측정값이지만, 동일하다는 이유로 위 기존 차이를 해결됐다고 판정하지 않는다. 이 범위를 추가 보완하거나 현재 PR에 한정해 기존 차이를 수용하는 명시적 결정 전에는 통합을 보류한다.


## 최종 base 대조 결론

2026-11-15 KST exact base 대조를 완료했다. 과거 공식값 미재현 **107행 전부**는 후보·최신 devel의 측정값과 실제 PNG bytes가 같다. Native 미재현 51쪽을 별도로 다시 대조했고 WASM 미재현 56쪽도 같은 결과다. [전 항목 동일성](existing-protected-differences.json). 나머지 Native 전 1,771행의 공식값도 base·후보가 같다.

이것은 이번 수정의 새 하락이 아님을 확인한 근거이며 과거 정본과의 차이를 해결한 결과는 아니다. 기존 차이에 관한 작업지시자 결정 전 현재 `머지 보류`를 유지한다. 새 회귀·90% 미달·쪽수 불일치·내용 누락·검사 실패를 허용하는 예외는 제안하지 않는다.


## 2026-11-15 작업지시자 결정과 추가 보완 범위

작업지시자는 “예외 없이 기존 차이도 추가 보완”을 선택했다. 앞서 제안한 한정 수용 경로는 채택하지 않으며 PR·push·댓글·merge·issue close 승인이 아니다. 기존 검증 head와 증거는 보존하고 같은 검토 branch에서 로컬 추가 보완을 진행한다.

1. 정본 생산·원 해결 PR·실제 입력/PDF/프로그램과 Native PNG·WASM Canvas·SVG의 소비 경로를 대조한다. 지표 재현 차이와 독립 Print에서 보이는 실제 결함을 구분한다.
2. 영문 face/style의 원문→열거·공급→실제 측정/paint 선택, 그림의 저장 테두리→공통 paint→실제 Canvas/Skia clip·stroke를 추적한다. sample16·nested·yeongwol은 첫 발산 glyph/줄과 나머지 문서의 공통 원인을 확인한다. 문서별 숫자 맞춤이나 글꼴/기준값 임의 변경은 하지 않는다.
3. 확인한 공통 원인을 보완하고 작은 실제 출력 대조로 방향을 확인한다. 기존 입력과 독립 Print를 유지하고 바뀐 소비 경로의 정상·반례를 함께 검증한다.
4. 최종 Native/fresh WASM 전쪽 보호 3,542행의 하락 0건, 원 PR 목표 범위·새 devel 기능 보존, 필수 회귀·lint·Native·편람 검사를 완료한다. 새 코드를 예전 통과 결과로 판정하지 않는다.

확정한 기존 차이 보완 범위에 대해 같은 승인을 반복 요청하지 않는다. 독립 근거가 부족한 원인은 조사 상태로 남기고 실제 보정·검증을 계속하며 원격 수용은 보류한다.


### 첫 원인 확인: 실제 대체 face 공급과 그림의 paint 소유 순서

- #8109/#8132의 `f45878c7caafb43ca1fd017ddd93484a0acc6ad2`와 기존 네 스타일 회귀는 원문 Helvetica·스타일 face와 명시 대체 Arial을 함께 보존한다. 독립 Print의 내장 프로그램도 ArialMT/Bold/Italic/BoldItalic이다. 이전 공급 준비는 #8896의 제한 목록을 재사용해 Arial 4개를 빠뜨렸다. 원문·font environment·제품 코드를 바꾸지 않고 실제 Arial을 추가 공급한 Native/fresh WASM 전 1쪽이 모두 100.00%다. Native body-list의 99.95%는 그대로이므로 이 발견으로 다른 미재현을 해결했다고 확대하지 않는다.
- 그림 프레임 공유는 `7b2c77dd34`에서 현재 `picture_border.rs`로 통합됐다. 하지만 `paint_order::children`는 Column 자식을 layer/z-order로 정렬하면서 owner Image는 앵커 plane, 그 테두리는 layer 없는 일반 흐름으로 분리한다. 실제 Print Canvas의 호출 기록에서 `strokeRect → drawImage`가 두 그림 모두 재현됐다. 그림이 자기 테두리를 나중에 덮으므로 위·왼쪽 외곽이 사라진다. 같은 원본의 독립 PDF에는 두 그림의 네 외곽과 fill+stroke가 존재한다.
- 보완할 공통 결과는 테두리의 paint 정렬 키다. `RectangleNode.image_frame_owner`로 같은 부모의 실제 Image를 찾아 그 Image의 layer/z-order/문서 주소 키를 소비한다. 안정 정렬에서 원래 발행된 image→border 순서는 유지한다. owner 없는 도형·다른 clip 부모·비그림 Rectangle은 기존 규칙을 유지한다. SVG·Native·Canvas는 같은 `paint_order::children` 결과를 소비하며 별도의 backend 보정은 추가하지 않는다. 좌표·선폭·이미지 crop·원본 z-order를 변경하지 않는다.

후속 Native/fresh WASM 실제 출력·뒤/앞 개체 대조와 정식 관계 회귀로 수정 전 FAIL/수정 후 PASS를 확인한 뒤 최종 보호 검증을 다시 실행한다.


### 실제 실행 복원과 Native 자동 공급 우선순위 보완

선행 #8896의 최종 `font-supply/sample16-*-font-environment.json`은 원래 HFT에 없는 기호를 검증한 Myeongjo·독립 구형 수식 프로그램으로 공급한다. 이번 최초 보호 명령은 앞선 `pr8389_8397` 설정을 사용해 이 연결을 빠뜨렸다. 글꼴 파일 목록만 복원한 것으로 환경 전체를 복원했다고 판단한 것은 검토자의 누락이다. 기존 최종 설정과 실제 파일을 함께 공급하자 선택한 sample16 2022의 6·30·43·51쪽과 HWPX/2024의 43·51쪽은 Native/fresh WASM 모두 100.00%다. 아직 전쪽 보호 gate 완료는 아니다.

Native 영문 대조군에는 공급 도구의 독립적인 우선순위 오류도 확인했다. `visual_sweep_native_webfonts.prepare`는 원 face가 없을 때 문서에 선언된 로컬 Arial이 있어도 registry의 Helvetica Black Condensed를 먼저 내려받았다. 실제 manifest의 `Helvetica → HelveticaNeueW02-97BlackCn`과 PNG의 원래 없는 굵은 압축체를 확인했다. #8565 `5551091085` → #8583 `96827b3acc` → 변환 신원 보완 `72216ada1c`를 대조한 뒤, 기존 원본/embedded/exact alias 우선순위는 유지하며 선언된 실제 로컬 대체를 registry보다 먼저 사용하도록 공통 준비 함수를 보완했다. 원문 IR·font mapping·정본 수치·비교 마스크는 바꾸지 않는다.

새 공급 경로 관계 검사는 수정 전 웹 다운로드 발생으로 FAIL, 수정 후 전체 관련 14건 PASS다. 합성 공급 계약이며 그 자체를 실제 시각 통과로 주장하지 않는다. 실제 입력의 Native 재출력·fresh WASM과 최종 전쪽 검증은 계속 진행한다. nested·yeongwol의 잔여 차이도 별도로 유지한다.


### Native CLI 공급과 원본 HFT Print 프레임의 추가 추적

Python 공급기 보완 뒤에도 실제 Native 재출력은 굵은 압축체를 그렸다. manifest의 Python mappings가 비어 있음을 확인하고 후속 호출 `cli/outputs::apply_font_environment → native_webfonts::prepare → merge_into_environment`까지 추적했다. Native의 공통 공급기에도 같은 registry 우선 분기가 있었다. #8597 `606c2bee0d`와 후속 `bf6282b640`·#8692 `0c888319aa`를 확인한 뒤 문서의 실제 로컬 대체를 registry보다 먼저 사용하도록 함께 보완한다. 일반 미공급 진단·원래 exact/embedded 우선순위는 유지한다.

중첩 표 16쪽은 실제 원문이 HFT 휴먼명조를 요청하고 한컴 Print는 Gulim의 동일 윤곽·hmtx를 쓴다. PDF의 Gulim과 Type3 모두 같은 종이 device CTM을 소비하며 15pt의 물리 글자 크기는 14.987749pt다. 현재 Print SVG의 해당 Gulim run은 source 20px/원점을 그대로 발행하고 transform이 없다. `style_resolver` 초기화는 치환된 슬롯의 HFT Print 자격을 먼저 버리고 이후 `preserveSourceMetrics` 환경도 복원하지 못한다. `paper_viewport::publish_glyph_geometry`가 이 자격을 소비하므로 backend별 원점 보정으로 해결하지 않는다.

보완은 원래 비내장 HFT 슬롯이 소유한 프레임을 표시 치환과 분리해 보존한다. 명시적 자연 프로그램 선택은 기존 환경 적용 단계에서 계속 프레임을 해제하며 원래 TTF 슬롯·내장 글꼴은 새로 활성화하지 않는다. 직전 `19f0847829`와 `2e262d0d4b`의 공통 프레임·원본 메트릭 보존 계약을 대조했다. 실제 원본 전쪽 재출력과 소유 관계 검사를 완료하기 전 통과로 판정하지 않는다.

추가 글꼴 공급 근거도 대상 PDF에서 추출한 윤곽을 재료로 사용하지 않았다. 로컬 SPMJ의 원래 U+2013 윤곽·1000em 전진은 독립 Print의 T6과 경로 방향/시작점만 다르며 같은 모든 선분이다. 원문의 기호 슬롯 HFT 한양신명조에 없는 글리프는 이 실제 Myeongjo 프로그램을, HCI Poppy에 없는 가운데점은 원문 기호 슬롯의 HY Sinmyeongjo를 사용하도록 명시한다. 영월 Print의 New Gulim은 실제 제공 Gulim의 네 표본 glyph 윤곽·hmtx가 모두 같고 Malgun Gothic과 다르므로 확인된 원 프로그램을 공급한다. 설정·프로그램·전체 쪽 결과를 최종 증거에 보존한다.


### 종이 기준 개체의 선행 정렬까지 연결

첫 `7bac062a66`은 Column의 `paint_order::children`만 보완했다. 최적화 WASM `f39e2f3b87`의 실제 Canvas는 여전히 두 그림 모두 `strokeRect → drawImage`, raw 94.65%였다. 따라서 첫 보완을 완료로 인정하지 않는다. 실제 tree에서는 bitmap·border의 layer가 이미 같으며, 더 앞의 `LayoutEngine::sort_paper_render_nodes`가 doc_path 없는 외곽을 owner보다 먼저 정렬한 것이 확인됐다. Column에 도달하기 전 원래 순서를 잃으므로 안정 정렬만으로 되돌릴 수 없었다.

공통 `paint_order::sibling_keys`가 같은 부모의 `image_frame_owner`에 해당하는 Image 키를 선택하고, Column paint 정렬과 종이 기준 `sort_paper_render_nodes`가 이 결과를 함께 사용하게 한다. 각 경로의 기존 plane/z-order/문서 주소 계산은 그대로 쓰며 picture_border가 아닌 도형과 실제 owner 없는 노드는 자기 키를 유지한다. `layout append → paper 정렬 → Column 소비 → LayerBuilder → 실제 replay` 전체 경로를 새 출력과 실제 PaintOp 순서로 재검증한다.


### 정밀 종이 CTM의 원본 TTF 대조

영월 원문의 `월별누계`는 새굴림 **TTF, 비내장, 13pt**다. 대상 Print의 New Gulim과 실제 공급 Gulim의 윤곽·hmtx가 같음을 확인했다. 원문 종이는 59528×84188HU이며 기존 600dpi 종이 프레임 계산은 sx=0.9994624739635827, sy=0.9989072595971114다. 독립 Print의 13pt TTF는 600dpi의 108 device em(12.96pt)에 같은 CTM을 적용한 12.949415pt이며 실제 PDF 값 12.9494152069pt와 맞는다. 대상 PDF에서 산출한 보정 상수나 임의 scale을 추가하지 않는다. 별도 font em 반올림 구현을 이번 값만으로 도입하지 않으며, 기존 종이 프레임을 원래 TTF도 소비해야 한다는 근거로 사용한다.

기존 HFT만의 자격 제한은 이 비내장 TTF 출력 계약을 누락했다. 원본의 명시적 TTF/HFT 슬롯과 기존 HWP3의 유형 미상 HFT 추론이 같은 `paper_viewport` 결과를 소비하도록 자격을 보존한다. Screen에는 적용하지 않으며 embedded 원본과 명시적 자연 프로그램 좌표 선택의 기존 경계는 유지한다. `preserveSourceMetrics=true`의 표시 치환은 프레임 해제로 해석하지 않는다. 초기 합성 대조군의 TTF 프레임 미적용 가정도 독립 TTF Print 근거에 따라 수정했으며 실제 대상 출력의 재검증을 계속한다.

`08c83701b8` Native의 실제 중간 재출력은 body-list 100.00%, 그림 100.00%, nested 16쪽 99.99%, 영월 1쪽 100.00%·3쪽 99.98%다. 이 중간 head의 두 Native focused harness 컴파일은 추가 좌표 보완 때문에 중단했고 통과 결과로 계상하지 않는다. 최종 source에서 정식 전체/집중 검사를 다시 실행한다.


### 최신 devel과 원본 TTF의 fill 이후 text 좌표

최종 전수 검증 전에 원격을 확인하니 #9083이 `f8234f6f90cb25d9601c9d5d762e0a565aea19e4`로 추가됐다. 원 기여 head 세 개와 모든 보완 SHA를 보존하기 위해 같은 통합 branch에 최신 devel을 충돌 없이 병합했다(`475d22a74d5a1b754748879aac401d272477c888`). 제품 변경은 paragraph/표/각주 소유, 영문 반각 em/탭, 공통 gradient·밑줄 paint, CanvasKit 전달이다. 해당 다섯 원본 389쪽과 편집/저장 반례의 유지도 최종 검증에 포함한다. 이전 기준의 통과를 이 새 base의 검증으로 대체하지 않는다.

`32ce8e6eca` Native의 작은 대조는 nested 16쪽·body-list·그림 각100.00%, 영월1쪽100.00%·3쪽99.98%다. 영월 Print SVG의 `누`와 바탕 본문에는 정밀 CTM이 여전히 없었고 현재 source y=963.5333은 sy를 적용하면 독립 PDF y≈962.4682와 일치한다. 원본 TTF text의 device 좌표와 도형 fill에 따른 기존 HFT Type3 그래픽 상태를 혼동해 TTF까지 정밀 좌표를 해제한 것이 남은 원인이다. 같은 비내장 source 자격을 사용하되 원래 TTF는 fill 뒤에도 정밀 text 프레임을 유지한다. HFT의 기존 fill 전/후 경계와 명시적 자연 프로그램 해제는 보존한다.

영월의 바탕 본문도 PDF의 실제 Batang과 로컬 Windows Batang의 `누·+·＊·월·계` 윤곽·hmtx가 모두 같다. 이전 관측 설정의 Haansoft Batang과 구분해 두 원 프로그램 Gulim/Batang을 명시 공급한다. 현재 진단을 대상 PDF의 글리프를 추출한 대체 글꼴이나 수치 맞춤에 사용하지 않는다.


### HWP 원문 유형과 실제 프로그램 종류의 분리 정정

앞 절의 “원문 TTF”라는 표현은 HWPX 변환본의 표기를 읽은 것으로, HWP 원본의 유형 확정 근거가 아니므로 정정한다. 별도 원문 font-decision 조회에서 영월의 새굴림·바탕은 **altType=0, embedded=false, substFont=null**이다. HWP5 FACE_NAME 하위 비트를 TTF로 확정하지 않는 기존 `try_resolve_styles_for_document`의 표20 해석을 보존한다. 원문 유형을1로 바꾸거나 변환본의 기본 TTF 표기로 덮어쓰지 않는다.

필요한 공통 결과는 기존 HFT 그래픽 프레임 자격과 별도의 `print_text_device_frame`이다. 명시적 HWPX TTF 슬롯은 초기값을 가지며 원 HWP5의 유형 미상 슬롯은 초기 TTF 자격을 얻지 않는다. `preserveSourceMetrics=true`로 실제 TTF를 선택한 세션만 프로그램의 text device 자격을 부여한다. 원문의 `source_font_is_ttf`는 변경하지 않는다. 자연 프로그램 선택·내장 원본의 기존 경계를 보존한다.

`ResolvedCharStyle → 언어별 TextStyle → PagePaintContext → TextRun/GlyphRun/GlyphOutline/CharOverlap`이 같은 결과를 전달·소비한다. TTF text는 fill의 Type3 그래픽 상태와 분리하고 기존 HFT의 fill 전/후 경계는 유지한다. 합성 관계 검사는 유형 미상 원문→명시 세션 TTF→fill 뒤 paint까지 실행하며 원문 altType=0 보존도 확인한다. 실제 시각 수용은 프로그램 파일을 함께 공급한 전쪽 실행으로 별도 확인한다.

## 2026-11-15 추가 전쪽 검증과 중첩 표 재검토

제품 `64f7bb449b`의 Native 보호 1,771쪽에서 하락은 중첩 표 1·8·9·12쪽의 네 행이다.
그 밖의 1,754쪽은 공식 두 자리 값 하락이 없다. 소규모 fresh WASM 보호 235쪽에서도 같은
중첩 표 네 행이 남았다. 편람·sample10을 포함하지 않은 이 WASM 실행은 전체 보호 완료가 아니다.

- 원 사례 네 경계는 최적화 fresh WASM에서 영월 1·3쪽, 본문 목록 1쪽, 중첩 표 16쪽,
  그림 crop 1쪽이 모두 raw 100.00%다. 그림의 실제 Canvas 호출은 각 image 뒤에 소유 테두리를 그린다.
- 중첩 표 8쪽의 `-`는 독립 Print T13 윤곽·전진 332가 로컬 HCI Hollyhock의 같은 문자와 일치한다.
  원문 글자모양 10도 영문 슬롯에 HCI Hollyhock을 선언한다. 그런데 한글 CFF 뒤의 표시 TTF가
  구두점을 포함하면 기존 coverage 판정이 영문 원 프로그램 선택을 막는다. 주 CFF coverage와
  대체 후보 전체 coverage를 분리해 공통 측정·run 소유 판정에 연결했다.
- 중첩 표 12쪽의 원 PUA 겹침은 내부 글자모양 목록이 없고 제어 슬롯 모양은 106(13pt)이다.
  기존 legacy 경로는 ID 0(10pt)을 만들었다. 누락을 제어 슬롯 모양 상속으로 고치고 명시 ID 0은 유지했다.
- 같은 12쪽의 작은 삼각형은 기존 #8896에서 확인한 현재 HBATANG 프로그램의 원 PUA다.
  글꼴 파일은 실제 공급 중이며 바뀌지 않았다. HY헤드라인M의 원문 슬롯에도 실제 기호 공급을 명시하니
  Native 99.82%에서 99.86%로 개선됐으나 공식 99.87%에는 아직 미달이다. 완료로 판정하지 않는다.
- 1쪽의 Gulim CID 프로그램은 원문 휴먼명조의 자연 프로그램 선택과 구분했다. 전체 문서에 적용하는
  실제 프로그램 선택 대조에서 1쪽은 98.58%(공식 98.44%)다. 쪽별 최고값은 선택하지 않는다.

새 수정의 Native/fresh WASM 및 전체 필수 게이트는 아직 실행 전이다. 기준값·허용치·쪽수는 유지하며
원격 처리 승인은 없다.

### 원 HFT의 독립 비트맵 기호와 명시 자연 TTF의 WASM Print

2026-11-15 12:27 KST. `7fbc8ba62b`에서 Latin 원 프로그램 소유 보정 후 Native 8쪽은
99.97%, 9쪽은99.99%이며 남은 차이는 U+2027이다. 원 Print의 해당 Type3 charproc에는
전진 500과 빈 fill만 있고, 실제 잉크는 별도 600dpi bitmap XObject다. 로컬 변환 HFT OTF에는
이 문자가 없다. 대상 PDF에서 윤곽·비트맵을 추출해 공급하지 않는다.

깨끗한 blank HWPX에 한양중고딕 HFT의 U+2027을 11·13pt로 넣고, 별도 원격 한컴 2020
11.0.0.9136의 전체 쪽 Print(method 0·one-up)로 독립 표본을 확보했다. job은
`b9c59e1c-a048-433c-93dd-79d88ca76a9a`다. 로컬 Windows Hwp.exe·COM·자동 인쇄는
실행하지 않았다. 독립 bitmap의 원 검정 픽셀과 실제 text/image 행렬로 em 윤곽을 만들며
대상 점수에 맞춘 이동·배율·획 보정은 하지 않는다. 입력 SHA-256은
`cf3ad6c2075bacb54c45d3c62f82f454c70232efe8b6a9e1308bc8962d17d2ce`, 독립 Print는
`a298eaa707b883c10572c43e4801ee62abb5c59693acfa8294798ac1a99a7fa0`, 로컬 표시 프로그램은
`71a1a1827048cd71cd23bfbd2f04e3403980ac723c1859723a6c6a0d354ed642`다. 상용 글꼴 자료는
재배포하지 않고 `/home/edward/mygithub/ttfs/local-validation/hft-symbols`에서 명시 공급한다.

이 동일 공급 조건의 Native 중첩 표 17쪽은 보호값 하락 0건이며, 8·9·12쪽은 100.00%다.
WASM도 세 쪽은 100.00%지만 자연 Gulim 선택을 사용한 10·11·14·15쪽에서 과거 보호값보다
0.06–0.15%p 낮았다. 글자 원점·내용·레이아웃 tree는 Native와 같다. Native Print와 WASM의 글자 재생 경로가 다르며, WASM은 원 HFT의 CFF 분기에서 TTF를 발견해도 Print TTF
공급 경로로 이어지지 않고 브라우저 글자 API로 되돌아간다.

`HftOutlineFonts::print_fallback_glyph → WebCanvasRenderer::draw_hft_outline → positioned
text paint`가 실제 등록된 정적 TTF의 같은 윤곽을 소비하게 한다. 적용은 기존 원 TTF 대체와
원 HFT의 **명시 자연 프로그램 선택**이며, 자동 표시 치환·화면·굵기·기울임·가변/컬러 face·
공급자 미확인·문맥 shaping은 확대하지 않는다. 원문 유형·레이아웃 전진·좌표는 바꾸지 않는다.
저장소의 OFL TTF로 실제 WASM·FontFace·Canvas를 실행한 새 관계 검사는 수정 전 package에서
Print의 `fillText` 두 호출과 path 0개를 검출해 실패했다. 수정 후 fresh WASM과 네 쪽의 재출력,
대조군·전체 보호·필수 게이트는 아직 완료하지 않았다. 앞서 시작한 7fbc 전체 회귀는 컴파일 단계에서
중단했고 통과 수에 넣지 않는다. Native 32개 focused와 Python 14개 검사는 해당7fbc 결과로 구분한다.

첫 WASM 보완 `bd77ca1b46`의 검증은 정상 대조 3건이 통과하고 새 자연 TTF 검사 1건이 실패했다.
명시 2024 Print로 같은 검사를 실행하면 통과했다. 원인은 실제 TTF 공급·윤곽 추출이 아니라
`preserves_raster_text_geometry`가 기존 2020/2022 Print에서 false인 점이다. 명시 자연 HFT
프로그램의 정적 TTF replay만 Print 전 세대에서 허용하고, 보통 원 TTF fallback의 기존 세대별
정책과 Screen·HighQuality는 보존한다. 같은 관계 검사가 2022·2024를 각각 실제 실행하도록
보완한다. 대상 글자의 좌표·크기·레이아웃 전진에는 새 보정을 넣지 않는다.

Native `7fbc8ba62b`의 보호 18 dataset·1,771쪽은 전쪽 raw 최저 98.58%, 공식값 하락 0건으로
완료했다. 별도 편람 383쪽 gate도 통과했다. 이후 두 Rust 변경은 wasm32 함수/모듈에 한정되며
Native 시각 producer를 이후 SHA로 바꿔 적지 않는다. 최종 Rust 게이트와 실제 새 WASM 검증은
계속 진행 중이다.

2026-11-15 13:08 KST 정정: Native의 `preserves_raster_text_geometry`도 세대별 정책이다.
따라서 앞선 “Native는 무힌팅” 설명을 2020/2022에 일괄 적용하지 않는다. 이 보완은 Native
정책을 변경하지 않고 명시 자연 TTF의 실제 윤곽을 WASM Print에 재생하는 처리다.
2024 경로의 진단에서 네 쪽은 각각 98.83·99.08·98.69·99.18%로 보호값 이상이지만, 이 결과를
2022 최종 통과로 계상하지 않는다. 최종 2022 fresh WASM을 별도로 빌드·측정한다.
보완 전 최적화 WASM 전수 1,771쪽의 하락은 위 네 쪽뿐이며 나머지 1,767쪽에는 하락이 없다.


### 전체 회귀에서 확인한 프로그램 자격 경계 보완

최적화 WASM `e29231c1f5`의 실제 2022 Print 재출력에서 중첩 표 10·11·14·15쪽은
98.83·99.09·98.69·99.19%로 각각 과거 보호값 이상이다. 8·9·12쪽과 그림 테두리·영문
대조군도 100.00%를 유지했다. 이는 작은 경계 확인이며 전쪽 최종 gate로 확대하지 않는다.

직전 전체 회귀는 13,597건 중 13,593건 통과, 4건 실패였다. 실제 기존 검사 3건은
`source_hft_original_program_print_frame`의 다른 프로그램 조회와
`issue_7445_hft_glyph_paint_frame`의 HWP3 대체·원 TTF 무인증 경로를 보호한다.
원 frame 보완 `19f08478296e`가 수용된 [통합 PR #8857](https://github.com/edwardkim/rhwp/pull/8857)과
독립 Print·보호 검사 복원 `c9b60993e79b`의 [PR #8188](https://github.com/edwardkim/rhwp/pull/8188),
기존 독립 Print 기록을 먼저 대조했다. 두 원격 수용 상태도 읽기 전용 API로 재확인했다.
새 보완의 원문 유형 비트·자동 표시 치환만으로 프레임을 부여한 처리가 이 경계를 넓혔다.
기존 기대값은 바꾸지 않고 공통 resolver가 이미 사용하는 실제 원 primary 선택 자격으로
TTF text device frame을 제한했다. 다른 프로그램을 조회하면 HFT 프레임을 해제하는
기존 계약도 복원했다.

나머지 실패 1건은 이번에 추가한 합성 프로그램 소유 검사다. 수동 `alt_type` 선언과 Gulim
표시 치환을 실제 원 프로그램 인증으로 가정한 입력을 고쳤다. 원 프로그램의 명시 선택,
유형 미상 원문 보존, 자연 프로그램 선택을 각각 검사하며 실제 기존 3개 검사를 유지한다.
수정 source는 `b5cda1957036dd4e800a9335e251469cf8ca172f`다. 이 source의 집중 회귀,
Native/fresh WASM 재빌드·전쪽 보호·필수 게이트가 끝나기 전에는 완료로 판정하지 않는다.


집중 재검증은 최종 합성 입력 보완 `411cb7a7e807abbea90c6ab6b6ca0c66b6c3ad71`에서
27건 모두 통과했다. 원래 정확한 검사 3건의 기대값은 유지했다. 새 합성 검사는 기존 폭 검증이
있는 Times New Roman primary를 명시하고, fill 대조는 `control_index`가 있는 원 도형을
사용한다. 일반 배경 사각형은 2022 Print의 polygon fill과 다른 계약이므로 이를 혼동하지 않는다.
`b5cda19570`/`5ae3222ad6`의 각 실패 결과도 보존하며 성공 수로 합산하지 않는다.


2026-11-15 14:12 KST 최적화 fresh WASM `411cb7a7e8`을 완료했다. 파일 SHA-256은
`4635f221ed76429fa7dd8c0fa1f0e943257afbd580297a3939130e69604f8d0d`다. 기존 CI가 수집하는
실제 브라우저 묶음 20건이 모두 통과했다. 새 자연 TTF의 2022·2024 및 기존 인쇄 기하,
Screen 복귀·미공급·가변 face 경계도 이 묶음에서 실제 실행했다. 경계 21쪽의 Native/WASM
보호값 하락은 양쪽 모두 0건이다. 필드·중첩 표·범위 초과 방어와 HWP/HWPX 두 차례 반복
저장·재열기도 새 WASM에서 통과했다.

현재 Native 목표·대조군 134쪽과 최신 devel 추가 389쪽은 측정을 완료했다. 같은 입력·PDF를
exact base `f8234f6f90cb25d9601c9d5d762e0a565aea19e4`와 대조한 정밀 raw 값(표시 반올림 전)의
최대 하락은 추가 389쪽에서 0.10884%p, 앞선 41쪽 보존군에서 0.05736%p다. 2%p 초과는 0건이다.
바탕쪽 66쪽·색상 6쪽·추가 여백/도형/저장 대조 18쪽은 모두 100.00%다. 이 수치를 최종
전쪽 보호 3,542행과 전체 회귀 완료로 확대하지 않으며 남은 전수·필수 게이트를 계속 실행한다.


### 확대 판독에서 남은 legacy Print의 그림 pen

`411cb7a7e8`의 전체 회귀 13,597건·Native 보호 1,771쪽·편람 gate는 모두 통과했다.
그러나 원본 PDF·현재 Native/WASM 그림의 네 변을 확대하자 2022 경로의 선폭이 여전히
얇았다. 높은 점수로 이 차이를 면제하지 않았다. 공개 `pic-crop-01-2022.pdf`의 compound
fill path에서 바깥/안쪽 차이는 0.480pt, 외곽 pen은 0.120pt다. 96dpi에서는 물리 띠
0.64px와 pen 0.16px이며 600dpi에서는 각각 4·1 device pixel이다.

원 보완은 `c417310cab62`·PR #8389다. 동일 source를 기존 2024 Print 경로로 출력하면
실제 pen은 `[0.64, 0.16]`, 최종 raster 선폭은 1.64px이고 독립 2022 PDF의 왼쪽 edge coverage와
같다. 기존 2022 경로는 이 metadata가 없고 최종 1px만 그린다. 원인은 공통 `LayerBuilder`가
`picture_border_pen`의 장치 픽셀까지 비트맵 색상 세대 옵션으로 걸러 낸 것이다.

`41be35bbb4`는 같은 공통 pen 함수에 장치 픽셀을 그대로 전달한다. 그림 소유·원문 폭·crop·
일반 사각형의 세대별 좌표·비트맵 색상 옵션은 바꾸지 않는다. 문서별 상수나 새 선폭 보정도
추가하지 않는다. 직전 411 checkpoint는 보존하고 새 Native/fresh WASM의 작은 실제 비교부터
진행한다. 최종 전수·필수 게이트를 다시 완료하기 전에는 새 source의 완료로 판정하지 않는다.


### 최신 devel 통합과 재검증 범위

2026-11-15. 기준을 `f8234f6f90cb25d9601c9d5d762e0a565aea19e4`에서
`e7ee272cfa33a960b0d0d924983d29e6509903e0`로 갱신한다. 수용된 PR #9088의
첫 표 조각 수용·Print 주 프로그램·소수 원점·HFT 막대 소유와 상향된 보호 하한 30행을 보존한다.
승인된 메인테이너 보완 범위에서 두 충돌을 통합한다. PUA 내부 서식 상속은 양쪽이 같은
구현이며 최신 설명을 채택한다. WASM은 원 CFF → 실제 CSS 주 프로그램 → 기존 자격을
통과한 정적 fallback 순서로 소비하며 명시 자연 프로그램·Print 경계를 보존한다.

이 통합 직전의 중간 결과는 `final-without-exceptions/`에 producer별로 보존했다.
새 base의 14쪽 원본·글꼴 경계와 기존 세 PR의 기능, 전체 보호·필수 검증을 새 source에서
다시 실행한다. 기존 결과를 새 source로 바꾸어 기록하거나 보호 하한을 되돌리지 않는다.


### 상향 하한의 실제 세 경계 재검토

통합 source `235da09ed62d23425dad71dddac2bffa9f70cdc1`의 Native 전쪽 1,771행은
하락 0건이다. 같은 source의 WASM 작은 경계에서는 중첩 표 4·11·14쪽이 새 하한보다
각각 0.01%p 낮았다. 기존 차이 예외나 표시값 재반올림으로 수용하지 않는다.

4쪽은 실제 형상이 다른 결측 기호였다. 원문 `굴림체` U+21E9를 독립 Print는
Haansoft Batang으로 표시한다. 로컬의 기존 명시 fallback은 `Gulim`만 대상으로 삼아
`GulimChe` 요청은 다른 시스템 기호로 넘어갔다. `GulimChe`·`굴림체`의 결측 체인을
같은 Haansoft Batang으로 연결한 문서 전체 설정에서 Native/WASM 4쪽이 모두 100.00%다.
글자·본문 좌표·문서 식별자나 쪽별 조건을 바꾸지 않는다. 글꼴 설치본 자체는 배포하지 않는다.

나머지 두 쪽은 별도 조사 중이다. 보호 생산자의 Gulim TTC 해시와 로컬 파일 해시는 다르지만,
기준 PDF에 포함된 370개 윤곽을 직접 대조했다. ToUnicode의 네 오기(3·데·접·남)를 분리하자
로컬 실제 글리프와 대응했다. PDF subset을 출력 공급자로 등록하거나 검증용 글꼴로 추출해
사용하지 않았다. 파일 해시 차이만으로 glyph 결함이나 새 글꼴 요구를 확정하지 않는다.
원본 프로그램·원 메트릭의 인증과 실제 공급한 TrueType paint 프로그램의 물리 text 좌표는
서로 다른 상태이므로, `NativeOtfMetrics → TextStyle → PagePaintContext::publish`의 소비를
추적한다. 미공급·CFF·명시 자연 잉크·Screen 대조 계약을 보존하고 실제 독립 출력으로 검증한다.


### 실제 공급 TrueType의 물리 text 프레임 보완 실험

정확한 devel `e7ee272cfa33a960b0d0d924983d29e6509903e0`의 frozen Native와 최적화
WASM을 현재와 같은 문서 전체 설정으로 실행했다. 두 경로의 11·14쪽은 현재 후보와 같으며,
WASM의 99.09%·98.69%도 그대로다. 이번 통합이 만든 하락은 아니지만 예외를 적용하지 않는다.

별도의 공개 TTF/CFF/가변 글꼴 최소 대조에서 실제 정적 TTF를 공급해도 Print text 프레임이
발행되지 않는 두 경계(2022·2024)를 재현했다. 미공급·CFF·가변·명시 자연 잉크·Screen/HQ
46개 음성 대조도 분리했다. `NativeOtfMetrics`는 cmap/hmtx만 남기는 축약을 수행하므로
paint에서 그 바이트를 다시 분석해 원 프로그램 종류를 추정할 수 없다. 원 글꼴 유형·폭 표
인증을 실제 paint 프로그램 형식의 대용으로 사용하는 현재 경계를 보완한다.

실험은 등록 시 실제 SFNT의 정적 glyf·비가변·비컬러·비SVG 사실을 보존하고, 그 family의
실제 공급 후보가 전부 이 계약을 지킬 때만 기존 Type0 text device 프레임을 사용한다.
원문 유형·전진·소스 bbox·기존 HFT의 fill 경계는 바꾸지 않는다. 명시 자연 잉크와 PDF의
자체 text paint 기하는 제외한다. `register_otf_for_source → NativeOtfProgram 형식 증거 →
NativeOtfMetrics → TextStyle.native_otf_metrics → PagePaintContext::publish_glyph_geometry →
동일 source의 TextRun/GlyphRun/GlyphOutline paintTransform`을 통해 공통 결과를 소비한다.
확인된 TTF의 Type0 text 좌표는 Type3 도형 fill 상태와 독립적이라는 기존 Print 계약을
사용하며 새 좌표나 글꼴별 상수를 만들지 않는다. 이 보완의 실제 출력·기존 대조·필수 검증은
아직 미완료이며, 점수가 개선되지 않거나 다른 계약이 후퇴하면 가정을 재검토한다.


첫 Native 재검증은 source `8ad1c5c08a`의 frozen binary다. 같은 문서 전체 환경에서
17쪽 모두 새 보호 하한 이상이며 11·14쪽은 각각 99.67%·99.42%로 개선됐다.
4·8·9·12·16쪽은 모두 100.00%다. 최저 98.63%와 전쪽 결과를 기록하며 아직 전체
최종 보호·새 WASM·필수 게이트의 완료로 확대하지 않는다.

이후 검증의 문서 전체 설정은 기존 원본 메트릭을 보존한다. 과거의 자연 Gulim 전진 선택을
제거하고, 독립 Print에서 확인한 `GulimChe`/`굴림체 → Haansoft Batang` 결측 연결을 추가했다.
두 backend에 같은 JSON을 공급하며 다른 페이지별 환경을 선택하지 않는다.


## 2026-11-15 최종 완료

제품 `8ad1c5c08a`의 Native/fresh WASM 각 1,771쪽은 최신 정본 대비 공식값 하락 0건이다. 기존 107행의 미재현 예외를 적용하지 않았고 영문 네 스타일·그림 외곽·원 기호의 직접 판독도 마쳤다. 실제 공급 정적 TTF의 형식을 메트릭 축약 전에 보존하고, 논리 원문 유형·전진을 유지한 채 공통 Print text 프레임을 소비한다. 정확한 최신 base와 원 사례 134쪽·추가 403쪽씩도 같은 조건으로 대조했고 2%p 초과 감소는 없다. 모든 필수 실행과 상승 19행 반영은 [최종 원장](final-source-8ad/README.md)에 있다. 이전 절의 미완료·실패 수치는 각 당시 source의 이력이며 현재 판정을 대신하지 않는다.
