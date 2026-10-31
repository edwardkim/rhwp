# #8403 시각 검증과 회귀 등록 근거

2026-10-31. 제품 구현 `19c6b60f97`, base `3c01421e89a562de4ba8b021a4cdf943af60b58d`.
이 기록은 정식 회귀 등록의 시각 선행 근거다. 최종 Rust 전체 게이트와 이후 추가한
패턴 문자 거절 경계의 fresh WASM 검증은 완료 보고에서 확정한다.

비교 방법은 96dpi print profile, `visual_sweep.py --silhouette-only --png-pair`의
색상 경계 보정 전 2px `silhouette_raw_match_percent`다. 평균으로 미달 쪽을 상쇄하지 않는다.

| 입력/동작 | 전체 쪽수 | Native 최저 | fresh WASM 최저 |
| --- | ---: | ---: | ---: |
| 원본 PDF | 8 | 94.70% | 94.60% |
| 이미지 패턴 대조군 | 5 | 99.98% | 99.98% |
| 패턴 HWP 재열기 | 5 | 99.98% | 98.91% |
| 패턴 HWPX 재열기 | 5 | 99.98% | 99.98% |
| 문자 획 대조군 | 7 | 100.00% | 100.00% |
| 문자 획 HWP/HWPX 재열기 | 각 7 | 100.00% | 100.00% |
| 14곳 글자 치환 후 HWP/HWPX 재열기 | 각 7 | 100.00% | 100.00% |

CanvasKit도 실제 `CanvasKitLayerRenderer`에 같은 LayerTree와 Noto Sans KR 파일을
공급해 7쪽 모두 100.00%를 확인했다. 직접 Canvas2D 검증을 CanvasKit 결과로 대체하지 않았다.
원본은 HWP/HWPX 재열기에서도 Native 94.70%, WASM 94.60%를 유지했다.

`*.tsv`는 전쪽 원문이다. `png/`는 공개 합성 문서의 독립 기준과 실제 출력이다.
원본의 내용·이미지는 외부 공개 자료에 넣지 않았으며 로컬 검토 경로에서 직접 판독했다.
원본의 로고·그래프·제목, 대조군의 색/투명도·타일 위상·q/Q·획만 표시·hairline·Form,
글자 치환 뒤 갱신된 획을 같은 페이지에서 확인했다. 정상 문자열의 누락·중복은 없다.
실루엣 100.00%가 색상·안티앨리어싱까지 바이트 동일하다는 뜻은 아니다.

원본의 1쪽 제목은 `2 Tr / 1.715 w`이며 글꼴 weight를 올려 맞추지 않았다. 내장 OpenType
name ID 1의 family를 연결했다. 원본 내장 프로그램과 설치된 family 버전/세부 위치 차이는
남지만 그림 검정 채움과 실제 획 누락은 복원됐다. 원본의 처음 Native 1쪽은 88.11%였다.

독립 기준은 원본 자체의 Poppler와 합성 PDF의 Poppler/MuPDF다. 제품 출력을 기준으로
복제하지 않았다. MuPDF가 생략한 hairline은 ISO 32000-1 §8.4.3.2와 Poppler로 교차
확인했다. PNG의 농도 차이를 고정하기 위해 기대값을 조정하지 않았다.

문자 획은 HWP/HWPX의 제한된 RHWP 확장으로 보존한다. 한컴 등 다른 편집기의 확장 해석을
보장하지 않으며 저장의 `MetadataReduced` 진단으로 이 경계를 알린다.

재현 입력과 생성기는 [PDF 디렉터리](../../../../../pdf/issue8403/README.md)에 있다.
초기 fresh WASM SHA와 Native 실행 파일/입력 SHA는 동봉한 `*-sha256.txt`에 기록했다.

## 브라우저 재현

최적화한 fresh WASM을 Studio에 적용한 뒤 저장소 루트에서 실행한다.
공개 대조군의 Canvas2D·CanvasKit, 실제 14곳 글자 치환, HWP/HWPX 저장·재열기와
11가지 거절 입력 뒤 정상 문서 복구를 검사한다.

```bash
cd rhwp-studio
node e2e/run-with-vite.mjs -- node ../mydocs/tech/investigations/issue-8403/validation/browser-verify.mjs --mode=headless
```

출력은 기본 `output/issue8403-browser/`다. `RHWP_8403_OUTPUT`으로 별도 경로를 지정할 수
있다. 원본을 보유한 검토자는 `RHWP_8403_ORIGINAL`에 원본 PDF 경로를 지정하면 같은
8쪽 검증과 두 저장 형식 재열기를 함께 실행한다. 원본 파일을 이 디렉터리에 복사하지 않는다.
