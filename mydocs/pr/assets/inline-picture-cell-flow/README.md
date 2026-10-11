# 인라인 그림 셀: 공개 합성 재현 자료

저장 줄 정보가 없는 셀에 인라인 그림 네 장을 넣은 합성 문서다. 회사 문서의 본문·사진·문서명·원시 스트림은 포함하지 않는다. 이미지는 직접 만든 색상 도형과 숫자 1~4다.

## 같은 입력의 독립 화면과 수정 전후

- 기준: 한컴 공식 WebHwp 데모 `10.80.0.2862`, 2026-10-08 캡처.
- 수정 전: #7668 코드 `733e2a302ff188dcd5b77e3fc972386dfadafc3b`에서 빌드한 Native. 그 뒤 `e9f43c0ce`는 공개 증적만 추가했다.
- 수정 후 코드: `2cceb050262555dd9fc96ad0feb3756a95788a14`.
- Native 바이너리 SHA-256: `a95ee9ccc872fb087f001a639f1edf4e620ca30e2324bda0e853bf82736d3bbc`.
- fresh WASM SHA-256: `d9cb4220063275d8786b1b1b85f82bdfe597931c242c78bbf3636562b6c15897` (`--dev --no-opt` 진단 빌드).
- `hancom-canvas-*.png`는 독립 기준의 실제 캔버스다. `*-before.png`, `*-after.png`, `*-wasm.png`는 같은 입력의 RHWP 출력이다.

대표 입력 `all-inline.hwp` SHA-256: `9da935ce21128324c7b5d4f799678bcb69b0c24369c1875243fd89dfa330b196`.

| 입력 | 확인한 의미 | 수정 후 그림 경계 최대 차이 |
| --- | --- | --- |
| all-inline.hwp | 내어쓰기: 첫 줄 1·2, 둘째 줄 3, 셋째 줄 4 | 1px |
| no-indent.hwp | 내어쓰기 없음: 2×2 | 1px |
| left.hwp / right.hwp / center.hwp | 줄별 가로 정렬 | 각각 1px |
| top.hwp / bottom.hwp | 큰 셀 안 세로 정렬 | 각각 1px |
| short.hwp | 첫 그림 높이가 작은 줄의 기준선 | 1px |
| following.hwp / stale-height.hwp | 뒤 문단 예약 및 작은 셀 높이 | 각각 1px |
| one-row.hwp | 네 그림이 한 줄에 들어가는 대조군 | 1px |
| positive-indent.hwp | 첫 줄 들여쓰기 | 1px |

12건 모두 Native/fresh WASM의 그림 데이터·좌표·쪽수가 같았다. PNG 전체는 10건 동일하며, 뒤 문장이 있는 2건은 글리프 차이가 남는다. 뒤 문단의 위·아래 경계는 기준 화면 대비 최대 2px 차이이며 그림 아래에 있다.

`top`/`bottom` 파일의 실제 저장 속성은 표 common.height=40309 HU, cell.height=60000 HU다. `stale-height`의 cell.height는 30000 HU다. 수동 생성 입력이며 정상 한컴 저장본이라고 주장하지 않는다. 기대값은 그 **같은 파일을 한컴에서 연 화면**에서 관측했다.

## 남은 검증

이 자료는 WebHwp 화면 대조이며 **한컴 Print PDF가 아니다**. 독립 Print PDF Visual Sweep 및 해당 근거를 갖춘 정식 회귀 테스트 추가는 남아 있다. 사용자가 이 미검증 상태를 안내받은 뒤 요청한 Draft PR용 자료이며, 시각 게이트 통과·병합 준비 완료·회사 문서 전체 정상 출력을 의미하지 않는다.

회사 원본 40개에서 셀 경계를 1px 넘는 그림 신호가 55→28개, 최대 초과가 847.1→47.8px로 줄었다. 쪽수·이미지 데이터별 출력 개수는 모두 유지됐다. 이는 로컬 render tree 집계이며 한컴 정답 판정을 대신하지 않는다. 원본과 개별 문서 식별 목록은 공개하지 않는다.
