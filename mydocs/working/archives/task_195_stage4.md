# Task #195 단계 4 완료보고서 — Renderer SVG Placeholder

> 구현계획서: [task_195_impl.md](../plans/task_195_impl.md)
> 단계: 4 / 5

## 작업 결과

### 수정 파일

- `src/renderer/layout/shape_layout.rs` — Chart/Ole 분기에서 placeholder 스타일 오버라이드

### 렌더링 규칙 (1차 범위)

| 타입 | 배경색 | 테두리 | 용도 |
|------|--------|--------|------|
| Chart (HWPTAG_CHART_DATA) | `#E8F0FE` (연한 파란) | `#4A90E2` 1px 대시 | 네이티브 HWP 차트 |
| Ole (SHAPE_COMPONENT_OLE) | `#F0F0F0` (연한 회색) | `#909090` 1px 대시 | OLE 개체 (MS Graph 등) |

- 기존 도형에 fill/stroke가 설정되어 있으면 그대로 유지 (placeholder는 `is_none()` 분기에만 적용)
- 범위 제외: 실제 차트 시리즈 렌더, OLE 프리뷰 이미지 추출 (별도 이슈)

## 1.hwp export-svg 검증

```
$ rhwp export-svg 1.hwp -o /tmp/task195_out -p 2 -p 3
$ grep -oE 'fill="[^"]+"' /tmp/task195_out/1_004.svg | sort -u
  fill="#000000"
  fill="#d9d9d9"
  fill="#ebf7fd"
  fill="#f0f0f0"    ← OLE placeholder (소문자 정규화됨, 입력 #F0F0F0)
  fill="#f2f2f2"
  fill="#ff0000"
  fill="#ffffff"
  fill="none"
```

- 8개의 `stroke-dasharray`가 SVG에 존재 → 점선 테두리 렌더링 확인
- OLE 2개(페이지 3, 4)가 각각 연한 회색 박스 + 점선 테두리로 표시됨 (기존: 빈 사각형)

## 테스트 결과

```
cargo build --release       # OK
cargo test --release --lib  # 878 passed; 0 failed; 1 ignored
```

## 설계 확정 사항

- **Rectangle 노드 재사용 유지**: SVG 출력은 기존 Rectangle 경로로 흐르므로 CSS/프린팅 호환성 문제 없음
- **placeholder 색은 hardcoded**: 추후 테마 연동 필요 시 별도 이슈
- **차트/OLE 내부 텍스트 라벨**은 이번 단계에서 제외. ShapeStyle 변경만으로 "빈 사각형"→"식별 가능한 placeholder"가 충족됨

## 미해결 이슈

- [ ] OLE 프리뷰 이미지 실제 추출 (BinData 스트림 압축 해제 + CFB 파싱)
- [ ] CHART_DATA 하위 태그 파싱 → 실제 시리즈 렌더
- [ ] placeholder 중앙 "차트"/"OLE" 텍스트 라벨 (텍스트 노드 오버레이)

**→ 이 3건은 별도 이슈 제안** (범위 과도로 Task #195 분리)

## 커밋 대상

- src/renderer/layout/shape_layout.rs
- mydocs/working/task_195_stage4.md

**커밋 메시지**: `Task #195: Renderer — Chart/Ole placeholder SVG (단계 4)`
