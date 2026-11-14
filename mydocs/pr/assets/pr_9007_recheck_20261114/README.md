# PR #9007 재검토 증거 — 2026-11-14 KST

판정은 [검토 문서](../../archives/pr_9007_review.md)다. 이 자료는 보호 기준 통과 증거가 아니다.

## 입력·생산·측정

[inputs.json](inputs.json)은 변경 대상 네 입력과 독립 Print PDF의 저장소 경로·SHA다.
base는 `dc3d0ff15e6e7cdb39138c72a5466994dea0e0d4`, 후보는 `abc4c1e82c34e4a112473c32886703b8356089d9`다.
각 SHA에서 별도로 Native Skia release-test 및 최적화 WASM을 빌드했다. 실행 파일·WASM 해시는 검토 문서에 있다.
측정 스크립트는 후보 checkout의 canonical `scripts/visual_sweep.py`를 양쪽에 공통 사용했다. 따라서 base 출력의 스크립트 checkout SHA와 실제 base producer SHA는 다르다.

명령 골격은 다음과 같다. `FONT_DIR`는 원본 글꼴을 묶어 명시 공급한 검증 전용 디렉터리이며 글꼴 파일은 이 증거에 포함하지 않는다.

```sh
python scripts/visual_sweep.py --silhouette-only --dpi 96 --compat VERSION   --hwp INPUT --pdf REFERENCE --key KEY --out OUTPUT   --rhwp-bin PRODUCER --font-path FONT_DIR --page-workers 4
# fresh WASM은 해당 producer의 --wasm-pkg PKG를 추가하고 --page-workers 1을 사용한다.
```

[font-programs.json](font-programs.json)은 실제 sweep catalog의 프로그램 이름·원본/정규화 SHA·family·style·보정 내역이다. Native의 문서 기본 글꼴과 수식·기호 대체 글꼴 공급을 구분했다. 보호 문서는 PDF의 글꼴 이름·문서 font plan·정본의 웹폰트 공급 근거를 대조해 공급했다. 현재 프로그램이 과거 수용 조건과 모두 같다는 확인은 끝나지 않았으며, 이 환경 미확인을 기여자 책임으로 돌리지 않는다.
공개 수식 글꼴 진단은 [GUST Latin Modern Math 1.959](https://www.gust.org.pl/projects/e-foundry/lm-math/download/latinmodern-math-1959.zip)를 별도 공급했다. 프로그램을 저장소에 추가하거나 시스템에 설치하지 않았다.

기준 PDF는 96 DPI PDFium RGB, 문서 출력은 Print profile을 사용한다. raw는 색상 경계 보정 전 2px 실루엣 값이다. 마스크를 추가하지 않았다.
`base-head-comparison.tsv`는 반올림 전 정수 미대응/합집합을 Fraction으로 비교한 결과이며, `*-native-base.tsv` 등의 각 TSV는 canonical 공식 표시값이다. [main-summary.json](main-summary.json)에 각 backend의 최저값과 정밀 하락 수가 있다.

## 직접 판독

무늬 HWP 1쪽과 hwpspec 23쪽의 Native·fresh WASM·독립 PDF PNG를 함께 보존했다. 무늬 방향, 격자, 본문 내용과 RECORD HEADER를 대조했다.
`sample16-p006-reference-native-crops.png`는 보호 문서 수식 차이의 확대 비교다. 이 그림은 전쪽 통과를 의미하지 않는다.

## 보호 기준 미충족의 대조

보호 기준과 입력/PDF 해시는 저장소의 `visual-protected-baseline/manifest.json` 및 `final.tsv`를 그대로 사용한다.
Native 수식 다섯 쪽은 `sample16-native-equation-head.tsv`와 `sample16-native-equation-base.tsv`가 동일하다.
fresh WASM의 18·35쪽도 `sample16-wasm-probe-head.tsv`와 `sample16-wasm-probe-base.tsv`가 동일하다. 이 두 쪽의 raw는 77.57·82.43%다. 색상 경계 보정 후 tolerant 값 약 99.9%를 raw 통과로 대체하지 않는다.
추가 수식 글꼴 공급 후에도 차이는 남았다. 18쪽 직접 판독에서는 서버 구성도 내부 배경의 채움·농도 차이가 보인다. 이를 수식 글꼴 차이만으로 설명하지 않는다. 같은 base에서도 재현되므로 이번 PR의 회귀로 판정하지 않으며 통합 담당자가 기존 출력 차이와 과거 수용 조건을 더 확인해야 한다. [WASM 18쪽](sample16-wasm-p018.png)과 [독립 PDF 18쪽](sample16-reference-p018.png)을 보존한다.

보호 전쪽 결과와 Decimal 비교는 별도 `protected-*-comparison.tsv`에 연결한다. 조건 정합이 미완료인 결과이므로 기존 정본을 갱신하는 데 사용하지 않는다.

## 전쪽 집계

Native·fresh WASM 각 18개 입력·1,771쪽, 합계 3,542쪽을 측정했다. 글꼴이 없는 마지막 두 WASM 입력은 exporter의 빈 공급 디렉터리 제한 때문에 기존 저장소의 Latin Modern Math 부분 글꼴을 명시 공급해 각 전체 1쪽을 완료했다. 글꼴 파일이나 제품 코드를 바꾸지 않았다.

- native: 보호 표시값 하락 78쪽, 90% 미만 0쪽, 최저 raw 96.25%.
- wasm: 보호 표시값 하락 87쪽, 90% 미만 5쪽, 최저 raw 31.61%.

[Native 비교](protected-native-comparison.tsv)와 [WASM 비교](protected-wasm-comparison.tsv)는 각 정본 행과 후보를 직접 연결하며, 후보 raw 정수 집계와 양쪽 PNG 해시도 포함한다. Native 수식 추가 글꼴 진단은 전체 기준 집계와 별도 자료로 보존했으며 페이지별 최댓값을 조합하지 않았다. 보호 기준 미충족이므로 승인·병합하지 않았다.
