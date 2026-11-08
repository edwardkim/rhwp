# 저장 TAC 원점과 직선 프레임의 독립 Print 증거

제품 source는 `6fc2b3490d`이며, Native079와 최적화 fresh WASM031의 같은 제품 소스로
검증했다. Native는 실제 Skia PNG, fresh WASM은 실제 Canvas PNG다. 전체 명령과 원본·PDF·
TSV SHA-256은 [provenance.json](provenance.json)에 있다. 실행 로그와 중간 자료는 ignored
`output/pr-review/semanticist21-20261019`에 보존하며 이 폴더에 복사하지 않았다.

## 원본과 독립 기준

- 당시 원본 `samples/2025 행정업무운영 편람(최종).hwp`의 SHA-256은
  `40d6d05eac4d55bdc4b0c62c42d93af104d5123b447581246f36fd15de7bd46f`다.
- 당시 한컴 2024+KoPub Print PDF `pdf/2025 행정업무운영 편람(최종)-hwp-kopub-2024.pdf`의
  SHA-256은 `3cdf329004cffad0ebacd122586e5e1702aff36b3015cbcb401d038857019593`다.
  원 #7751의 실제 통합 #7964와 같은 독립 기준이다. 기존 정상 Print와 출처를 재사용하며
  RHWP 저장본이나 RHWP export PDF를 기준으로 대체하지 않았다. 원본과 PDF는 이미 커밋돼 있다.
- 비교는 compat2024, 96DPI, 종이 원점 CSS px다. 시스템 글꼴 경로와 기존 설치 글꼴을
  사용했고 글꼴 프로그램을 새로 포함하지 않았다.

## 전쪽 및 고정 페이지 조건

원본과 PDF의 전체 쪽수는 383쪽으로 일치한다. 전쪽 raw 최저는 Native 92.44708%,
fresh WASM 92.45435%이며 양쪽 모두 90% 미만인 쪽은 없다.
원문 TSV는 [Native](native_silhouette.tsv), [fresh WASM](wasm_silhouette.tsv)에 바이트 그대로 있다.
raw는 색상 경계 보정 전 2px 이웃 실루엣 값이다. 엄격 RGB 픽셀 일치를 뜻하지 않는다.

| 쪽·핵심 경계 | Native raw | fresh WASM raw | Native review·overlay | fresh WASM review·overlay |
| --- | ---: | ---: | --- | --- |
| 111: 장식과 같은 TAC 줄, 뒤 두 표, 셀 안 화살표 | 99.93763% | 99.91756% | [review](native_review_111.png) · [overlay](native_overlay_111.png) | [review](wasm_review_111.png) · [overlay](wasm_overlay_111.png) |
| 242: 빈 줄 뒤 TAC 제목과 후속 본문 | 98.84506% | 98.87142% | [review](native_review_242.png) · [overlay](native_overlay_242.png) | [review](wasm_review_242.png) · [overlay](wasm_overlay_242.png) |

두 실패 지점을 직접 판독했다. 111쪽의 세 표 시작과 뒤 본문, 마지막 상자 화살표의 위 방향·
끝점·marker를 확인했고, 242쪽 제목과 뒤 문단의 위치·순서를 확인했다. 괘선·글자 래스터의
농도 차이는 남으며 RGB 완전 일치를 주장하지 않는다. Native079의 242쪽 실제 PNG는 이전에
직접 판독한 Native078 PNG와 SHA-256까지 같음을 확인했고, 새 fresh031 PNG도 직접 판독했다.

위 두 쪽만 독립 좌표 ±2px를 추가로 보호한다. 문서 전체가 98% 이상이라고 보고하지 않는다.
좌표 검사와 별도로, 이후에도 같은 쪽의 Native/fresh raw를 각각 98% 이상 유지해야 한다.
한쪽이라도 미달하면 좌표 검사 통과와 무관하게 회귀로 판정한다.

## 좌표와 관계 회귀

[독립 좌표](independent_coordinates.json)는 PDF의 실제 vector 선·채움 경로를 72pt에서
96DPI로 환산한 값이다. 구현의 계산값을 기대값으로 복사하지 않았다.

당시 정식 회귀 파일 `tests/cases/stored_tac_source_and_line_frames.rs`은 다음을 검사했다.

- 111쪽 표의 구역·문단·컨트롤 소속, 셀 내용 순서·누락·중복, 앞뒤 표의 겹침 방지와
  독립 PDF 윗변·아랫변을 함께 보호한다.
- 242쪽 제목 표의 외곽과 셀 내용을 보호하고 뒤의 구축 목적·주요 기능 본문 순서를 확인한다.
- 111쪽 마지막 셀 직선은 원본 회전 속성을 보존하면서 완성된 출력 좌표에 회전을 재적용하지
  않는지 검사한다. 독립 PDF 끝점·위 화살표 marker·셀 소속·표 내부 포함도 함께 확인한다.

같은 정식 검사 소스를 동결 제품에 strict rustc로 연결해 실제 실행했다.

| 제품 | 실제 source SHA | 실행 결과 |
| --- | --- | --- |
| Native076, 원점 보정 전 | `96038d8647` | 0PASS·3FAIL: 두 표 원점과 추가 회전 검출 |
| Native078, 원점·위 여백 보정 후 | `9810f26ccf` | 2PASS·1FAIL: 화살표의 추가 회전만 검출 |
| Native079, 직선 프레임 보정 후 | `6fc2b3490d` | 3PASS·0FAIL |

실제 실행은 `stored-tac-new-regressions-1170.log` 및 해당 제품별 로그에 있다. 컴파일 실패를
수정 전 결함의 검출 증거로 세지 않았다. 기존 회귀의 기대값·golden·허용치는 변경하지 않았다.
이 폴더의 통과는 전체 통합 후보의 Rust·lint·다른 문서·원격 CI 완료를 뜻하지 않는다.

## 현재 통합 후보의 범위

위 내용은 당시 실행의 역사적 기록이다. 사용자의 범위 지정에 따라 행정업무운영 편람과
해당 회귀 파일은 현재 통합 PR 검토에서 제외·제거했다. 이 자료를 현재 후보의 제출 증거나
활성 검사로 연결하지 않으며 삭제된 회귀 파일과 파생 참조를 다시 생성하지 않는다.
