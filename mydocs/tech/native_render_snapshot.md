---
kind: reference
status: active
canonical: mydocs/tech/native_render_snapshot.md
last_verified: 2026-11-07
---

# 불변 Native 렌더 입력 snapshot

`NativeDocumentSession::render_snapshot(page, profile)`은 현재 페이지의
`Arc<PageLayerTree>`와 문서 revision·font generation·page·명시적 글꼴 공급을
함께 보유한다. 동일 페이지·profile의 재조회는 기존 Core의 공유 layer tree를
재사용하며 트리 전체를 깊게 복제하지 않는다. profile과 layer output options는
보유한 tree의 값이다. scale·viewport·device generation은 소비하는 호스트가 관리한다.

snapshot은 `Send + Sync`인 불변 입력이다. Native session·Skia renderer·GPU context를
다른 스레드로 옮기거나 `unsafe impl Send`로 바꾸지 않는다. 소비자는 자신이 소유한
render thread에서 renderer를 만들고 기존 borrowed Canvas/Surface replay API에 tree를
넘긴다. snapshot은 원래 Core나 문서 세션을 보유하지 않아 편집·문서 교체·세션 종료
뒤에도 재생할 수 있다. snapshot 자체는 GPU target이나 표시 가능한 프레임이 아니다.

```rust
use rhwp::paint::RenderProfile;

let snapshot = session.render_snapshot(0, RenderProfile::Screen)?;
drop(session);
// native-skia를 켠 소비자 스레드에서 생성한다.
let renderer = snapshot.create_renderer()?;
renderer.render_to_surface_strict(&mut target, snapshot.tree(), Default::default())?;
```

## 글꼴과 자원 수명

`set_font_sources`에서 읽고 검증한 원래 bytes는 성공할 때만 세션의
`Arc<[Vec<u8>]>`에 게시된다. snapshot은 이 Arc를 공유하고 `create_renderer`는 같은
불변 입력을 기존 `try_with_font_data`에 빌려 준다. source path를 다시 읽거나 입력
bytes를 snapshot마다 복사하지 않는다. 실패한 글꼴 교체는 이전 공급 상태와 bytes를
교체하지 않는다. Skia가 renderer를 구성하며 만드는 native font 객체와 per-face
복사본은 별도 수명과 예산을 갖는다.

이 계약이 고정하는 것은 **명시적으로 공급한 글꼴 bytes**다. `SkiaLayerRenderer::new`
는 시스템 FontMgr fallback을 사용할 수 있으며 시스템 글꼴 설치·내용·선택 결과는
snapshot의 fingerprint로 고정하지 않는다. 현재 native session 경로와 동일하게
호출자 font path나 `RHWP_FONT_PATH`는 renderer 재생성 때 읽지 않는다. 다른 호스트로
옮겨도 모든 glyph가 같은 모양을 낸다는 portable font 인증을 의미하지 않는다.

이미지·SVG·embedded font blob과 텍스트·geometry는 shared tree의 owned 값이다.
Core의 임시 borrow나 원본 파일 경로에 대한 borrow는 없다. 리소스 decode·GPU 업로드와
strict replay 오류 정책은 [기존 Surface 계약](native_skia_host_surface.md)을 따른다.
이 API가 누락된 이미지나 지원하지 않는 glyph 계약을 자동으로 보완하지 않는다.

## 한도와 계상 범위

`render_snapshot_with_limits`에서 다음 기본 상한을 낮출 수 있다. 공개 상한을
초과한 값과 `usize::MAX`는 입력 오류이며, 항목 수·바이트의 덧셈·곱셈은 overflow를
검사한다. 0은 해당 종류의 입력을 허용하지 않는 제한이다.

| 항목 | 공개 상한 |
| --- | --- |
| layer node | 65,536개 |
| paint operation | 262,144개 |
| 자원 참조 | 8,192개 |
| 계상한 입력 payload | 128 MiB |
| layer 깊이 | 256단계 |

자원 계상은 arena의 image·SVG·font blob과 leaf가 직접 보유하는 image·변환된 image·
쪽 배경 image·SVG·EMF·수식 SVG 및 명시적 host font bytes를 포함한다. 중복 참조도
보수적으로 각각 계상한다. 텍스트/source table, 주요 glyph·path·clip·gradient 배열과
node/operation 구조 크기를 별도로 더한다. traversal은 callback이나 재귀 없이
제한된 pending stack을 사용한다. 한도를 넘은 export는 오류로 끝내며 이미 발행한
snapshot이나 편집 상태를 바꾸지 않는다. Core의 일반 layer 생성·캐시 조회는 수행될 수 있다.

`NativeRenderSnapshotUsage`는 이 규칙에 따른 **논리 입력 크기 추정값**이다. allocator
capacity·lookup metadata·보조 style/layout 상태·일부 공유 metric 객체·decoded image·
Skia cache·GPU target·driver 메모리를 모두 측정하는 RSS/VRAM 값이 아니다. 원래 Core의
layer 생성 과정도 export 전 별도 예산을 소비한다. snapshot이 제한을 통과했다는
사실을 프로세스 전체 메모리 상한이나 무제한 동시 보유의 허가로 사용하지 않는다.

호스트는 문서 ID, view/device generation, snapshot/frame 보유 개수와 동시 참조의
총예산을 관리하고 소비가 끝난 snapshot을 해제해야 한다. GPU context·flush·fence·
프레임 acquire/release·표시와 Flutter/Qt transport는 이 Rust 입력 계약 위의 별도 계층이다.

## 검사 범위

`tests/cases/issue_536_native_render_snapshot.rs`는 기본 빌드에서 공유 tree 재사용,
profile 분리, 편집 전후의 revision·내용 독립성, 세션 종료 후 다른 스레드에서의
입력 사용과 한도 거절을 검사한다. `native-skia`에서는 기존 허가 글꼴 fixture의
경로 변경·삭제 후 frozen bytes로 renderer를 다시 만들고 borrowed CPU Surface에
재생한다. 합성 PNG의 retained payload 제한과 세션 종료 후 출력 보존도 검사한다.
한컴 시각 일치율·실물 페이지 좌표·GPU 표시·시스템 fallback 이식성을 판정하지 않는다.
