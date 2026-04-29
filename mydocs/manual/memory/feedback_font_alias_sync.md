---
name: 한글 폰트 추가 시 resolve_metric_alias 동기화 필수
description: 새 한글 폰트를 지원할 때 style_resolver 외에도 font_metrics_data의 resolve_metric_alias에 매핑을 등록하지 않으면 SVG 글자 겹침 발생
type: feedback
originSessionId: 67d1cb8f-86d4-4672-b831-a8d028a1cfcf
---
한글 폰트 이름 해석은 **2-계층**. 새 한글 폰트 추가 시 두 계층 모두에 매핑 등록 필수.

- **Layer 1 — `src/renderer/style_resolver.rs`**: 한국어 별칭 → 한국어 정규명 (예: 한양중고딕 → HY중고딕)
- **Layer 2 — `src/renderer/font_metrics_data.rs::resolve_metric_alias`**: 한국어 정규명 → 영문 DB 이름 (예: HY중고딕 → HYGothic-Medium)

**Why:** Layer 1만 구현하고 Layer 2를 누락하면 `find_metric()` 이 FONT_METRICS 에서 한국어명을 찾지 못해 None 반환 → 기본 폭 fallback → SVG 에서 영문/숫자 글자가 좁게 겹친다. Task #259 에서 HY 계열 7종 + 본한글/본명조 계열 전체가 이 증상을 보여 작업지시자가 발견.

**How to apply:**
1. 신규 한글 폰트 지원 PR/이슈 시 두 파일 모두 수정 여부를 체크리스트로 확인.
2. FONT_METRICS 배열에 영문 DB 이름이 존재하지 않으면:
   - (A) `extract_metrics` 파이프라인으로 TTF → FONT_METRICS 정식 엔트리 추가, 또는
   - (B) 기존 유사 폰트로 근사 (본한글 → Pretendard, 본명조 → Noto Serif KR — Task #259 선례). 근사 시 주석에 한계 명시.
3. `#[cfg(test)] mod tests` 에 매핑 검증 테스트 추가 (Task #259 의 hy_family_all_map 패턴).
4. 상세 매뉴얼: `mydocs/tech/font_fallback_strategy.md` 부록 A.

**핵심 증상**: SVG 에서 특정 폰트의 라틴/숫자 글자가 좁게 겹침. 폰트명을 grep 으로 확인하면 font-family 속성은 정상이나 x 좌표 간격이 기본값 (보통 ~7.67px). DB 엔트리 실측 폭 (예: HYGothic-Medium = 9.04px) 과 비교하면 차이 식별 가능.
