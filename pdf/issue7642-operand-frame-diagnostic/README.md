# 피연산자·세로 묶음의 독립 한컴 Print 진단

수동 입력은 기존 한컴 생성 HWPX의 스타일·구역을 사용했다. 크기·baseline·LineSeg를 지우고 MCP engine 2020에서 정상 HWP 저장 후 Print했다. 실제 제품은 한컴 11.0.0.9136, PrintMethod 0, `hancom2020_pdf_driver_one_up`이며 총 18쪽이다. 최초 카탈로그의 일치를 대신하지 않으며 검증된 회귀 fixture가 아니다.

SAVE job: `99959971-a032-474b-985f-6fe29732b213`. Print job: `863d38f4-f275-49e7-a8ed-12e6c749747e`.

| 파일 | SHA-256 |
| --- | --- |
| `operand-frame-hancom-2020.hwp` | `50d60eccf10f404968975560bb77e43a91aafa1b509108f9ce514756a275b36d` |
| `operand-frame-hancom-2020.pdf` | `1e3f2803886ffcc354171cf1652fc9e59b9b434f604ea7bb6bc9b0289229f2fb` |
| `operand-frame-input.hwpx` | `632365e85c4c358622fc8039853ce1c9802b84be5d58609dc0f3ce46166b4c41` |

## 현재 Native 관측

생산 SHA `cbd9d4676b648bee9696e19f4456f3cb8d999762`. 최저 8.23529%, 90% 미만 16쪽. 쪽수는 동일한 18쪽이며 누락이 없다. 본문·수식 혼재 줄의 캐시 누락 조건, 공백 피연산자, PILE 행 프레임을 보정하기 전의 실패 증거다. 실제 TSV·MCP/source frame receipt·로그는 ignored output에 있다. 대표 그림은 [같은 producer의 Native review/overlay](../../mydocs/pr/assets/issue7642-operand-frame-20261013/README.md)다.

| 쪽 | 대조군 | script | Native 일치율 |
| --- | --- | --- | ---: |
| 1 | `over_tight` | `a+b OVER c+d` | 71.20921% |
| 2 | `over_spaced` | `a + b OVER c + d` | 100.00000% |
| 3 | `over_left_tight` | `a+b OVER c + d` | 83.91194% |
| 4 | `over_right_tight` | `a + b OVER c+d` | 83.15047% |
| 5 | `over_times` | `a TIMES b OVER c TIMES d` | 100.00000% |
| 6 | `over_grouped` | `{a+b} OVER {c+d}` | 13.92097% |
| 7 | `over_chain` | `1 OVER 2 OVER 3` | 10.08878% |
| 8 | `atop_tight` | `a+b ATOP c-d` | 8.23529% |
| 9 | `sqrt_mixed` | `SqRt x` | 54.19847% |
| 10 | `sqrt_normal` | `SQRT x` | 36.50362% |
| 11 | `root_mixed` | `RoOt {a+b}` | 53.16872% |
| 12 | `root_normal` | `ROOT {a+b}` | 37.05621% |
| 13 | `pile_three` | `PILE{a # b # c}` | 13.45756% |
| 14 | `pile_two_frac` | `PILE{1 OVER 2 # 3 OVER 4}` | 16.22896% |
| 15 | `pile_eight_frac` | `PILE{1 OVER 2 # 2 OVER 3 # 3 OVER 4 # 4 OVER 5 # 5 OVER 6 # 6 OVER 7 # 7 OVER 8 # 8 OVER 9}` | 26.84604% |
| 16 | `eqalign_six_frac` | `EQALIGN{1 OVER 2 &= 3 OVER 4 # 2 OVER 3 &= 4 OVER 5 # 3 OVER 4 &= 5 OVER 6 # 4 OVER 5 &= 6 OVER 7 # 5 OVER 6 &= 7 OVER 8 # 6 OVER 7 &= 8 OVER 9}` | 51.56720% |
| 17 | `lpile_mixed` | `LPILE{1 OVER 2 # x}` | 12.69325% |
| 18 | `rpile_mixed` | `RPILE{1 OVER 2 # x}` | 11.56069% |
