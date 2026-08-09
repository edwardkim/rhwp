# 완료 보고서 — #4224 U+F02FB 작은 오른쪽 방향 삼각형

- **Issue**: #4224
- **대상**: 한컴 문자표 `U+F02FB` 일반 `TextRun` tofu
- **브랜치**: `task_m100_4224_pua_f02fb_small_triangle`
- **stack 기준**: `task_m100_4158_char_overlap_boxed_pua` `27932685b`
- **devel 기준**: `upstream/devel` `5a4f26d0d`
- **구현 커밋**: `3f0974dc8`
- **전체 게이트 후보**: `5f6569062`
- **계획서**: [`mydocs/plans/task_m100_4224.md`](../plans/task_m100_4224.md)
- **작업 기록**: [`mydocs/working/task_m100_4224_stage1.md`](../working/task_m100_4224_stage1.md)

## 결과

검증된 한컴 PUA 표시 표에 `U+F02FB → U+25B8(▸)`를 추가했다. 원문 IR은 보존하면서
Canvas2D·SVG·Native Skia와 텍스트 추출 표면이 공개 글꼴에서도 작은 오른쪽 방향 삼각형을
결정적으로 사용한다. 인접 `U+F02FC → ►`와 다른 PUA 동작은 변경하지 않았다.

실제 `pau-004.hwp` Rust·SVG 2건, 검증 표 1건, 인접 PUA 13건, Native Skia feature 2건,
Clippy·fmt·diff, release WASM, Canvas2D 6개 계약과 Native Skia PNG 출력을 통과했다. #4158 head
위로 재배치한 뒤 네이티브 `rhwp`와 WASM을 다시 만들었고, 동일 산출물에서 #4158 사각 번호 7개와
삼각형 6개 Canvas2D 계약, E2E manifest 88/88을 통과했다. 시각 증적은 `output/4158/`과
`output/pau-004/`에 있다. 작업지시자는 결합 WASM의 사각 번호와 작은 오른쪽 방향 삼각형에 대한
rhwp-studio 시각 판정을 통과시켰다.

승인된 전체 PR 게이트도 완료했다. IR sweep은 815개 샘플·112,314건에서 기존 baseline과 동일했고,
overflow-cell sweep은 감소한 값만 현재 결과로 엄격하게 낮춘 뒤 676개 샘플·1,849줄과 일치했다.
release library 3,307건, 전체 release-test integration suite, Native Skia 58+2+4건,
Clippy·fmt·diff·rustdoc, Studio 타입 검사와 단위 테스트 802건, release WASM을 모두 통과했다.

새 WASM에서 E2E manifest 88/88, #4158 7개 계약, 삼각형 6개 계약을 재확인했다. OVR5는
`devel@5a4f26d0d` 대비 5문서·142쪽·11개체에서 geometry 회귀 0건이다.

중복 검색 뒤 GitHub 이슈 #4224를 등록하고 문서·Rust·E2E 식별자를 이슈 번호로 정규화했다. 새
이름의 focused Rust 2건, E2E manifest 88/88, Canvas2D 6개 계약, 변경 Markdown 523개 링크 검사가
통과했다. branch push와 PR 생성은 아직 수행하지 않았다.
