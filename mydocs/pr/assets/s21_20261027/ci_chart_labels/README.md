# 기존 #2293 차트 라벨 검사 — 고정 영역을 가시 출력 관계로 교체

원 입력은 기존 [표식이있는꺽은선형.hwp](../../../../../samples/chart/라인/표식이있는꺽은선형.hwp)다.
한컴 2020 MCP의 `PrintMethod=0`, `hancom2020_pdf_driver_one_up`, 실제 빌드 11.0.0.9136으로
내용 변경 없이 [독립 Print PDF](../../../../../pdf/semanticist21-20261019/ci-chart-label-4819/표식이있는꺽은선형/표식이있는꺽은선형-2020.pdf)를 출력했다.
입력/PDF SHA·job ID·producer source는 [증거](evidence.json)에 있다. 실제 source는
`04c92da78165ccbf63b467855591df183655a98d`이며 이후 두 CI 검사 보정은 제품을 변경하지 않았다.

| 전체 1쪽 | 고정 base Native | 고정 base fresh WASM | 후보 Native | 후보 fresh WASM |
| --- | --- | --- | --- | --- |
| raw 실루엣 | 85.57830% | 78.70130% | 97.45943% | 96.57949% |
| 원문 TSV | [base/native](base/native.tsv) | [base/wasm](base/wasm.tsv) | [current/native](current/native.tsv) | [current/wasm](current/wasm.tsv) |

base는 `faa885e8b54ffc7cdf4196cd43a3103f362499a2`, Print profile·compat 2022·96dpi다.
변경 전 미달은 비교 자료이고 후보는 양쪽 모두 90% 이상이다. 이미 보정된 후보를 base의
과거 미달만으로 이관하지 않는다. 이 페이지는 양쪽 98% 미만이므로 **px 좌표·고정 영역·
기존 bbox 하단 6% 비율을 회귀 기대값으로 사용하지 않는다.**

원인 검사는 [작업 기록](../../../../working/task_semanticist21_20261027_ci_chart_labels.md)에 있다.
원본의 제목·눈금·카테고리·범례 12개를 각각 SVG에서 하나만 제외해 같은 Native backend로
출력한 대조와 원 출력의 차이를 검사한다. 그 글자가 실제로 보이지 않으면 기여가 0이 되어
실패한다. 좌표·픽셀 패턴·차이 픽셀 개수를 기대값으로 고정하지 않는다. 다른 글자나 격자
잉크가 사라진 라벨의 표시를 대신하지 못한다. SVG는 이 소비 단계의 입력이며 독립 조판
정답지로 쓰지 않는다. 기존 산점도 검사는 변경하지 않았다.

정상·CI 최소 글꼴 환경에서 실제 두 검사 모두 통과했다. 글꼴을 전혀 공급하지 않은
음성 대조는 실제 텍스트 소실로 exit 101이었다. 첫 호출에서 파생 suite가 갱신되지 않아
0건으로 끝난 결과는 검증 근거에서 제외했고, 재생성 후 실제 2건 실행을 확인했다.

[Native review](native-review.png)·[overlay](native-overlay.png)와
[fresh WASM review](wasm-review.png)·[overlay](wasm-overlay.png)를 같은 배율로 직접 판독했다.
제목·네 눈금·네 카테고리·세 범례가 모두 보이며 표식·플롯·쪽 소속도 유지된다.
기존 글꼴 굵기·선/마커 raster 잔차는 남아 있고 raw를 RGB 완전 일치로 해석하지 않는다.
이 테스트 수정은 원본 배치의 독립 좌표를 고정하거나 제품의 모양을 바꾸는 변경이 아니다.
