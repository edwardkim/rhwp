# Task #660 Stage 2 — 빌더 + CLI 명령 + e2e 보고서

## 인도물

- `src/document_core/mod.rs` — `pub mod builders;` 등록
- `src/document_core/builders/mod.rs` — 모듈 진입점 + 책임 명세
- `src/document_core/builders/exam_paper.rs` — `build_exam_paper(ingest: &IngestDocument) -> Document`
  - 텍스트 위주 (이미지 placeholder 처리, placement 4모드는 #661에서 본격)
  - 선택지 ①~⑤ 텍스트 직접 포함 (spike #654 결정 정책)
  - 문제 번호 자동 prepend (`{번호}. {지문}`)
  - 문제 간 빈 문단 자동 삽입 (마지막 제외)
- `src/main.rs` — `rhwp build-from-ingest <ingest.json> [--media-dir <dir>] -o <out.hwpx>` 신규 명령

## 검증

### Unit 테스트 (4/4 통과)

```
cargo test --release --lib document_core::builders
running 4 tests
test test_build_with_image_placeholder ... ok
test test_build_single_question ... ok
test test_build_stem_without_blocks ... ok
test test_build_multiple_questions_separator ... ok
test result: ok. 4 passed
```

### e2e 변환 (sample_minimal.json → HWPX)

```
$ cargo run --bin rhwp --release -- build-from-ingest \
    tools/rhwp-ingest/schema/sample_minimal.json -o output/sample_minimal.hwpx
저장 완료: output/sample_minimal.hwpx (5356바이트, 문제 3개, 문단 21개)
```

HWPX 구조:
```
Archive: output/sample_minimal.hwpx
  Length    Name
       19   mimetype
      309   version.xml
     1243   Contents/header.xml
    10676   Contents/section0.xml          ← 21개 문단 본문
        2   Preview/PrvText.txt
       68   Preview/PrvImage.png
      279   settings.xml
      867   META-INF/container.rdf
     1428   Contents/content.hpf
      475   META-INF/container.xml
      134   META-INF/manifest.xml
```

### 라운드트립 (`rhwp dump`)

3개 문제 × (1 stem + 1 추가 stem block + 5 choice) + 2 빈 문단(간격) = 21 문단 모두 정상.

문제 1 (예시):
- 문단 0.0: "1. 다음 글의 주제로 가장 적절한 것은?" (cc=24, text_len=23)
- 문단 0.1: "환경 오염은 현대 사회의 중요한 문제..." (cc=69)
- 문단 0.2~0.6: ① ~ ⑤ 선택지
- 문단 0.7: 빈 문단 (다음 문제 간격)

①~⑤ 유니코드 모두 정상 직렬화 (`<hp:t>① ...</hp:t>`).

## 알려진 한계 (본 단계 의도)

- 이미지: `[이미지: img/q1.png]` placeholder 텍스트로 처리 — 본격 Picture 빌드는 #661, 출력은 #182 의존
- placement 4모드: Placement enum 정의만, 실제 IR 매핑은 #661
- ParaShape: 모든 문단 default(id=0) — 시험지 표준 ParaShape 풀(`exam_styles.rs`)은 #661

## 다음 단계

Stage 3: 단계별 보고서 마무리 + 최종 보고서 + 커밋
