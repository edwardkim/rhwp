# PR #6214 검토 기록

## 대상

| 항목 | 값 |
| --- | --- |
| PR | [#6214](https://github.com/edwardkim/rhwp/pull/6214) |
| 작성자 | `edwardkim` |
| base | `devel` |
| source | `task_m100_4968` |
| Full CI code candidate | `9c568b40a` |
| 최신 base 병합 head | `469ac96fd` |
| 병합한 `upstream/devel` | `2db6fb29b` |
| 규모 | 문서 작성 시점 참고값 117 files, +21,488 / -73 |
| 상태 | 문서 작성 시점 `MERGEABLE` / `CLEAN`; merge 직전 재확인 필요 |

관련 이슈는 [#4968](https://github.com/edwardkim/rhwp/issues/4968)이고, CI generator 분류 후속은
[#6215](https://github.com/edwardkim/rhwp/issues/6215)로 분리했다. 이 검토에는
`collaborator_self_merge` 기본 경로와 `review_only_fast_pass` 보조 경로를 적용했다.

## 검토 범위

- `ResolvedCharStyle.kerning` 의도를 exact font slot/source, bounded pair measurement, fresh line boundary와
  최종 `TextRunNode.layout_positions`까지 전달하는 흐름을 확인했다.
- Canvas2D, CanvasKit, SVG, HTML, native Skia가 layout owner의 검증된 positions만 재생하고, 손상되거나
  replay 문자열과 맞지 않는 sidecar는 기존 K0 계산으로 닫히는지 확인했다.
- exact source 등록의 32 MiB 개별 font, 64 MiB registry, 256 face, 4,096 slot·scalar/glyph,
  4,097 positions, 256 paragraph segment 상한과 slot conflict fail-closed를 확인했다.
- 문서 embedded font와 native/WASM 명시 등록 API가 family 이름을 재탐색하지 않고
  `(char_shape_id, language_index, bytes, face_index)` provenance를 공유하는지 확인했다.
- Q0~R4E 단계 보고서, 공개 fixture·canonical JSON, 한컴 PDF 교차검증과 성능 참고 계측을 대사했다.
- 최신 `devel` 병합으로 들어온 CI 정책 변경은 #4968 source diff와 분리해 계약 테스트와 최신 PR check로
  다시 확인했다.

## 발견 사항

### 차단 결함

없다. 구현은 K0 무변화, exact-source 격리, stored `LineSeg` feature detection, bounded work와
native/WASM/backend parity 보호 불변식을 유지한다.

### merge 전 정정할 메타데이터

PR 본문의 `current code head`는 초기 `4f28e195f`, source-unit 수는 4,224로 남아 있다. 최종 source-unit
기준은 4,221이고 최신 PR head는 `469ac96fd`다. 코드나 검증 결과의 결함은 아니지만 merge 전 PR 설명을
현재 검토 기록과 맞춰야 한다.

### 범위 밖·잔여 위험

- Studio가 실제 선택한 font bytes를 exact slot에 자동 등록하는 연결은 이번 PR에 포함하지 않았다. family
  이름 추측으로 연결하지 않고 fallback·selection owner 후속 범위로 유지한 판단이 타당하다.
- fallback metric DB, GSUB, vertical metrics, variable axis는 [#4969](https://github.com/edwardkim/rhwp/issues/4969)
  범위로 남았다.
- native 참고 계측에서 K0는 약 +0.4~1.1%, K1은 약 +3.1% 중앙값 증가가 관측됐다. 정식 허용 상한이 없는
  기초 데이터이며, 4,096 scalar·256 segment 상한과 K0 source-lookup 생략으로 입력 비용은 제한된다.
- 최종 Docker WASM은 이전 R4E-1 측정보다 10,884 bytes(+0.113449%) 컸지만 그 사이 `devel` 변경을 함께
  포함하므로 #4968 단독 회귀로 귀속하지 않았다.

## 로컬 검증

최신 `devel@2db6fb29b`를 merge한 `469ac96fd`에서 다음 검증을 완료했다.

| 검증 | 결과 |
| --- | --- |
| `cargo fmt --all` / `cargo fmt --all -- --check` | PASS |
| `git diff --check` | PASS |
| Rust unit-tier base-ref | 4,221 tests / 299 modules, PASS |
| integration manifest 정책 | 18/18 PASS |
| review worktree manifest prepare/check | 983 sources / 4,417 attrs / 32 suites + 9 exceptions / 41 targets, PASS |
| #4968 public integration | 29/29 PASS |
| trusted post-merge reuse Node 계약 | 2/2 PASS |
| duration policy Node 계약 | 7/7 PASS |
| trusted reuse workflow Python 계약 | 3/3 PASS |
| nextest archive workflow Python 계약 | 14/14 PASS |

R4E 최종 후보에서는 전체 nextest 8,451/8,451, Native Skia lib·#2225·direct PDF, Docker WASM,
native/WASM 22문맥 canonical parity와 failure matrix 6/6도 통과했다. 같은 source candidate의 GitHub Full
CI가 성공했으므로 최신 base merge 뒤 광범위 로컬 회귀를 중복 실행하지 않고 current-base 계약 검증에
집중했다.

## CI

- `9c568b40a`의 [Full CI](https://github.com/edwardkim/rhwp/actions/runs/33068377156), CodeQL,
  Render Diff, Proptest roundtrip과 Adapter inter-diff가 모두 성공했다.
- 최신 base merge head `469ac96fd`에서는 trusted post-merge 검증과 CI·CodeQL·Render Diff·Proptest·Adapter
  preflight, 최종 `Build & Test`가 모두 성공했다. 11개 check가 성공했고 heavy worker 19개는 검증된
  candidate 재사용 정책에 따라 의도적으로 skip됐다.
- 이 review·오늘할일만 추가하는 trailing commit은 `mydocs/` 허용 범위다. push 뒤 최신 head의 preflight와
  required aggregate 성공을 다시 확인해야 한다.

## 시각·오라클 증적

- R4C-4 공개 실문서 교차검증에서 96 DPI pixel match 96.77153%, flagged page 0을 기록했다. 글상자 inline
  image의 가로 위치와 크기는 한컴 PDF와 사실상 같았고, 세로 차이는 앞 본문의 기존 page-flow 잔여로
  분리했다.
- 이 오라클의 글상자 본문은 공백과 inline image이므로 문자 kerning 줄 경계의 직접 정답으로 과장하지
  않았다. visible text의 body·table-cell·text-box K0/K1 차이와 결정성은 공개 synthetic integration이 맡는다.
- HWP 2020 `11.0.0.9136`에서는 flag 보존과 K0/K1 PDF 위치 무차이까지만 관측했다. 구현은 한컴 버전 분기가
  아니라 exact OpenType capability 기능 탐지를 사용한다.
- 단계별 결과와 위 한계를 메인테이너가 승인했다. 최신 code candidate의 Render Diff도 성공했다.

## 결론

코드 차단 결함은 발견되지 않았다. PR 설명의 오래된 head·test count를 정정하고, trailing 문서 head의
required checks와 최신 mergeability를 확인한 뒤 정상 merge 후보로 권고한다. merge와 #4968 close는 각각
메인테이너 승인 후 수행한다.
