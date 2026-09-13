# #3587 Stage 4 — B1 문단 블록 반복 사전검사

- 일자: 2026-09-12
- 승인: 메인테이너 「B 구현계획서를 승인합니다」.
- 상태: **B1 예산·지원 종류·참조 경계·공유 자원 등록 검사 구현·집중 검증 완료. 실제 사본 생성 비용 실측 및 B2/B3는 미완료.**
- 최신 제품/테스트 검증 SHA: `e77607148f` (`task_m100_3587`). 최초 자원 절편 SHA는 `b1de53c326`.
- 근거: [승인된 B 계획](../plans/task_m100_3587_impl_b.md), [A 통합 결과](task_m100_3587_stage3.md).

## 1. 이번 절편

`paragraph_block_budget_native`는 복제 전 주소·자원 비용을 읽기 전용으로 조사한다.
아직 `repeat_paragraph_block_native`를 제공하지 않는다. 비용 검사를 통과했다는 이유로
컨트롤 복제·참조 완결성·저장 호환성이 보증되는 것은 아니다.

- source `[start,end)`와 사전 좌표 목적지 검사. 원형 내부 삽입 거부, 양 끝 경계 허용.
- count=0은 주소·옵션만 검사하고 내용/전체 원문 순회 없이 반환한다.
- 사본·본문 문단·소유 노드·소유 깊이·구조 바이트·대응표·전체 원문 노드 예산 적용.
  호출자는 기본 hard ceiling을 낮출 수만 있다. 계산은 checked arithmetic이다.
- 소유 트리를 sibling 하나씩 읽는 명시적 stack으로 순회한다. 넓은 표의 모든 셀을
  먼저 stack에 복사하지 않는다. 문단·컨트롤·도형·셀·글상자·캡션·master/메모 문단을 포함한다.
- 원문 순회는 구역 자체도 한 노드로 세므로 문단 없는 구역 배열도 제한한다.
  원문 ID/참조를 아직 수집하는 것은 아니다. 해당 수집의 비용을 대체한다고 주장하지 않는다.
- 구조 비용은 serde 값 방문으로 계측하되 JSON이나 복제 IR은 만들지 않는다.
  `source_line_seg_vertical_pos`는 serde에서 빠지므로 모든 중첩 문단에서 별도 합산한다.
  고정 저장폭을 중복 계산하는 보수적 논리 예산이며 allocator/RSS 상한은 아니다.
- 대응표는 노드마다 양쪽 경로의 단계당 16 bytes 및 entry 고정비 64 bytes로 예비 산정한다.
  실제 typed path 결과 타입을 연결할 때 이 저장폭을 다시 검증해야 한다.
- Form의 사용자 정의 serializer는 정렬용 Vec를 할당하므로 원형 비용 방문 전에 거부한다.
  이는 B 초기 Form 미지원 방침에 맞춘 방어이며, 다른 종류의 비용 계측 성공은 지원 승인이 아니다.

## 2. 검증 계약

`tests/cases/issue_3587_paragraph_block_budget.rs`의 10건:

1. 경계 좌표와 기존 빈 문단 보존.
2. 잘못된 주소는 count=0도 거부.
3. count=0에서 unsupported 내용/원문 순회 생략, 무변경.
4. 모든 한도의 0/상향 지정 거부.
5. 사본/문단 제한 및 곱셈 overflow 거부.
6. 구조/노드/대응표/원문 예산의 정확한 경계와 한 단계 초과.
7. UTF-8/raw/serde 제외 버퍼 비용 포함.
8. 중첩 글상자 문단·깊이·제외 버퍼 포함.
9. 1/10/100회 비용 선형성과 요청당 원문 1회 순회 비용.
10. 실제 표/clipboard가 있는 core에서 성공·실패 모두 문서/이벤트/clipboard 보존.

검증은 source를 커밋한 후 `rhwp-review-3587`의 동일 SHA에서 수행했다.
생성 suite는 해당 worktree에서만 준비하고 `target/pr-review`를 재사용했다.
이번 예산 수치의 선형성 검사는 사본 생성 시간/RSS 계측이 아니다.

## 3. 실행 결과

로그: Git 제외 `output/3587/b1/`. 아래 검사는 모두 `b1de53c326`의 동일 제품/테스트에서 수행했다.

| 검사 | 결과 |
| --- | --- |
| manifest `--prepare` | PASS, review worktree에서만 실행 |
| `cargo fmt --all` 및 `--check` | PASS, tracked source 변경 없음 |
| `run-rust-test.mjs issue_3587_paragraph_block_budget` | **10 PASS / 0 FAIL**, 필터 제외 194건 |
| A 4개 source의 `issue_3587_` focused nextest | **25 PASS / 0 FAIL**, 필터 제외 539건 |
| `cargo clippy --locked --target-dir …/target/pr-review -- -D warnings` | PASS, 46.76초 |
| manifest `--check` | PASS |
| B 계획·본 기록 파일 단위 Markdown 링크 검사 | PASS |

B 테스트 컴파일 2분 10초, 실행 0.011초. A 테스트 컴파일 9.83초, 실행 0.229초.
이는 해당 환경의 집중 테스트 시간이지 자동화 복제 API 성능 수치가 아니다.
두 실행의 nextest 버전 0.9.137 권장 버전 경고 및 `report-skipped` 설정 키 경고는 A 통합
검증 때와 동일하다. 도구 설정을 이번 변경으로 수정하지 않았다.

이번 절편은 제품/테스트 실패 없이 통과했다. 전체 nextest·WASM/workspace Clippy·Docker WASM·
한컴 시각 검증은 이번 B 코드에서 아직 실행하지 않았으며, 이전 A의 전체 통과로 갈음하지 않는다.
원격 push 또는 PR 전에는 규정된 전체 lint 묶음을 수행해야 한다.

## 4. 자원 절편 직후 남았던 범위 (후속은 §5 이후)

- B1: 종류/안전 raw 슬롯 지원표, 최초 문제의 typed path, 필드/연결선 양방향 경계 검사,
  중복 참조 검사, 공유 자원 존재 검사. 현재 budget 함수는 이것들의 대체물이 아니다.
- B1 비용 검증: 실물 두 블록의 실제 사본 생성 1/10/100회 시간·메모리와 추정치 대조.
- B2: 공유 allocator·사본별 map·단일 삽입·원본 Box/표 reflow 출처 보존·오류 무변경.
- B3: HWP/HWPX 저장·native 및 WASM/한컴 사용 검증.
- 기존 clipboard/조판 코드는 수정하지 않았다. 원격 push/PR/게시도 하지 않는다.

## 5. 후속 절편 — 지원표와 참조 경계 (구현 전 고정)

메인테이너의 다음 절차 승인에 따라 strict 사전검사를 추가한다. 삽입 구현은 아직 별도다.
검사는 문단/컨트롤/셀/글상자/캡션/그룹 자식의 typed path를 오류에 제공한다.

| 대상 | 첫 지원 범위 / 거부 조건 | 코드 근거 |
| --- | --- | --- |
| 일반 문단 | 빈 문단·Page/Column break 유지. Section/MultiColumn 및 raw break bit 0/1 거부 | `parser/body_text.rs::parse_para_header` |
| 문단 raw 헤더 | 없음 또는 10 bytes(ID 슬롯 6..10), 12 bytes 중 변경추적 값 0 허용. 기타 확장/변경추적 거부 | `serializer/body_text.rs`의 instanceId/변경추적 기록 |
| 문단 range tag | 초기 strict profile은 tag kind/data의 확장 의미·참조 재매핑을 보증하지 않으므로 비어 있지 않으면 거부 | `model/paragraph.rs::RangeTag`, 기존 A remapper의 미해석 범위 |
| 표·수식 common raw | 없음 또는 36/40 bytes, 이후 길이가 맞는 UTF-16 설명문까지 허용. 선택 prevent_page_break가 없고 빈 설명문 길이만 있는 38 bytes도 허용. common 미해석 tail 및 표 레코드 미해석 tail 거부 | `parser/control/shape.rs::parse_common_obj_attr`, `commands/object_ops/table.rs`의 기존 생성기, `clone_identity/remap.rs` |
| 셀 LIST_HEADER raw | 없음, 폭+zero padding 13 bytes, 또는 고정 필드명 marker/UTF-16 이름/zero trailer의 완전한 형태 허용. 그 밖은 거부 | `serializer/control.rs::build_cell_list_extra` (model의 필드명 offset 주석보다 실제 writer/parser의 15/17 사용) |
| 기본 도형/그룹/글상자 | 모델링된 소유 구조 순회. 미해석 connector/polygon tail, textbox LIST_HEADER tail은 초기 거부 | `model/shape.rs`, `identity/walk.rs` |
| 그림 | payload 없음 또는 알려진 5/17/18 bytes, own ID 1..5 및 크기/alpha 보존. 기타 확장 거부 | `parser/control/shape.rs` 그림 extra 파싱 |
| ClickHere | begin ID와 종료 마커의 소유/순서/범위 검사. shared fieldid는 unique ID와 구별. 이름은 변경하지 않음 | `model/paragraph.rs::FieldRange/OrphanFieldEnd`, `clone_identity/remap.rs` |
| ClickHere 이름 | 이름만 포함한 단일 ParameterSet의 고정 헤더/길이/UTF-16 이름이 IR 이름과 일치하면 CTRL_DATA 허용. 추가 항목/tail은 거부 | `serializer/control.rs`의 `0x021b/0x4000` 이름 기록 |
| ClickHere 확장 | raw parameter XML/비어 있지 않은 parameters 및 위 이름 형식 외 CTRL_DATA payload는 아직 의미 검증 전이므로 거부 | `model/control.rs::Field`의 HWP/HWPX 보존 경계 |
| 그 밖의 컨트롤 | Section/Column/Header/Footer/쪽 설정·Form·각주/메모·Unknown·OLE/Chart·기타 필드/책갈피 등은 명시적 미지원 | B 계획 §4 |

읽기 전용 사전검사는 포맷 유효성 전체를 인증하지 않는다. 지원 경계 밖의 실제 샘플은 오류 경로와
종류를 기록하고, 통과시키기 위해 payload를 지우지 않는다. 이 절편에서는 참조 폐쇄성까지 검증하며,
공유 스타일/BinData의 존재 및 전체 저장 검증은 후속으로 남긴다.

### 5.1 참조 판정 방식

- 새 `validate_paragraph_block_native`는 성공 시 비용 정보를 반환하며 삽입하지 않는다.
  실패는 `ParagraphBlockValidationError {code, path, detail}`이다. path는 요청 원형 시작을
  기준으로 한 상대 소유 경로이며 section은 요청의 section_index다. 일반 DSEL 주소나 영구 ID가 아니다.
- 필드의 고유 begin ID와 공유 fieldid는 별개다. 같은 이름과 공유 fieldid를 이유로 중복 오류를 내지 않는다.
- 같은 문단의 FieldRange는 control index/문자 범위/내부 슬롯 범위를 검사한다. 다문단 종료 마커는
  동일 소유 문단 목록의 뒤 문단에서 끝나야 한다. 셀/글상자/캡션 경계를 넘어 닫는 필드는 거부한다.
- 원문 전체에서 해당 begin ID의 소유자와 종료 마커를 확인한다. 블록 안 end→밖 begin뿐 아니라
  안 begin→밖 end, 누락/복수 end, 원문의 중복 begin도 거부한다.
- 연결선은 A remapper와 같은 common ID/legacy alias/drawing ID 집합으로 대상을 찾는다.
  같은 객체가 여러 alias를 갖는 것은 중복이 아니며 다른 객체가 같은 참조 번호를 가지면 모호하다.
  안에서 밖을 참조하거나 밖 연결선이 안 객체를 참조하는 경계는 strict API에서 거부한다.
- 원문은 기존 bounded owned walker로 검사한다. 원형 관련 ID만 원장에 보관하고, 소유자는
  첫 소유자+중복 여부만 기록하여 원문의 중복 개수에 비례한 집합을 만들지 않는다.
  소유 노드와 별도로 FieldRange/OrphanFieldEnd 레코드도 max_document_nodes로 제한한다.
- 이번 단계는 비용용 원문 순회와 참조용 원문 순회가 각각 있다. B2의 ID 예약/검사 통합 시
  재사용할 수 있으나 현재 단일 원문 순회라고 보고하지 않는다.

### 5.2 첫 검증에서 발견한 누락

`c32c8475d1`에서 신규 16건 중 15 PASS / 1 FAIL. 실패는 기존 `create_table_native`가 생성한
표를 지원 검사에서 거부한 것이다. `commands/object_ops/table.rs`는 common fixed 36 bytes 뒤에
빈 설명문의 길이 2 bytes를 넣은 38-byte raw를 생성한다. 파서도 선택 prevent_page_break를
읽을 4 bytes가 없으면 빈 설명문을 읽는다. 최초 지원표에 이 선택 필드 조합이 누락되었다.

원본 생성기·기대값을 변경하지 않고 이 레코드 형식을 지원표에 보완했다. 이와 함께 실제 writer에
정의된 셀/이름 전용 CTRL_DATA 형식과 unknown tail의 경계 계약 2건을 추가했다.
정정 소스는 `125272a5e6`이며 최종 재검증 결과를 아래에 기록한다.

### 5.3 검증 실행 경로 정정과 최종 후보

이후 제가 종전 suite 번호를 고정해 실행한 명령은 자동 재배정 결과와 달라 **0건 실행 / exit 4**로
끝났다(`focused-final.log`). 이 실행은 통과 증거에 포함하지 않는다. 현재 manifest를 한 번 도출해
`buildCaseIndex`에서 `issue_3587_` source 6개를 선택하고, 그 target들을 중복 제거해 nextest에
전달하도록 실행 명령을 정정했다. 단일 source는 기존 `run-rust-test.mjs`로 같은 방식으로 실행할 수 있다.
앞으로 이 타스크에서 generated suite 번호를 고정해 재사용하지 않는다.

`125272a5e6`의 동적 선택 실행은 신규 18 + 이전 예산 10 + A 25 = **53 PASS / 0 FAIL**이며
native Clippy도 PASS였다. 이후 초기 strict 범위가 문단 range tag의 미확인 확장까지 포함하도록
보완하고 경계 테스트 1건을 추가했다. 최종 제품/테스트 후보는 `f56e831dcb`다.
이 후보는 직전 53건 결과로 갈음하지 않고 동일 SHA에서 재검증한다.

최종 `f56e831dcb`의 동일 SHA review worktree에서 재검증을 완료했다.

| 검사 | 최종 결과 | 로그 (`output/3587/b1-references/`, Git 제외) |
| --- | --- | --- |
| 현재 manifest의 관련 source 6개를 선택한 nextest | **54 PASS / 0 FAIL**, 868 filtered/skipped, 실행 0.255초 | `focused-final2.log` |
| native Clippy `--locked … -- -D warnings` | PASS, 28.32초 | `clippy-native-final2.log` |
| fmt 적용 및 `--check` | PASS, 검증 후 tracked 변경 없음 | 로컬 실행 |
| manifest `--prepare` / `--check` | PASS, review worktree에서만 파생물 준비 | `prepare-final2.log`, `manifest-final2.log` |

54건은 신규 지원/참조 계약 19건 + 기존 예산 10건 + A 계약 25건이다. 전체 회귀 테스트 숫자가
아니며, nextest 0.9.137과 권장 0.9.140의 버전 차이 경고는 남아 있다. 이번 후보의
WASM/workspace Clippy·전체 nextest·Docker WASM·한컴 시각 검증은 수행하지 않았다.
전체 B 완료 또는 PR 제출 가능 판정이 아닌 이번 읽기 전용 절편의 집중 검증 결과다.

### 5.4 참조 절편 직후 남았던 작업 (공유 자원 후속은 §6)

1. 공유 스타일/BinData 존재 검사와 실제 두 블록의 지원 경계 대조. 현재 API 설명처럼 지원/참조
   검사 성공이 모든 자원·저장 호환성을 인증하지는 않는다.
2. 사본 생성 1/10/100회 실측과 구조 예산 대조. 현재까지의 숫자는 집중 테스트 결과다.
3. B2 실제 반복 삽입: 공유 ID allocator, 사본별 참조 map, 표 reflow 출처, 단일 commit 및 이벤트.
4. B3 HWP/HWPX 저장/재열기와 사람에게 제공할 실제 결과물. C/D 및 PR 작업은 별도 승인 범위다.

이 절편은 삽입/붙여넣기/조판 코드를 변경하지 않는다. source worktree의 기존 A 코드와 증적,
별도 review worktree 및 고정 target cache를 보존한다.

## 6. 후속 절편 — 공유 자원 참조 검사

다음 절차 승인으로 `validation/resources.rs`를 추가한다. 원형에서 도달하는 **모델링된 참조의
등록 여부**를 확인하며 DocInfo를 복제·정리·수정하지 않는다. 이미지의 바이트 무결성이나 opaque
DocInfo raw payload의 모든 숨은 참조까지 인증하지 않는다.

| 참조 | 확인 방식 |
| --- | --- |
| 문단·글자 모양·스타일·탭 | 0-based 인덱스. 빈 char run은 기존 0번 기본 모양 확인 |
| 스타일 내부 참조 | 다음 스타일·문단/글자 모양을 따라 검사. 방문 집합으로 자기/순환 참조 종료 |
| 글자 모양 | 7개 언어별 글꼴 등록 및 테두리 참조 확인. OS 폰트 설치 여부 검사가 아님 |
| 테두리 | 0은 없음, 양수는 1-based. 문단·글자·표·셀·zone 및 배경 이미지 참조 |
| 개요·번호·글머리표 | head_type에 맞는 정의. 개요 ID 0은 구역 기본값, 둘 다 0이면 내장 기본값. 이미지 글머리표는 초기 unsupportedResource |
| 도형·그림 | 내부 그림과 배경 이미지의 BinData 등록 슬롯. ordinal 우선, 범위 밖만 storage ID 검색 |
| 내장 폰트·대체 내장 폰트 | 그림 ordinal과 구별하여 resolved storage ID로 확인 |

근거는 `foreign_paste.rs`의 서식 매핑, `renderer/layout/utils.rs`의 BinData/개요 해석,
`bin_data_prune.rs` 및 `queries/rendering.rs`의 내장 폰트 storage ID 처리다.
shared resource edge/lookup는 `max_document_nodes` 이내에서 별도 계수한다. 원형 밖의 사용하지
않는 자원은 검사하지 않는다. BinData의 `load`뿐 아니라 `len/is_empty`도 호출하지 않는다.
따라서 Lazy 등록 항목은 실제 파일이 읽히는지 아직 확인되지 않은 상태로 구분한다.

### 6.1 검증 준비와 오류 기록

- 최초 제품 `9a4118442a`의 native Clippy에서 구역 개요 ID 경로를 `Section` 직접 필드로 잘못
  접근한 컴파일 오류를 검출했다. 실제 소유 위치 `Section.section_def.outline_numbering_id`로
  제품·테스트를 정정한 후보는 `b6f2c23210`이다. 최초 후보는 검증 통과로 세지 않는다.
- 주 checkout에서 전체 fmt를 먼저 실행한 시도는 낡은 파생 suite의 누락 source 참조로 실패했다.
  변경한 파일은 직접 rustfmt로 정리하고, 전체 fmt는 파생물을 준비한 review worktree에서만
  실행하는 절차로 복귀했다. 사용자 파일이나 파생 파일을 source에 stage하지 않았다.
- 신규 `issue_3587_paragraph_block_resources`는 9개 자원 계약 + 1개 실물 무변경 관측이다.
  실물 관측 테스트 통과는 입력 불변을 의미하며, 블록 지원 또는 시각 통과가 아니다.
- 실물 원형: `hwp_table_test.hwp` s0/pi3, `table-in-tbox.hwp` s0/pi0,
  `rnote/labnote-001.hwp` s0/pi0. 컨트롤을 제거하거나 블록을 임의 수정하지 않고 최초 오류를 남긴다.
- 검증 결과·실물 판정은 동일 후보 실행 후 기록한다. 로그 위치는 Git 제외
  `output/3587/b1-resources/`다.

`b6f2c23210` 실행은 64건 중 22건 실행, **19 PASS / 3 FAIL**, 실패 즉시 중단으로 42건 미실행이다.
세 실패는 신규 합성 입력의 구성 오류였다. 실제 blank 템플릿에는 번호 정의·추가 스타일·테두리가
있고, 0번 글자 모양도 0번 글꼴만 참조하지 않는다. 저는 이를 한 항목뿐인 자원 그래프로 가정해
① 참조되지 않는 글꼴을 바꾸고 실패를 기대했고 ② 사용 중인 Border(2)를 제거했으며
③ 이미 있는 Number(1)이 없다고 기대했다. 제품 검사를 완화하지 않고 합성 계약용 DocInfo를
명시적인 한 항목 그래프로 구성했다. 실제 문서 관측 경로의 입력에는 이 초기화를 적용하지 않는다.
정정 테스트 후보는 `e77607148f`이며 `--no-fail-fast`로 관련 64건을 다시 실행한다.

첫 실물 결과: `hwp_table_test.hwp`의 `[3,4)`는 통과, 나머지 두 파일의 `[0,1)`는
`unsupported / paragraph(0) / section/multicolumn paragraph boundary`였다. 구역 경계를
제외한다는 승인된 B 계약에 따른 거부이며 문서 손상이나 렌더링 실패 판정이 아니다.
`labnote-001.hwp`의 다른 본문 문단(최대 16개)도 후보별로 읽기 전용 검사한다. 이 관측을 위해
임의로 SectionDef/ColumnDef를 지우거나 첫 문단의 raw 플래그를 변경하지 않는다.

입력 해시: 기존 두 파일은 Stage 1 §2.1의 해시와 일치하며, 연구노트 SHA-256은
`8401e778edc386f0a87a0df4120a2cf0733aee36c84070ddc864499bde73a1af`다.

### 6.2 최종 집중 검증과 실물 경계

최종 `e77607148f`의 review worktree에서 **64 PASS / 0 FAIL**, 필터 제외 1,309건.
기존 관련 54건 + 신규 자원 계약 9건 + 실물 입력 불변 관측 1건이며 전체 nextest 결과가 아니다.
컴파일 20.71초, 실행 0.266초다(`focused-final.log`). native Clippy는 44.60초 PASS
(`clippy-final.log`). 최종 fmt 및 manifest `--check`도 PASS였다
(`fmt-final.log`, `manifest-final.log`). review worktree의 tracked 변경은 없다.
nextest 버전/설정 키 경고는 기존과 같으며 설정을 이번 절편에서 바꾸지 않았다.
전체 회귀·WASM/workspace Clippy·Docker WASM·한컴 시각 판정·원격 push/PR은 이번 후보에서 미수행이다.

| 입력과 블록 (s0, 0-based 본문 문단) | 사전검사 결과 | 해석 |
| --- | --- | --- |
| `hwp_table_test.hwp` `[3,4)` | PASS. 소유 노드 26, 구조 예산 10,684 bytes | 기존 표 블록으로 B2 검증 가능 |
| `table-in-tbox.hwp` `[0,1)` | 구역 경계 unsupported | 기존 컨트롤 복제 A와 문단 전체 복제 B의 범위 차이. 자동 제거하지 않음 |
| `labnote-001.hwp` `[0,1)` | 구역 경계 unsupported | 첫 구역 설정은 원본에 유지 |
| `labnote-001.hwp` pi 1~12 각각의 단일 문단 | **12/12 PASS** | 13개 본문 문단을 모두 조사. 복제 성공·시각 판정은 아직 아님 |

연구노트의 컨트롤 있는 통과 문단은 pi 2/4/6/8/11(각 1개), pi 12(3개)다.
pi 12 한 문단의 소유 노드는 94, 구조 예산 37,883 bytes, 대응표 예산 16,320 bytes다.
이는 자식 구조를 포함한 검사 수치이며 실제 반복 생성 시간/RSS가 아니다. 여러 문단을 묶을 때는
선택된 묶음으로 필드·연결선 경계를 다시 검사해야 하므로 단일 문단 통과를 무조건 합성하지 않는다.

### 6.3 다음 순서

1. 통과한 실물 문단에서 반복 원형과 보존할 앞뒤 문단을 정하고 B2 사본 생성·단일 삽입을 연결한다.
2. 실제 생성된 사본의 1/10/100회 시간·메모리와 B1 논리 예산을 대조한다. 현재는 실제 복제 API가
   없으므로 해당 실측을 완료했다고 하지 않는다.
3. B3 저장·재열기/사람 판정, 이어 C 공개 API·채우기 구현 후 Gym으로 연구노트 시나리오를 실행한다.
4. `table-in-tbox`의 구역 설정 혼재 문단 전체 지원은 자동 확대하지 않는다. A 컨트롤 복제 회귀는
   그대로 유지하며, B 원형 선정/지원 범위 변경이 필요하면 별도로 근거를 제시한다.
