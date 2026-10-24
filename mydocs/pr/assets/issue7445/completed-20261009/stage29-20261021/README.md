# 보정29 체크포인트 — 설치된 스타일 프로그램의 이름 조회

Issue: #7445, 관련 #7009/#6389. 부모 `b89bf4213c`와 같은 Native1075 런타임 후보다. 이 체크포인트는 전쪽 수용·새 fresh WASM 검증·PR 준비 완료가 아니다.

## 원 PR과 설치 프로그램 증명

#7009 등록 커밋 `e5984786bc`와 `mydocs/tech/investigations/issue-7009/hancom_2024_kopub_oracle.json`의 원본/PDF/job을 대조했다. 원본 `samples/2025 행정업무운영 편람(최종).hwp` SHA `40d6d05eac4d55bdc4b0c62c42d93af104d5123b447581246f36fd15de7bd46f`, 독립 한컴2024 KoPub 설치환경 Print383쪽 `pdf/2025 행정업무운영 편람(최종)-hwp-kopub-2024.pdf` SHA `3cdf329004cffad0ebacd122586e5e1702aff36b3015cbcb401d038857019593`, job `5a2b89ea-dda5-4343-838b-b169e7b9035a`다. 기준 PDF를 교체하지 않았다.

원래 글꼴 공급 경로 `8e3f08f267`→주소별 선택 `4e953d6c76`→한도 보존 `aa3d0e25a2`와 현재 `FontEnvironment::from_document_lookup`을 대조했다. 원본 프로그램 조회가 실패하면 선언된 HCR 대체가 선택된다. 설치된 KoPub 6개 파일의 name1/4는 굵기 이름을 포함하지만 fontdb 조회는 name16의 typographic family를 쓸 수 있다. 같은 파일의 name6 PostScript 이름을 함께 조회하면 원래 프로그램의 존재가 확인된다. `installed_font_program_aliases`에 이 동일 프로그램의 한글/영문/공백 이름을 연결했다. 서로 다른 굵기·KoPubWorld·generic 대체를 합치지 않았다.

원본 선언→공통 document_requests의 primary 이름→Native 실제 SFNT 조회/브라우저 공급 snapshot→from_document_lookup→선택 환경→측정과 실제 paint 순서다. 파일 6개는 `/Users/tsjang/Library/Fonts/KoPub *.ttf`이며 name1/2/4/6/16/17과 SHA는 ignored1073에 기록했다. 글꼴을 복사하거나 커밋하지 않았다.

## 실제 전쪽 선행 검증과 집중 보정

보정 전1070의 Native/fresh WASM 각383쪽 산출에서 Native16쪽 미달/최저63.53324%, WASM2쪽 미달/최저63.48332%였다. 이는 보정 전1055/1057 증거이며 새 후보 통과로 쓰지 않는다. fixed4a07의 집중 대조에서도242쪽63.78331%로 재현됐다.4a07→최신base2e3의 제품 파일 차이는 Qt clipboard뿐이지만 이를2e3에서 새로 빌드한 전쪽 결과라고 쓰지 않는다.

후보1075는 정식96DPI/Print/PDFium smooth_rgb_v1/raw2px90%·같은 입력/PDF/글꼴 공급으로 Native 집중8쪽만 다시 산출했다. 원본383쪽 소속은 유지했다.

| 쪽 | 이전 Native1055 | 후보 Native1075 |
|---:|---:|---:|
| 27 | 89.79086% | 99.99766% |
| 111 | 80.86530% | 82.21971% |
| 130 | 97.03510% | 98.96134% |
| 242 | 63.53324% | 63.83827% |
| 277 | 86.79912% | 99.99781% |

법령2/13쪽은99.20447/96.87649%,PrEP1쪽100%로 이전과 같다. #8064 Native 품질 하한을 통과한 것으로 처리하지 않는다. 전체 편람의 새 Native/fresh WASM TSV와 전체 영향 검증은 아직 미완료다.

## 직접 판독과 잔여

- 277쪽 전: [review](handbook-native-p277-before-review.png) · [overlay](handbook-native-p277-before-overlay.png)
- 277쪽 후: [review](handbook-native-p277-after-review.png) · [overlay](handbook-native-p277-after-overlay.png)
- 242쪽 전: [review](handbook-native-p242-before-review.png) · [overlay](handbook-native-p242-before-overlay.png)
- 242쪽 후: [review](handbook-native-p242-after-review.png) · [overlay](handbook-native-p242-after-overlay.png)

277쪽 제목·목차의 KoPub 굵기와 원문 순서·밑줄을 직접 읽었다. 배경 외곽 미세 차이는 남는다. raw99.99781을 엄격RGB 동일성으로 표현하지 않는다.242쪽 제목도 원래 굵은 돋움 프로그램으로 복구됐으나, 표와 후속 본문이 약9.97px 위로 당겨진 간격 문제는 그대로다.111/242쪽을 숨기거나 좌표 핀으로 고정하지 않았다.

1077의 실제 `RHWP_DIAG_ADV`는 p410의 원문1250+748HU+뒤1000HU와 달리 커서가748HU를 소비하지 않고 진행한 뒤 lazy base748을 설정함을 확인했다. p413빈 줄 원점과 p414 TAC 표 사이 원문 차이1600HU와 현재 배치 차이852HU가 다르다. 이 관계는 글꼴 선택과 독립적인 후속 보정 항목이다. 아직 제품 간격 코드를 수정하지 않았다.

## 프로그램 회귀 계약

`issue_7445_installed_style_program`은24개 동일 SFNT 이름 조건과 실제 원본 결측·다른 굵기 반례를 공개 프로그램 선택 API로 검사한다. 미달 실물 문서의 px 배치를 고정하지 않는다. 동일 최종 검사 SHA `5a8f641bf6c47797f95a246406411b6422b0f947916a8e1a7858eecaa433a838`: frozen1055에서1FAIL→1075에서1PASS(각0.01초). source-unit attrs는 늘리지 않았다. 1078의 --prepare 후 실제 자동 suite001은1PASS(실행0.026초/총112.632초)다. 기존 환경5PASS·원본 프로그램 소유21PASS로 집중 합계27PASS/0FAIL이다. manifest/check·unit-tier/check(base2e3)·fmt/diff도 exit0이다. 필터미선택249/247/237은 전체 회귀 skip으로 합산하지 않는다.

Native1075는 jobs8 release-test/native-skia CLI+lib 빌드 성공이며 producer/source hash는 ignored1075에 고정했다. 새 fresh WASM/실제 초기화·전쪽/전체회귀·범위 gate·Actual36은 후속 필수 항목이다. 기존Stage28 WASM을 새 alias source 통과로 재인용하지 않는다. `.log`·생성JSON·진단Rust는 ignored output1070~1078에만 보존한다. 지침/remote 변경과 PR 생성은 하지 않았다.

Native1075 CLI SHA `f09788a14b83f26e52301259d98d8b91d7bc932e3a331e7c8dcb78a8efd7dff7`, lib `6173ad2cea0086e95dfa5906504455f09e3f7dcd54fe2a0a50dc6875c53159a7`. 이 로컬 커밋은 부분 보정 체크포인트이며 새 full nextest·strict Clippy·fresh WASM·전쪽/실제 소비를 완료한 최종 제출 커밋이 아니다.
