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

## 처리 범위 선택

원래 승인한 패턴 이미지 추출·배치 수정은 계속 검증한다. 최종 전쪽 90% gate를 충족하려면
별도로 확인한 문자 외곽선의 공통 추출·paint 보존도 수정해야 한다. 이 경우 글꼴을 강제로
굵게 치환하거나 제목 영역을 비교에서 숨기지 않고 실제 PDF 획과 좌표를 근거로 처리해야 한다.
이미지 수정에 한정하려면 작업지시자의 기존 문자 차이에 한정한 명시적 예외가 필요하다.
어느 경우든 이미지·일반 대조군·WASM·저장 후 재열기 검증은 생략하지 않는다.
