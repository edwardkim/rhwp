# Task #318 구현 계획서

상위: 수행계획서 `task_m100_318.md`
브랜치: `task318`

## 단계 구성

4단계 분할.

### 1단계 — A 수정 (`table_partial.rs` 가드)

목표: 분할 표 셀 수식 중복 emit 해소. z-table 4 값 중 0.1915/0.3413/0.4332 가 1회로 정상화되는지 확인 (0.4772 는 B 잔존 예상).

작업:
1. `src/renderer/layout/table_partial.rs:766` `Control::Equation` 분기에 `tree.get_inline_shape_position(section_index, cp_idx, ctrl_idx).is_some()` 가드 추가 (`#301` 의 `table_layout.rs:1610` 동일 패턴)
2. 가드 진입 시 `inline_x += eq_w` 로 인라인 너비만 진행, render 스킵
3. 4샘플 페이지 수 측정 + 골든 SVG 6건 무회귀 확인
4. issue_301 부분 통과 (3 of 4) 확인

산출:
- `src/renderer/layout/table_partial.rs` (수정)
- `mydocs/working/task_m100_318_stage1.md`

### 2단계 — B 정밀 진단

목표: pi=27 호스트 텍스트 중복 emit 경로 + 인라인 수식 ci 매핑 붕괴 root cause 식별. 코드 변경 없음.

작업:
1. SVG 출력에 `data-origin` 등 디버그 속성을 임시로 부착하여 각 `<text>`/`<g equation>` 노드가 어느 layout 함수에서 생성됐는지 추적 (env-gated)
2. exam_math 페이지 12 (index 11) 의 pi=27 호스트 텍스트 + ci=1..5 수식 emit 위치 매핑 도출
3. PartialParagraph PageItem 경로 (`layout.rs:1799` `layout_partial_paragraph`) vs wrap-around 경로 (`layout.rs:2522` 같은 함수 호출) 의 중복 호출 지점 확인
4. ci 매핑 붕괴 (두 위치 모두 0.4772 출력) 원인 파악

가설:
- (a) wrap-around 경로가 호스트 본문 모든 줄을 wrap area 에 그리는데, PartialParagraph 도 같은 줄을 그려서 두 번 출현
- (b) wrap-around 의 `compose_paragraph` / line 매핑이 인라인 수식을 모두 마지막 ci 의 수식으로 잘못 출력 (ci 인덱스 유실/혼선)

산출:
- `mydocs/working/task_m100_318_stage2.md`

### 3단계 — B 수정

목표: 한 경로만 호스트 텍스트를 그리도록 일원화 + ci 매핑 정상화.

작업:
- 2단계 진단 결과에 따라:
  - (a) 옵션: PartialParagraph PageItem emit 측에서 wrap-around host 가 본문 줄을 처리할 경우 자체 emit 스킵
  - (b) 옵션: wrap-around 측에서 호스트 본문 줄 emit 비활성화 후 PartialParagraph 만 처리
  - (c) 옵션: 두 경로 모두 같은 줄을 그리되 inline_shape_position 가드를 통해 수식 dedup
- 4샘플 페이지 수 + 골든 SVG 무회귀
- issue_301 4 값 모두 1/1/1/2 로 정상화 확인

산출:
- 코드 + `mydocs/working/task_m100_318_stage3.md`

### 4단계 — issue_301 재활성화 + 최종 보고

작업:
- `tests/issue_301.rs::z_table_equations_rendered_once` 의 `#[ignore]` 제거
- 진단 도구 (env-gated 디버그 속성 등) 회수
- `cargo test` 전체 통과
- 최종 보고서 + 오늘할일 갱신

산출:
- `mydocs/working/task_m100_318_stage4.md`
- `mydocs/report/task_m100_318_report.md`

## 회귀 검증 명령

```bash
cargo test
cargo test --test issue_301 -- --ignored  # 단계별 부분 통과 확인 시
cargo build --release
for f in samples/{21_언어_기출_편집가능본,exam_math,exam_kor,exam_eng}.hwp; do
  pages=$(./target/release/rhwp dump-pages "$f" 2>/dev/null | grep -c "^=== 페이지")
  echo "$(basename $f): $pages 쪽"
done
# 기대: 15 / 20 / 24 / 9

# 골든 SVG 무회귀
cargo test --test svg_snapshot
```

## 위험 / 롤백

| 위험 | 대응 |
|------|------|
| 2단계 진단이 길어짐 | 가설 (a)/(b) 를 binary search 로 빠르게 좁히기. 1.5일 초과 시 user 보고 |
| (b) 가설이 맞으면 wrap-around layout 깊이 수정 | wrap-around 측의 emit 비활성화 (c 옵션 선호) 로 minimal 수정 |
| 4샘플 다른 표 (TopAndBottom, 일반) 회귀 | 매 단계 4샘플 + 골든 SVG 재측정 |
| 다른 wrap=Square 표 회귀 | exam_kor / 21_언어 시각 확인 (wrap=Square 표 존재 여부 사전 조사) |

## 승인 요청

위 분할 승인 시 1단계 시작.
