# #8413–#8416 반복 이미지 공유 검증

- 측정·직접 판독: 2026-11-01 KST. 기준 커밋 `62720ae4052693605e6e3e70e53fbc451c04f6ff` + 제품 diff SHA-256 `13488c6d73fc0526a13c530eada0ce50e2d78e44fc21485b2f4dc8c79218473a`.
- 최적화 WASM: `1e6b3794e268a1ea09f47af4e2848c6eef404523e21d5674f39ad0c99a0fb35a`, release + wasm-opt -O, 25,952,222바이트. 실제 Studio 네트워크 응답 해시 확인.
- 96 dpi, `visual_sweep.py --silhouette-only`, 색상 경계 보정 전 2px `silhouette_raw_match_percent`.
- PDF 가져오기이므로 독립 기준은 입력 PDF 자체의 Poppler 래스터 출력이다. HWP를 역변환한 기준이 아니다.

| 입력 | 전체 쪽수 | Native 최저 | fresh WASM 최저 | 근거 |
| --- | ---: | ---: | ---: | --- |
| img_bug_01.pdf | 37/37 | 99.70% | 99.75% | [Native TSV](native-silhouette.tsv), [WASM TSV](wasm-silhouette.tsv) |
| work-limit-jpx.pdf | 1/1 | 100.00% | 100.00% | [Native TSV](jpx-native-silhouette.tsv), [WASM TSV](jpx-wasm-silhouette.tsv) |

첫 입력 SHA-256: `3fc955a5ceec37d0c5d83c28d09fce54a5fa2654884e82b5ecabe090f20e810b`.
JPX 입력은 [기존 PDF](../../../../../pdf/issue8183/work-limit-jpx.pdf)를 사용한다.

## 글꼴과 직접 판독

37쪽은 원본 Gibson CFF subset의 윤곽·advance를 모은 로컬 검증용 OpenType과 Windows Arial/Consolas를 공급했다.
Gibson ascent/descent는 원본 FontDescriptor의 800/-200을 사용했다. 추출 글꼴 프로그램은 ignored output에만 있으며 배포하지 않는다.
원본 폰트 없이 실행한 최초 Native 최저는 99.69%로 [별도 TSV](native-fallback-silhouette.tsv)에 구분했다.

9쪽과 최저 32쪽에서 큰 배경·스크린샷·로고·제목·본문·빨간 설명을 대조했다.
이미지는 보존되지만 32쪽 제목과 `enter KeX` 등의 공백·문자 배치 차이가 남는다.
이를 점수로 면제하지 않으며 37쪽 문서의 최종 시각 gate나 공통 작업 완료로 판정하지 않는다.

- 9쪽: [기준](reference-9.png), [Native](native-9.png), [WASM](wasm-9.png).
- 32쪽: [기준](reference-32.png), [Native](native-32.png), [WASM](wasm-32.png).
- JPX 1쪽: [기준](jpx-reference.png), [Native](jpx-native.png), [WASM](jpx-wasm.png).

JPX는 글자가 이미지 안에 포함되어 host 글꼴에 의존하지 않는다. 양쪽 출력과 기준의 표·한글·마지막 행을 직접 판독했다.
같은 이미지의 세 번 배치를 보존하며, 이전 중복 디코딩 기반 실패 검사는 공유 성공·명시적 낮은 작업 한도 거부·다음 요청 복구로 갱신한다.
원 해결 근거는 #8190 통합 `e7a0dec55c6c6c6365e4ff50babe7583e1cf1c70`와 [#8183 단계 3](../../../task_m100_8183_stage3.md)이다.

## 실제 실행

37쪽 Native 가져오기·HWPX 저장·재열기: 성공. 최종 재실행 wall 24,012ms, peak RSS 418,750,464바이트,
누적 decoded 1,478,483,157바이트, codec work 491,881,360. 이전 실행 출력과 HWPX SHA-256도 동일했다.
직접 WASM 가져오기·저장·재열기: 성공, 28,107ms(세 작업 포함), linear memory capacity 554,827,776바이트.
capacity는 RSS가 아니며 두 시간을 같은 측정 구간의 성능 비교로 해석하지 않는다.
Studio 실제 Worker: 기본 30초 실행은 timeout, 고급 설정 60초 실행은 27,299ms에 37쪽 열기 성공.
동일 WASM에서 암호 재시도·취소와 대화창 timeout·재시도·테마·키보드 E2E도 통과했다.

실행 로그·JSON·임시 HWPX·글꼴 manifest는 `output/pdf-import-errors-20261031/`에 두었다.
이 문서는 자원 실패 개선의 증거이며 #8413–#8416 전체의 PR 준비 완료를 뜻하지 않는다.
