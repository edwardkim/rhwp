# 수행 계획서: Task #240 — BMP 임베딩 이미지의 SVG 렌더링 호환성 확보

- 이슈: edwardkim/rhwp#240 (유형: 버그, 마일스톤 없음)
- 브랜치: `local/task240` (← `local/devel` ← `devel`)
- 작성일: 2026-04-22

## 배경

`bitmap.hwp`, `한셀OLE.hwp` 등 BMP 이미지(또는 OLE 객체의 BMP 미리보기)를 포함한 HWP 문서를 `rhwp export-svg`로 변환하면, SVG 내부에 `<image xlink:href="data:image/bmp;base64,...">` 형태로 BMP 원본이 그대로 임베딩된다. 주요 브라우저는 SVG `<image>` 요소 내부의 `data:image/bmp` URI를 표준 지원하지 않아 이미지 영역이 빈 공백으로 보인다.

## 목표

SVG 내보내기 경로에서 BMP 이미지 데이터를 PNG로 재인코딩하여 `data:image/png;base64,...`로 임베딩함으로써, 브라우저 호환성을 확보한다.

## 범위

### 포함
- `src/renderer/svg.rs` 의 SVG 임베딩 경로 (HWP 본문 이미지)
- `bitmap.hwp`, `한셀OLE.hwp` 두 샘플의 실제 렌더 검증

### 제외 (후속 과제로 분리 가능)
- WMF/EMF, TIFF 등 BMP 외 포맷 호환성 (현 작업에서는 미변경, BMP만 다룸)
- HWPX 직렬화 경로 (BinData 저장 규격에 영향 주면 안 되므로 수정 제외)
- web_canvas/wasm_api 등 다른 렌더 경로 (SVG 내보내기만 대상)
- EMF 변환기가 생성하는 DIB→BMP data URL (이는 SVG 내부가 아닌 `<img>` 컨텍스트에서 쓰일 수 있어 별건)

## 산출물

1. 코드 변경
   - BMP→PNG 변환 유틸 (신규 내부 함수, `src/renderer/svg.rs` 또는 `src/renderer/image_util.rs`)
   - SVG data URI 생성 경로에 BMP 감지 시 PNG 변환 호출
2. 의존성 추가
   - `image = { version = "0.25", default-features = false, features = ["bmp", "png"] }`
3. 단위 테스트
   - 최소 BMP(BI_RGB 32-bit)를 PNG로 변환하여 PNG 시그니처(`89 50 4E 47`)로 시작하는지 확인
4. 샘플 검증
   - `samples/bitmap.hwp`, `samples/한셀OLE.hwp` → SVG 재생성 후 `<image>` 의 mime이 `image/png`임을 확인
5. 문서
   - 구현 계획서 (`task_m100_240_impl.md`)
   - 단계별 완료 보고서 (`task_m100_240_stage{1,2,3}.md`)
   - 최종 보고서 (`task_m100_240_report.md`)
   - `mydocs/orders/20260422.md` 타스크 상태 갱신

## 검증 방법

- `cargo test` 단위 테스트 통과
- `cargo run --bin rhwp -- export-svg samples/bitmap.hwp` 및 `samples/한셀OLE.hwp` 재실행
- 결과 SVG 내 BMP data URI(`data:image/bmp`)가 모두 사라지고 `data:image/png`로 바뀌었는지 grep 확인
- (수동) 생성된 SVG를 브라우저에서 열어 이미지가 표시되는지 육안 확인 — 작업지시자 검수 대상

## 위험/영향

- `image` crate 추가로 바이너리/WASM 크기 증가 가능 → `default-features = false`로 최소 feature(`bmp`, `png`)만 활성화해 완화
- BMP 디코드 실패 시 원본 BMP 유지(폴백)하므로 기능 회귀 없음
- 출력 SVG 크기는 일반적으로 감소 (32-bit BI_RGB BMP → 압축 PNG)

## 일정 (3단계)

1. Stage 1 — 의존성 추가 + BMP→PNG 변환 유틸 + 단위 테스트
2. Stage 2 — SVG 임베딩 경로에 변환 적용 + 기존 테스트 회귀 확인
3. Stage 3 — 샘플 두 건 재검증 + 최종 보고서 + local/devel 병합
