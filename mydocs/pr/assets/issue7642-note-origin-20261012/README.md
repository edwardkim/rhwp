# #7642 저장 미주 0 원점 보정 증거

- code source: `fcd86c8ae30515dfdac3cdbaedaa466f3b81ebe5`
- 입력: `samples/3-09월_교육_통합_2022.hwpx`
- 입력 SHA-256: `b3ed3d5c4f0f95f9a8990b17720da3bef566b5a36174153aa3def28fd46e6029`
- 독립 한컴 2024 Print: `mydocs/pr/assets/pr_7600_control_print_20261006/input-02-3-09월_교육_통합_2022-2024.pdf`
- PDF SHA-256: `052f390b36e53a2374b3883a82c96164802784497259f1e7d5e3fe06b9a163f6`
- Native binary SHA-256: `1fb5a8fda95bac598f7d0e8514bbc85e2e56745f9f7fb365c8a9fdc8d91d2f98`
- 동일 원문·PDF, compat 2024, 전체 글꼴 바이트, print profile, 96dpi다.

| 쪽 | Native 167 | Native 168 | 증거 |
| --- | ---: | ---: | --- |
| 10 | 35.26860% | 98.92453% | [전체 review](native168-p010-review.png) |
| 11 | 62.88459% | 98.82582% | [전체 review](native168-p011-review.png) |

전체 23쪽 Native 검증의 최저값은 91.85820%이며 모두 90% 이상이다. 쪽수는
23/23, 소스 해시 유지 true다. fresh WASM도 23/23쪽, 최저 91.85820%, 미달 0쪽, 소스 해시 유지 true로
완료됐다. 별도 WASM 대표와 수식 확대 판독은 계속 확인한다. 대표 점수로 전체 검증을 대신하지 않는다. 미주 문7 마지막 수식 줄이
다음 쪽 첫 단에 표시되고 문8–13 내용의 단 소속이 독립 Print와 맞는 것을 확인했다.
수식의 장평·기준선·첨자·장식·미세 간격은 fresh WASM까지 같은 영역을 확대 확인해야
하며 이 자료만으로 최종 승인을 선언하지 않는다.

컴파일된 원본 파서 진단에서 `SO-SUEOP.hwpx` 미주 161의 연속줄 0,0은 0,1200으로
복원되고, 9월 원본 미주 7의 마지막 0은 보존됐다. 이는 파서 반례의 실제 실행
증거이며 전체 문서 시각 검증이나 정식 렌더링 회귀의 통과를 대신하지 않는다.

실행 명령·Native/WASM 영수증·전쪽 TSV·글꼴 공급/복원 해시는 ignored
`output/pr-review/issue7642/failed-inputs-20261008/stage15-current-validation/`의
`stage168-*`에 보존한다. TSV와 중간 JSON/log는 Git에 넣지 않는다.

## fresh WASM와 확대 대조

실제 fresh WASM 빌드 SHA-256은
`ab407d8560f8d8bb339ad5ad3143a9f819f27c13fb8cfe039aefe19e98ff5b71`다.
[10쪽 WASM review](wasm168-p010-review.png)와
[11쪽 WASM review](wasm168-p011-review.png)를 별도로 생성했다.
전체 Native/fresh WASM PNG 23쌍 중 20쌍은 실제 바이트가 같고, 나머지
3쌍은 바이트가 다르다. 페이지별 기록을 별도로 보존한다. 10쪽은 실제 PNG 바이트가 같고, 11쪽은 다르다. 다른 쪽은 9·11·13쪽이며
실제 픽셀 차이도 있어 각각의 WASM 출력으로 확대 판독한다. 같은 SVG/PNG를 복사해
두 backend의 실행을 대신하지 않았으며 각각 실제 exporter와 whole gate를 완료했다.

[수식 확대](native168-selected-math-detail.png)는 같은 좌표의 독립 Print와
Native PNG를 4배 nearest로 확대한다. 미주 문7 마지막 계산, 문12 극한/분수
연속 계산, 문13 삼각비/근호를 대조했다. 줄 소속, 계산 내용, 정체 AH/CH와
장식·분수·근호·첨자의 보존을 확인했다. 원문 서체/래스터의 미세 윤곽 차이가
있으며 픽셀 일치와 수학 기호/내용 소속의 판독은 구분한다. 확대 이미지는
진단 범위로만 선택했으며 같은 crop 좌표를 회귀 검사 기대값으로 쓰지 않는다.
다른 원본이나 변경 영역의 확대 판독을 대신하지 않는다.

[같은 영역의 실제 fresh WASM 확대](wasm168-selected-math-detail.png)도 별도로 생성했다.
Native 이미지의 바이트 동일성을 가정해 WASM 확대 판독을 대신하지 않는다.

## 확대 판독에서 남은 배치 결함

11쪽의 `sqrt{1-(2/sqrt5)^2}`에서 큰 닫는 괄호 뒤 첨자 2가 독립 한컴
Print보다 낮다. Native와 fresh WASM에서 같은 영역을 확인했다. 수학 내용의
존재와 첨자 배치의 정확성을 구분하며 이를 글꼴 윤곽 차이로 면제하지 않는다.
전체 23쪽 점수 통과 후에도 이 영역의 배치는 **미충족**이다. 원본 script와
레이아웃의 script 부착 기준을 추적해 보정·재출력 후 회귀/승인 여부를 판정한다.
