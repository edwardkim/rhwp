---
타스크: #259 한글 폰트명 → 메트릭 DB 영문명 매핑 누락
구현계획서
브랜치: local/task259
작성일: 2026-04-23
---

# 구현계획서

## 1. 파일 단위 변경

### 1.1 `src/renderer/font_metrics_data.rs`

#### 함수 `resolve_metric_alias(name: &str) -> &str` 확장

현재 24줄 match. HY 7건 + 본한글 계열 ~10건 추가.

**추가 매핑 (HY 계열)**:

```rust
// HY 계열 한글 정규명 → 메트릭 DB 영문명
// style_resolver.rs 가 한국어 별칭을 여기 정규명으로 먼저 변환한 뒤 find_metric 이 호출된다.
"HY중고딕" => "HYGothic-Medium",
"HY견고딕" => "HYGothic-Extra",
"HY헤드라인M" => "HYHeadLine-Medium",
"HY견명조" => "HYMyeongJo-Extra",
"HY신명조" => "HYSinMyeongJo-Medium",
"HY그래픽" => "HYGraphic-Medium",
"HY궁서" => "HYGungSo-Bold",
```

**추가 매핑 (본한글 계열 — Pretendard / Noto Serif KR 근사)**:

```rust
// Source Han Sans 계열 (본한글 · 본고딕) → Pretendard (한글 원천 동일, OFL 호환, 이미 번들)
// 본한글vf 의 임의 weight 도 Pretendard Regular/Bold 로 근사 (CJK 는 weight 별 한글 폭 차이 미미)
"본한글"
    | "본한글vf"
    | "본한글 Medium"
    | "본한글M"
    | "본고딕"
    | "본고딕vf"
    | "Source Han Sans"
    | "Source Han Sans K"
    | "Source Han Sans KR"
    | "SourceHanSans"
    | "SourceHanSansKR"
    | "SourceHanSansK"
    | "Noto Sans CJK KR"
    => "Pretendard",

// Source Han Serif 계열 (본명조) → Noto Serif KR (serif 원천 근사)
"본명조"
    | "본명조vf"
    | "본명조M"
    | "Source Han Serif"
    | "Source Han Serif K"
    | "Source Han Serif KR"
    | "SourceHanSerif"
    | "SourceHanSerifKR"
    | "SourceHanSerifK"
    | "Noto Serif CJK KR"
    => "Noto Serif KR",
```

**주석 블록 추가** — 매핑 정책 설명:

```rust
/// 한국어 폰트 이름 → 내장 메트릭 영문 이름 별칭.
///
/// 계층:
/// 1. style_resolver.rs 가 한국어 별칭 → 한국어 정규명 (예: 한양중고딕 → HY중고딕)
/// 2. 본 함수가 한국어 정규명 → 영문 DB 이름 (예: HY중고딕 → HYGothic-Medium)
/// 3. find_metric 이 FONT_METRICS 에서 영문 이름으로 조회
///
/// 본한글/본명조 는 정식 메트릭 DB 엔트리가 없어 Pretendard/Noto Serif KR 로
/// 근사. 근거: 같은 한글 원천, 이미 번들, OFL 호환. 한계: Latin 폭 미세 차이,
/// weight 축은 2단계로 근사 (본한글vf 는 wght 중간값을 Regular/Bold 중 가까운
/// 쪽으로). CJK 폰트는 weight 별 한글 폭 차이가 작으므로 실무 허용.
/// 정식 DB 엔트리 추가는 별도 이슈.
```

### 1.2 `src/renderer/font_metrics_data.rs` — 단위 테스트

`FONT_METRICS` 배열 밖 아무데나 `#[cfg(test)] mod tests { ... }` 추가.

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hy_gothic_medium_maps_correctly() {
        // "HY중고딕" → resolve → "HYGothic-Medium" → find_metric Some
        let m = find_metric("HY중고딕", false, false);
        assert!(m.is_some(), "HY중고딕 매핑 실패");
        assert_eq!(m.unwrap().metric.name, "HYGothic-Medium");
    }

    #[test]
    fn hy_family_all_map() {
        for (korean, expected_english) in &[
            ("HY중고딕",     "HYGothic-Medium"),
            ("HY견고딕",     "HYGothic-Extra"),
            ("HY헤드라인M",  "HYHeadLine-Medium"),
            ("HY견명조",     "HYMyeongJo-Extra"),
            ("HY신명조",     "HYSinMyeongJo-Medium"),
            ("HY그래픽",     "HYGraphic-Medium"),
            ("HY궁서",       "HYGungSo-Bold"),
        ] {
            let m = find_metric(korean, false, false);
            assert!(m.is_some(), "{} 매핑 실패", korean);
            assert_eq!(
                m.unwrap().metric.name, *expected_english,
                "{} 이 {} 에 매핑되지 않음", korean, expected_english
            );
        }
    }

    #[test]
    fn source_han_sans_family_maps_to_pretendard() {
        for name in &[
            "본한글", "본한글vf", "본한글 Medium", "본한글M",
            "본고딕", "본고딕vf",
            "Source Han Sans", "Source Han Sans K", "Source Han Sans KR",
            "SourceHanSans", "SourceHanSansKR", "SourceHanSansK",
            "Noto Sans CJK KR",
        ] {
            let m = find_metric(name, false, false);
            assert!(m.is_some(), "{} 매핑 실패 (Pretendard 기대)", name);
            assert_eq!(m.unwrap().metric.name, "Pretendard");
        }
    }

    #[test]
    fn source_han_serif_family_maps_to_noto_serif_kr() {
        for name in &[
            "본명조", "본명조vf", "본명조M",
            "Source Han Serif", "Source Han Serif K", "Source Han Serif KR",
            "SourceHanSerif", "SourceHanSerifKR", "SourceHanSerifK",
            "Noto Serif CJK KR",
        ] {
            let m = find_metric(name, false, false);
            assert!(m.is_some(), "{} 매핑 실패 (Noto Serif KR 기대)", name);
            assert_eq!(m.unwrap().metric.name, "Noto Serif KR");
        }
    }

    #[test]
    fn hy_family_bold_fallback() {
        // HY 계열은 bold 변형 DB 에 없을 수 있음 → bold_fallback: true 동작 확인
        let m = find_metric("HY중고딕", true, false);
        assert!(m.is_some());
        // bold 가 없으면 regular 로 fallback, bold_fallback=true
        // (HY 계열 bold 엔트리가 DB 에 있는지는 Stage 1 에서 확인)
    }

    #[test]
    fn non_korean_font_unchanged() {
        // 기존 매핑이 회귀 없음
        let m = find_metric("함초롬바탕", false, false);
        assert!(m.is_some(), "함초롬바탕 기존 매핑 회귀");
        assert_eq!(m.unwrap().metric.name, "HCR Batang");
    }
}
```

## 2. 단계별 실행

### Stage 1 — 매핑 테이블 확정 (실측)

#### 작업
1. FONT_METRICS 배열을 스크립트로 스캔 → HY / Source Han / Pretendard / Noto 관련 엔트리 전수 출력
2. HY 계열 bold 변형 존재 여부 확인 (있으면 bold 테스트도 정확 매칭, 없으면 bold_fallback)
3. 수행계획서 §4.1.A 의 7건 매핑이 DB 실체와 일치하는지 검증:
   - `HYGothic-Medium` · `HYGothic-Extra` · `HYHeadLine-Medium` · `HYMyeongJo-Extra` · `HYSinMyeongJo-Medium` · `HYGraphic-Medium` · `HYGungSo-Bold`
4. 불일치 시 Stage 2 진입 전 수행계획서 정정 + 재승인

#### 산출물
- `mydocs/working/task_m100_259_stage1.md` — 실측 결과 + 매핑 확정본

### Stage 2 — `resolve_metric_alias` 수정 + 단위 테스트

#### 작업
1. `src/renderer/font_metrics_data.rs` 의 `resolve_metric_alias` 에 Stage 1 확정 매핑 반영
2. 주석 블록 추가 (매핑 정책 · 근사 한계 설명)
3. 단위 테스트 6건 추가 (`mod tests`)
4. `cargo test --lib` 전체 그린 확인 (현재 947 → 신규 테스트 포함)
5. `cargo clippy --lib -- -D warnings` 그린 확인

#### 산출물
- `src/renderer/font_metrics_data.rs` 수정
- `mydocs/working/task_m100_259_stage2.md`

### Stage 3 — 회귀 검증 (text-align.hwp + svg_snapshot)

#### 작업
1. `cargo test --test svg_snapshot` 실행
   - form-002 / table-text golden 영향 여부 확인 (HY 중고딕 같은 폰트가 form-002 에 쓰이는지)
   - 영향 있으면 `UPDATE_GOLDEN=1` 재생성 후 결정성 재확인 (PR #221 / #251 재생성 교훈)
2. `text-align.hwp` CLI 렌더:
   ```bash
   ./target/release/rhwp export-svg samples/text-align.hwp -o output/svg/text-align-fix/
   ```
3. 4번 문단 (HY중고딕 지자체 열거) 글자 간격이 자연스러운지 **시각 확인**
4. 필요 시 작업지시자 웹 에디터 재확인 (WASM 재빌드 후)

#### 산출물
- `mydocs/working/task_m100_259_stage3.md`
- 필요 시 갱신된 `tests/golden_svg/*/page-0.svg`

### Stage 4 — 스모크 회귀

#### 작업
1. 다른 한글 폰트 사용 샘플 렌더 비교:
   - `samples/hwp/exam_kor.hwp` (25 페이지)
   - `samples/hwp/biz_plan.hwp` (6 페이지)
   - `samples/hwpx/hwpx-02.hwpx`
2. 렌더 결과 SVG 를 이전 커밋 (local/devel tip) 대비 diff 로 비교
   - 의도된 변화 (HY 폰트 폭 개선) 외에 의도치 않은 변화 없는지 확인
3. 본한글 또는 본명조 사용 샘플이 저장소에 있는지 확인 + 있으면 동일 검증

#### 산출물
- `mydocs/working/task_m100_259_stage4.md`

### Stage 5 — 문서화

#### 작업
1. 최종 결과 보고서 `mydocs/report/task_m100_259_report.md`
2. 오늘할일 `mydocs/orders/20260423.md` 에 "11. Task #259 …" 섹션 추가
3. 메모리 등록 (`~/.claude/.../memory/feedback_font_alias_sync.md`):
   > 한글 폰트 추가 시 `font_metrics_data.rs::resolve_metric_alias` 에도 매핑 동기화 필수. 누락 시 find_metric 이 None 반환 → 기본 폭 사용 → 글자 겹침.
4. MEMORY.md 인덱스 갱신
5. 매뉴얼: `mydocs/tech/font_fallback_strategy.md` 에 본한글 → Pretendard 근사 정책 기록

#### 산출물
- 최종 보고서 · orders 갱신 · 메모리 신규 1건 · MEMORY.md 갱신 · font_fallback_strategy 갱신

## 3. 커밋 단위 제안

Stage 별로 커밋:

1. `Task #259 Stage 1: HY/본한글 매핑 실측 결과` (문서만)
2. `Task #259 Stage 2: resolve_metric_alias 에 HY + 본한글 매핑 추가 (+ 단위 테스트 6건)`
3. `Task #259 Stage 3: text-align.hwp 회귀 검증 + svg_snapshot golden (필요 시 재생성)`
4. `Task #259 Stage 4: 스모크 스위프 (exam_kor / biz_plan / hwpx-02)`
5. `Task #259 Stage 5 + 최종 보고서: 메모리 + orders + font_fallback_strategy 갱신`

## 4. 위험 관리

| 위험 | 완화 |
|---|---|
| HY 영문 대응 추정 오류 | Stage 1 에서 확정, 오류 시 수행계획서 정정 후 재승인 |
| svg_snapshot golden 재생성 필요 | 지난 2회 (PR #221, #251) 경험으로 절차 확립됨 — UPDATE_GOLDEN + 결정성 재확인 |
| 본한글 Latin 폭 차이 | 근사 수용 (수행계획서 §5 에 기록됨) |
| 스모크 샘플 부족 | 저장소 샘플 한정해서 돌리고, 없는 폰트 계열은 추후 수집 대상으로 분리 |

## 5. 예상 소요

- Stage 1 (실측): 10분
- Stage 2 (코드 + 테스트): 15분
- Stage 3 (svg_snapshot 재생성 포함): 10분
- Stage 4 (스모크): 10분
- Stage 5 (문서): 20분
- **합계: ~65분**

## 6. 승인 요청

본 구현계획서 승인 시 Stage 1 착수.
