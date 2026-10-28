# PR #7200 메인터너 보정 1회차

## 분석

- 사용자 요청: 빈 선행 문단 정렬 회귀를 해결하고 CLAUDE.md의 실행 지침을 보완한다.
- 시작 code head `25e0d8c74`, review commit `281795d20`, base `222c8c4819`.
- 원인: sequential 측정은 빈 줄의 높이·줄간격을 계상하지만 실제 block 표 원점은
  가시 텍스트 유무로 선택한다. 구성된 줄이 점유한 공간과 글자가 보이는지를 혼동했다.
- 독립 기대값: 기존 fixture의 빈 줄은 line_height=1000 HU, line_spacing=200 HU로
  96 DPI에서 16px를 점유한다. 중첩 표 앞 빈 줄도 흐름에 보존하고, 동일 Top 배치의 점유
  영역으로 Center/Bottom 이동량을 검증한다. 한컴 외형 일치로 확대하지 않는다.
- 수정 방향: 초기화된 저장 위치의 셀에 대해 구성 줄과 표 그룹에서 상대 원점·하단을 산출하고
  높이 측정 및 실제 block 표 배치가 같은 결과를 소비하게 한다. 빈 문자 여부로 높이를 지우지 않는다.
- 지침 원인: 공통 결과·반례·소비 경로 심사 원칙은 이미 있다. helper 호출 여부와 정상 사례의
  assertion만 확인하고 최종 원점 선택·덮어쓰기 분기를 대조하지 못했다. 기존 정책을 반복하기보다
  조판 수정 시 실제 소비 지점 역추적, 의미가 다른 경계의 대조, 정식 회귀 편입을 제출 단계에 연결한다.
- 검증: 새 경계 수정 전 FAIL → 보정 후 PASS, 기존 stored_nested_content_flow 및 관련 경계,
  전체 nextest·Rust lint 3종·Native Skia 3종, fresh WASM 및 같은 HWP/PDF의 Visual Sweep/overlay.
  검증 후 실제 결과를 이 문서와 review에 기록하고 커밋한다.

## 수정 및 선행 검증

- `SequentialNestedCellLayout`이 구성 줄과 저장 표 그룹에서 셀 상대 표 원점과 점유 하단을 산출한다.
  `calc_nested_controls_bottom_height`는 이 하단을, 실제 셀의 block 표 배치는 같은 결과의 원점을
  사용한다. 빈 문단을 높이 계산에서 제거하지 않는다. 기존 저장 앵커·wrap offset·분할 clip 분기는 유지한다.
- `empty_leading_paragraph_keeps_its_line_space_in_nested_table_alignment`를 기존 정식 회귀에 추가했다.
  수정 전 빈 줄 16px assertion FAIL, 수정 후 기존 3개와 새 1개 모두 PASS.
  파생 suite 번호는 source weight에 따라 바뀌므로 최종 fmt 뒤 prepare와 case wrapper로 실행했다.
  중간의 stale harness 0건 실행은 통과로 세지 않았다.
- `CLAUDE.md`와 공통 `AGENTS.md`에 생산 값부터 실제 paint 원점까지의 소비 경로 대조,
  가시성/점유/전진의 구분, 반례의 정식 테스트 편입, Visual Sweep·standalone overlay 직접 확인을 명시했다.
  기존 원칙이 없었던 것이 아니라 이번 변경에서 최종 원점 선택과 반례 검사가 누락된 것이다.
  작성자가 지침을 실제 읽었는지 여부는 제출 코드와 테스트만으로 단정하지 않는다.
- 새 빈 문단 HWPX 3개도 engine 2020 / Hancom 12.0.0.4605로 PDF 변환했다.
  PDF의 `End` 텍스트 y(96 DPI)는 Top 163.38 / Center 208.82 / Bottom 254.11px,
  보정 Native는 162.0 / 207.4 / 252.7px다. 기존 원 PR은 146.0 / 191.4 / 236.7px였다.
  PDF/renderer 글리프 상자의 약 1.4px 차이는 세 정렬에서 일정하다. 가로 원점·테두리 차이는 남아 있으며
  PDF 전체 외형 일치로 표현하지 않는다. 원점을 16px 내려 빈 줄 공간을 보존하는 방향은 한컴 관측과도 일치한다.


## 사용자 시각 검토에 따른 재분석 (진행 중)

- 사용자가 overlay의 큰 외형 차이를 지적하여 진행 중 전체 nextest와 소유 하위 프로세스를
  중단했다(exit 143). 완료되지 않은 전체 테스트를 PASS로 세지 않으며 이후 lint/Skia/WASM 단계도
  실행하지 않았다. 시각 수정 방향을 확인하기 전에 전체 검증을 재개하지 않는다.
- 빈 줄 정렬 불변식만으로 보류 사유를 해결했다고 볼 수 없다. 동일 PDF에 남은 가로 앵커,
  문단 테두리와 표 외곽·뒤 문단까지 직접 대조한다.
- PDF 벡터 확인: 글 앞으로 중첩 표 왼끝은 48.32px, rhwp는 128px이다. 좁은 비-TAC 표에
  적용하던 무조건 가운데 배치가 명시한 LEFT/offset=0을 무시한다. overlay의 Para/Column
  가로 앵커를 보존하고 기존 자리차지/Square 호환 경로와 구분한다.
- 추가 원인: 원본은 셀 문단에도 SOLID 테두리를 명시하지만 renderer는 cell_ctx가 있으면
  문단 테두리를 수집하지 않는다. 본문 큐 오염 방지가 셀 문단 테두리의 생략으로 구현되어 있다.
  PDF의 추가 선을 단순 합성 입력의 이상으로 제외할 근거가 없으며 별도 수정이 필요하다.

- 추가 수정: 셀별 문단 테두리 큐를 분리하여 부모/본문에 누출하지 않고 그린다. block 표의 빈 호스트
  문단도 테두리가 있으면 배치한다. `border_connect`를 resolved style에 전달해 연결이 꺼진 문단을
  임의 병합하지 않는다. TAC 표가 별도 PageItem인 경우에도 호스트 문단 테두리 점유 범위를 수집한다.
- 원본 width-top.hwp를 한컴 HWPX로 단순 저장해도 200자에 저장 줄 1개가 남는다. 줄 정보를 삭제한
  진단 대조군을 한컴에서 재조판/저장한 HWP는 textpos 0/45/90/135/180의 5개 줄을 가진다.
  같은 엔진 PDF에서 겹침 없이 5줄 출력된다. 원본 합성 HWP/PDF를 이 대조군으로 바꾸어 성공 처리하지 않는다.
- 대조군의 줄 메트릭은 line_height=1000, text_height=1056, spacing=632, 다음 vpos 차=1688HU다.
  composed 줄 높이가 1000만 사용해 줄당 56HU를 놓쳤다. 일반 저장 줄은 max(line_height,text_height)를
  공통 구성 높이에 실어 측정/배치가 소비하게 수정했다. TAC 전후 텍스트 분리 호환 경로는 유지한다.
- 현재 Native 대조군은 5줄 피치와 중첩 표 가로 위치가 개선됐다. 글자처럼 취급하는 바깥 표 주변의
  추가 테두리와 미세한 좌표 차이는 남는다. 아직 최종 시각 일치나 보류 해제로 보고하지 않는다.


## 이번 회차 결과보고

- 보정 경계 5개와 관련 대조군 4개: 총 **9 PASS**, 0 FAIL. 전체 회귀는 재개하지 않았다.
- 최종 Native CLI와 fresh WASM release/wasm-opt 빌드 완료. 원본 width 3개, 빈 선행 문단 3개,
  한컴 재조판 대조군 3개의 총 9쪽에서 Native/WASM PNG가 바이트 동일하다. 각 compare/overlay/review를
  다시 열어 판독했다. 원본 width의 PDF 글자 겹침과 표 주변 글자 테두리 차이는 여전히 미충족이다.
- 실문서 #2470/#6787의 각 1·2쪽도 두 backend로 재캡처했고 base와 4/4 바이트 동일하다.
- 새 한컴 대조군은 원본과 다른 실제 저장 줄을 가진 HWP 3개와 대응 PDF를 보존한다. 원본을 이름만
  바꿔 중복 커밋하지 않는다. 글자 테두리의 미해결 영역을 분리한 통제 HWPX/PDF도 보존한다.
- source 위치: `composer.rs`(줄의 텍스트 점유 높이), `style_resolver.rs`(border_connect),
  `layout.rs`(문단 테두리 연결/별도 TAC 호스트), `paragraph_layout.rs`(셀 문단 테두리·마지막 간격),
  `table_layout.rs`(공통 sequential 배치 결과·overlay 가로 앵커·셀별 테두리 수집).
- `cargo fmt --all -- --check`, `git diff --check`, 변경 Markdown 6개 상대 링크, review 메타데이터 검사는 모두 PASS. 전역 메타데이터 검사에는 기존 변경 밖 4파일/16건 누락이
  존재한다. 이번 변경과 무관한 문서를 임의로 수정하지 않는다.
- **미완료**: TAC 표에 적용된 글자 테두리의 하단 확장 계약, 원본 합성 HWP의 PDF 일치,
  최종 전체 회귀/Clippy 3종/Native Skia/통합 head CI. 이 회차는 부분 보정이며 머지 보류를 유지한다.
- 최신 수치·입력 계보·PNG·실행 명령/환경·산출물 해시는 PR 검토 문서에 기록했다. raw log/JSON/TSV는
  `/private/tmp/rhwp-pr7200-review-20260916`에만 두고 커밋에 포함하지 않는다.

- 독립 PDF 위치 계약을 저장된 보정 전/후 WASM render tree에 동일하게 대조했다.
  원 PR의 3문서에는 14개 글자 위치가 기준을 벗어났고 보정 후에는 0개였다.
  원 PR의 End x=128px와 누적 줄 간격 손실을 실제 이전 출력에서 검출했다.
