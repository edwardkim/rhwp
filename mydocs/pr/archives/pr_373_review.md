# PR #373 검토 — Task #372: SVG 렌더러 형광펜 배경(charPr.shadeColor) 누락 수정

## PR 정보

| 항목 | 값 |
|------|-----|
| PR 번호 | [#373](https://github.com/edwardkim/rhwp/pull/373) |
| 작성자 | [@planet6897](https://github.com/planet6897) (Jaeuk Ryu) |
| base / head | `devel` ← `local/task372` |
| state | OPEN |
| mergeable | MERGEABLE |
| mergeStateStatus | **BEHIND** (devel 보다 뒤, rebase 필요) |
| 이슈 | [#372](https://github.com/edwardkim/rhwp/issues/372) |
| 변경 통계 | +49 / -0, 3 files |

## 결함 요약

`samples/hwpx/hwpx-h-02.hwpx` 9쪽 PDF 의 민트색(`#CDF2E4`) 형광펜 배경이 SVG 출력에서 누락.

원인: HWPX → IR → `TextRunStyle.shade_color` 까지 정상 보존되지만 `src/renderer/svg.rs::draw_text` 가 해당 필드 미사용.

## 변경 내용

`src/renderer/svg.rs::draw_text` 의 클러스터 분할 직후에 `<rect>` 배경 출력 추가:

```rust
let shade_rgb = style.shade_color & 0x00FFFFFF;
if shade_rgb != 0x00FFFFFF && shade_rgb != 0 {
    let text_width = *char_positions.last().unwrap_or(&0.0);
    if text_width > 0.0 {
        self.output.push_str(&format!(
            "<rect x=\"{:.4}\" y=\"{:.4}\" width=\"{:.4}\" height=\"{:.4}\" fill=\"{}\"/>\n",
            x, y - font_size, text_width, font_size * 1.2,
            color_to_svg(style.shade_color),
        ));
    }
}
```

| 항목 | 값 |
|------|-----|
| 변경 파일 | `src/renderer/svg.rs` (+13 줄), `mydocs/plans/task_m100_372.md`, `samples/hwpx/hwpx-h-02.pdf` |
| 가드 | `shade_rgb != 0x00FFFFFF && shade_rgb != 0` (흰색/미설정 제외) |
| 좌표 시멘틱 | `y - font_size` 시작, 높이 `font_size * 1.2` (web_canvas.rs 와 동일) |

## 변경 평가

### 강점
1. **기존 검증 패턴 이식**: `html.rs:334-337` 와 `web_canvas.rs:1271-1279` 가 이미 동일 패턴 사용. 새 시멘틱 도입 없음
2. **변경 범위 매우 작음**: svg.rs 단일 함수에 13 줄 추가, 다른 경로 (회전 / char_overlap) 미변경
3. **가드 정확**: 흰색(`0xFFFFFF`) / 미설정(`0`) 제외 → 기존 문서 영향 없음
4. **<rect> 가 <text> 앞에 출력**: 텍스트가 배경 위에 보임 (z-order 정확)
5. **PDF 와 시각 일치 검증**: 12개 강조구 모두 민트 배경 정상 표시 확인
6. **테스트**: cargo test 1055 ok / 0 failed (회귀 없음)

### 약점 / 점검 필요
1. **mergeStateStatus = BEHIND** — devel rebase 필요 (PR #366, #371 머지 후 1014 → 1015~ 로 변동)
2. **회전 텍스트 / char_overlap 경로 미적용** — PR 본문에 명시적 비범위 (web_canvas 도 일반 경로에만 적용). 향후 별도 task 후보일 뿐 본 PR 의 결함은 아님
3. **PDF 파일이 git 에 추가됨** (`samples/hwpx/hwpx-h-02.pdf`) — 검증 정답 PDF. 다만 다른 PDF 들도 이미 samples 에 있음 (예: `samples/exam_eng.pdf`) → 정상 패턴

### 메인테이너 동시 정정 여부

검색 결과 — 본 결함의 메인테이너 동시 정정 없음. PR #373 단독 정정.

## 처리 방향

### 옵션 A: 정상 머지 (cherry-pick)

근거:
- 변경 범위 매우 작고 명확 (13 줄)
- 기존 검증 패턴 이식 (html, web_canvas 와 일관)
- 결함 진단 + 시각 검증 정확
- 작성자 절차 정상 (이슈 #372 + 계획서 + PR)
- 메인테이너 동시 정정 없음

### 옵션 B: 정상 머지 (rebase + force-push)

작성자가 같은 저장소 브랜치를 사용 (외부 fork 아님). 그러나 메인테이너가 PR 브랜치를 직접 push 할 수 없을 수 있음.

## 권장

**옵션 A** — cherry-pick 방식 (PR #371 처럼). 이유:
- BEHIND 상태 자동 해결
- 작성자 attribution 보존
- devel 의 최신 변경 (PR #366, #371) 위에 깔끔히 적용

## 충돌 사전 분석

PR #373 의 변경 파일 3개:
- `src/renderer/svg.rs` — devel 에 PR #366, #371 의 svg.rs 변경 없음 → **자동 머지 예상**
- `mydocs/plans/task_m100_372.md` — devel 에 없음 → 충돌 없음
- `samples/hwpx/hwpx-h-02.pdf` — devel 에 없음 → 충돌 없음

`mydocs/orders/20260427.md` 의 변경은 PR #373 에 없음 (작성자가 orders 안 갱신했거나, BEHIND 라 base 시점에 #361/#362/#370 항목 없음). cherry-pick 시 orders 충돌 발생할 수도.

## 다음 단계 — 작업지시자 결정

옵션 A (cherry-pick) 진행 시:
1. 구현계획서 작성 + 승인
2. cherry-pick → 충돌 해결 (orders 가능성)
3. 검증 (cargo test, svg_snapshot, sample 회귀, hwpx-h-02 9쪽 SVG 출력)
4. devel merge + push
5. PR close + 이슈 close + 보고서

## 검토 항목 (Claude 점검 완료)

- [x] 결함 진단 정확성 (svg.rs 누락, html/web_canvas 동일 패턴 존재) ✅
- [x] 변경 범위 (단일 함수 13 줄) ✅
- [x] 가드 조건 정확 ✅
- [x] z-order (<rect> 가 <text> 앞) ✅
- [x] 메인테이너 동시 정정 없음 ✅
- [ ] cherry-pick 시 orders 충돌 가능성 — 진행 시 확인
- [ ] cargo test 회귀 0 — rebase 후 재검증

## 참고

- 이슈: [#372](https://github.com/edwardkim/rhwp/issues/372) (OPEN)
- PR: [#373](https://github.com/edwardkim/rhwp/pull/373) (OPEN, BEHIND)
- 동일 패턴 참조: `src/renderer/html.rs:334-337`, `src/renderer/web_canvas.rs:1271-1279`
