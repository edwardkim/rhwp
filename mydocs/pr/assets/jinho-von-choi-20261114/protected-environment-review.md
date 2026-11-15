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
