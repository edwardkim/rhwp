# 최종 전쪽 시각 검증

실제 제품 source는 `cda7c8cb33`이며 회귀 source `263b1e753c`와 제품 코드가 같다. 96 DPI·PDFium·Print·compat 2022, 색상 경계 보정 전 2px `silhouette_raw_match_percent`를 사용했다. [실행·산출물·입력 출처](../build-provenance.json), [검증 입력의 commit 일치](../validation-input-commit-receipt.json), [글꼴 공급](../font-supply-provenance.json)을 따른다.

## 문서별 전체 페이지

| 입력 / 독립 Print | 전체 쪽수 | Native 최저 | fresh WASM 최저 | 전쪽 Native TSV | 전쪽 WASM TSV |
| --- | --- | --- | --- | --- | --- |
| `tests/fixtures/gradient/radial-corner-print-contract.hwpx` / `tests/fixtures/gradient/radial-corner-print-contract-2020.pdf` | 6 | 100.00% | 100.00% | [Native](radial-native.tsv) | [WASM](radial-wasm.tsv) |
| `samples/eq-01.hwp` / `pdf/eq-01-hwp-2020.pdf` | 1 | 95.46% | 95.51% | [Native](eq-01-native.tsv) | [WASM](eq-01-wasm.tsv) |
| `samples/shape-group-02.hwp` / `pdf/shape-group-02-hwp-2020.pdf` | 1 | 100.00% | 100.00% | [Native](shape-group-native.tsv) | [WASM](shape-group-wasm.tsv) |
| `samples/pic-crop-01.hwp` / `pdf/pic-crop-01-2022.pdf` | 1 | 100.00% | 94.65% | [Native](picture-crop-native.tsv) | [WASM](picture-crop-wasm.tsv) |
| `mydocs/pr/assets/leading-definition-slots/input/report-implicit-column.hwpx` / `pdf/leading-definition-slots/report-implicit-column-hancom-print.pdf` | 1 | 100.00% | 100.00% | [Native](leading-definition-native.tsv) | [WASM](leading-definition-wasm.tsv) |
| `samples/issue8555/21868765_cell_text_edit.hwp` / `pdf/issue8555/21868765_cell_text_edit-hwp-mac-12.30.pdf` | 4 | 98.63% | 98.53% | [Native](recent-8555-native.tsv) | [WASM](recent-8555-wasm.tsv) |
| `samples/issue8556/21868765_cell_picture_insert.hwp` / `pdf/issue8556/21868765_cell_picture_insert-hwp-mac-12.30.pdf` | 5 | 98.89% | 98.77% | [Native](recent-8556-native.tsv) | [WASM](recent-8556-wasm.tsv) |
| `samples/issue8557/21868765_relocated_picture.hwp` / `pdf/issue8557/21868765_relocated_picture-hwp-mac-12.30.pdf` | 4 | 98.73% | 98.60% | [Native](recent-8557-native.tsv) | [WASM](recent-8557-wasm.tsv) |
| `samples/issue7593/repro-equation-issue7593.hwpx` / `pdf/issue7593/repro-equation-issue7593-hwpx-2020.pdf` | 1 | 99.60% | 99.48% | [Native](issue8337-full-fonts-native.tsv) | [WASM](issue8337-full-fonts-wasm.tsv) |

Native/fresh WASM 각각 전 24쪽이며 90% 미만·누락·측정 불가·독립 PDF와의 쪽수 불일치는 없다. [정확한 devel/후보 비교](base-candidate-comparison.json)의 같은 입력·같은 글꼴 PNG 48쌍은 모두 바이트 단위로 같다. base TSV도 같은 디렉터리에 보존했다. 이는 수식 글꼴 공급 조건의 대조이며 원 글꼴 부재 시 새 개선은 [#8337 별도 비교](../issue8337/latest-base-missing-face-comparison.json)를 따른다.

## 대표 review·standalone overlay

기존 최종 PNG와 독립 PDF raster에서 canonical Visual Sweep 함수를 사용해 생성했다. 새로 렌더한 것처럼 표시하지 않는다. [페이지별 raw·pixel match·visual accuracy proxy·hash](representative-review-metrics.json)를 따른다. 자동 수치는 사람의 최종 판정을 대체하지 않는다. 모든 쪽의 overlay 합성을 요구하지 않으며 전쪽 TSV와 대표 이미지를 구분한다.

| 입력·쪽 | Native review / overlay | fresh WASM review / overlay |
| --- | --- | --- |
| radial p1 | [review](radial-native-p001-review.png) / [overlay](radial-native-p001-overlay.png) | [review](radial-wasm-p001-review.png) / [overlay](radial-wasm-p001-overlay.png) |
| radial p3 | [review](radial-native-p003-review.png) / [overlay](radial-native-p003-overlay.png) | [review](radial-wasm-p003-review.png) / [overlay](radial-wasm-p003-overlay.png) |
| radial p6 | [review](radial-native-p006-review.png) / [overlay](radial-native-p006-overlay.png) | [review](radial-wasm-p006-review.png) / [overlay](radial-wasm-p006-overlay.png) |
| eq-01 p1 | [review](eq-01-native-p001-review.png) / [overlay](eq-01-native-p001-overlay.png) | [review](eq-01-wasm-p001-review.png) / [overlay](eq-01-wasm-p001-overlay.png) |
| shape-group p1 | [review](shape-group-native-p001-review.png) / [overlay](shape-group-native-p001-overlay.png) | [review](shape-group-wasm-p001-review.png) / [overlay](shape-group-wasm-p001-overlay.png) |
| picture-crop p1 | [review](picture-crop-native-p001-review.png) / [overlay](picture-crop-native-p001-overlay.png) | [review](picture-crop-wasm-p001-review.png) / [overlay](picture-crop-wasm-p001-overlay.png) |
| leading-definition p1 | [review](leading-definition-native-p001-review.png) / [overlay](leading-definition-native-p001-overlay.png) | [review](leading-definition-wasm-p001-review.png) / [overlay](leading-definition-wasm-p001-overlay.png) |
| recent-8555 p1 | [review](recent-8555-native-p001-review.png) / [overlay](recent-8555-native-p001-overlay.png) | [review](recent-8555-wasm-p001-review.png) / [overlay](recent-8555-wasm-p001-overlay.png) |
| recent-8556 p1 | [review](recent-8556-native-p001-review.png) / [overlay](recent-8556-native-p001-overlay.png) | [review](recent-8556-wasm-p001-review.png) / [overlay](recent-8556-wasm-p001-overlay.png) |
| recent-8557 p1 | [review](recent-8557-native-p001-review.png) / [overlay](recent-8557-native-p001-overlay.png) | [review](recent-8557-wasm-p001-review.png) / [overlay](recent-8557-wasm-p001-overlay.png) |
| issue8337-full-fonts p1 | [review](issue8337-full-fonts-native-p001-review.png) / [overlay](issue8337-full-fonts-native-p001-overlay.png) | [review](issue8337-full-fonts-wasm-p001-review.png) / [overlay](issue8337-full-fonts-wasm-p001-overlay.png) |

## 직접 판독과 남은 차이

- radial의 중심·반경·색 변화·표식은 대응 쪽에서 직접 확인했다. 실제 Direct PDF의 독립 출력은 [별도 6쪽 검증](../direct-pdf/README.md)을 따른다.
- #8337의 원 숫자 4·3과 AB/CD·장식·앞뒤 본문은 원 수식 face 공급/부재 대조와 독립 Print를 직접 비교했다. 숫자 의미 보정과 전진폭·기준선·shear 보존을 구분해 확인했다.
- `eq-01.hwp`의 본문 굵기·수식 간격/분수 모양과 `pic-crop-01.hwp`의 가는 테두리는 독립 Print와 차이가 남는다. 같은 devel PNG에도 동일하며, 90% 통과로 면제하지 않는다. [원격 수용안](../remote-action-plan.md)에 이 기존 범위에 한정한 예외를 명시했고 아직 승인 전이다.
- 최근 devel의 #8555·#8556·#8557 그림·셀, #8711 선행 정의 저장 재열기의 대응 내용과 쪽·개체 배치를 대조했다. 전쪽 base/후보 PNG가 같고 실제 편집·저장 계약은 해당 실행 원장과 집중 회귀를 함께 따른다.
