# Task #631 최종 결과 보고서

> **이슈**: [#631](https://github.com/edwardkim/rhwp/issues/631) — aift.hwp 18페이지 하단 1줄 다음 페이지로 밀림
> **브랜치**: `local/task631`
> **완료일**: 2026-05-06

---

## 요약

aift.hwp 우리 렌더러 page 18 의 `pi=222` 단락에서 PDF 정답은 2줄을 담지만 우리는 1줄만 담고
나머지를 page 19 로 밀던 회귀를 수정. `typeset.rs::typeset_paragraph` 의 줄 단위 분할 루프에 
**HWP 권위값 더블체크** 를 추가하여 한컴 엔진이 LINE_SEG vpos-reset 으로 명시한 페이지 경계를
존중하도록 함.

## 원인

`src/renderer/typeset.rs::typeset_paragraph` 의 partial split 에서
`LAYOUT_DRIFT_SAFETY_PX = 10` 이 두 군데에서 차감 (합계 20px 보수 마진):

- `typeset.rs:1046`: `base_available = base_available_height() - 10`
- `typeset.rs:1074`: `avail_for_lines = page_avail - sp_b - 10`

`pi=222` 진입 시점:
- `current_height = 912.2px`, `base_available_height = 971.4px`
- `page_avail = 961.4 - 912.2 = 49.2`, `avail_for_lines = 49.2 - 0 - 10 = 39.2`
- li=0: `0+16=16 ≤ 39.2` ✓ → end_line=1
- li=1: `25.6+16=41.6 > 39.2` ✗ → break (line 1 탈락)

20px 보수 마진이 본문 잔여 59.2px → 39.2px 로 축소시켜 2줄 합(41.6) 을 넣지 못함.

이는 Task #332 stage4b 커밋(`0211e574`) 에서 명시적으로 알려진 콘텐츠 손실 회귀 — 
"21_언어 pi=10 line 1, hwp-multi-002 pi=68, **aift pi=222**" — 가운데 마지막 미해결분이었다.

## 수정 내용

**`src/renderer/typeset.rs:1078-1098`** — 줄 단위 분할 루프 break 직전 HWP 권위값 우회:

```rust
for li in cursor_line..line_count {
    let content_h = fmt.line_heights[li];
    if cumulative + content_h > avail_for_lines && li > cursor_line {
        // [Task #631] HWP 권위값 더블체크
        let hwp_authoritative = para.line_segs.get(li + 1)
            .map(|next| next.vertical_pos == 0)
            .unwrap_or(false)
            && para.line_segs.get(li).map(|cur| {
                let bottom_px = crate::renderer::hwpunit_to_px(
                    cur.vertical_pos + cur.line_height, self.dpi);
                bottom_px <= st.base_available_height()
            }).unwrap_or(false);
        if !hwp_authoritative {
            break;
        }
    }
    cumulative += fmt.line_advance(li);
    end_line = li + 1;
}
```

조건 (모두 AND):
1. typeset 누적 추정으로 fit 실패 (기존 break 조건)
2. 다음 줄 `vpos == 0` — HWP 가 페이지 경계 신호 인코딩
3. 현재 줄 `vpos+lh ≤ body_available_height` — HWP 좌표상 본문 안

## 검증 결과

### 단위 테스트
- `cargo test --lib`: **1125 passed; 0 failed; 2 ignored**

### 광범위 회귀 검증 (155 샘플 / 1255 페이지)

byte-level SVG 비교 (HEAD~1 vs HEAD):

| 카테고리 | 샘플 | 효과 |
|----------|------|------|
| 의도된 수정 | aift, 20250130-hongbo(-no), hwp3-sample4/5 | page 끝 1~수 줄 페이지 내 복원 |
| 회복 — 페이지 통합 | loading-fail-01 | 17→16 페이지, drift 잉여 페이지 통합 |
| 회복 — 누락 콘텐츠 복구 | hwpctl_API_v2.4 | +2 페이지, **+460 text elements** |
| 회복 — 누락 콘텐츠 복구 | hwpspec | +1 페이지, **+855 text elements** |
| 변화 없음 (147개 샘플) | exam_kor/eng/math/science, synam-001, hwp-multi-002, 21_언어, 2010-01-06 등 | byte 동일 |

**부정적 회귀 0건. 1300+ 누락 콘텐츠 복구.**

### 타겟 케이스 검증

```
페이지 18 (수정 후):
  PartialParagraph  pi=222  lines=0..2  vpos=67980..0 [vpos-reset@line2]
페이지 19 (수정 후):
  PartialParagraph  pi=222  lines=2..4  vpos=0..1920 [vpos-reset@line2]
```

PDF 정답과 일치 (page 18 에 2줄, page 19 에 2줄).

## 영향 범위

- **Task #332 stage4b 의 알려진 회귀 종결**: aift pi=222 명시 해결
- **숨어 있던 동일 클래스 회귀 6건 회복**: 사용자가 인지 못했던 콘텐츠 손실 자동 복구
- 다른 포맷 영향 없음 (`Document` IR 공통이지만 HWP3/HWPX 도 LINE_SEG vpos 정보 제공)

## 산출물

- `mydocs/plans/task_m100_631.md` — 수행계획서
- `mydocs/plans/task_m100_631_impl.md` — 구현계획서
- `mydocs/working/task_m100_631_stage1.md` — 정밀 진단
- `mydocs/working/task_m100_631_stage2.md` — 구현 + 단일 샘플 검증
- `mydocs/working/task_m100_631_stage3.md` — 광범위 회귀 검증
- `mydocs/report/task_m100_631_report.md` (본 문서)
- `mydocs/orders/20260506.md` — 오늘할일

## 커밋 이력 (`local/task631`)

```
63a51d76 Task #631 Stage 3: 광범위 회귀 검증 — 155 샘플 / 1255 페이지 / 회귀 0건
ba12598f Task #631 Stage 2: HWP 권위값 더블체크 구현 — vpos-reset 인접 line 보존
a8d3b239 Task #631 Stage 2: 구현계획서 — LAYOUT_DRIFT_SAFETY_PX 이중 차감 + HWP 권위값 더블체크
a108061e Task #631 Stage 1: 정밀 진단 — pi=222 LINE_SEG vpos-reset 미활용 + 누적 drift 의심
```

## 후속 검토 권고

- 본 권위값 더블체크는 `vpos-reset` 신호가 인접한 경우만 적용. HWP 파일이 신호 없이 단순히 
  페이지 끝까지 채울 수 있는 케이스는 여전히 보수 마진(20px) 적용. 추가 회귀 발견 시 
  조건 완화 검토.
- `RHWP_USE_PAGINATOR=1` 경로의 `engine.rs::paginate_text_lines` 도 동일 마진을 가질 가능성. 
  본 task 범위 외 (현재 default 경로는 typeset.rs).
