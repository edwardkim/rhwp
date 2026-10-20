# PR #7699 구현 검토와 메인터너 보정

## 적용 규칙

- Square는 글 배제이며 allowOverlap은 개체끼리의 겹침 허용이다. 동일 HWPX에서 allowOverlap만 0→1로 바꾼 독립 Hancom Print 전3쪽이 래스터 동일하다.
- 본문 폭80%라는 비율로 전폭 띠를 강제하면 실제 옆 차선이 남는85%폭 표에서도 본문을 모두 아래로 보낸다. 정상 Hancom 저장으로 LineSeg를 다시 만든 반례는 실제로 옆 차선을 사용한다.
- 재조판과 동일한 LayoutFrame::carve가 본문 상단에서 사용 가능한 차선을 판정한다. 원본 저장 줄의 시작이 표·바깥여백 하단을 소유하는 경우에만 전단 상단 예약을 만든다. 중간/본문 밖·인라인·문단 기준·캡션·편집/합성 저장 줄은 새 경로를 사용하지 않는다.
- 빈 줄의 앞/뒤 저장 사다리가 줄 높이+뒤 간격을 닫는다는 사실만으로 흐름 소유를 정하지 않는다. 문단 상대 표는 통째 예약·이월 뒤에도 안내 줄을 소유한다. 쪽·종이 기준 표는 기존 typeset_block_table의 page_anchored_square 분기에서 표 높이를 본문에 예약하지 않고 host 본문만 전진하므로, 이 좌표계에 속한 정상 저장 빈 줄은 일반 본문에서 한 번 소비한다.
- 저장 원점 비교는 HWPUNIT 정수로 수행한다. 동일한43240HU 경계도 px 변환 합산 순서에서1e-13px 차이가 나므로 부동소수점 비교는 줄 소유 증거가 아니다.

## 생산→측정→실제 배치

1. float_placement/stored_square_top_band.rs::table_reserved_height_hu: 기존 calculate_shape_reserved_heights의 셀/패딩 높이 producer를 그대로 이동해 공유한다. 입력 table IR을 변형하지 않는다.
2. stored_square_top_band: 실제 수평 anchor와 absolute_table_vertical_geometry의 동일 세로 원점·바깥여백 → LayoutFrame의 기존 차선 배제 → 원본 LineSeg의 시작 소유를 결합해 절대 하단을 반환한다.
3. typeset.rs::compute_body_wide_top_reserve_for_para → section/post_flow → reserve_body_wide_top → state/transition::advance_column_or_new_page: 다음 단의 current_height/current_start_height는 같은 하단에서 body.y를 뺀 상대값이다. zone 원점은 이동하지 않는다.
4. layout/shape_layout.rs::calculate_shape_reserved_heights 및 calculate_body_wide_shape_reserved는 같은 절대 하단을 받는다. 신규 Square 경로는 legacy TopAndBottom의80%/상단1/3 및 별도높이 재계산으로 내려가지 않는다.
5. layout.rs::build_single_column은 body_wide_reserved로 y_offset/col_anchor_y를 같은 하단에서 시작하고, 기존 저장 줄의 상대 간격을 소비한다. 타이틀 표 자체의 Page/Paper anchor는 변경하지 않는다.
6. typeset_block_table의 page_anchored_square 분기는 절대 위치 표 높이를 예약하지 않고 host 본문만 전진시킨다. 문단 상대 표는 통째 예약·이월 뒤에도 안내 줄을 소유한다. 옆 흐름 경로는 수용한 표의 occupied_bottom과 source origin으로 register_square_band를 호출하지만, 그 임시 밴드 하나가 소유의 유일한 증거는 아니다. controls/wrap_flow::place는 표 anchor로 분류된 문단 상대 표이면 기존 whole_paragraph→commit_wrap_absorption→record_wrap_around_para 경로를 유지하고, layout_wrap_around_paras가 같은 표 원점과 저장 offset으로 안내 줄을 그린다. 종료는 close_square_band가 바닥을 흐름에 반영한다. Page/Paper 절대 위치 표에서만 닫힌 원본 빈 줄을 기존 wrap anchor와 함께 일반 fit/paint로 보낸다. 원본이 아닌 입력·편집 세션·dirty ladder는 소유 증거로 인정하지 않는다.

## 분할/이어받기 적용 여부

이번 변경은 표 셀 내용 컷/rowspan/continuation을 변경하지 않는다. 실제 반례의 본문은 PartialParagraph로 단·쪽을 이어받으므로 원본 pi0/pi2 전내용 순서·누락·중복, 둘째 단 소유 및 전체 Print3쪽을 최종 render tree에서 검사한다. 표 내부 분할의 일반성까지 새로 해결했다고 주장하지 않는다.

## 수정 전 관측

- 초기 진단 devel375aff(최종 기준은4662c069e): 원본/정상 HWPX/allowOverlap p1 Native54.09142%, WASM53.88339%(초기 Hwp 글꼴 경로 누락 진단).
- 원 PR 초기 체리픽 ef981(리베이스 뒤 b6a865ca6): 원본/정상 HWPX Native99.98326%, allowOverlap54.09142%.
- 좁은 표: devel Native91.87685% → 원 PR61.24616%. devel에도 빈 줄20px 점유 누락이 존재한다.
- 보존한 원 PR 라이브러리에서 동일 회귀 후보4건: 원본/정상 HWPX2 PASS, allowOverlap/좁은 표2 FAIL. 환경/빌드 실패를 재현으로 세지 않는다.
- 보정1의 새 좁은 표 검사 PASS, 전폭3건 FAIL: 위 부동소수점 경계 판정 오류를 발견해 HWPUNIT 비교로 바로잡았다. 이 실패 후보는 제출하지 않는다.

## 전체 검사에서 발견한 메인터너 보정 회귀

첫 보정은 저장 사다리의 닫힘을 본문 소유로 일반화했다. 표 프레임의 안내 줄도 같은 사다리를 가지므로 이 가정은 틀렸다. 초기 전체 nextest는10,420 PASS/12 FAIL이었으며 표 이월·각주 번호/들여쓰기·Square 이어받기·쪽수·화면 밖/겹침 래칫에서 드러났다. 원 PR 라이브러리로 동일한 이월 표3건·각주4건을 실행한 결과는7 PASS다.

현재 단의 square_band_top 유무로 구분한 중간 보정도26건 중5건이 실패했다. 표를 통째로 예약하거나 첫 조각이 이월되면 그 임시 밴드가 없어도 표 프레임의 소유가 유지되기 때문이다. 이후 원본 좌표계와 실제 typeset_block_table 예약 분기에 따라 소유를 구분하고 helper 이름도 stored_empty_line_has_closed_source_box로 고쳐 좌표 연속성이 소유 증거라는 잘못된 주장을 제거했다. 기존12실패 검사·기대값·golden·baseline은 변경·제거·이관하지 않는다. 최종 source는 새 devel4662c069e 위로 충돌 없이 리베이스했다. 최종 좌표계 보정9e2cca42a의 관련26건은 모두 PASS다. 최종 기본 전체 nextest도10,440 PASS/45 skipped/실패0이며, 이전12실패 각각의 통과를 확인했다. 원본 읽기 전용 probe에서 병원 문서의 연속 빈 줄260–262와 KPS 문서의675 이후 안내 줄은 모두 Para 기준 Square 표에 속했다. 해당 문서의 기대값은 변경하지 않는다.

## 보정 후 시각 검증

최종 생산 소스 `9e2cca42a015c965106698a1689040fa56e871e3`를 Native/fresh WASM 각각 빌드해 보존했다. 최종 글꼴 목록에 실제 Human 글꼴 설치 경로 Hwp도 포함하여 base/head를 모두 다시 실행했다. 초기 누락 상태의 WASM 점수는 최종 판정에서 제외했다.

4개 대상 각3쪽 및 기존 정상 대조군5개 각1쪽, 양쪽 각17쪽을 같은 Print·compat2022·96dpi로 비교했다. 최저99.69093%, 90% 미만·쪽수 차이·누락·중복0. Decimal로 비교한34개 페이지/backend 쌍에 devel 대비0.01%p 이상 하락0이다. 원본·정상 HWPX·allowOverlap의 전3쪽 실제 RGB는 backend별 동일하며, 좁은 표 p3도 동일하다.

[입력 SHA·전쪽 수치·대표 PNG·독립 px 기준](../assets/pr7699/README.md)을 같은 PR에 보존한다. 원본 p1 표 아래 본문·두 단·원래 겹친 안내 글상자, 좁은 표의 옆 차선·빈 줄, 원본/좁은 표 p2의 표·그림·캡션·수식·번호·참고문헌과 p3 안내 표를 직접 읽었다. 수식 Σ/φ·괄호·첨자·이탤릭·식 번호 및 주변 간격은 같은 배율의 Native/Print/fresh WASM 확대 PNG에서 확인했다. p1 `영문제목`에는 기존 글꼴 차이가 남는다. 원본은 HFT 계열 `한양견명조`를 요청하지만, 동일4경로를 공급한 Native 실제 decision trace는 bundled `Noto Sans KR` 선택을 관측했다. 독립 Print보다 얇은 glyph를 단순 AA 차이로 표현하지 않는다. base/head의 같은 제목 crop을 별도로 대조하고 본 PR의 본문 흐름·기준선 보정과 분리한다. 글꼴 프로그램·메타데이터를 바꾸거나 완전한 서체 일치를 주장하지 않는다.

## 정식 회귀

`tests/cases/issue_7699_stored_square_top_band.rs`의5함수는 다음 의미를 보호한다.

- 원본 HWP와 정상 HWPX 각각: 전3쪽·pi0/pi2의 전내용 순서/누락/중복·양단 본문 소유·표/바깥여백 아래·제목 뒤 본문 순서.
- allowOverlap: 같은 본문 배제 관계 유지. 원본 속성 하나만 바꾼 독립 Print는 전쪽 동일하다.
- 좁은 표: 실제 옆 차선, 표 내부 비겹침 및 Print 첫 본문 기준선.
- 빈 글줄: 제목과 본문 사이의 소유·한 번만 존재·점유 보존. 가시 글자 유무를 흐름 유무로 대용하지 않는다.
- 모든 입력: Print에 의도적으로 겹친 안내 글상자는 숨기거나 중복하지 않는다.

완료 p1의 양쪽99%대 일치율과 직접 판독을 근거로 Print 문자 origin 기준선을 ±2px로 추가했으며 관계 검사도 유지한다. 원본3종의 첫 단/둘째 단/제목과 좁은 표의 첫 본문을 각각 보호한다. 무관한 높은 p3 대신 핵심 동작의 높은 완료 페이지를 선택했다.

보존한 원 PR 라이브러리와 동일한 정식 테스트 소스의2 PASS/3 FAIL은 `output/pr-review/pr7699-20261016/logs/pre-fix-formal-regression.log`다. 과도한 밀어내기·allowOverlap 겹침·빈 줄 누락이 각기 실패한다. 후보 standalone과 정식 Native/default suite의 대응5건은 모두 PASS다. 최종 전체 nextest는10,440 PASS/45 skipped/실패0이다. 빌드/환경 오류를 결함 재현으로 세지 않았다.

## 적용 범위와 남은 한계

- 생산→페이지네이션 예약→paint 시작 원점의 동일 결과 공유는 위 세 소비 지점과 본문/다음 단의 실제 render tree로 확인했다. helper 호출 뒤 legacy 재계산으로 값을 바꾸는 신규 경로는 두지 않았다.
- 셀 내용 컷·rowspan·continuation의 규칙은 변경하지 않는다. 본문의 PartialParagraph 단·쪽 이어받기는 전내용 보존과 전체 쪽수로 함께 검사한다.
- 별도 캡션 상자, 편집 후 저장 줄의 재사용, 다른 anchor 정렬과 일반적인 선언/실측 높이 차이까지 해결했다고 주장하지 않는다. 해당 조건은 기존 경로에 남기고 #7548 전체를 종료하지 않는다.
- 기존 정상 대조군의 관계 검사와 golden/baseline은 완화·제거하지 않았다. 새 source·fixture·대표 PNG만 추가한다.


## 실제 backend 소비와 검증 상태

- 최종9e2cca42a의 Native 관련26 PASS, Native lib4,043 PASS/12 ignored, 그림2 PASS·직접 PDF4 PASS, fresh WASM build 완료. format·Native/WASM Clippy·workspace build/all-target Clippy·manifest PASS, 기본 전체 nextest10,440 PASS/45 skipped/실패0이다.
- 실제 Studio Canvas2D·CanvasKit compat/default × screen/print ×3입력18캡처는 요청한 backend와 profile을 사용했고 fallback 및 replay hard 위반0이다. 성공한 WebAssembly.instantiateStreaming/instantiate에 사용된 바이트를 해시해18캡처 모두9e2cca42a package의 실제 WASM과 일치함을 확인했다. 생성 JS와 실제 Vite JS 응답도 구분해 해시했다.
- 실제 CanvasKit default/software Print 첫 쪽은 원본99.94575%, 좁은 표99.93156%, 정상 reflow99.84876%다. 원본/좁은 표의 실제 PNG를 직접 읽었다. Studio에는 연한 용지 안내선이 남는다. 순수 파일 export와 같은 화면이라고 주장하지 않는다.
- backend 사이 픽셀 비교(report-only, 채널 차이8 이하 무시·diff ratio0.5% 기준)은12건 미달이다. 이를 PASS로 바꾸거나 제품 전체 CanvasKit readiness 통과로 표시하지 않는다. 독립 Print의2px 실루엣·실제 배치·기호를 별도로 확인했다.
- 첫 UI 해시 계측은16MB WASM 응답이 Chromium inspector cache에서 제거되어 실패했다. 제품 회귀로 세지 않았고 실제 초기화 소비 바이트를 관측하도록 ignored harness를 고쳐 재실행했다. 임시 samples3개는 SHA를 확인하고 삭제했다.
- `영문제목` Native 글꼴 차이의 최신 base4662c069e와 보정본96dpi crop `[305,196,420,238]`는 RGB 동일(SHA-256 `2756e405597c78372400b5fe74fa5ac108e86793cfa2186955f78c6ace84b8e8`)이다. 본문 기준선 px 검사는 이 기존 glyph 선택을 고정하지 않는다. Studio host provider의HYmjrE는 기존 fallback chain의 TTF 대체 선택이며 원래 HFT와 동일 프로그램이라고 주장하지 않는다. 독립 TTF/OTF 파일을 PR에 추가하거나 원본 글꼴 name table을 바꾸지 않았다.

## 코드 위치

- [공통 상단·높이 producer](../../../src/renderer/float_placement/stored_square_top_band.rs#L8)
- [페이지네이션 예약 소비](../../../src/renderer/typeset.rs#L5716)와 [다음 단 전환](../../../src/renderer/typeset/state/transition.rs#L564)
- [단별 측정](../../../src/renderer/layout/shape_layout.rs#L4455), [전단 측정](../../../src/renderer/layout/shape_layout.rs#L4651), [paint 시작 원점 소비](../../../src/renderer/layout.rs#L7978)
- [빈 줄의 좌표계 판정](../../../src/renderer/typeset/controls/wrap_match.rs#L126), [일반 흐름/표 소유 분기](../../../src/renderer/typeset/controls/wrap_flow.rs#L48)

## 디스크 부족과 검증 재개

첫 workspace build는 linker의 ENOSPC 및 SIGBUS로 종료했다. 모든 Cargo/Rust 종료, debug cargo lock과 열린 파일 참조 없음 확인 후15시 이전의 종료된 incremental 하위 캐시202개(8.96GiB)를 정리했다. 최신16시 검증 캐시·deps·fingerprint·실행 파일·증적은 보존했고 여유0.86→9.76GiB를 확인했다. 같은 jobs8·공유 target·동일 build 명령 재시도는44.43초에 성공했다. workspace all-target Clippy와 manifest 정책도 통과했다. 최종 전체 nextest는 compile16분08초·실행1,307.752초에10,440 PASS/45 skipped/실패0으로 종료했다.

## 최종 로컬 검증 식별

모든 위 결과의 코드 head는 `9e2cca42a015c965106698a1689040fa56e871e3`다. 이후 추가한 문서·PNG는 생산6파일·정식 테스트·입력9파일을 바꾸지 않는다. 검사 종료 뒤 `git fetch upstream devel`에서 기준이 여전히4662c069e임을 확인했으며 동일 head를 다시 리베이스하지 않았다. GitHub CI·원격 PR·merge를 로컬 통과와 혼동하지 않는다. 현재 push·PR 생성은 사용자 승인 전이다.

```bash
CARGO_BUILD_JOBS=8 cargo nextest run --locked \
  --cargo-profile release-test --target-dir /home/tsjang/rhwp/target/pr-review \
  --tests --test-threads 12 --no-fail-fast
```

최종 로그는 ignored `output/pr-review/pr7699-20261016/logs/corrected-final-full-nextest.log`, 종료 코드와 모든 단계 명령은 `corrected-final-status.json`, 이전12실패의 최종 PASS 대조는 `final-local-result.json`에 보존한다. 원시 로그·TSV·중간 JSON은 Git에 추가하지 않는다.
