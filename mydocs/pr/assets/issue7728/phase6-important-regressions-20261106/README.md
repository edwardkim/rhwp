# #7728 중요 열 프레임·괘선 회귀

대상은 `samples/issue6854/70833-electrical-safety-rule-regulatory-analysis.hwp`이며 SHA-256은
`a80b09754ce4f8e73398af9a9467f570c02be237815fec203ee8ff2db1bb8cbb`이다. 독립 기준은
`pdf/issue7728/70833-electrical-safety-rule-hancom2020-print-20261106.pdf`, SHA-256
`6cec821ff5897714669602c8707d66f669cfcb1187362c69101a0255c8d17956`, 전 18쪽이다.

## 검사 분류와 독립 근거

- **관계형:** `continuation_column_frames_keep_opening_span_and_row_edges`는 6→7쪽의 같은
  원본 표에서 선언 폭이 같은 전체 열 병합 칸과 행별 열 경계가 같은 프레임을 유지하는지 확인한다.
  셀 간격·병합을 포함하는 선언 폭을 그대로 실제 칸 폭으로 고정하지 않는다. 시작·계속 조각의
  상대 원점·폭과 서로 다른 행의 열 소유를 대조한다. `1e-6`은 같은 원본 프레임을 계산한
  부동소수 끝자리 비교이며 독립 PDF 좌표의 픽셀 허용값이 아니다.
- **고정형:** `independent_print_page10_keeps_all_grid_edges`는 독립 PDF 10쪽의 표 벡터
  괘선 중심을 72pt→96 CSSpx로 환산해 7개 셀의 좌·우·상·하 경계를 ±2px로 보호한다.
  좌표 원점만 검사하던 기존 검사에 병합 머리와 두 본문 행의 끝점 보호를 추가한다.
  독립 PDF 벡터 원값 (로컬 진단: `output/open-issues-oldest-20261105/pr-final-20261107/rebase-5077b216-20261108/temporary-asset-json/mydocs/pr/assets/issue7728/phase6-important-regressions-20261106/independent-page10-grid-vectors.json`)을 사용하며 후보 좌표를 정답으로 복사하지 않는다.

코드 `04f03cfafde42d2f77946a8a076ab527b8956468`의 같은 원본·PDF·compat2022·96DPI·PDFium·
2px raw 전쪽 [Native TSV](whole-native.tsv)와 [fresh WASM TSV](whole-wasm.tsv)를 연결한다.
Native 최저 94.62153%, fresh WASM 최저 94.27517%, 누락·90% 미달은 없다.
6쪽 N/W는 94.62153/94.82126%, 7쪽은 94.79966/94.27517%이므로 관계형으로 검사한다.
10쪽 N/W는 99.57724/99.47384%이므로 직접 판독한 독립 기대 좌표의 ±2px 검사가 가능하다.

## 검출력과 실행 상태

같은 최종 검사 파일 2건을 고정 devel `5716a226`과 후보의 frozen 라이브러리에 연결해
수정 전 2 FAIL→후보 2 PASS를 확인했다. 컴파일은 양쪽 모두 성공했다. devel은 6쪽의
계속 표 소속과 10쪽 표 윗선 497.82667px 대 독립 439.83866px 차이로 실패했다.
이 두 검사는 기존 열 너비 축소 검사를 대체하지 않는다. 열 너비 보정 직전 단계에서도
통과하는 보완 검사이며, 그 단계의 폭 축소 검출은 기존 4건 중 해당 검사의 FAIL→PASS로 입증했다.

파생 suite에 실제 등록한 뒤 최소 글꼴 환경의 Native Skia 실행에서 기존 4건과 새 2건이
6 PASS/0 FAIL, 0.218초, exit 0이었다. 명령·최종 파일 hash·전후 결과 (로컬 진단: `output/open-issues-oldest-20261105/pr-final-20261107/rebase-5077b216-20261108/temporary-asset-json/mydocs/pr/assets/issue7728/phase6-important-regressions-20261106/validation.json`)에
고정했다. 새 2건을 포함한 최종 전체 회귀는 대기 중이며 이전 12,761 PASS를 새 검사 포함
결과로 표시하지 않는다.
기존 4개 #7728 검사와 기존 반례 검사의 입력·기대값·문턱은 유지한다.

[현재 Native 10쪽 review](current-native-p010-review.png)·[overlay](current-native-p010-overlay.png),
[현재 fresh WASM review](current-wasm-p010-review.png)·[overlay](current-wasm-p010-overlay.png)의
원본 PNG·PDF raster와 canonical raw 재계산을 출처 (로컬 진단: `output/open-issues-oldest-20261105/pr-final-20261107/rebase-5077b216-20261108/temporary-asset-json/mydocs/pr/assets/issue7728/phase6-important-regressions-20261106/review-provenance.json`)에 기록했다.

## 웹폰트 응답 재점검

활성 공급 규칙과 Native 공개 공급 경로 99개를 확인하고, 편람 gate 전용 공급 7개도 추가로
실제 GET·고정 hash를 확인했다. 중복 URL을 제외한 총 105개(외부 CDN 74개와 Studio 상대
URL 31개) 모두 HTTP 200·글꼴 파싱에 성공했고, 응답 없는 경로는 0개였다.
최장 응답은 약 1.21초다. 따라서 제외하거나 대체한 설정은 없다.
응답·글꼴 식별·바이트 hash (로컬 진단: `output/open-issues-oldest-20261105/pr-final-20261107/rebase-5077b216-20261108/temporary-asset-json/mydocs/pr/assets/issue7728/phase6-important-regressions-20261106/webfont-response-audit.json`)를 남겼다. 이것은 URL 응답 확인이며
원본 문서의 CanvasKit 글꼴 준비 실패를 통과로 바꾸는 근거가 아니다. 다운로드한 글꼴은
임시 검사 후 제거했고 바이너리는 이 증적에 포함하지 않는다.

Mac WASM은 locked wrapper의 web/no-opt 로컬 대체 빌드다. Docker 최적화 통과가 아니다.
도구·바이너리·입력·PDF·글꼴 조건과 새 회귀 추가의 선행 시각 증거를 구분해 기록한다.
실행 로그는 ignored `output/open-issues-oldest-20261105/post-rebase-ece9/rebase-5716a226-20261106-115256/`에만 보존한다.
