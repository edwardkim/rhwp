# Native CI의 PDF 표 글자 가시성 검사 증거

PR #8319의 Native Skia 실패를 원 해결 [PR #7953](https://github.com/edwardkim/rhwp/pull/7953)과
대조했다. 검사 목적은 positional 배경이 표 글자를 가리는 결함을 검출하는 것이다.
제품 출력·글꼴 선택·표 추론은 변경하지 않고, 얇은 대체 글꼴을 글자 없음으로 오판하던
검사의 절대 밝기 조건을 원본 PDF의 배경색과 검은 글자 사이의 색상 관계로 바꾼다.

기준 base는 `faa885e8b54ffc7cdf4196cd43a3103f362499a2`, 후보 제품 source는
`04c92da78165ccbf63b467855591df183655a98d`이며, 제출 head `f528dba6d974df25aa2b28891e0a1ad24da47624`와
제품 파일이 같다. 아래는 검사 보정 전에 같은 제품 producer로 다시 실행한 결과다.
이후 테스트만 보정한 head의 새 제품 빌드·새 캡처라고 표시하지 않는다.

| 기존 원본 PDF | 전체 쪽수 | base Native / fresh WASM | 후보 Native / fresh WASM |
| --- | ---: | --- | --- |
| [contiguous-table.pdf](../../issue7949_7950_20261017/contiguous-table.pdf) | 1 | 100% / 100% | 100% / 100% |
| [interleaved-table.pdf](../../issue7949_7950_20261017/interleaved-table.pdf) | 1 | 100% / 100% | 100% / 100% |
| [plain-table.pdf](../../issue7949_7950_20261017/plain-table.pdf) | 1 | 100% / 100% | 100% / 100% |

기준은 원본 PDF의 독립 Poppler raster이며, PDF 입력의 수용 검사를 HWP 한컴 Print 검증으로
표시하지 않는다. Print profile·compat 2022·96dpi를 고정했다. TSV는 `base/`와 `current/`의
`native/`·`wasm/` 아래에 원문 그대로 보존한다. raw는 색상 경계 보정 전 2px 실루엣 값이다.
이 점수를 RGB·글꼴 모양의 완전 일치로 해석하지 않는다.

`native/`와 `wasm/`의 세 원본에 대한 `*-review.png`와 `*-overlay.png`를 모두 직접 판독했다.
H1/H2/B1/B2 표시, 표 영역·배경·일반 표의 셀 소속을 확인했다. 기존의 획 농도·격자 raster 차이는
같은 원본의 base/후보 차이 및 글자 가림과 구분한다.

`minimal-font-review.png`·`minimal-font-overlay.png`는 호스트 글꼴을 바꾸지 않고 DejaVu만
공급한 읽기 전용 bubblewrap 환경에서 같은 원본을 import→HWPX 저장→PNG로 출력한 진단이다.
왼쪽 아래 B1은 실제로 보이지만 가장 어두운 RGB 값이 95여서 종전 `<80` 검사를 통과하지 못했다.
원본의 흰 배경보다 어두운 글자 픽셀은 100개였다. raw 100%와 별도로 얇은 글꼴 차이를 기록한다.

검사 보정·실제 종료 코드와 CI 명령 실행 결과는
[작업 기록](../../../../working/task_semanticist21_20261027_ci_pdf_paint.md)에 연결한다.
새 테스트는 원본 셀 내부·검은 글자·파란 배경·일반 표 추론 계약을 유지한다.
