# 저장 앵커와 중첩 표 흐름의 합성 입력

빈 문서에서 구성한 독립 입력이다. 사용자 양식이나 실제 환자 정보는 포함하지 않는다.

- `cell-*`: 1칸 표에 `Start` 문단과 작은 표를 둔다. `collapsed`는 두 문단의 저장 위치가 0이며, `intact`는 두 번째 문단에 1200 HWPUNIT의 정상 앵커를 둔다. Top 배치의 콘텐츠 점유 영역으로 Center/Bottom의 이동량을 검증한다.
- `body-*`: `Heading` 다음 저장 줄에 본문 표를 두고 다음 문단에 `Footer`를 둔다. 파일 이름의 숫자는 앞 텍스트 줄의 간격이며 표 소속 줄의 간격은 0이다.
- 기본 `cell-*`·`body-*` 입력은 HWP5 계보 마커로 해당 호환 경로를 선택한다. `pure-` 입력은 같은 XML에서 마커만 제거한 일반 HWPX 통제군이다.
- `overlay-*`: 순수 HWPX의 배경 표 뒤에 별도 문단의 표를 둔 입력이다. 배경 객체 높이는 다음 문단을 밀지 않지만, 빈 호스트 줄의 1200HU는 점유한다. 두 번째 표는 첫 표 너비(12000HU)만큼 오른쪽에 명시 배치한다. 가로 앵커 보정 전 우연한 가운데 배치에 의존하던 offset=0을 바로잡아 두 라벨이 겹치지 않도록 했으며, 독립 PDF에서 확인한 1200HU 줄 전진을 별도로 검사한다.
- `square-*` / `flow-*`: 연속 두 표의 첫 표를 어울림 / 자리차지로 바꾼 대조군이다. 어울림 표는 앵커 줄 높이만 전진한다.
- `width-*.hwp`: 긴 앞 문단과 뒤 중첩 표를 한컴에서 정상 저장한 HWP다. 최종 저장 5줄과 뒤 표의 위치를 독립 PDF로 검증한다. 저장 줄 1개/앵커 0으로 손상한 재조판 경계는 테스트 메모리에서 별도로 만든다.

cell/body/square/flow 합성 입력은 조판 기하 계약이며 PDF 일치 증거가 아니다. width 및 overlay의 PDF 대응과 검증 범위는 아래와 PR 검토 문서에 기록한다.

## 메인터너 보정의 overlay 대조 (PR #7200)

2026-09-17 보정은 `overlay-*.hwpx`의 두 번째 중첩 표 `horzOffset`만 0→12000HU로 바꿨다.
텍스트·세로 위치·표 크기·저장 줄과 첫 표의 배경 속성은 그대로다. 원래 입력의 `End`/`Second`는
가로 앵커를 올바로 적용하면 x=48px에서 겹쳤다. 보정 입력은 x=48/208px로 나뉜다. 한컴 PDF의 두 라벨 y 차이는 15.9968px이며, 실제 배치에도 빈 호스트 줄 16px를 반영한다.
`layout-anomaly`의 text-overlap은 Top/Center/Bottom 각각 1→0이며 baseline을 변경하지 않았다.

보정한 동일 HWPX를 engine 2020으로 출력한 PDF는 `pdf/pr7200/pr7200-overlay-*-2020.pdf`다.
start → status succeeded → download를 확인했다. PDF 전체 일치 여부와 저장 메트릭 차이는
`mydocs/pr/archives/pr_7200_review.md`의 직접 비교 판정을 따른다.

| 정렬 | PDF 변환 job |
| --- | --- |
| Top | `8580dbc3-851c-49f6-bbea-b6e259b1aa08` |
| Center | `667040ea-ace0-43a1-8792-7d33b94f5385` |
| Bottom | `61397e2d-f4d0-478e-8d9d-50d00d782d4c` |

| 최종 입력 / PDF | 입력 SHA256 | PDF SHA256 |
| --- | --- | --- |
| `overlay-top.hwpx` / `pr7200-overlay-top-2020.pdf` | `b2f8069202597e006e0cc2dacb17f89f9c95db6a718ac65669ef634bc002a5a4` | `24e0f744bfc5049de512f0a36c60e142fd3bf95ec13183250b7c0902f749b7bf` |
| `overlay-center.hwpx` / `pr7200-overlay-center-2020.pdf` | `82f47168e773ceb84809552703d6a72a8311173659374f54d214e500b962fe38` | `80764f6e12b79358facbcc7390c619af13be6b7ad3412856f887a6284b84a830` |
| `overlay-bottom.hwpx` / `pr7200-overlay-bottom-2020.pdf` | `6f457a17897e2417674ed730daf36dd70dadf199a817337a471ffaccdfc1f2f6` | `3126a88f633b3b0af4922cc3b906c6f17f7f06d1840b2ae5f4b414d474f1b28c` |

## width 입력 보정 (3회차)

원본의 200자/1개 저장 줄은 한컴에서도 글자를 겹쳐 출력했다. 정상 재생성한 HWP로 현재 파일을
갱신하고 `tests/fixtures/pr7200_hancom_recomposed`의 동일 파일을 제거했다. 텍스트와 표/셀/스타일을
유지한 채 저장 줄을 제거하고 한컴에서 재저장한 절차·job·해시는
[생성 기록](../../tests/fixtures/pr7200_hancom_recomposed/README.md)을 따른다.
원본 및 실패 증거는 `78fd17cf8`의 동일 경로에 보존한다.

현재 width 입력은 `pdf/pr7200/width-{top,center,bottom}-recomposed-2020.pdf`와 대응한다.
`pdf/pr7200/pr7200-width-*-2020.pdf`는 **과거 입력의 실패 PDF**이며 현재 입력의 oracle이 아니다.
`damaged_width_nodes`는 현재 정상 5줄을 먼저 확인한 뒤 메모리에서 1줄로 손상시키므로, 파일을
정상화해도 부실 저장 복구 검사를 삭제하거나 그 기대값을 완화하지 않는다.
