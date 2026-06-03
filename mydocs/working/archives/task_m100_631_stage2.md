# Task #631 Stage 2: 권위값 더블체크 구현 + 단일 샘플 검증

> **이슈**: [#631](https://github.com/edwardkim/rhwp/issues/631)
> **브랜치**: `local/task631`
> **작성일**: 2026-05-06

---

## 수정 내용

`src/renderer/typeset.rs::typeset_paragraph` 줄 단위 분할 루프(line 1079~1086)에 
HWP 권위값 더블체크 추가.

```rust
for li in cursor_line..line_count {
    let content_h = fmt.line_heights[li];
    if cumulative + content_h > avail_for_lines && li > cursor_line {
        // [Task #631] HWP 권위값 더블체크
        // 누적 추정으로는 fit 실패하지만 HWP 파일 자체가 다음 줄(li+1)에
        // vpos-reset(=0) 을 인코딩한 경우, 한컴 엔진이 직접 li 까지를 현재
        // 페이지에 배치한 것이다. typeset 보수 마진(20px) 으로 인한 콘텐츠
        // 손실을 차단하기 위해 HWP 신호를 우선한다.
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

### 동작 조건 (모두 AND)

1. typeset 누적 추정으로는 fit 실패 (기존 break 조건)
2. **다음 줄의 vpos == 0** — HWP 가 명시적으로 페이지 경계 신호를 인코딩
3. **현재 줄의 vpos+lh ≤ body_available_height** — HWP 좌표상 본문 안에 들어감

조건 2~3 이 모두 참일 때만 break 우회. 조건 2가 매우 좁아 회귀 위험 최소.

## 검증 결과

### 1. aift.hwp 18페이지 (목표 케이스)

**Before** (수정 전):
```
페이지 18 단 0 (items=15, used=937.8px, hwp_used≈922.4px, diff=+15.4px)
   ...
   PartialParagraph  pi=222  lines=0..1  vpos=67980..69900
페이지 19 단 0 (items=15, used=879.0px, hwp_used≈948.0px, diff=-69.0px)
   PartialParagraph  pi=222  lines=1..4  vpos=69900..1920 [vpos-reset@line2]
```

**After** (수정 후):
```
페이지 18 단 0 (items=15, used=963.4px, hwp_used≈948.0px, diff=+15.4px)
   ...
   PartialParagraph  pi=222  lines=0..2  vpos=67980..0 [vpos-reset@line2]
페이지 19 단 0 (items=15, used=853.4px, hwp_used≈948.0px, diff=-94.6px)
   PartialParagraph  pi=222  lines=2..4  vpos=0..1920 [vpos-reset@line2]
```

→ **page 18 = 2줄, page 19 = 2줄**. PDF 정답과 일치.

### 2. cargo test --lib

```
test result: ok. 1125 passed; 0 failed; 2 ignored
```

### 3. 주요 샘플 회귀 검사 (Task #332 주의 케이스)

| 샘플 | 페이지 수 | diff |
|------|----------|------|
| 21_언어_기출_편집가능본 | 15 | 0 |
| hwp-multi-002 | 6 | 0 |
| synam-001 | 35 | 0 |
| 2010-01-06 | 6 | 0 |
| exam_kor | 20 | 0 |
| exam_eng | 8 | 0 |
| exam_science | 4 | 0 |
| exam_math | 20 | 0 |
| aift | 77 | 2 (page 18, 19 — 의도된 수정) |

**의도된 변경 외 회귀 0건**.

## 검증 방법

```bash
# Before: stash + build + export
git stash
cargo build --release
./target/release/rhwp export-svg samples/{샘플}.hwp -o /tmp/svg_diff_before/{샘플}/

# After: pop + build + export
git stash pop
cargo build --release
./target/release/rhwp export-svg samples/{샘플}.hwp -o /tmp/svg_diff_after/{샘플}/

# byte-diff
for f in /tmp/svg_diff_before/{샘플}/*.svg; do
    cmp -s "$f" "/tmp/svg_diff_after/{샘플}/$(basename $f)" || echo "DIFF: $f"
done
```

## 다음 단계

Stage 3: 광범위 회귀 검증 (전체 샘플) → 최종 보고서.
