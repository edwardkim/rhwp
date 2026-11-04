# 성능 검증 문서의 기존 조판 차이와 수용 범위

작성: 2026-11-05 KST. 비교 base `22947f6a90c998e95fa51238d5b907a0ebd37162`, 최종 runtime `2f44d7d0ab19c743089ae309a8ca64eee57efae2`. **실측 raw gate의 `re_review_required`는 유지한다. 2026-11-05 작업지시자가 변경 전부터 있던 동일 출력 차이에 한정한 수용 예외를 승인했다.**

## 글꼴 대조를 완료한 최종 전쪽 비교

| PR | 원본 / 독립 Print | 쪽수 | Native 최저(base → 최종) | fresh WASM 최저(base → 최종) | 최종 PNG 동일성 |
| --- | --- | --- | --- | --- | --- |
| #8331 | `mydocs/pr/assets/issue_8236/regulatory_sandbox/156492236-regulatory-sandbox-full.hwpx` / 같은 폴더 `reference-full-2020.pdf` | 17 | 75.22 → 75.22% | 75.47 → 75.47% | Native 17/17·WASM 17/17 바이트 동일 |
| #8332 | `samples/3-11월_실전_통합_2024-구분선위0미주사이20구분선아래2.hwpx` / `pdf/`의 같은 stem `-hwpx-2024.pdf` | 22 | 42.22 → 42.22% | 42.20 → 42.20% | Native 22/22·WASM 22/22 바이트 동일 |

96 DPI·Print profile·PDFium exact physical DPI v2·2px raw 실루엣이다. 원본·PDF·compat·공급 글꼴·backend를 base/후보 사이에 바꾸지 않았다. JPEG 7·15쪽과 수식 8·11·12·13·18·19·20·21쪽이 양쪽 모두 90% 미만이다. 전 39쪽씩 총 **78쌍의 실제 PNG**가 base와 최종 후보 사이에 바이트 단위로 같다. 누락 쪽은 없고 PDF와 쪽수도 각각 17·22쪽으로 일치한다.

최초 10개 글꼴 프로그램 비교 뒤 reference PDF의 `pdffonts`를 대조해 HCR Batang Bold·HCR Dotum Bold·Malgun Gothic Bold·Haansoft Dotum·휴먼고딕을 보완했다. 총 15개 프로그램의 경로·이름·해시는 [최종 글꼴 목록](performance-fonts-complete.json)에 있다. 휴먼고딕은 메인테이너가 로컬 사용 한정으로 준비한 HFT 변환 OTF다. 프로그램을 공급했다는 사실을 독립 원본과 같은 메트릭의 보증이나 재배포 허가로 해석하지 않는다. 글꼴 파일을 Git에 넣지 않았다.

최초 10개 프로그램의 TSV는 `preliminary-performance-10-fonts/`에 보존했고 현재 `base-*`·`candidate-*` 8개 TSV는 보완된 15개 프로그램의 최종 실측이다. [전쪽 결과·양쪽 PNG 해시](performance-font-complete-results.json), [PDF metadata](pdf-reference-metadata.json). 다른 기능 문서와 편람은 각각 해당 독립 Print의 검증 글꼴 환경을 유지했다.

## 직접 판독과 변화의 원인 구분

- JPEG 7쪽: 독립 Print의 그림 상자와 실제 출력의 폭·높이 및 주변 본문 흐름이 다르다. devel과 최종 후보의 출력은 완전히 같아서 픽셀 인계가 새로 만든 배치 차이가 아니다. [기준/Native/WASM](jpeg-sandbox-font-complete-review.png).
- 수식 19쪽: 이전 쪽에서 이어받는 시작 내용, 풀이의 줄·단 흐름과 다음 문단 원점이 독립 Print와 다르다. Native와 WASM 모두 이 기존 차이를 보인다. 본문·수식 영역을 같은 배율로 직접 판독했고 성능 개선으로 해결됐다고 기록하지 않는다. [기준/Native/WASM](equation-flow-font-complete-review.png).
- base/후보 PNG가 같은 것을 독립 Print 일치로 승격하지 않는다. 공급 글꼴을 보완한 뒤에도 기존 미달은 남았다. 예외 필요 범위를 글꼴 대조 전에 질문했던 순서는 바로잡았으며, 최종 판단은 위 측정 결과를 기준으로 한다.

## 같은 브라우저의 실제 성능·출력 보존

Chrome 155.0.8059.79, 양쪽 최적화 WASM·release codegen-units 16, 배율 2, 새 브라우저 페이지·새 WASM memory에서 base/후보 순서를 번갈아 각각 3회 실행했다. CPU 빌드나 다른 렌더 검사는 이 측정 중 실행하지 않았다. 측정은 10건 누적 후보의 관측값이며 단일 최적화의 독립 인과 추정이 아니다.

- JPEG 11쪽 첫 그리기 중앙값: **1772.8 → 1118.6ms (-36.9%)**. WASM linear memory의 그리기 후 최고 크기: **443,482,112 → 315,555,840바이트 (-28.8%)**. 프로세스 전체 RSS 상한으로 해석하지 않는다.
- JPEG 재그리기는 34.5 → 35.1ms, 열기는 323.3 → 333.3ms였다. 3회의 작은 차이를 속도 악화의 확정 근거로 쓰지 않는다. `allMs`는 1·11쪽을 먼저 그린 뒤 17쪽을 재생한 값이며 cold 전체 그리기로 기록하지 않는다.
- 수식 문서 열기 중앙값: 2481.5 → 2386.7ms, 문자 삽입 뒤 재조판: 1732.4 → 1725.3ms. 최신 base의 #8348 flow/paint 캐시를 보존했으므로 원 PR이 옛 base에서 주장한 15초 → 2.7초 개선을 현재 개선량으로 인용하지 않는다. 메모리는 274,792,448 → 275,513,344바이트(+0.26%)였다.
- 매 회 JPEG 전17쪽 Canvas RGBA hash와 수식 전22쪽의 편집 전/후 SVG hash가 각각 base/후보 사이에 일치했다. 총 3쌍 × (17+22+22) = 183개 출력 비교다. 별도의 공식 전쪽 sweep에서도 Native/fresh WASM PNG 39쪽씩 바이트가 같다.

실측 원본은 `wasm-performance-measurements.json`, 요약은 `wasm-performance-summary.json`, 실제 실행 소스 보존본은 `performance-probe.mjs.txt`다. 원 측정의 기본 작업공간 경로를 그대로 보존했으며 일반 자동 테스트로 등록하지 않는다.


위 성능 측정은 기록된 10개 글꼴 프로그램과 source `5c540c6c3e`에서 실행한 사전 측정이다. 이후 `2f44d7d0ab`의 executable 차이는 구역 setter의 불필요한 전체 스타일 재해소 제거이며 JPEG 인계·수식 경로의 코드는 같다. 최종 15개 글꼴 실측은 시간 측정과 구분하고, 빌드/다른 sweep과 동시에 측정한 시간을 성능 근거로 추가하지 않았다.

## 승인된 예외 범위

**권고:** #8331·#8332의 위 두 성능 문서에서 변경 전부터 존재하며 최종 출력이 바이트 단위로 같은 조판 차이에 한정해 수용 예외를 적용한다. 새 회귀·쪽수 차이·출력 누락·테스트 실패·새 baseline 핀·90% 미만인 새 렌더링 회귀 등록을 허용하는 예외가 아니다. 나머지 8개 PR의 기능 검증과 필수 로컬 gate를 유지한다. 기존 차이를 이번 PR의 해결 성과로 쓰거나 기존 관련 이슈를 자동 종료하지 않는다.

2026-11-05 작업지시자가 #8331·#8332의 변경 전부터 있던 동일 출력 차이에 한정한 수용 예외와 10건의 통합 push·Open PR·해당 head CI·self-review·merge·원 PR 완료/감사 댓글·동기화·정리를 승인했다. 새 회귀·검사 실패·쪽수 차이·누락·90% 미만의 새 렌더링 회귀 추가는 예외 대상이 아니다. [승인 기록](maintainer-approval.json).
