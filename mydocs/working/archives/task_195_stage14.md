# Task #195 단계 14 완료보고서 — shape_layout 통합 + 회귀 + 최종 보고서

## 수행 내용

`src/renderer/layout/shape_layout.rs`의 Ole arm에 EMF 폴백 경로를 연결하여, OOXMLChartContents가 없고 EMF 프리뷰 바이트만 있는 OLE 객체도 네이티브 SVG로 렌더되도록 했다. 전체 회귀 확인 후 최종 보고서를 갱신했다.

## 수정 파일

| 파일 | 변경 |
|------|------|
| `src/renderer/layout/shape_layout.rs` | Ole arm: OOXML 시도 실패 시 `container.preview_emf` → `emf::convert_to_svg(...)` 호출 → 성공 시 `RawSvg` 노드 삽입, 실패 시 기존 placeholder 유지 |
| `mydocs/working/task_195_report.md` | 최종 보고서에 3차 스코프(단계 9~14) 섹션 추가 |
| `mydocs/orders/20260419.md` | #195 상태 "구현 완료 / 14단계", 커밋 이력 확장, 후속 이슈 5건 등록 |

## 렌더 분기 체계 (최종)

```
ShapeObject::Ole(ole) =>
  BinData 조회 + OLE 컨테이너 파싱
  ├── OOXMLChartContents 존재 → OOXML 파서 → 네이티브 차트 SVG (1순위)
  ├── preview_emf 존재       → emf::convert_to_svg → 네이티브 EMF SVG (2순위)
  └── 둘 다 없음 or 변환 실패 → 회색 placeholder + 라벨 (3순위)
```

## 검증 결과

```
$ cargo build --release       → OK
$ cargo test --release --lib  → 915 passed; 0 failed; 1 ignored
$ for f in samples/*.hwp samples/*.hwpx; do
    ./target/release/rhwp export-svg "$f" -o /tmp/rhwp-svg-check/
  done
  → 전체 샘플 크래시·에러 없음
```

### 1.hwp 수동 검증

- 페이지 3, 4의 OLE 차트 2개: **OOXML 네이티브 SVG 렌더 유지** (1순위 경로)
- EMF 폴백 경로는 **OOXML 부재 시에만 활성화**되므로 본 파일에서는 트리거되지 않음
- 회귀 확인: 모든 페이지 정상 출력, 크래시·빈 사각형 없음

### EMF 폴백 경로 트리거 조건

1차 구현 완성도를 검증하려면 다음 조건의 샘플 필요 (후속 수집):
- OLE 객체가 포함되고, 내부 CFB에 `OOXMLChartContents`는 없고 `\x02OlePres000`만 있는 HWP
- 예: 워드/엑셀/Visio 임베딩, 수식, 단순 도형 그림

## 전체 통계 (단계 1~14 총합)

| 항목 | 값 |
|------|-----|
| 총 커밋 | 14 (단계별 커밋 + 스코프 확장 2회) |
| 신규 파일 | 약 40개 (src/emf/ 15, ooxml_chart 3, bin_data/ole_container 2, tech 4, working 15+) |
| 수정 파일 | model/shape.rs, parser/control/shape.rs, renderer/layout/shape_layout.rs, renderer/svg.rs 등 |
| 단위 테스트 | 단계 5 시점 878 → 단계 14 시점 915 (+37) |
| EMF 단위 테스트 | 25건 (단계 10: 6, 11: 7, 12: 7, 13: 5) |
| 지원 EMF 레코드 | 38개 분기 (제어/객체/상태/드로잉/패스/텍스트/비트맵) |
| 외부 crate 추가 | 0 (기존 base64 0.22, cfb, flate2, quick-xml 재사용) |

## 다음 단계 (작업지시자 결정 필요)

1. **승인 + devel 머지**: `local/task195` → `local/devel` (local merge) → `local/devel` → `origin/devel` (push) — 이전 대화에서 push는 보류되어 있음
2. **GitHub Issue #195 close**
3. **후속 이슈 5건 등록** — EMF+ / WorldTransform 개별 적용 / 텍스트 회전 / 한글 폰트 폴백 / EMF-only 샘플 수집
