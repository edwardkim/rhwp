# PR #2721: 도형·그림 hp:sz widthRelTo/heightRelTo/protect 하드코딩 제거 (#2712)

이슈 #2697 잔여 — 도형(shape.rs)과 그림(picture.rs)의 write_sz가 widthRelTo/heightRelTo/protect를 리터럴("ABSOLUTE"/"0")로 방출하던 것을 IR 값에서 유도하도록 수정. size_criterion_width_str/height_str 헬퍼를 shape.rs에 pub(crate)로 추가하고 picture.rs에서 재사용.

## 검증
- hwpx 테스트 471/471 통과
