# #7375 곡률·번호 도형 편집 검증 증적

현재 `ece9c3383b` 리베이스 뒤 결과는 [새 프로그램 검증](rebased-ece9/README.md)을 따른다.
아래 base `a95d1d5afa`·source `c72f967575`의 실행은 리베이스 전 이력이다. 초기 파일 접근
차단만의 Mac 글꼴 결과도 이력으로 보존하며, 새 최소 글꼴 수용 근거는 개인 Font Registry
미노출·실제 한국어 표시까지 확인한 새 결과와 구분한다.

입력은 `samples/issue7333/aaaaaa.hwp`, 독립 기준은 `pdf/issue7333/aaaaaa-2020.pdf`다.
원본 SHA는 검증 수치 (로컬 진단: `output/open-issues-oldest-20261105/pr-final-20261107/rebase-5077b216-20261108/temporary-asset-json/mydocs/pr/assets/issue_7375/verification-20261105/visual-validation.json`)에 고정했다. 기준 PDF를 rhwp 출력으로 대체하지 않았다.

- base: `a95d1d5afad1bfd61998794fa78fb0db100d690a`.
- 제품 보정 source: `c72f967575c49b068b9217830670abddc3785712`.
- Native Skia: 50/50쪽, 최저 98.31507%, 90% 미만 0쪽. 고정 base의 50쪽 PNG와 모두 바이트가 같다.
- fresh WASM Canvas2D: 50/50쪽, 최저 98.32714%, 90% 미만 0쪽.
- 같은 96dpi·compat2022·print 프로필, 정본 2px 실루엣 비교와 PDFium 물리 DPI 정책을 사용했다.
- WASM은 Mac locked wrapper의 `--target web --out-dir pkg --no-opt` 로컬 대체 빌드다.
  Docker 최적화 빌드 결과가 아니다.

[base Native TSV](base-native.tsv) · [후보 Native TSV](candidate-native.tsv) ·
[후보 WASM TSV](candidate-wasm.tsv) · Native base 대조 (로컬 진단: `output/open-issues-oldest-20261105/pr-final-20261107/rebase-5077b216-20261108/temporary-asset-json/mydocs/pr/assets/issue_7375/verification-20261105/native-base-comparison.json`).
TSV·PNG 표시는 0.01%p 단위이며 반올림 전 수치는 `visual-validation.json`에 기록했다.

| 쪽 | Native raw / review / overlay | fresh WASM raw / review / overlay | 독립 PDF raster |
| --- | --- | --- | --- |
| 27 | [raw](native-p027-raw.png) · [review](native-p027-review.png) · [overlay](native-p027-overlay.png) | [raw](wasm-p027-raw.png) · [review](wasm-p027-review.png) · [overlay](wasm-p027-overlay.png) | [raw](pdf-p027-raw.png) |
| 28 | [raw](native-p028-raw.png) · [review](native-p028-review.png) · [overlay](native-p028-overlay.png) | [raw](wasm-p028-raw.png) · [review](wasm-p028-review.png) · [overlay](wasm-p028-overlay.png) | [raw](pdf-p028-raw.png) |

번호 원의 외곽선·스크린샷 위 소속·강조 상자·후속 본문을 직접 대조했다.
공통 RectangleNode의 반지름은 원본의 50%에서 짧은 변의 절반이며, 0·20·37·50%의
HWP/HWPX 저장·재열기와 같은 기하 소비는 실제 Native API 검사로 보호한다.
Native SVG·직접 PDF 검증 (로컬 진단: `output/open-issues-oldest-20261105/pr-final-20261107/rebase-5077b216-20261108/temporary-asset-json/mydocs/pr/assets/issue_7375/verification-20261105/native-vector-validation.json`)에서는 27·28쪽을 각각 독립 호출했다.
SVG의 반지름은 짧은 변 25.04px의 절반인 12.52px다. 각 1쪽짜리 직접 PDF를 원본의 해당 쪽과
대조한 raw 값은 99.99754%/99.99745%다. rhwp 생성 PDF를 독립 기준으로 사용하지 않았다.

실제 브라우저 편집 결과 (로컬 진단: `output/open-issues-oldest-20261105/pr-final-20261107/rebase-5077b216-20261108/temporary-asset-json/mydocs/pr/assets/issue_7375/verification-20261105/browser-edit-validation.json`)는 27·28쪽 각각 독립 선택·이동·삭제·
Ctrl+Z·Command+Z를 실행했다. 삭제 전후 자식 개수는 4→3→4이며, Undo 뒤 저장 offset과 실제
표시 경계가 원래 상태와 같다. 선택만으로 문서 변경 명령을 만들지 않는다.
[27쪽 복원 화면](studio-p027-undo-restored.png) · [28쪽 복원 화면](studio-p028-undo-restored.png).

Native producer (로컬 진단: `output/open-issues-oldest-20261105/pr-final-20261107/rebase-5077b216-20261108/temporary-asset-json/mydocs/pr/assets/issue_7375/verification-20261105/native-producer.json`) · WASM producer (로컬 진단: `output/open-issues-oldest-20261105/pr-final-20261107/rebase-5077b216-20261108/temporary-asset-json/mydocs/pr/assets/issue_7375/verification-20261105/wasm-producer.json`) ·
고정 base producer (로컬 진단: `output/open-issues-oldest-20261105/pr-final-20261107/rebase-5077b216-20261108/temporary-asset-json/mydocs/pr/assets/issue_7375/verification-20261105/base-producer.json`). Root pkg와 Studio public의 JS/WASM 파일 해시를 대조했고,
실제 초기화 WASM 응답도 일치했다. 브라우저 JS는 Vite가 변환한 응답이므로 원본 glue 파일 해시와
구분해 `browser-edit-validation.json`에 기록했다. 실제 Studio backend는 Canvas2D다.

필수 편람 Native gate (로컬 진단: `output/open-issues-oldest-20261105/pr-final-20261107/rebase-5077b216-20261108/temporary-asset-json/mydocs/pr/assets/issue_7375/verification-20261105/handbook-native-gate.json`)는 383쪽 전체와 기존 기준 대비 하락 0쪽을 확인했다.
전체 회귀·Rust lint·Native Skia 3종은 현재 Mac에서 직접 최소 글꼴 격리 검증을 진행 중이다.
프로세스 파일 sandbox로 개인·추가 설치 글꼴과 이전 글꼴 캐시 접근을 차단했다. CoreText의
이름 캐시는 남지만 개인 글꼴 1,206개 파일에 속한 모든 face가 실제 개인 프로그램 대신
LastResort를 사용하고, OS 기본 글꼴 294개 파일의 정보는 바뀌지 않았음을 대조했다.
저장소의 공개 기본 fallback과 명시적 OFL fixture는 별도로 허용했다. 이 증적만으로 PR 전체
검증 완료나 이슈 통합 완료를 선언하지 않는다. 로그와 임시 driver는 ignored `output/`에 보존한다.
Mac 글꼴 격리 사전 검증 (로컬 진단: `output/open-issues-oldest-20261105/pr-final-20261107/rebase-5077b216-20261108/temporary-asset-json/mydocs/pr/assets/issue_7375/verification-20261105/macos-font-isolation.json`)에 실제 프로필·허용 파일 해시·CoreText
프로그램 대조와 일반 사용자 프로세스의 글꼴 상태가 바뀌지 않은 대조 결과를 기록했다.
