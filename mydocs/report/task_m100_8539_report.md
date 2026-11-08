# #8539 최종 보고 — 쪽을 넘는 RowBreak 표 칸의 한 줄 이른 분할

- 이슈: [#8539](https://github.com/edwardkim/rhwp/issues/8539) (#8198에서 분리)
- 브랜치: `fix/issue-8539-rowbreak-stored-restart-cut`, 코드 커밋 `1543ae62f`, base `upstream/devel` `7d06d5c70`
- 기록: [계획](../plans/task_m100_8539.md) · [2단계 원인](../working/task_m100_8539_stage2.md) ·
  [3단계 구현·코퍼스](../working/task_m100_8539_stage3.md) · [4단계 head 검증](../working/task_m100_8539_stage4.md)

## 결과

국민건강보험법 시행규칙 [별표 1] 부양요건(HWP 5.0.3.4)의 18×3 RowBreak 표는 두 쪽 경계 모두에서 한/글 2020 Print와
같은 위치에서 나뉜다.

| 경계 | devel | 수정 후 = 한/글 2020 |
|---|---|---|
| 1→2쪽 9행 | `[2,2,2]` | `[2,3,3]` |
| 2→3쪽 16행 | `[9,8,9]` | `[10,8,10]` |

전쪽 raw 실루엣(Native·fresh WASM 같음): devel 97.72 / 57.68 / 63.72 → 수정 후 99.90 / 100.00 / 100.00.

## 적용한 규칙

1. **선언 행 상자 안의 옛 저장 재시작**: 행의 첫 조각에서 선언 상자(병합 안 한 칸 저장 높이의 최댓값)가 이 쪽 남은
   공간에 통째로 들고, 셀 줄의 저장 vpos 0 재시작이 그 상자의 내용 높이 안쪽에 있으면, 그 재시작은 쪽 경계가 아니다.
   이때 다음 줄 수용은 용량 컷과 같은 끝 줄간격 제외로 잰다(공통 closure `trimmed_capacity_fit`).
2. **이어받은 조각의 저장 프레임 인증**: 이어받은 조각의 시작 컷이 저장 프레임 경계가 아니면
   (`row_start_owns_stored_frame` 거짓), 뒤 행의 저장 프레임 끝도 이 쪽의 끝으로 인증하지 않는다.

조건 없이 "끝 줄간격을 빼면 들어가면 저장 재시작을 흡수"하는 v1은 16418295·148776468·issue1853·issue1937에서
다음 쪽이 33~91%로 회귀했다. 이 문서들은 한/글이 저장 재시작을 지키는 사례이며, 선언 상자 조건으로 모두 devel과 같아졌다.

## 검증

- 회귀: `tests/cases/issue_8539_rowbreak_stored_restart_cut.rs` 3개. devel에서 9행으로 FAIL,
  규칙 1만 넣으면 16행으로 FAIL, 수정 후 PASS. 보존·겹침 검사 2개는 #8198 증상의 재발 방지용이다.
- 코퍼스: `samples/**` 1,424개 `dump-pages` 대조에서 컷이 바뀐 문서는 대상 1개, 쪽수 변화 0.
- Rust: fmt, Clippy 3종, workspace build, manifest, release-test 전체 nextest 12,418 passed / 0 failed,
  Native Skia 3종 통과(4단계).
- 시각: 커밋 head에서 Native(release-test Skia)·fresh WASM 전쪽 TSV, 일반 모드 gate `passed`, 1~3쪽 직접 판독.

## 남은 범위

- HWPX 저장본과 한/글 2024 Print에서의 같은 상황은 실물·기준 출력이 없어 미검증이다.
- 블록(rowspan) 컷 경로의 같은 모양 조건은 이번 표에서 소비되지 않고, 코퍼스에서도 차이가 나오지 않아 바꾸지 않았다.
- HY신명조가 없는 환경이라 NanumMyeongjo로 공급했다. 글꼴 예외 없이 최저 99.90%다.
