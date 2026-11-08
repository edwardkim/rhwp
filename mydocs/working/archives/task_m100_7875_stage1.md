# #7875 1단계 — 프로젝트 적합성과 WASM 라이브러리 주장 검증

Issue: [#7875](https://github.com/edwardkim/rhwp/issues/7875)

원 PR: [#7876](https://github.com/edwardkim/rhwp/pull/7876)

## 목적과 현재 단계

사용자의 지시에 따라 체리픽보다 프로젝트 적합성·문제 재현을 먼저 확인했다.
이 문서는 적용 전 조사와 보정 계획이다. 적용과 완료된 검증 결과는 [2단계](task_m100_7875_stage2.md)에 기록했다.

- 기준 devel: `515bd72dcde0eecdeece333666b675354b7fa91d`.
- 원 PR head: `b273e3fc44dd713f33d22846faa6fb1d98f01d3b`, 기능 1커밋·2파일.
- 작성자 `9ood2s`의 전체 PR 조회에서 rhwp 첫 기여임을 확인했다. reviewer는 `jangster77`로 요청했다.
- 기본 작업공간의 가시성 branch: `review/9ood2s-7876-20261017`.
- 기본 경로: `collaborator_external_pr` 9.1.1. 보조 경로: 첫 기여자·접수/리뷰·로컬 검증.
- 이 단계에서 contributor 코드·fork는 변경하지 않았다. 최신 devel을 먼저 동기화했다.

## 프로젝트 범위 판정

`Cargo.toml`의 `[lib]`는 Rust 라이브러리 우선과 내부 CLI·Native·tools의 rlib 재사용을 명시한다.
[로드맵의 업스트림·다운스트림 경계](../../../ROADMAP.md#업스트림과-다운스트림의-경계)는 공통 엔진·Web/WASM 재사용과 공용 API 호환 개선을 업스트림 범위로 둔다.
JS export를 선택적으로 끄는 범용 호환 개선은 이 범위에 맞는다.

다만 [#5446의 메인터너 결정](https://github.com/edwardkim/rhwp/issues/5446#issuecomment-5327810241)은
v1.0까지 신규 배포 채널을 동결하며 안정적인 외부 Rust API·전체 feature/MSRV 지원 계약은 미확정이라고 명시한다.
따라서 이 작업은 기존 git/path 소비자의 좁은 호환 개선으로 한정한다. crates.io 게시·공식 parser-only SDK·장기 API 호환 약속을 추가하지 않는다.

## 독립 근거와 최소 재현

실제 Cargo.lock·CLI와 같은 wasm-bindgen **0.2.127**로 최소 소비 앱을 빌드하고 Node에서 초기화했다.
[해당 버전의 공식 start 설명](https://github.com/wasm-bindgen/wasm-bindgen/blob/0.2.127/guide/src/reference/attributes/on-rust-exports/start.md)은
여러 명시적 start를 연결해 실행하며 순서는 보장하지 않는다고 명시한다.

| 소비 앱 조건 | 일반 main 실행 | 명시 app start 실행 | 의존성 start 실행 |
| --- | --- | --- | --- |
| 일반 main, 의존성 start 없음 | true | false | false |
| 일반 main, 의존성 start 있음 | false | false | true |
| 앱·의존성 명시적 start 모두 있음 | false | true | true |
| 앱에만 명시적 start 있음 | false | true | false |

따라서 일반 Rust main 기반 wasm 바이너리의 자동 시작 문제는 재현됐다.
명시적 start까지 대체한다고 일반화할 수는 없다. 시작만은 앱의 명시적 start로 우회할 수 있고,
불필요한 export 제거는 별개의 문제다.

## 최신 rhwp를 링크한 실제 소비 앱

`rhwp = { path = "/home/tsjang/rhwp", default-features = false }`를 사용했다.
Cargo metadata의 실제 rhwp feature 목록도 **빈 배열**임을 확인했다. `parse_document`만 호출해
다음 공개 HWP5/HWPX를 읽었다. 입력 파일과 Git blob의 바이트 일치도 확인했다.

| 입력 | bytes | SHA-256 |
| --- | ---: | --- |
| `samples/hwpx/blank_hwpx.hwpx` | 12707 | `cf7291b4aab53f25b48df4fce83e33db636fea8600beed369a5eb34880c803ef` |
| `samples/hwpx/hancom-hwp/blank_hwpx.hwp` | 12800 | `297a9b5f80e6ea590f1364d81fee287c781de49e4a656e16fef86f39c6756837` |

두 입력은 모두 `sections=1;paragraphs=1`로 파싱됐다. 이 값은 파서 사용의 smoke 계약이며,
한컴 출력이나 조판 정확성의 근거로 승격하지 않는다.

| 소비 앱 | main 실행 | app start 실행 | HwpDocument export | 전체 export | WASM bytes |
| --- | --- | --- | ---: | ---: | ---: |
| base-plain-main | false | false | 492 | 526 | 16,753,644 |
| base-explicit-start | false | true | 492 | 527 | 16,753,711 |

Chrome 155.0.8059.79의 web-target에서도 Node와 동일한 시작 상태·두 입력의 파싱 결과를 확인했다.
Trunk 본체는 설치하지 않았으며 `Cargo → wasm-bindgen web → Chrome` 경로를 실행했다.
기여자의 사유 앱에서 측정한 49.0→38.4MB는 해당 앱을 확보하지 않아 독립 재현했다고 쓰지 않는다.

실제 소비 코드의 핵심은 다음과 같다. `explicit-app-start` 유무만 바꿔 정상 대조군을 만들었다.

```rust
use std::sync::atomic::{AtomicBool, Ordering};
use wasm_bindgen::prelude::*;
static MAIN_STARTED: AtomicBool = AtomicBool::new(false);
static APP_STARTED: AtomicBool = AtomicBool::new(false);
fn main() { MAIN_STARTED.store(true, Ordering::SeqCst); }
#[cfg(feature = "explicit-app-start")]
#[wasm_bindgen(start)]
pub fn consumer_start() { APP_STARTED.store(true, Ordering::SeqCst); }
#[wasm_bindgen]
pub fn main_started() -> bool { MAIN_STARTED.load(Ordering::SeqCst) }
#[wasm_bindgen]
pub fn app_started() -> bool { APP_STARTED.load(Ordering::SeqCst) }
#[wasm_bindgen]
pub fn document_summary(bytes: &[u8]) -> String {
    match rhwp::parse_document(bytes) {
        Ok(doc) => format!("ok;sections={};paragraphs={}", doc.sections.len(), doc.sections.iter().map(|s| s.paragraphs.len()).sum::<usize>()),
        Err(err) => format!("error:{err}"),
    }
}
```

Rust 1.93.1·wasm-bindgen CLI 0.2.127·release/LTO·codegen-units 16 조건이다.
Cargo 작업은 jobs 8·공용 `/home/tsjang/rhwp/target/pr-review`에서 순차로 실행했다.
소비 앱은 독립 workspace이며 현재 저장소의 `vendor/svg2pdf` patch를 그대로 지정했다.
의존성 검증용 새 Cargo.lock만 만들었고 원본 Cargo.lock은 변경하지 않았다.

```bash
cargo build --locked --offline --release --target wasm32-unknown-unknown   --target-dir /home/tsjang/rhwp/target/pr-review --no-default-features
# 명시적 app start 대조군에는 --features explicit-app-start 추가
wasm-bindgen <소비-app.wasm> --target nodejs --out-dir <case-output>
# 같은 raw wasm을 --target web으로 만들고 Chrome에서 별도 초기화했다.
```

최초 Node 점검 스크립트의 CommonJS `exports` 변수명 충돌은 점검 코드에서 고쳤다.
이미 완료된 WASM을 재사용했고, 이를 제품 결함·수정 전 FAIL 증거로 세지 않았다.

## 보정 계획과 수용 전 조건

1. 현재 기본 feature `pdf-import`를 보존하면서 원 기능 1커밋을 `-x` 체리픽한다.
2. 주석의 시작 함수 설명을 일반 main 경로로 한정하고 export 제거와 구분한다.
   기본 feature를 끈 기존 소비자가 Rust `wasm_api` facade를 쓰려면 명시적 `wasm-api` opt-in이 필요함을 설명한다.
3. 현재 기본 패키지의 JS·d.ts·export 표면을 적용 전후에 대조한다.
   feature 이름만 추가한 대조군도 필요하면 사용해 실제 코드 차이와 crate metadata 차이를 분리한다.
4. 같은 소비 앱에서 main 자동 시작 복원·HwpDocument export 제거·두 포맷 파싱 보존을 실행한다.
   명시적 start 정상 대조군도 유지한다. default/비기본 native·wasm lib와 CLI·FFI 소비를 검증한다.
5. Rust lint·focused·전체 nextest·Native Skia·fresh WASM의 해당 게이트를 수행한다.
   불필요한 CI 개별 호출은 추가하지 않는다. 기존 fixture/golden/허용치를 완화하지 않는다.
6. 기본 빌드의 실제 조판·렌더 소비 경로를 바꾸지 않는지 코드와 산출물로 판정한다.
   이 작업의 파싱·시작/export 검증을 Visual Sweep 통과로 바꾸어 기록하지 않는다.
7. 정확한 제출 code CI가 녹색인 뒤 원 PR 번호 기준 archive review·구현 기록과 오늘할일을 같은 통합 PR의 trailing 문서로 포함한다.
   merge가 승인되고 실제 완료되면 첫 기여자에게 존댓말로 감사·환영·실제 보정과 결과를 안내하며 fork branch를 보존한다.

## 적용 전 단계의 증거 위치와 미검증

작업 중 원본 로그·각 JSON·raw consumer WASM과 frozen web/Node 패키지는
`output/pr-review/pr7876-20261017/`에 있다. 이 1단계 완료 시 제품 source는 기준 devel과 같았다.
기본 패키지 비교 기준도 기준 source에서 wrapper `--target web --no-opt`로 생성 완료했다(exit 0, 6분 35초). Docker는 이 호스트에 없다.
이는 최적화된 Docker 배포 빌드 성공을 뜻하지 않는다.
원 PR의 과거 green CI는 최신 통합 head의 증거가 아니며 현재 원 PR은 feature 목록 충돌 상태다.
이 단계에서는 보정 후 소비 앱·최종 lint/회귀/CI·merge를 완료로 판정하지 않았다. 이후 실행 결과는 [2단계](task_m100_7875_stage2.md)를 따른다.

## 적용 전 기본 패키지 지문

실제 producer는 위 기준 SHA이고 Cargo.toml·Cargo.lock·src/lib.rs의 전후 SHA가 같았다.
기본 export는 522개다.

| 파일 | bytes | SHA-256 |
| --- | ---: | --- |
| `rhwp.js` | 474830 | `c085ddebd0d26befc7a105deba0c041f17c962e46deeaaa35c34cc925abf714e` |
| `rhwp.d.ts` | 191405 | `c56c89a9fe1060e04ca5acd84b3eb1d2f2b56e17ecb0ab945d0b9d1230fcd382` |
| `rhwp_bg.wasm.d.ts` | 64886 | `53002a58e6455e6937bf091e84797c81c2e50d909b51eebd4d63402a4f94398e` |
| `rhwp_bg.wasm` | 22305631 | `ba855693573fea3dfc14b7ffaa637d616c0fbeebd5be8dc60e61e85fdbfad1a3` |
