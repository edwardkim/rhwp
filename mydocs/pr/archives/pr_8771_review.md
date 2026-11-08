# PR #8771 self-review — 승격 검증의 샘플 전달과 본문 clipping 보정

## 판정과 범위

승인 — 제품 구현·로컬 검증·제품 CI 검토에서 추가 보정할 결함은 발견하지 않았다. 정확한 code head의 모든 필수 검사와 최종 정책 감사가 성공했다. 이 기록 뒤의 문서-only trailing head는 실제 검사·병합 가능 상태를 별도로 확인한다.

- PR: https://github.com/edwardkim/rhwp/pull/8771
- code head: `339f77f6f27fd15bae4db0b4201af3bd5e408eef`.
- 실제 검증 제품 source: `8118ccb14d092f767cd1caf1aaece4753a8e61d8`. 이후 차이는 증적·기록뿐이다.
- base: `80fc0c6b44bb270e5c14946cfb52e0eb0475a3a8`.
- 경로: collaborator_self_merge; 보조 절차 intake_and_review, local_validation, visual_fixture_evidence, review_only_fast_pass, post_merge.
- 작업지시자가 본문 clipping 보정·필수 검증과 이 후보의 push·Open PR·CI·self-review·머지를 승인했다. 정책 감사의 두 차례 재실행도 각각 별도로 승인했다.

## 코드 검토

1. `/samples`의 기존 저장소 파일 우선순위를 유지한다. 저장소 읽기의 `ENOENT`일 때만 public의 실제 regular file을 확인해 Vite로 넘긴다. 기존 상위 경로 차단을 통과한 같은 상대 경로를 사용하며, 없거나 디렉터리이면 404로 끝낸다. 실패를 SPA HTML로 바꾸지 않는다. 실제 HTTP 검사가 11개 입력 바이트·두 제공 경로·우선순위·404·403을 확인한다.
2. 원 해결 #6590 → #6591 → #7059와 #6269의 본문 잉크 경계를 대조했다. layout이 만든 `Body.clip_rect`는 유지하고 `body_clip_with_editor_marks`가 기존 비인쇄 표식 gutter만 적용한다. LayerBuilder는 같은 결과 뒤 기존 텍스트 그림자 합집합을, Legacy는 같은 결과로 Canvas `rect`·`clip`을 적용한다. Legacy의 비표식 4px 중복 확장을 제거했다.
3. 측정·줄 구성·표 좌표·pagination·continuation을 변경하지 않는다. 반환값 뒤의 Layer replay·Print device clip·Legacy overflow controls도 확인했다. 문서 ID나 우연한 속성에 따른 분기·좌표 clamp를 추가하지 않았다. 기존 48px는 편집 표식의 공통 계약이며 표 외곽을 맞추기 위한 새 상수가 아니다.
4. 기존 실제 Canvas 비교의 보정 전 FAIL → 보정 후 PASS와 #6590/#7059/#6269의 집중 회귀를 재사용했다. 구현의 산식을 반복하는 새 Rust 단위 검사를 추가하지 않았다. BookReview의 기존 raw 100% 독립 우단 ±1px 검사는 유효하며 기대값·fixture·시각 임계치를 변경하지 않았다.
5. Native와 기본 Layer의 전후 PNG가 같고, 측정·배치 경계 변경이 없어 편람 383쪽 조판 gate는 비해당이다. 공통 paint 경로의 Native 공식 3종·전체 회귀·fresh WASM·실제 브라우저 검증은 완료했다.

## 로컬 검증과 직접 판독

[정식 검증 원장](../../tech/investigations/release-0.9.0-renderer-sweep/README.md)에 입력·독립 Print·빌드/폰트 신원, 전쪽 TSV, 직접 판독 PNG와 명령·로그 해시를 보존했다.

- 최소 글꼴 환경에서 전체 회귀 12,997건·집중 9건·Native 공식 4,077건 통과. 기존 skip/ignored는 원장에 구분했다. fmt·Native/WASM Clippy·workspace build·all-target Clippy·suite manifest도 통과했다.
- Studio 2,778건 통과·기존 skip 2건, TypeScript·production build·최적화 WASM 성공.
- BookReview 독립 Print의 전 1쪽은 Native/fresh WASM 각각 raw 100%, 쪽수 일치. 동일 영역을 4배 확대해 오른쪽 괘선·행 경계·아래 모서리·글자·비인쇄 표식을 직접 대조했다. PDF와 전체 RGB 픽셀이 같다는 뜻은 아니다.
- 실제 Canvas 원 11개 모두 기존 0.05% 기준 통과. 원 사례의 두 Canvas 차이는 570 → 0픽셀이다. 표식 네 조합과 다시 숨김의 실제 clip도 같고 기존 48px gutter를 유지한다. 표식 glyph의 별도 19/25픽셀 차이는 clip과 구분해 기록했다.
- Native screen/print와 기본 Layer screen/WASM print의 변경 전후 4쌍은 바이트 단위로 같다. `lseg-05-tab.hwp`의 Canvas2D·CanvasKit 두 모드 screen/print 6개 대조군도 확인했다.

## GitHub 제품 CI

- [CI](https://github.com/edwardkim/rhwp/actions/runs/42442759346) 성공: 기본 feature Archive A–E 12,803건, 선택 Native 237건, Studio 2,778건·기존 skip 2건. 로컬 전체 feature/Native 공식 실행과 범위를 구분한다.
- [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/42442759254)·[Render Diff](https://github.com/edwardkim/rhwp/actions/runs/42442759209)·[Adapter](https://github.com/edwardkim/rhwp/actions/runs/42442759206)·[Proptest](https://github.com/edwardkim/rhwp/actions/runs/42442759169) 성공.
- Render Diff artifact는 실제 병합본 `425bcfbc3b82d611d73c8fc5049f6e63638ed765`의 최적화 WASM과 초기화 응답 18개가 일치한다. 필수 Canvas 3건·CanvasKit readiness 8건·browser parity 8건·필수 PDF 3건이 통과했다. 별도 PDF report-only 4건의 advisory 차이는 성공으로 바꾸어 표현하지 않는다.
- 최종 정책 감사 첫 실행은 CodeQL `Analyze (actions)`의 단계 목록을 GitHub API가 빈 배열로 반환해 `missing-step`으로 실패했다. 실제 로그에는 정상 skip이 있고 현재 API도 10개 단계를 반환한다. 첫 재실행은 CI·Render Diff의 늦게 갱신된 단계 때문에 pending이었다. 전체 API 단계가 정상화되고 동일 trusted-base 감사 코드의 읽기 전용 판정이 성공한 뒤, 별도로 승인된 추가 1회를 실행했다. [최종 감사](https://github.com/edwardkim/rhwp/actions/runs/42444774638) attempt 3과 원격 정책 status가 모두 success다. 제품 CI·코드·workflow·임계치를 다시 실행하거나 바꾸지 않았다.

## 남은 절차

최종 정책 감사 성공 뒤 이 기록·오늘할일을 문서-only trailing commit으로 추가했다. 최신 devel과의 실제 merge tree에서 링크·기존 기록 보존을 확인하고, trailing head의 실제 fast-pass와 required check 성공 뒤 일반 merge한다. 병합 뒤 최종 devel SHA를 고정해 이미 승인된 승격 13개 workflow를 실행한다. 릴리즈 범위는 현재 검증한 소스와 이 보정까지이며 이후 들어오는 PR을 임의로 추가하거나 검증 범위를 늘리지 않는다. main 승격·태그·공개 게시 완료로 표시하지 않는다. 사용 중인 릴리즈 output/target과 다른 작업은 보존한다.

[실제 CI 결과](../assets/pr8771/ci-results.json) · [실제 Render Diff 산출물 판독](../assets/pr8771/render-evidence.json) · [정책 감사 복구 근거](../assets/pr8771/policy-recovery.json).
