# #8153 리베이스 후 편람 전쪽 검증

2026-10-30에 렌더링 코드 `5c33032fe1`의 Native Skia Print와 표준 최적화 fresh WASM
Canvas Print를 각각 실행했다. 두 입력은 각각 383쪽이며 기존 독립 한컴 2024 Print PDF를
재사용했다. 기준과 입력 경로·SHA 및 검증용 글꼴 환경은 [기존 증거](../handbook-validation-20261028/README.md)를
따른다. PDF를 다시 출력하지 않았다.

96dpi PDFium 래스터와 `scripts/visual_sweep.py`의 색상 보정 전 2px 내용 실루엣을 비교했다.
각 경로에서 382쪽은 소수 둘째 자리 표시 100.00%, 130쪽은 99.96%다. 130쪽 WMF 내부의
차이는 사용자 지정 예외이며 ±2px 경계 검사를 유지한다. 표시된 점수는 색상별 픽셀 완전 일치를
뜻하지 않는다.

| 원본 | Native 전쪽 TSV | fresh WASM 전쪽 TSV | 205쪽 Native | 205쪽 WASM |
| --- | --- | --- | --- | --- |
| HWP | [TSV](handbook-hwp-native.tsv) | [TSV](handbook-hwp-wasm.tsv) | [review](handbook-hwp-p205-native-review.png) · [overlay](handbook-hwp-p205-native-overlay.png) | [review](handbook-hwp-p205-wasm-review.png) · [overlay](handbook-hwp-p205-wasm-overlay.png) |
| HWPX | [TSV](handbook-hwpx-native.tsv) | [TSV](handbook-hwpx-wasm.tsv) | [review](handbook-hwpx-p205-native-review.png) · [overlay](handbook-hwpx-p205-native-overlay.png) | [review](handbook-hwpx-p205-wasm-review.png) · [overlay](handbook-hwpx-p205-wasm-overlay.png) |

205쪽의 글상자 Y/N 소유와 순서를 확인하고, 독립 PDF의 문자 원점과 기준선을 96dpi CSSpx로
환산한 좌표에 ±1px를 허용하는 회귀 검사를 추가했다. 두 원본의 205쪽은 두 출력 경로 모두
100.00%다. 그림·문자 경계는 위 같은 쪽 overlay와 review에서 대조했다.

Native 실행 파일 SHA-256은 `4db09178e15359b6cbd2f0601e31f9217b77a4451d027065c7f92973b40d90b4`,
최적화 WASM SHA-256은 `d6dd0b6ce157ea50b6291b541854b2fec8437d2104e51e0597dccff7634df408`다.
빌드 작업 수는 8, WASM `codegen-units`는 16이다. 후속 변경은 회귀 주석·Rust 서식·lint 범위와
증거를 정리하며 렌더 동작을 변경하지 않았다. 검증 글꼴은 로컬에서 공급했으며 글꼴 파일은
저장소에 추가하지 않았다.

전쪽의 내용·독립 좌표 회귀는 `tests/cases/issue_8153_all_pages.rs`와
[기준 자료](../../../../../tests/fixtures/issue8153/README.md)에 정리했다. 양 원본에서 문자
370쪽·빈 12쪽·표지 그림 1쪽을 확인한다. HWP와 HWPX 두 검사는 각각 모든 쪽을 실행해
통과했고, Fontconfig가 글꼴을 0개 반환하는 환경에서도 다시 통과했다.

## 리베이스 기준과의 대조

기준 `ad4bf269ffef5adf2142b33004a9981fad7ce804`의 기존 Native 실행 파일로 같은
HWP·PDF·96dpi와 같은 글꼴 경로·환경 파일을 지정해 전 383쪽을 비교했다. 기준 Native는
실제 글꼴 환경 메타데이터를 출력하지 않으므로 선택된 글꼴의 동일성까지 확인한 것은 아니다. [기준 Native TSV](base-handbook-hwp-native.tsv)와
위 후보 TSV의 소수 둘째 자리 표시값을 같은 쪽끼리 대조하면 227쪽 개선, 156쪽 동일,
하락 0쪽이다. 기준 최저는 59.60%, 90% 미만은 154쪽 59.60%, 155쪽 71.46%,
156쪽 74.74%이며 후보의 세 쪽은 모두 100.00%다.
[후보 154쪽 Native overlay](handbook-hwp-p154-native-overlay.png)에서 표 조각과 후속 내용을 확인한다.
[HWPX 기준 Native TSV](base-handbook-hwpx-native.tsv)는 같은 383쪽에서 231쪽 개선,
152쪽 동일, 하락 0쪽이다. 기준 최저는 96.25%, 후보 최저는 99.96%다.
기준과 후보 HWP WASM 출력 메타데이터의 글꼴 환경 설정은 동일하다.
[HWP 기준 fresh WASM TSV](base-handbook-hwp-wasm.tsv)는 225쪽 개선, 158쪽 동일,
하락 0쪽이다. 기준 최저는 60.01%, 90% 미만은 154쪽 60.01%, 155쪽 71.56%,
156쪽 74.76%이며 후보의 세 쪽은 모두 100.00%다. 기준 WASM은 재시도에서 초기 조판을
위해 브라우저 프로토콜 제한을 60분으로 지정했다. [HWPX 기준 fresh WASM TSV](base-handbook-hwpx-wasm.tsv)는 229쪽 개선, 154쪽 동일,
하락 0쪽이며 기준 최저는 96.20%, 후보 최저는 99.96%다. 기준과 후보 HWPX WASM 출력의
글꼴 환경 설정도 동일하다.

[Studio 205쪽 실화면](studio-p205/README.md)은 두 원본의 Canvas2D와 CanvasKit 요청 시
실제 전환 결과, 수신 WASM 해시를 별도로 기록한다.
