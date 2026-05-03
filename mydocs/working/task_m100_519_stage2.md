# Task #519 Stage 2: ImageNode.transform 채우기 — 완료 보고서

## 변경 내용

`ImageNode` 생성 시 `transform: extract_shape_transform(&pic.shape_attr)` 를 추가하여 HWP 의 Picture 변환 속성을 SVG 렌더 트리로 전달.

### 변경 파일 (4 파일, 6개 ImageNode 생성 지점)

| 파일 | 라인 | 비고 |
|------|------|------|
| `src/renderer/layout/picture_footnote.rs` | import + 107-117 + 316-326 | 메인 그림 렌더링 (앵커/floating) |
| `src/renderer/layout/paragraph_layout.rs` | import + 1787 + 2060 + 2169 | 문단 내 인라인 TAC 그림 (3 분기) |
| `src/renderer/layout/table_cell_content.rs` | import + 636 | 표 셀 내부 그림 |
| `src/renderer/layout.rs` | 2764 | 페이지 컨텐츠 직접 그림 (Task #347 경로) |

`utils::extract_shape_transform` 헬퍼 (기존, `shape_layout.rs:530` 의 다른 도형에서 이미 사용 중) 를 재사용 — 단위 변환 / 신규 코드 0 줄.

## 검증

### 컴파일

```
cargo build --release: Finished in 4m 22s (0 errors, 0 warnings)
cargo clippy --lib -- -D warnings: Finished in 2m 30s (0 errors)
```

`cargo clippy --all-targets` 의 46+2 errors 는 사전 존재 (test 파일의 `Box::new(_)` 등 stylistic) — `git stash` 후 확인 완료, 본 변경 무관.

### 회귀 — 단위 테스트

```
cargo test --lib: 1103 passed; 0 failed; 1 ignored
cargo test --test svg_snapshot: 6 passed; 0 failed
```

### 회귀 — exam_eng.hwp 8 페이지 SVG diff

| 페이지 | diff lines | 비고 |
|--------|-----------|------|
| 001 | 0 | 변화 없음 |
| 002 | 0 | 변화 없음 |
| 003 | 0 | 변화 없음 |
| 004 | 4 | `<g transform="...">` 래퍼 1개 추가 (28번 그림) |
| 005 | 0 | 변화 없음 |
| 006 | 0 | 변화 없음 |
| 007 | 0 | 변화 없음 |
| 008 | 0 | 변화 없음 |

페이지 4 에 추가된 transform:
```xml
<g transform="translate(1602.43,0) scale(-1,1) translate(0,2203.93) scale(1,-1)">
  <!-- 28번 그림 (BIN0002.jpg) -->
</g>
```

수평+수직 대칭 (180° 회전 효과). 다른 페이지·다른 그림은 일체 변화 없음 (Stage 1 분포 조사와 일치).

## 시각 비교

| | 위치 |
|---|------|
| `output/svg/task519_baseline/pdf_p28.png` | PDF (정답): curl 좌상단 명확 |
| `output/svg/task519_baseline/before_p28.png` | SVG-before: curl 우하단 (본문과 겹쳐 사실상 비가시) |
| `output/svg/task519_baseline/after_p28.png` | SVG-after: curl 좌상단 (PDF 와 일치) ★ |

## 완료 조건 충족

- [x] exam_eng p4 28번 curl 데코가 박스 좌상단에 표시 (PDF 와 시각 일치)
- [x] 빌드 0 error, lib clippy 0 warning
- [x] cargo test --lib + svg_snapshot 통과
- [x] 다른 페이지·다른 샘플 SVG diff 0

## 다음 단계

Stage 3 — 임시 도구(`scan_picture_transforms.rs`) 제거 + 최종 보고서.
