# Task M100 #6641 PR 초안

## 제목

```text
[field] 필드 편집 후 소유 문단 LineSeg 재조판 복구
```

## 본문

```markdown
## 변경 요약

필드 값을 바꾼 뒤 `LineSeg`를 비우기만 하고 다시 조판하지 않아 메모리 문서와 저장·재적재 문서가
달라지던 원인을 해결합니다. `batch fill --verify`의 diff나 종료 코드를 완화하지 않고, 편집 문단을
본문·표 셀·글상자의 실제 소유 폭에서 즉시 reflow합니다.

- 본문은 편집 전 흐름 끝을 보존하고 reflow 뒤 section vpos를 다시 잇습니다.
- 중첩 필드는 `TableCell`·`TextBox` 소유 의미를 보존해 셀 padding 또는 글상자 margin을 반영합니다.
- HWPX 가상 셀의 공개 field ID도 by-name 경로와 같은 cell mutation으로 수렴시킵니다.
- 소유자 해석 또는 reflow가 실패하면 stale 조판을 성공으로 반환하지 않습니다.
- 기존 integration source 안에서 본문·표 셀·글상자·깊이 2 중첩과 저장 왕복 계약을 강화합니다.

## 관련 이슈

Closes #6641

이 수정은 부모 Gym 정상화 이슈 #6628의 BO05·BO15 차단 원인을 해소합니다. #6628 자체는 남은
sub-issue #6669와 부모 최종 감사를 마친 뒤 별도로 종료합니다.

## 테스트

- [x] `cargo fmt --all` 및 `cargo fmt --all -- --check`
- [x] native root, WASM32 lib, workspace all-targets Clippy `-D warnings`
- [x] `cargo build --locked --workspace --target-dir target/pr-review`
- [x] 기존 field focused 56건 + 최신 devel 인접 layout 4건: 60/60
- [x] 전체 integration nextest: 8,973/8,973 pass, 정책상 ignored 46건
- [x] integration manifest: 1,132 sources / 4,825 attrs / 48/48 targets
- [x] unit-tier 정책 확인: 4,221 tests / 299 modules
- [x] 새 integration source, generated suite·manifest, Cargo 파생 target을 제출 diff에 포함하지 않음
- [x] `git diff --check` 및 변경 Markdown 링크 검사
- [x] #6628 Gym 재검증: BO05·BO15 2/2, positive 1,035/1,035,
  discrimination false-pass 0, trajectory 239/239 load-bearing

Native Skia·Docker WASM·시각 검증은 renderer 일반 정책, serialization, WASM 또는 프런트엔드를
변경하지 않는 필드 mutation 경로 정정이므로 승인된 수행계획의 비확대 원칙에 따라 추가하지 않았습니다.
WASM 전용 cfg는 WASM32 lib Clippy로 확인했습니다.

검증한 source/test candidate는 `7f1174f1d59bc020aaa38ceb7e148a8ae77b2784`, 최신 기준은
`upstream/devel@900b56edcaff3c1f84567c3f7c9e398a0dd9e8bb`입니다. nextest 0.9.137이 저장소 권고
0.9.140보다 낮고 `junit.report-skipped` 키를 인식하지 못한다는 기존 환경 경고가 있었으나,
버전 검사 우회 없이 전 테스트가 종료 코드 0으로 완료됐습니다.

## 성능 영향 및 측정 결과

- 예상 영향: 필드 mutation 때 대상 문단 reflow와 vpos 재연결 비용이 추가됩니다. 읽기·일반 렌더
  경로에는 추가되지 않습니다.
- 동일 호스트 단일 wall-clock 기초값: Gym 세 축 합계 2,320.59초 → 2,280초(-1.75%).
- host cache·부하와 함께 최신 devel 변경도 포함된 값이므로 microbenchmark나 성능 보장은 아닙니다.

## 스크린샷

해당 없음. 저장 왕복 `LineSeg`·vpos·필드 metadata와 CLI verify 계약으로 판정했습니다.
```

## 게시 경계

- base: `edwardkim/rhwp:devel`
- head: `edwardkim/rhwp:task_m100_6641`
- 검증 source/test candidate: `7f1174f1d59bc020aaa38ceb7e148a8ae77b2784`
- 직전 validation-evidence commit: `a233f1f6e`
- 원격 push, Open PR 생성·게시, self-review, merge, issue close는 각각 별도 승인 게이트다.
