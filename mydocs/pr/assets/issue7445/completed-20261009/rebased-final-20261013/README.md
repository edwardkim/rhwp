# 리베이스 후보 최종 검증 증적

검증 후보: `a7ce883cbbb284b83740e63e2b4f03502996104d`. 제출 시 추가되는 증적 commit은 제품·회귀 코드를 변경하지 않습니다.

- 전체 로컬 검증（로컬 중간 자료 `final-validation.json`）
- 전쪽 비교 요약·원본/PDF/환경 해시（로컬 중간 자료 `whole-visual-summary.json`）
- 실제 Native Skia·Studio Canvas2D/CanvasKit 검증（로컬 중간 자료 `actual-consumer-summary.json`）
- 대표 변경·사용자 지적 영역 직접 판독（로컬 중간 자료 `direct-inspection.json`）

[전쪽 Native·fresh WASM TSV 원본 ZIP](https://github.com/user-attachments/files/33360752/7445-final-native-fresh-wasm-tsv.zip) · SHA-256 `8cf352615813da4e9b802cf28b50978d6b0809953c2c2a5609748ada2e8b70fb`. ZIP 내부 `native/<입력>/silhouette.tsv`, `wasm/<입력>/silhouette.tsv`로 구분하며 TSV는 Git에 커밋하지 않습니다.

## 전쪽 비교

42입력의 Native/fresh WASM을 각각1,049쪽씩 대조했습니다. 90% 미만·누락·쪽수 불일치0건이며 기존 비교 기준과 독립 Print PDF를 유지했습니다.

| 입력 | 쪽수 | Native 최저 | WASM 최저 | 전쪽 TSV |
| --- | ---: | ---: | ---: | --- |
| biotoxin205 | 205 | 92.75882% | 92.75882% | [Native](https://github.com/user-attachments/files/33360752/7445-final-native-fresh-wasm-tsv.zip) / [WASM](https://github.com/user-attachments/files/33360752/7445-final-native-fresh-wasm-tsv.zip) |
| regulatory17 | 17 | 95.45455% | 95.45455% | [Native](https://github.com/user-attachments/files/33360752/7445-final-native-fresh-wasm-tsv.zip) / [WASM](https://github.com/user-attachments/files/33360752/7445-final-native-fresh-wasm-tsv.zip) |
| ktx003 | 1 | 95.81702% | 95.81702% | [Native](https://github.com/user-attachments/files/33360752/7445-final-native-fresh-wasm-tsv.zip) / [WASM](https://github.com/user-attachments/files/33360752/7445-final-native-fresh-wasm-tsv.zip) |
| low-slack6535 | 1 | 95.86412% | 95.86412% | [Native](https://github.com/user-attachments/files/33360752/7445-final-native-fresh-wasm-tsv.zip) / [WASM](https://github.com/user-attachments/files/33360752/7445-final-native-fresh-wasm-tsv.zip) |
| asset60 | 60 | 91.86922% | 91.86922% | [Native](https://github.com/user-attachments/files/33360752/7445-final-native-fresh-wasm-tsv.zip) / [WASM](https://github.com/user-attachments/files/33360752/7445-final-native-fresh-wasm-tsv.zip) |
| pii66-declared | 66 | 92.53112% | 92.53112% | [Native](https://github.com/user-attachments/files/33360752/7445-final-native-fresh-wasm-tsv.zip) / [WASM](https://github.com/user-attachments/files/33360752/7445-final-native-fresh-wasm-tsv.zip) |
| sample16-hwp | 64 | 91.28697% | 91.28697% | [Native](https://github.com/user-attachments/files/33360752/7445-final-native-fresh-wasm-tsv.zip) / [WASM](https://github.com/user-attachments/files/33360752/7445-final-native-fresh-wasm-tsv.zip) |
| sample16-hwpx | 64 | 91.28697% | 91.28697% | [Native](https://github.com/user-attachments/files/33360752/7445-final-native-fresh-wasm-tsv.zip) / [WASM](https://github.com/user-attachments/files/33360752/7445-final-native-fresh-wasm-tsv.zip) |
| sample16-2010-alias | 64 | 91.28697% | 91.28697% | [Native](https://github.com/user-attachments/files/33360752/7445-final-native-fresh-wasm-tsv.zip) / [WASM](https://github.com/user-attachments/files/33360752/7445-final-native-fresh-wasm-tsv.zip) |
| sample16-2022 | 64 | 91.28697% | 91.28697% | [Native](https://github.com/user-attachments/files/33360752/7445-final-native-fresh-wasm-tsv.zip) / [WASM](https://github.com/user-attachments/files/33360752/7445-final-native-fresh-wasm-tsv.zip) |
| press12 | 12 | 90.17175% | 90.17175% | [Native](https://github.com/user-attachments/files/33360752/7445-final-native-fresh-wasm-tsv.zip) / [WASM](https://github.com/user-attachments/files/33360752/7445-final-native-fresh-wasm-tsv.zip) |
| project2097-declared | 2 | 91.12756% | 91.12756% | [Native](https://github.com/user-attachments/files/33360752/7445-final-native-fresh-wasm-tsv.zip) / [WASM](https://github.com/user-attachments/files/33360752/7445-final-native-fresh-wasm-tsv.zip) |
| issue265 | 16 | 91.17993% | 91.17993% | [Native](https://github.com/user-attachments/files/33360752/7445-final-native-fresh-wasm-tsv.zip) / [WASM](https://github.com/user-attachments/files/33360752/7445-final-native-fresh-wasm-tsv.zip) |
| hwp3-alias | 16 | 91.17993% | 91.17993% | [Native](https://github.com/user-attachments/files/33360752/7445-final-native-fresh-wasm-tsv.zip) / [WASM](https://github.com/user-attachments/files/33360752/7445-final-native-fresh-wasm-tsv.zip) |
| converted16 | 16 | 97.05201% | 97.05201% | [Native](https://github.com/user-attachments/files/33360752/7445-final-native-fresh-wasm-tsv.zip) / [WASM](https://github.com/user-attachments/files/33360752/7445-final-native-fresh-wasm-tsv.zip) |
| rowbreak18 | 18 | 92.75195% | 92.75195% | [Native](https://github.com/user-attachments/files/33360752/7445-final-native-fresh-wasm-tsv.zip) / [WASM](https://github.com/user-attachments/files/33360752/7445-final-native-fresh-wasm-tsv.zip) |
| tac3 | 3 | 92.54967% | 92.54967% | [Native](https://github.com/user-attachments/files/33360752/7445-final-native-fresh-wasm-tsv.zip) / [WASM](https://github.com/user-attachments/files/33360752/7445-final-native-fresh-wasm-tsv.zip) |
| civil2 | 2 | 98.64096% | 98.64096% | [Native](https://github.com/user-attachments/files/33360752/7445-final-native-fresh-wasm-tsv.zip) / [WASM](https://github.com/user-attachments/files/33360752/7445-final-native-fresh-wasm-tsv.zip) |
| ole-hwp | 1 | 100.00000% | 100.00000% | [Native](https://github.com/user-attachments/files/33360752/7445-final-native-fresh-wasm-tsv.zip) / [WASM](https://github.com/user-attachments/files/33360752/7445-final-native-fresh-wasm-tsv.zip) |
| ole-hwpx | 1 | 100.00000% | 100.00000% | [Native](https://github.com/user-attachments/files/33360752/7445-final-native-fresh-wasm-tsv.zip) / [WASM](https://github.com/user-attachments/files/33360752/7445-final-native-fresh-wasm-tsv.zip) |
| rnote-cp01 | 3 | 100.00000% | 100.00000% | [Native](https://github.com/user-attachments/files/33360752/7445-final-native-fresh-wasm-tsv.zip) / [WASM](https://github.com/user-attachments/files/33360752/7445-final-native-fresh-wasm-tsv.zip) |
| rnote-cp02 | 4 | 100.00000% | 100.00000% | [Native](https://github.com/user-attachments/files/33360752/7445-final-native-fresh-wasm-tsv.zip) / [WASM](https://github.com/user-attachments/files/33360752/7445-final-native-fresh-wasm-tsv.zip) |
| table32 | 1 | 100.00000% | 100.00000% | [Native](https://github.com/user-attachments/files/33360752/7445-final-native-fresh-wasm-tsv.zip) / [WASM](https://github.com/user-attachments/files/33360752/7445-final-native-fresh-wasm-tsv.zip) |
| table33 | 2 | 100.00000% | 100.00000% | [Native](https://github.com/user-attachments/files/33360752/7445-final-native-fresh-wasm-tsv.zip) / [WASM](https://github.com/user-attachments/files/33360752/7445-final-native-fresh-wasm-tsv.zip) |
| table40 | 2 | 100.00000% | 100.00000% | [Native](https://github.com/user-attachments/files/33360752/7445-final-native-fresh-wasm-tsv.zip) / [WASM](https://github.com/user-attachments/files/33360752/7445-final-native-fresh-wasm-tsv.zip) |
| table30 | 2 | 100.00000% | 100.00000% | [Native](https://github.com/user-attachments/files/33360752/7445-final-native-fresh-wasm-tsv.zip) / [WASM](https://github.com/user-attachments/files/33360752/7445-final-native-fresh-wasm-tsv.zip) |
| ole-enter-hwp | 1 | 100.00000% | 100.00000% | [Native](https://github.com/user-attachments/files/33360752/7445-final-native-fresh-wasm-tsv.zip) / [WASM](https://github.com/user-attachments/files/33360752/7445-final-native-fresh-wasm-tsv.zip) |
| ole-enter-hwpx | 1 | 100.00000% | 100.00000% | [Native](https://github.com/user-attachments/files/33360752/7445-final-native-fresh-wasm-tsv.zip) / [WASM](https://github.com/user-attachments/files/33360752/7445-final-native-fresh-wasm-tsv.zip) |
| roadmap302 | 302 | 92.86894% | 92.86894% | [Native](https://github.com/user-attachments/files/33360752/7445-final-native-fresh-wasm-tsv.zip) / [WASM](https://github.com/user-attachments/files/33360752/7445-final-native-fresh-wasm-tsv.zip) |
| disaster | 2 | 96.21501% | 96.21501% | [Native](https://github.com/user-attachments/files/33360752/7445-final-native-fresh-wasm-tsv.zip) / [WASM](https://github.com/user-attachments/files/33360752/7445-final-native-fresh-wasm-tsv.zip) |
| empty-cell | 1 | 95.96280% | 95.96280% | [Native](https://github.com/user-attachments/files/33360752/7445-final-native-fresh-wasm-tsv.zip) / [WASM](https://github.com/user-attachments/files/33360752/7445-final-native-fresh-wasm-tsv.zip) |
| land-table | 5 | 98.68082% | 96.81731% | [Native](https://github.com/user-attachments/files/33360752/7445-final-native-fresh-wasm-tsv.zip) / [WASM](https://github.com/user-attachments/files/33360752/7445-final-native-fresh-wasm-tsv.zip) |
| ledger | 1 | 98.97669% | 98.97669% | [Native](https://github.com/user-attachments/files/33360752/7445-final-native-fresh-wasm-tsv.zip) / [WASM](https://github.com/user-attachments/files/33360752/7445-final-native-fresh-wasm-tsv.zip) |
| lighting | 2 | 94.56196% | 94.56196% | [Native](https://github.com/user-attachments/files/33360752/7445-final-native-fresh-wasm-tsv.zip) / [WASM](https://github.com/user-attachments/files/33360752/7445-final-native-fresh-wasm-tsv.zip) |
| none-image-fill | 14 | 97.76365% | 97.76365% | [Native](https://github.com/user-attachments/files/33360752/7445-final-native-fresh-wasm-tsv.zip) / [WASM](https://github.com/user-attachments/files/33360752/7445-final-native-fresh-wasm-tsv.zip) |
| yeongwol4 | 4 | 99.54887% | 99.54887% | [Native](https://github.com/user-attachments/files/33360752/7445-final-native-fresh-wasm-tsv.zip) / [WASM](https://github.com/user-attachments/files/33360752/7445-final-native-fresh-wasm-tsv.zip) |
| court2 | 2 | 90.06028% | 90.06028% | [Native](https://github.com/user-attachments/files/33360752/7445-final-native-fresh-wasm-tsv.zip) / [WASM](https://github.com/user-attachments/files/33360752/7445-final-native-fresh-wasm-tsv.zip) |
| superscript3 | 3 | 93.36519% | 93.36519% | [Native](https://github.com/user-attachments/files/33360752/7445-final-native-fresh-wasm-tsv.zip) / [WASM](https://github.com/user-attachments/files/33360752/7445-final-native-fresh-wasm-tsv.zip) |
| field-marker1 | 1 | 99.97837% | 99.97837% | [Native](https://github.com/user-attachments/files/33360752/7445-final-native-fresh-wasm-tsv.zip) / [WASM](https://github.com/user-attachments/files/33360752/7445-final-native-fresh-wasm-tsv.zip) |
| broken-ladder1 | 1 | 99.52965% | 99.52965% | [Native](https://github.com/user-attachments/files/33360752/7445-final-native-fresh-wasm-tsv.zip) / [WASM](https://github.com/user-attachments/files/33360752/7445-final-native-fresh-wasm-tsv.zip) |
| missing-lineseg1 | 1 | 95.45311% | 95.45311% | [Native](https://github.com/user-attachments/files/33360752/7445-final-native-fresh-wasm-tsv.zip) / [WASM](https://github.com/user-attachments/files/33360752/7445-final-native-fresh-wasm-tsv.zip) |
| quote1 | 1 | 92.76945% | 92.76945% | [Native](https://github.com/user-attachments/files/33360752/7445-final-native-fresh-wasm-tsv.zip) / [WASM](https://github.com/user-attachments/files/33360752/7445-final-native-fresh-wasm-tsv.zip) |

## 대표 review·overlay

각 입력의 원본·기준 PDF는 전쪽 요약에 기록한 저장소 경로를 사용합니다. 동일 바이트의 파일을 별도로 복사하지 않았습니다. Native/fresh WASM의 현재 PNG 해시와 직접 판독 근거를 함께 보존합니다.

### asset60

| 경로 | review | overlay |
| --- | --- | --- |
| native · p20 · 98.47349% | [review](asset60-p020-native-review.png) | [overlay](asset60-p020-native-overlay.png) |
| wasm · p20 · 98.47349% | [review](asset60-p020-wasm-review.png) | [overlay](asset60-p020-wasm-overlay.png) |

### regulatory17

| 경로 | review | overlay |
| --- | --- | --- |
| native · p11 · 99.52781% | [review](regulatory17-p011-native-review.png) | [overlay](regulatory17-p011-native-overlay.png) |
| wasm · p11 · 99.52781% | [review](regulatory17-p011-wasm-review.png) | [overlay](regulatory17-p011-wasm-overlay.png) |

### ktx003

| 경로 | review | overlay |
| --- | --- | --- |
| native · p1 · 95.81702% | [review](ktx003-p001-native-review.png) | [overlay](ktx003-p001-native-overlay.png) |
| wasm · p1 · 95.81702% | [review](ktx003-p001-wasm-review.png) | [overlay](ktx003-p001-wasm-overlay.png) |

### low-slack6535

| 경로 | review | overlay |
| --- | --- | --- |
| native · p1 · 95.86412% | [review](low-slack6535-p001-native-review.png) | [overlay](low-slack6535-p001-native-overlay.png) |
| wasm · p1 · 95.86412% | [review](low-slack6535-p001-wasm-review.png) | [overlay](low-slack6535-p001-wasm-overlay.png) |

### pii66-declared

| 경로 | review | overlay |
| --- | --- | --- |
| native · p66 · 99.87429% | [review](pii66-declared-p066-native-review.png) | [overlay](pii66-declared-p066-native-overlay.png) |
| wasm · p66 · 99.87429% | [review](pii66-declared-p066-wasm-review.png) | [overlay](pii66-declared-p066-wasm-overlay.png) |

### sample16-2022

| 경로 | review | overlay |
| --- | --- | --- |
| native · p40 · 91.41748% | [review](sample16-2022-p040-native-review.png) | [overlay](sample16-2022-p040-native-overlay.png) |
| wasm · p40 · 91.41748% | [review](sample16-2022-p040-wasm-review.png) | [overlay](sample16-2022-p040-wasm-overlay.png) |

### biotoxin205

| 경로 | review | overlay |
| --- | --- | --- |
| native · p46 · 99.59703% | [review](biotoxin205-p046-native-review.png) | [overlay](biotoxin205-p046-native-overlay.png) |
| wasm · p46 · 99.59703% | [review](biotoxin205-p046-wasm-review.png) | [overlay](biotoxin205-p046-wasm-overlay.png) |
| native · p47 · 99.99625% | [review](biotoxin205-p047-native-review.png) | [overlay](biotoxin205-p047-native-overlay.png) |
| wasm · p47 · 99.99625% | [review](biotoxin205-p047-wasm-review.png) | [overlay](biotoxin205-p047-wasm-overlay.png) |
| native · p48 · 99.99303% | [review](biotoxin205-p048-native-review.png) | [overlay](biotoxin205-p048-native-overlay.png) |
| wasm · p48 · 99.99303% | [review](biotoxin205-p048-wasm-review.png) | [overlay](biotoxin205-p048-wasm-overlay.png) |
| native · p49 · 99.99697% | [review](biotoxin205-p049-native-review.png) | [overlay](biotoxin205-p049-native-overlay.png) |
| wasm · p49 · 99.99697% | [review](biotoxin205-p049-wasm-review.png) | [overlay](biotoxin205-p049-wasm-overlay.png) |

## 실제 paint와 제한

실제 Canvas2D 화면·인쇄58캡처의 독립 PDF 대조는 전부90% 이상입니다. Native Skia64개 export 명령은 exit0입니다. CanvasKit은 기존 지원6입력의 software 화면·인쇄24캡처를 같은 96 DPI 독립 PDF와 대조했으며 최저96.41465%, 미달0건입니다. 나머지23입력의 CanvasKit 재검증과 GPU는 미실행입니다.

sample16 40쪽 실제 결측 글자는 U+119E 옛한글 아래아였으며 기존 bundled Source Han Serif K Old Hangul을 paint 전에 준비해 보정했습니다. glyph 공급과 메트릭 신뢰를 구분합니다.

작은 획·가는 선·KTX 제목 굵기와 sample16 40쪽 일부 줄끝 차이는 남아 있습니다. 위 점수는 허용된 실루엣 보조값이며 픽셀 완전 일치가 아닙니다.

Rust 전체 검증은 `a7ce883cbbb284b83740e63e2b4f03502996104d`의 10,219 PASS / 0 FAIL / 45 SKIP(448.660초)입니다. 현재 후보 Studio는1,835 PASS/0 FAIL/2 SKIP이며 TSC·build도 exit0입니다. 기존 글꼴 원장 감사17 PASS/1 FAIL은 base의 두 역사적 증거 문서 누락입니다.

검증 로그와 전체 임베딩 글꼴 SVG는 ignored output에 보존·정리 기록을 남겼고 이 자산에는 .log·글꼴 프로그램을 넣지 않았습니다. GitHub CI·PR review·오늘할일은 PR 채번과 code head CI 성공 뒤 별도 기록합니다.
