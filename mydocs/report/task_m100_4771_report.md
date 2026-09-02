# Task M100 #4771 최종 보고서 — 원본 IR과 파생 조판 상태 분리

- Issue: #4771
- 기준: `upstream/devel@900b56edcaff3c1f84567c3f7c9e398a0dd9e8bb`
- 브랜치: `ir/derived-layout-state`

## 구현 결과

1. `Paragraph::serializable_line_segs()`를 HWP5/HWPX 공통 source view로 만들었다. template·cell
   merge·문단 split/merge·memo root도 vector와 suffix/source-vpos provenance를 한 lifecycle로 다룬다.
2. #2004 stack admission은 같은 reference frame에서 alignment를 적용한 실제 2D bounds가 양수로
   겹칠 때만 허용한다. 기존 fixture는 page별 그림 ID와 최종 bbox까지 고정했다.
3. `Table.dirty`, source paragraph overflow memo, `Table.text_reflowed_after_edit`를 제거했다. 측정
   invalidation은 외곽 문단 revision이, overflow 판정은 renderer session cache가, text-reflow frame
   provenance는 live `Box<Table>` identity projection이 소유한다.
4. `Table.local_resize_*`와 minority morphology 추론을 제거했다. HWP/HWPX가 보존하지 못하는
   `localResize:true`는 JSON whitespace와 무관하게 mutation 전에 거부하고 Studio도 같은 이유를
   사용자에게 표시한다. 일반 공유-grid resize와 저장 cell geometry는 유지한다.
5. `HwpExportSnapshot`이 lowering·DocInfo reseal의 단일 작업 표현이다. 일반/report/password/verify
   저장과 CLI 검증은 같은 snapshot을 사용하며 live `Document`는 바뀌지 않는다.

## 검증 결과

- `tests/cases/issue_4771_derived_layout_state.rs`: 6/6 통과
- 전체 release-test: 8,968/8,968 통과, 46 skip
- 필수 fmt/native Clippy/WASM Clippy/workspace build/all-target Clippy: 통과
- Rust suite manifest 및 unit-tier policy: 통과
- Native Skia lib 3,959건 실행 및 PNG/PDF focused suite: 통과
- WASM native wrapper `--no-opt`: 통과 (Docker daemon 부재로 매뉴얼 fallback 사용)
- Studio production build: 통과
- Studio Node 22 suite: 1,347 통과, 1 skip
- #2004 HWP/HWPX OVR: 각 8→8쪽, 10→10개체, 허용오차 ±2px에서 회귀 0건
- 변경 문서 10건 상대 링크 검사: 통과
- 전체 문서 metadata 검사는 변경 밖 기존 누락 16건만 보고했다.
- Gestell code review 및 boundary review: PASS
