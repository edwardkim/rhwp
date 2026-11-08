# 이슈 #7642 — 남은 미주 수식 줄 구성과 Square 그림 원점

이전 단계: [첫 물리 줄 이월](task_m100_7642_stage22.md), source `984d5b194`.

## 독립 관측과 범위

같은 Nov2022 원본과 독립 MCP2020 Print의 19/20쪽은 96.58035 / 96.49176%로 개선됐다.
18쪽 83.43951%, 21쪽 86.60499%는 미달이다. 18쪽 오른쪽의 도함수·삼각 함수 수식 줄과
그림 앞 공간, 21쪽 왼쪽의 작은 Square 그림과 오른쪽 풀이 간격을 직접 review에서 확인했다.
같은 원본의 실제 source와 한컴 Print를 계속 사용하며 별도 이슈 이관·검사 삭제로 해소하지 않는다.

## 실제 경로 추적 계획

- 공개 render-tree JSON은 Equation의 script와 원본 미주 소유를 생략한다. 현재 source의
  실제 DocumentCore/RenderNode에서 `note_ref`와 원본 Equation을 함께 읽어 저장 프레임,
  구성 결과, 실제 표시의 연결을 확인한다. 가려진 정보로 소유를 추정하지 않는다.
- 수식 폭 → 저장 줄 수용/재구성 → 본문 순서와 줄 높이 → 미주 배치의 소비 지점을 대조한다.
  전체 페이지 점수와 별도로 같은 배율에서 수식 기호·첨자·장식·앞뒤 간격을 확인한다.
- 작은 Square 그림은 원본 문단·그림 control 소유를 따라 저장 기준/offset → 실제 문단 원점 →
  최종 그림 원점을 추적한다. 각 출력의 좌표계를 일치시켜 비교하며 그림만 움직이는
  고정 px 예외나 clamp를 만들지 않는다. flow 참여와 표시 소유도 구분한다.
- 영향 페이지를 먼저 재출력한 뒤 전쪽 Native/fresh WASM 90%와 확대 판독을 완료하고 필요한
  회귀를 소유·내용 순서·포함·겹침 관계로 교정한다. 미달인 현재 문서의 기대값은 변경하지 않는다.

진단 값·로그·TSV·중간 JSON·nextrun은 ignored output에 보존한다. 계획 시점의 진단과 이후
구현·실행 결과를 아래에서 구분한다. 전체 90개 실패와 요청 52개 원본의 최종 검증·PR·CI·
후속 처리는 진행 중이다.

## 원본 소유와 그림의 독립 관측

source `984d5b194`의 실제 librhwp를 사용한 읽기 전용 진단을 컴파일하고 실행했다.
처음에는 진단 코드가 Equation의 크기를 직접 필드로 읽어 컴파일에 실패했고, 실제
`common.width/height`로 바로잡아 exit 0으로 실행했다. 이 도구 오류를 제품 회귀로 세지 않는다.
`nov2022-residual-source-provenance.json`에 library 해시·명령·두 시도와 실행 결과를 남겼다.

작은 Square 그림은 원본 본문 p426의 미주 ci0, 내부 note2의 picture ci0 소유다.
`vert_rel_to=Para`, `vert_align=Top`, `vertical_offset=2583HU`, `treat_as_char=false`이며
같은 문단의 저장 두 줄 사이 거리는 1530HU다. Native의 최종 그림 원점과 독립 Print의
실제 이미지 사각형을 같은 좌표계로 비교하면 한 줄 전진에 가까운 차이가 있다. 그림의
가로 위치·크기는 거의 대응한다. 다음에는 실제 소비자가 어느 문단/줄 원점을 선택하는지
코드에서 확인한다. 원점이나 offset을 특정 문서의 px 수치로 덮지 않는다.

18쪽의 그림도 원본 picture 소유와 독립 PDF 이미지 사각형을 확인했다. 실제 표시 높이는
거의 대응하지만 앞 본문에서 누적된 세로 위치가 다르다. source의 수식 script·저장 프레임·
실제 배치·원본 미주 주소는 진단 로그로 연결했으며, 줄 구성과 앞뒤 간격의 차이가 어디서
시작하는지 후속 대조한다. 아직 원인 계층의 판정이나 구현 보정은 완료하지 않았다.

## 구현 전에 확인한 원인과 반례

`layout_shape_item`은 Square/Para 그림에 첫 좁은 줄과 문단 첫 줄의 저장 높이 차이를
더한 뒤 `layout_body_picture`에 전달한다. 후자는 원본의 문단 상대 offset을 다시 적용한다.
측정의 `ObjectPlacementFrame::position`은 문단 원점과 같은 원본 offset을 직접 소비한다.
좁은 줄은 그림의 제외 영역을 반영한 저장 결과이므로 그 위치를 그림의 새 문단 기준으로
추론하는 paint 경로는 측정과 일치하지 않는다. Nov2022 21쪽의 한 줄 차이도 이 추가 전진과
대응한다. offset이 양수인지 문서 ID가 무엇인지로 분기를 좁히지 않고 잘못된 원점 추론을
제거한다. 별도 저장 개체 배치와 다음 쪽 deferred 원점 처리는 유지한다.

Task1245의 과거 회귀는 좁은 줄 정렬을 구현 기대값으로 고정했으며 당시 기록은 독립 Print
확인을 남은 일로 기재했다. 해당 검사를 통과시키기 위해 잘못된 원점을 유지하지 않는다.
같은 저장 방식의 Sep2022 7쪽 문25·문28과 첫 vpos가 0인 8쪽 문29를 독립 Print 대조군으로
실행한다. TAC 그림·Paper/Page 기준 그림은 이 변경의 비적용 경로이며 대표 출력도 확인한다.
회귀 검사 변경은 전쪽 Native/fresh WASM 및 확대 판독의 선행 조건 이후에 진행한다.

기존 1만 줄 이상의 layout 파일 전체를 재작성하지 않고 변경 경로의 약 1천 줄 메서드를
글줄 그림과 나머지 개체 처리의 1천 줄 미만 파일로 추출한다. 같은 모듈의 lexical include로
private 타입·호출 경로를 보존하고 추출 전후 본문 일치를 확인한다. 기존 루트의 나머지 대형
코드는 이 단계에서 건드리지 않는다.

## 첫 후보 실행과 소유 줄 확인

첫 Native 후보는 컴파일에 성공했다. 11월 17–21쪽은
95.47163 / 83.43951 / 96.58035 / 96.49176 / 88.46158%로, 작은 그림의
위치가 개선됐지만 18·21쪽은 미달이다. 9월 7·8쪽 대조군은 86.10514 /
72.32625%였고 직접 review에서 그림이 위로 올라가는 반례를 확인했다.
따라서 모든 그림을 문단 첫 줄 기준으로 삼는 후보를 채택하지 않는다.

실제 원본에서 9월 문25의 picture ci11은 텍스트 위치 88, 문28의 ci9는
77이며 모두 저장 마지막 줄 소유다. 11월 작은 그림 ci0은 텍스트 위치
0으로 첫 줄 소유다. `control_line_seg_index`가 원본 UTF-16 위치와 글자
좌표를 같은 축에서 해소한다. 기존 composer의 `stored_current_picture_wrap_rows`
경로도 그림 control의 원시 위치가 속한 줄을 앵커로 사용한다. 이 규칙을
어울림 paint와 실제 배제 상자에도 공유하며 첫 좁은 줄 위치로 소유를 추측하지 않는다.
유효한 저장 줄에 한정하고 저장 구성이 dirty이거나 구현이 만든 줄·좌표 되감김은
같은 계약으로 판단하지 않는다. 재조판의 기존 줄 구성 경로는 보존한다.

첫 빌드의 실행 그룹은 오류 메시지 없이 143으로 종료됐다. 모든 Cargo/Rust
종료를 확인한 뒤 같은 캐시·같은 source로 재실행해 release-test build exit 0을
확인했다. 잘못 생성한 Sep 진단 runner가 Nov 원본을 선택한 실행은 대조군
증거에서 제외했으며 실제 Sep 원본·해시로 교정해 위 결과를 얻었다.

## 저장 소유 줄 후보의 한계와 실제 소비 지점

소유 줄 후보 Native는 release-test build exit 0이다. Sep 7쪽 93.65377%,
8쪽 88.69235%이며 8쪽 그림은 여전히 내려갔다. 원본 p429의 control raw 시작은
347이고 저장 줄에서는 7번째 인덱스다. 독립 Print의 그림 사각형 시작은 문단 시작에서
저장 첫 좁은 줄의 전진과 대응한다. 저장 좁은 줄 때문에 marker의 뒤 텍스트가 밀린 결과를
최초 앵커로 다시 사용하면 한 줄을 중복 전진시킨다. raw owner를 읽는 것만으로 이 순환을
해소했다고 판정하지 않는다.

`layout_picture_band`의 원점을 구하는 코드가 한 control의 raw 위치와 **전폭으로 구성한
실제 줄 결과**를 연결하는 기존 근거다. 이 함수의 실제 caller는 text_editing의 편집
경로이며 현재 원본 load/paint가 이 결과를 소비한다는 근거로 사용하지 않는다. 저장 입력의
어울림 앵커도 이 전폭 줄 구성 producer를 사용하도록 연결하고, 배제 상자와 paint가 같은
결과를 소비하게 한다. 저장 없는 재조판·별도 저장 개체 배치는 기존 소유 경로를 보존한다.
새 producer에서 쓸 문단 상자는 기존 `ParagraphBox::body_for_style`로 만들고 original
control raw 시작을 해소한다. 임의의 한 줄·px 보정이나 LargestOnly/offset 값의 예외로
대상을 좁히지 않는다. 같은 producer가 저장 줄을 비운 입력을 처리할 때에는 저장 앵커
보정이 발동하지 않아 재귀나 같은 float의 중복 적용을 피한다.

paint 이후의 `paragraph_float_placements` 선택·vpos 높이 예약·flowWithText 제한도
확인했다. 별도 저장 placement가 있는 분기는 그 결과를 그대로 소비하고 이 일반 앵커로
덮지 않는다. 이 단계에서는 해당 분기의 새로운 시각 통과를 주장하지 않는다.

## 전폭 재구성 후보의 반례와 저장 배제 영역 대조

전폭 후보 Native build exit 0 이후 Sep 7/8쪽은 91.10607 / 72.32625%다.
8쪽 원본 p429는 실제 미주 control을 포함한다. `layout_paragraph_in_frame`은
중첩 흐름 소유가 있는 이 입력의 재구성을 지원하지 않아 None을 반환했고, paint의
0 폴백으로 그림이 문단 첫 줄에 올라갔다. 미주를 폭이 없는 control로 처리해
지원을 넓히지 않는다. 이 후보는 시각 개선으로 채택하지 않는다. Nov 17–21쪽은
95.47163 / 83.43951 / 96.58035 / 96.49176 / 88.46158%로 미달이 남았다.

후속 후보는 원본 raw control로 그림 소유를 먼저 식별한다. 단일 Square 개체,
Para/Column 가로 기준, Para/Top·Inside 세로 기준, 캡션 없음의 경로에서 같은
개체 위치 계산의 가로 경계와 저장 줄 차선, 원본 offset과 세로 점유를 대조한다.
control 소유 줄까지의 저장 줄 원점 중 최초 옆줄과 겹치면서 이전 전폭 줄을 침범하지
않는 후보가 **유일할 때만** 수용한다. 저장 정밀도 한 HWPUNIT 외의 px 허용치나
특정 문서·offset 상수로 그림을 이동하지 않는다. 다른 좌표계·복수 개체·모호한
후보는 이 저장 배제 영역 계약으로 판정하지 않는다. 저장 줄을 비운 재구성 경로와
별도 저장 개체 placement는 다른 경로로 구분한다.

실제 Sep p429의 첫 옆줄은 raw marker 소속보다 앞이다. 독립 Print의 그림 시작은
그 줄과 대응한다. Nov 작은 그림은 첫 줄 소유와 원본 offset을 적용했을 때 다음
옆줄의 글자 영역과 겹친다. 두 관측을 같은 점유 관계로 설명하며, offset 유무로
분기하지 않는다. 변경된 공통 결과는 `ObjectPlacementFrame::picture_exclusion`과
`layout_shape_item`의 그림 원점에서 소비한다. 원본 저장 문단의 모든 측정 경로가
이 helper를 실행한다는 주장은 하지 않는다. 기존 NO_LS 등록 조건, 이후 별도
placement 선택·vpos 예약·flowWithText 제한의 실제 소비 경계를 별도로 추적했다.

이번 작업에서는 전체 90개 실패를 현재 브랜치에서 보정한다. base의 기존 미달 여부는
원인 분리에만 사용하며 검사 삭제·새 이슈 이관으로 완료하지 않는다. 기존에 사용자가
지정한 HML 이관은 과거 별도 범위다. 현재 후보 빌드와 영향 페이지 재검증은 진행 중이며,
회귀 검사·기대값·baseline은 변경하지 않았다. 진단 산출물은 모두 ignored output에 있다.

## 저장 배제 영역 후보의 Native 결과와 확대 후 추가 보정

`picture-band-anchor` Native build exit 0, 바이너리 SHA-256
`8a3b0f7164784dfd06477ec51af2b782a3b0189d729d46a8fff9adcaa8036e62`.
Sep 7/8쪽은 93.65377 / 98.15647%, 누락 없음이다. 8쪽 그림의 실제 소유는
p429 ci21로 보존됐으며 최종 원점은 독립 Print 사각형과 대응한다. 두 페이지의
review PNG를 생성해 사용자에게 직접 표시했다. Nov 17–21쪽은
95.47163 / 83.43951 / 96.58035 / 96.49176 / 88.46158%, 누락 없음이다.
18·21쪽은 계속 미달이다. 이 후보의 전체 문서나 fresh WASM 통과를 주장하지 않는다.

93%인 Sep 7쪽의 확대에서 타원 그림이 여전히 Print보다 내려간 것을 확인했다.
앞 수식 줄의 메트릭은 재발행됐지만 그림의 상대 전진은 원래 저장 사다리를 썼다.
기존 `stored_row_projection::project_equation_rows`가 같은 저장 줄의 새 메트릭을
후속 물리 원점에 투영하는 producer다. 이 함수 → `compose_paragraph_in_context` →
`compose_paragraph_scoped`는 저장 줄·글자 메트릭을 구성하며 ObjectPlacementFrame의
배제 계산을 호출하지 않는다. 투영을 공통 그림 앵커에 적용해 측정/paint에 같은
물리 줄 결과를 전달한다. 줄 소속이 바뀐 입력은 투영을 수용하지 않는다. 구현상
재귀나 앞 조각의 투영량을 다시 더하는 경로를 새로 만들지 않는다.

`picture-projected-band` 후속 Native 빌드와 같은 Print 재검증을 시작했다.
위 결과는 직전 frozen 후보의 증거이며 후속 source 결과로 재사용하지 않는다.
현재 90개 실패를 삭제·이관하지 않는 이번 작업 지시는 계속 유효하다.

## 원본·Print 계약과 재현 명령

| 원본 | SHA-256 | 한컴 Print 기준 | SHA-256 | 확인 쪽 |
|---|---|---|---|---|
| `samples/3-09월_교육_통합_2022.hwp` | `d451ffbc72e8c84433f380c0cbcc8472f23286d53dce4b793a4bbb82f8715108` | `mydocs/pr/assets/pr_7600_control_print_20261006/input-01-3-09월_교육_통합_2022-2020.pdf` | `523477f1bd0572f95665c3f3651d1deb4fb94701e96301abe8835edaee69dd04` | 7–8 |
| `samples/3-11월_실전_통합_2022.hwp` | `bc8bccbb954c337d813d1af96f4e3047242124c2f2939163e282634eb721accd` | `mydocs/pr/assets/pr_7600_control_print_20261006/input-09-3-11월_실전_통합_2022-2020.pdf` | `b5dc3580c0c3716963c9686ec62fd779cedfea97f50d8f49eb98a2ec8fae369a` | 17–21 |

두 기준은 한컴 `11.0.0.9136`, PrintMethod 0의 MCP2020 실행 결과이며 이미 Git에
같은 바이트로 보존돼 있다. 새 사본으로 대체하지 않는다. 각각 job
`40c81c4e-94c4-4949-9139-840389147099`, `cc8b4953-2dbd-4bfd-b46a-4a3db2d8fc30`이다.
이번 변경의 Native 재현 빌드는 다음과 같다.

```bash
cargo build --locked --profile release-test --target-dir target/pr-review --bin rhwp
venv/bin/python scripts/visual_sweep.py \
  --hwp samples/3-09월_교육_통합_2022.hwp \
  --pdf mydocs/pr/assets/pr_7600_control_print_20261006/input-01-3-09월_교육_통합_2022-2020.pdf \
  --rhwp-bin target/pr-review/release-test/rhwp \
  --key sep2022-picture-projected-band --pages 7-8 --embed-fonts=full \
  --silhouette-only --dpi 96 --out output/pr-review/issue7642/stage23-sep
```

Nov는 표의 원본·Print를 사용하고 `--pages 17-21`로 실행한다. 실제 진단 실행은
빌드마다 바이너리를 freeze하고 해당 입력·Print·코드 파일과 바이너리 해시, 전체
명령·exit·TSV·누락 쪽을 `stage15-current-validation/*-provenance.json`에 기록했다.
이 로그·JSON·TSV·nextrun은 ignored output 전용이며 커밋하지 않는다. 두 입력의
전체 페이지·fresh WASM·전체 90개 회귀의 최종 통과는 아직 미완료다.

## 원점 보정 소단계의 최종 Native 관측

`picture-projected-band` Native build exit 0, 6분 4초. 바이너리 SHA-256
`e7b6fe64cb4c724b025d9303289b4b611c11ce69e5bc074f55f7053af7c5b3f3`.
두 원본과 표의 동일 Print PDF를 다시 실행했다.

| 원본/쪽 | 직전 원점 후보 | 줄 메트릭 투영 포함 후보 | 상태 |
|---|---:|---:|---|
| Sep 7 | 93.65377% | 95.93481% | Native 90% 충족, 전쪽/fresh WASM 미검증 |
| Sep 8 | 98.15647% | 98.15647% | Native 90% 충족, 전쪽/fresh WASM 미검증 |
| Nov 17 | 95.47163% | 95.47163% | Native 90% 충족, 전쪽/fresh WASM 미검증 |
| Nov 18 | 83.43951% | 83.43951% | 미충족, 수식 폭/줄 구성 보정 계속 |
| Nov 19 | 96.58035% | 96.58035% | Native 90% 충족, 전쪽/fresh WASM 미검증 |
| Nov 20 | 96.49176% | 96.49176% | Native 90% 충족, 전쪽/fresh WASM 미검증 |
| Nov 21 | 88.46158% | 88.46158% | 미충족, 수식/간격 보정 계속 |

선택한 7쪽의 누락은 없다. public Core 진단에서 Sep 7쪽 타원과 8쪽 구의
원본 소유를 보존했고, 줄 투영 뒤 타원 원점이 Print와 대응하는 것을 확인했다.
Native 실제 PNG의 review/overlay를 새로 생성했으며 옛 후보 캡처를 새 결과로
대체하지 않았다. 직접 판독에서 타원·구의 큰 세로 차이는 해소됐지만 Sep의
색상 테두리·수식 세부와 Nov의 잔여 수식/간격은 별도 보정 범위다.

대표 PNG는 ignored `stage15-current-validation/representative-native/`의
`sep2022-picture-projected-band-p007`, `sep2022-picture-projected-band-p008`,
`nov2022-picture-projected-band-p021` 안에 있다. 단계 코드와 관측을 소단계 커밋으로
고정한다. 기존 회귀 검사·기대값·baseline은 아직 변경하지 않았다. 전체 회귀의
새 head 실행·fresh WASM·전쪽 시각 검증·필수 lint·PR·CI는 후속 필수 작업이다.
