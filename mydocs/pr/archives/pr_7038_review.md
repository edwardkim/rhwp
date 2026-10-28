# PR #7038 검토 기록

## 판정: 승인

- 대상: [PR #7038](https://github.com/edwardkim/rhwp/pull/7038), [이슈 #5551](https://github.com/edwardkim/rhwp/issues/5551).
- 검토 코드 head: `d2668706f7f7ac9a1c4f070e311d1ded57998100`.
- 2026-09-11 21:43 KST 기준 해당 head의 CI gate 완료 및 `MERGEABLE/CLEAN`을 확인했다.
- 승인은 캡션 소유자 메타데이터와 컨트롤 원본 주소 계약 범위다. 한컴과의 전체 문서 시각 일치나 머리말·꼬리말 캡션 소유자 확장을 승인한 것이 아니다.
- 이 문서는 코드 CI 완료 뒤 추가하는 문서 전용 trailing commit이다. 문서 head의 CI 완료 및 merge 가능 상태를 다시 확인한 뒤 병합한다. 아직 발생하지 않은 병합·devel CI 성공은 기록하지 않는다.

## 구현 및 요구사항 대조

- 캡션 TextLine에 선택적 `captionOwner`를 추가했다. `secIdx`, `paraIdx`, `controlIdx`, `controlKind`, `captionOrdinal`은 본문 컨트롤의 원본 주소를 사용하며 텍스트나 화면 좌표로 추정하지 않는다.
- 표·그림·도형 캡션을 구분한다. 같은 캡션 문단의 줄바꿈에는 같은 ordinal을 유지하고 다음 캡션 문단에는 다음 ordinal을 부여한다.
- 일반 텍스트 및 소유자를 확정할 수 없는 경로는 필드를 생략한다. 기존 `pi`, 트리 구조 및 내부 paint 순서용 `stableIndex`를 소유자 주소로 재해석하지 않는다.
- 컨트롤의 문단·컨트롤 주소가 부분적으로 없는 경우에도 알려진 `secIdx`는 보존한다. 빈 section에 컨트롤이 없는 것은 오류로 취급하지 않는다.
- 머리말·꼬리말 상속 컨트롤에는 실제 source section과 `headerFooter` 참조를 제공한다. 내부 sentinel/cache key는 유지하며 외부 API에 잘못된 section으로 노출하지 않는다.
- 머리말·꼬리말의 하위 목록 주소를 본문 주소로 위조하지 않는다. 해당 경로의 `captionOwner`는 생략하며 향후 별도 주소 계약이 필요하다.
- Studio 편집 소비자는 본문 컨트롤과 머리말·꼬리말 컨트롤을 구분한다. 원본 section 보정으로 머리말·꼬리말 컨트롤이 본문 편집 대상에 잘못 매칭되지 않도록 그림·연결선·키보드·표 관련 소비 경로를 보정했다.
- 분석, 코드 수정, 검증 결과, 커밋의 단계 구분은 [Stage 1](../../working/task_m100_5551_stage1.md)과 [Stage 2](../../working/task_m100_5551_stage2.md)에 기록했다.

## 로컬 검증과 실제 동작 증거

### PR 준비 코드 `888725201` 기준

- 전체 default-feature 회귀: 8 threads, `--no-fail-fast`, 9,479개 통과, 46개 skip, 실패 0. 실행 시간 365.225초는 빌드 시간을 제외한다. 사용자 지시에 따라 PR 직전 한 번 실행했다.
- Native Skia lib: rhwp 3,930개 통과·13개 ignored, workspace 관련 lib 182개 통과. PNG placeholder 2개 및 direct PDF 4개 통과.
- Rust 집중 6개, Studio 정책·연결선 집중 4개 통과. TypeScript 검사와 Studio 전체 1,661개 통과·2개 skip.
- fmt, native/WASM32/workspace Clippy, workspace build, WASM build 및 manifest 검사 통과. generated harness drift는 prepare로 재생성해 해결했고 source-side 테스트 기준선을 완화하지 않았다.
- 새 WASM을 실제 Chrome에서 실행해 입력 4개, 총 42페이지의 공개 API를 점검했다. 캡션 줄 41개가 컨트롤 소유자 주소와 일치했고 `secIdx` 누락·범위 초과는 0이었다.
- 3-section 합성 HWPX: 5페이지, 18컨트롤, 24캡션 줄. 표·그림·도형과 줄바꿈 8그룹을 확인했으며 빈 section 2에 컨트롤이 없는 것은 정상이다.
- `samples/hwp3-table-caption.hwp`: 1페이지·1컨트롤·1표 캡션 줄.
- `samples/issue6284/child_policy_top_caption_charts.hwpx`: 34페이지·206컨트롤·16캡션 줄, 그림·표 및 줄바꿈 4그룹.
- `samples/issue5802/hf_cross_section_inherit.hwp`: 2section·2페이지·4컨트롤. 상속 머리말·꼬리말의 source section과 내부 stableIndex 보존을 확인했다.
- Stage 2 전후 42페이지 SVG hash가 동일했다. 대표 페이지를 실제 Canvas에 렌더링해 비어 있지 않음을 확인했다. 이는 API 메타데이터 변경의 출력 불변성 증거이며 한컴 PDF 시각 대조를 수행했다는 뜻이 아니다.
- 사용 WASM SHA-256: `96a7e033501d3765f6ea11626755d7ef1dd277da8940e7b53d9e2d912a2b3dd7`. 위 브라우저 결과를 이후 devel 동기화 head에서 재빌드한 결과로 표기하지 않는다.

### devel 동기화 코드 `d2668706f` 기준

- 기존 승인된 devel 동기화 commit을 유지했다. 이후 오늘할일 갱신을 위해 추가 source merge·rebase는 하지 않는다.
- #5551 및 함께 들어온 #7028 집중 테스트 15개 통과. fmt, 세 Clippy, workspace build 및 manifest 검사 통과.
- source-unit 기준: 4,205 tests / 298 modules. suite manifest: 1,260 sources / 5,352 static attributes / 28 suites와 20 exceptions.
- 전체 로컬 회귀를 중복 실행하지 않았으며 아래 최종 코드 head의 원격 CI 결과를 별도로 확인했다.

## 실제 코드 head CI

| 검사 | 결과 및 증거 |
| --- | --- |
| CI | [34598844280](https://github.com/edwardkim/rhwp/actions/runs/34598844280) success. Build & Test, lint, Native Skia, archive A-D build/test와 실행된 frontend gate 성공 |
| CodeQL | [34598844306](https://github.com/edwardkim/rhwp/actions/runs/34598844306) success. Rust, JavaScript/TypeScript, Python 분석 성공 |
| Adapter inter-diff | [34598844269](https://github.com/edwardkim/rhwp/actions/runs/34598844269) success. preflight 성공, worker는 정책 분기 skip |
| Proptest | [34598844316](https://github.com/edwardkim/rhwp/actions/runs/34598844316) success. preflight 성공, worker는 정책 분기 skip |
| Render Diff | [34598844092](https://github.com/edwardkim/rhwp/actions/runs/34598844092) success. preflight 성공, Canvas worker는 정책 분기 skip |
| CI Impact Policy | 최종 status success, pending 없음 |

- skipped worker를 실제 실행·통과한 테스트로 합산하지 않는다. 최종 코드 CI는 heavy worker가 실행된 full 검증이며 문서 fast-pass라고 부르지 않는다.

## 검증 한계와 잔여 범위

- 이슈에서 언급한 `minimal-caption.hwpx`는 실제 첨부되지 않아 사용하지 않았다. 저장소 공개 fixture와 합성 입력을 구분해 검증했다.
- 머리말·꼬리말 캡션의 완전한 owner 주소 계약은 이번 범위가 아니다. 본문 owner로 잘못 표기하지 않는 것을 검증했다.
- 실제 Studio UI의 클릭·삭제 전체 E2E 및 한컴 PDF 전 페이지 시각 대조는 수행하지 않았다. 정책 함수·연결선 소비 함수 실행과 실제 WASM API/Canvas 검증 결과만 주장한다.
- 임시 로그, 합성 입력, SVG, JSON, 브라우저 스크립트 및 임시 PNG는 커밋하지 않는다. 이번 변경은 출력 차이가 없는 API 메타데이터 계약이므로 시각 fidelity를 주장하는 이미지 코멘트를 만들지 않는다.

## Merge 후 PR·이슈 comment 계획

- `post_merge.md`에 따라 최신 문서 head CI, 일반 merge commit SHA와 해당 devel push의 실제 CI·CodeQL 및 적용되는 gate 완료를 먼저 확인한다.
- PR #7038에는 self-review 포함 사실, 실제 merge SHA, PR/devel CI URL과 위 검증 범위를 UTF-8 body file로 한 번 기록한다. Stage 1·2 및 이 리뷰는 실제 merge SHA의 영구 링크를 사용한다.
- 이슈 #5551에는 본문 `Closes #5551`과 API의 실제 종료 상태를 대조한다. devel 대상 PR의 closing-reference 목록이 비어 있다는 이유로 본문의 closing keyword를 무시하지 않는다. 자동 종료 여부를 확인하고 필요하면 종료 처리한다.
- 이슈가 이미 CLOSED여도 같은 merge SHA의 검증 결과 comment가 없다면 구현 내용, 실제 테스트 결과, 머리말·꼬리말 owner 범위 한계를 한 번 기록한다. 기존 comment가 있으면 중복 게시하지 않는다.
- 게시한 comment body는 API로 재조회한다. 임시 로컬 파일을 공개 증거 링크로 사용하지 않으며 실행하지 않은 UI·PDF 검증을 주장하지 않는다.
- 기본 작업공간이 clean이고 merge SHA가 upstream/devel에 포함되며 사용 중인 관련 작업이 없을 때만 local branch `fix/5551-caption-owner-20260911`와 전용 target `target/review-5551-20260911`을 정리한다. 기본 작업공간과 공유 target은 보존한다. 이번 작업의 동일 이름 upstream 임시 head branch도 최종 SHA·보호 여부·다른 OPEN PR 사용 여부를 확인한 뒤 별도 승인 질문 없이 lease를 사용해 자동 정리한다.
