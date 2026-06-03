# Task #195 단계 9 완료보고서 — EMF 스펙 조사 + IR 설계

## 수행 내용

스코프 재확장(단계 9~14)의 첫 단계로, MS-EMF 스펙을 조사하고 rhwp 내부 구현(IR)을 확정했다. **코드 변경 없음**, 문서만 작성.

## 산출물

| 파일 | 역할 |
|------|------|
| `mydocs/tech/emf_spec.md` | MS-EMF 스펙 요약 (rhwp 1차 범위) — 파일 구조 / EMR_HEADER / 기본 구조체 / RecordType 카탈로그 / WMF와 차이 / 좌표 변환 파이프라인 / 제외 목록 |
| `mydocs/tech/emf_ir_design.md` | `src/emf/` 모듈 IR 설계 — 모듈 트리 / 공개 API / Record enum / DeviceContext / ObjectTable / Player / 좌표 변환 / SVG 출력 규약 / 테스트 전략 |
| `mydocs/working/task_195_stage9.md` | 본 보고서 |

## 주요 결정 사항

1. **WMF 모듈과 완전 분리** — `src/emf/` 독립, 코드 공유 금지. 이유: RecordType 완전히 다름, 좌표 크기(16bit→32bit) 차이, 패스/텍스트 레코드 구조 상이.
2. **1차 범위 고정** — GDI 기본 레코드(선/사각형/타원/패스/텍스트/비트맵), DC 스택, 객체 핸들 테이블.
3. **EMF+ / 고급 레코드 제외** — GradientFill, AlphaBlend, Clipping Region, ICM 색상 관리는 후속 이슈.
4. **좌표 매핑 단순화** — MapMode는 `MM_ANISOTROPIC`만 지원. WorldTransform은 SVG `<g transform="matrix(...)">` 로 직접 출력.
5. **에러 복구** — 미지 RecordType은 `Record::Unknown`으로 보존 후 스킵. 파싱 실패 시 상위 렌더에서 기존 placeholder로 폴백.

## 모듈 구조 (확정)

```
src/emf/
├── mod.rs                         공개 API
├── parser/
│   ├── mod.rs
│   ├── constants/                 RecordType, PenStyle, BrushStyle, MapMode, TextAlign
│   ├── objects/                   Header, RECTL/POINTL, XFORM, LogPen/Brush/FontW
│   └── records/
│       ├── header.rs              단계 10
│       ├── object.rs              단계 11
│       ├── state.rs               단계 11
│       ├── drawing.rs             단계 12
│       ├── path.rs                단계 12
│       ├── text.rs                단계 13
│       └── bitmap.rs              단계 13
└── converter/
    ├── mod.rs                     Player
    ├── device_context.rs          DeviceContext, DcStack, ObjectTable
    └── svg/                       SvgBuilder
```

## 공개 API (확정)

```rust
pub fn parse_emf(bytes: &[u8]) -> Result<Vec<Record>, Error>;
pub fn convert_to_svg(bytes: &[u8], render_rect: (f32, f32, f32, f32)) -> Result<String, Error>;
```

## 테스트 결과

- 해당 없음 (문서만)

## 미해결 이슈

- PNG 인코딩 crate 선정 — 단계 13 착수 시 기존 의존성 조사 후 결정
- EMF+ 레코드 감지 시 처리 방침(경고 vs 폴백 유지) — 단계 10 파서 구현 시 확정

## 다음 단계

**단계 10**: EMF 모듈 골격 + EMR_HEADER 파서 구현 + 단위 테스트.
