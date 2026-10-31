# #8403 패턴 이미지 수정 중 확인한 문자 외곽선 차이

2026-10-31. 패턴 이미지 구현·검증은 승인된 범위대로 진행 중이다.
이 기록은 전체 완료 보고가 아니라 시각 gate의 추가 발견 사항이다.

## 관측

패턴 이미지를 복원한 첫 Native 빌드에서 원본 8쪽을 96dpi·print profile로 출력했다.
원본 PDF의 Poppler 출력과 `visual_sweep.py --silhouette-only --png-pair`로 비교했다.

| 쪽 | Native raw 실루엣 일치율 |
| --- | --- |
| 1 | 88.11% |
| 2 | 98.48% |
| 3 | 99.69% |
| 4 | 98.36% |
| 5 | 99.92% |
| 6 | 95.68% |
| 7 | 96.92% |
| 8 | 98.55% |

그림은 복원됐지만 1쪽 한글 제목이 기준보다 얇다. 실제 콘텐츠에는 `/F2 60 Tf`,
`1.715 w`, `2 Tr`가 있다. ISO 32000-1 §9.3.6의 text rendering mode 2는 글리프의
채움과 외곽선을 모두 그린다. 단순히 굵은 글꼴로 바꾸라는 선언이 아니다.
해당 Font는 `CAAAAA+NanumSquare`, 내장 TTF의 OS/2 weight 400, Regular다.

`--font-path /usr/share/fonts/truetype/nanum`과 Windows 글꼴 경로를 명시해도
1쪽 PNG는 동일했다. 같은 이미지 데이터를 직접 배치한 조사 진단본의 기존 Studio 출력에서도
제목이 얇다. 따라서 패턴 수정이 만든 문자 변화나 글꼴 파일 부재로 분류하지 않는다.
현재 PDF lowering은 글꼴 weight·fill colour를 보존하지만 이 실제 획 두께를 글자 paint로
연결하지 않는다. `issue_7966_pdf_text_paint_style`의 기존 검사는 혼합 글자색 보존이며
이 입력의 `Tr=2` 외곽선 지원을 입증하는 검사가 아니다.

## 증거와 한계

`output/black-image-20261031/` 아래에 다음 자료가 있다.

- `native-prototype/`: 첫 수정 Native 8쪽 PNG.
- `reference96/`: 원본 PDF의 독립 Poppler 96dpi PNG.
- `native-prototype-scores/silhouette.tsv`: 전쪽 원문 TSV.
- `native-fonts-prototype/`: 글꼴 경로 명시 후 1쪽 출력.
- `page-1-content.txt`: 실제 PDF 콘텐츠의 연산자.
- `direct/`: 제품 수정 전 직접 이미지 진단본의 기존 WASM 출력.

이번 수치는 초기 진단 빌드다. 후속 캐시·자원 경계 보정을 포함한 최종 Native/fresh WASM
결과를 대신하지 않는다. 1쪽 미달이 있으므로 전체 시각 검증 완료나 PR 준비 완료를 선언하지
않으며, 새 정식 렌더링 회귀 등록도 아직 하지 않는다.

## 처리 범위 결정

2026-10-31 작업지시자는 처음에는 이미지 수정에 한정한 예외를 승인했으나, 이어서
“결정을 번복합니다. 문자 외곽선도 함께 처리하세요.”라고 지시했다. **이전 예외는 철회됐다.**
패턴 이미지와 실제 문자 획을 함께 수정하며 전쪽 Native/fresh WASM 90% 기준을 적용한다.

글꼴을 강제로 굵게 치환하거나 제목 영역을 비교에서 숨기지 않는다. ISO 32000-1 §9.3.6의
채움/획 모드와 §8.4.3의 획 두께·색·선 연결 속성을 공통 모델로 전달한다. 획은 편집 가능한
글리프를 다시 그릴 때 적용하며 고정 그림을 덧씌우지 않는다. HWP/HWPX의 기본 속성으로
정확히 표현하지 못하는 paint는 기존 제한된 RHWP 확장 메타데이터 계약을 통해 왕복 보존하고,
다른 편집기에서의 표현 한계는 저장 진단에 명시한다.

구현 추적: PDF page/Form 그래픽 상태와 실제 text-show 순서 → 텍스트 paint 관측 →
PDF CharShape(문단·표·글상자에서 공유) → TextRun의 공통 TextStyle → Native Skia/SVG/Canvas2D paint.
폭·줄 높이·기준선 계산은 실제 글꼴 메트릭을 유지한다. 획 설정이 없는 정상 텍스트는 기존 경로를 쓴다.

현재는 구현 중이며 최종 검증 결과가 아니다. 새 회귀 등록은 전쪽 시각 선행 조건을 충족한 뒤 진행한다.

## 문자 획과 family 연결 보정

Native 중간 검증에서 실제 획을 복원한 뒤에도 1쪽은 88.91%였다. 추가로 원본의
`BAAAAA+NanumGothic-Bold`가 PostScript 표기이고 내장 TTF name ID 1은 `NanumGothic`임을
확인했다. CSS/Skia family를 임의 문자열 분해로 추정하지 않고 실제 name 테이블로 연결했다.
F2 `NanumSquare`의 Regular weight와 별도 `1.715 w / 2 Tr`는 그대로 유지한다.

이 보정의 Native 전쪽 결과는 94.70, 98.54, 99.69, 98.36, 99.92, 95.66, 96.91,
98.55%다. `native-family-fixed-scores/silhouette.tsv`가 원문이며 기준은 원본 PDF의
동일 Poppler 96dpi 출력이다. 1쪽을 직접 대조해 검정 이미지가 아닌 로고와 제목 획의 복원을
확인했다. 설치된 family와 원본 내장 프로그램의 버전·글자 위치 차이까지 완전히 같다는 뜻은 아니다.

공개 가능한 별도 대조군은 저장소의 OFL Noto Sans KR로 만든 PDF다. 채움과 획·획만 표시·
ExtGState의 색/투명도·선 연결·균일 CTM·hairline·중첩 Form을 포함한다. 앞선 Native 7쪽
중 6쪽은 100.00%였고 hairline은 54.73%여서, 폭 0을 채움으로 혼동한 Native 소비 경계를
보정하고 재검증 중이다. 이 중간 결과를 최종 통과나 회귀 등록 근거로 사용하지 않는다.

Studio TypeScript와 전체 2,692개 테스트 통과(2개 skip)를 확인했다. 최적화한 fresh WASM,
직접 CanvasKit, 편집·저장 재열기와 전체 Rust/Native 게이트는 아래 후속 결과로 확정한다.

### Native 문자 획 대조군 재검증

hairline을 폭 0의 stroke로 분리한 뒤에도 MuPDF 기준만 낮게 남았다. 직접 판독 결과
MuPDF 96dpi 출력은 해당 글자를 완전히 생략했고, Poppler는 같은 원본에서 얇은 외곽선을
그렸다. ISO 32000-1 §8.4.3.2의 폭 0은 장치 해상도의 최소 획이며 채움이나 생략이 아니다.
참조 PDF를 바꾸거나 기대 이미지를 제품 출력에서 만들지 않고, 원본 실물과 같은 Poppler로
대조군 전쪽을 다시 출력했다. Native 7쪽 및 HWP/HWPX 재열기 각각 전쪽 raw 100.00%다.
실루엣 점수와 별도로 hairline의 농도 차이는 남으며, 문자 외곽선과 일반 채움 대조군의
구분·내용·q/Q 복원은 직접 확인했다. MuPDF의 생략 출력도 진단 증거로 보존한다.

자료: `stroke-reference-poppler/`, `stroke-native-poppler-scores/silhouette.tsv`,
`stroke-native-final-hwp-scores/silhouette.tsv`, `stroke-native-final-hwpx-scores/silhouette.tsv`.
이 기록은 WASM 완료를 뜻하지 않는다.
