# Task #1116 Stage 15 보고서 — 원본 HWP3 3mm 격자 흐름 반영

- 이슈: [edwardkim/rhwp#1116](https://github.com/edwardkim/rhwp/issues/1116)
- 브랜치: `local/task1116`
- 작성일: 2026-05-25
- 상태: 원본 HWP3 및 HWP3-origin HWP5 p3 세로 흐름 보정 완료

## 1. 추가 지시

작업지시자가 한컴오피스의 원본 HWP3 3mm 격자 화면을 추가 제공했다.

Stage 14는 HWP3-origin HWP5 변환본의 `spacing_before`만 본문 흐름에서 복원했다. 그러나 원본 HWP3도 같은 3mm 격자 흐름을 따라야 하므로, HWP3 원본과 HWP5 변환본을 같은 본문 흐름 spacing 보정 경로에 태워야 한다.

## 2. 수정

수정 파일:

```text
src/document_core/queries/rendering.rs
src/renderer/height_measurer.rs
src/renderer/layout.rs
src/renderer/layout/paragraph_layout.rs
tests/issue_1116.rs
```

변경:

1. 기존 `is_hwp3_variant`는 HWP3-origin HWP5 변환본 전용 의미로 유지했다.
2. 별도 플래그 `use_hwp3_origin_flow_spacing_before`를 추가했다.
3. 이 플래그는 `document.is_hwp3_variant || document.header.version.major == 3`일 때 켜진다.
4. 높이 측정과 실제 paragraph layout의 `spacing_before` 흐름에만 적용한다.
5. 원본 HWP3 p3 heading y 좌표 테스트를 추가했다.

## 3. 좌표 확인

원본 HWP3 `samples/hwp3-sample16.hwp`를 다음 명령으로 재생성했다.

```text
target/debug/rhwp export-svg samples/hwp3-sample16.hwp \
  -o output/poc/render-spacing/stage15-hwp3-sample16-p3-after-flow-spacing \
  -p 2 \
  --show-grid=3mm
```

확인된 주요 y 좌표:

| 항목 | RHWP 수정 후 |
| --- | ---: |
| `Ⅰ. 사업개요` | 98.3px |
| `1.추진목적` | 147.8px |
| `2. 추진방향` | 337.2px |
| `3. 주요 추진내용` | 749.2px |
| 마지막 `□ 활용가능...` | 975.8px |

## 4. 검증

완료:

1. `cargo test --test issue_1116 -- --nocapture`
2. `cargo test --test issue_1035_alignment -- --nocapture`
3. `cargo test --test issue_1086 -- --nocapture`
4. `cargo fmt --all -- --check`
5. `cargo build --bin rhwp`
6. `git diff --check`
