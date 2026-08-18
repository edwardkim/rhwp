---
kind: reference
status: active
canonical: mydocs/tech/render_backend_fixture_catalog.md
last_verified: 2026-08-18
---

# render_backend 픽스처 카탈로그 (M06-f)

합성 장면 196 장의 목록이다. 각 장은 `tests/fixtures/render_backend/scenes/<id>.json`
이고, TraceBackend 기대 로그를 포함한다. 실제 HWP 가 아니다.

생성기: `tools/render_backend/gen_m06f.py`.
스키마: `FixtureScene::SCHEMA == 1`.

## 장면 목록

| id | 치수(px) | op 수 | 계약 |
| --- | --- | --- | --- |
| `c00-rectangle-ellipse-path` | 400×300 | 3 | flow 클러스터 rectangle+ellipse+path 순서 유지 |
| `c01-line-textRun-textDecoration` | 400×300 | 3 | flow 클러스터 line+textRun+textDecoration 순서 유지 |
| `c02-image-placeholder-rawSvg` | 400×300 | 3 | flow 클러스터 image+placeholder+rawSvg 순서 유지 |
| `c03-formObject-equation-footnoteMarker` | 400×300 | 3 | flow 클러스터 formObject+equation+footnoteMarker 순서 유지 |
| `c04-charOverlap-tabLeader-textControlMark` | 400×300 | 3 | flow 클러스터 charOverlap+tabLeader+textControlMark 순서 유지 |
| `c05-rectangle-textRun-image` | 400×300 | 3 | flow 클러스터 rectangle+textRun+image 순서 유지 |
| `c06-ellipse-path-line` | 400×300 | 3 | flow 클러스터 ellipse+path+line 순서 유지 |
| `c07-placeholder-formObject-rawSvg` | 400×300 | 3 | flow 클러스터 placeholder+formObject+rawSvg 순서 유지 |
| `m-charOverlap-160x120` | 160×120 | 1 | charOverlap 를 160x120 페이지에 올리면 begin_page 헤더가 160.00x120.00 |
| `m-charOverlap-240x180` | 240×180 | 1 | charOverlap 를 240x180 페이지에 올리면 begin_page 헤더가 240.00x180.00 |
| `m-charOverlap-320x240` | 320×240 | 1 | charOverlap 를 320x240 페이지에 올리면 begin_page 헤더가 320.00x240.00 |
| `m-charOverlap-480x360` | 480×360 | 1 | charOverlap 를 480x360 페이지에 올리면 begin_page 헤더가 480.00x360.00 |
| `m-charOverlap-640x480` | 640×480 | 1 | charOverlap 를 640x480 페이지에 올리면 begin_page 헤더가 640.00x480.00 |
| `m-charOverlap-80x60` | 80×60 | 1 | charOverlap 를 80x60 페이지에 올리면 begin_page 헤더가 80.00x60.00 |
| `m-ellipse-160x120` | 160×120 | 1 | ellipse 를 160x120 페이지에 올리면 begin_page 헤더가 160.00x120.00 |
| `m-ellipse-240x180` | 240×180 | 1 | ellipse 를 240x180 페이지에 올리면 begin_page 헤더가 240.00x180.00 |
| `m-ellipse-320x240` | 320×240 | 1 | ellipse 를 320x240 페이지에 올리면 begin_page 헤더가 320.00x240.00 |
| `m-ellipse-480x360` | 480×360 | 1 | ellipse 를 480x360 페이지에 올리면 begin_page 헤더가 480.00x360.00 |
| `m-ellipse-640x480` | 640×480 | 1 | ellipse 를 640x480 페이지에 올리면 begin_page 헤더가 640.00x480.00 |
| `m-ellipse-80x60` | 80×60 | 1 | ellipse 를 80x60 페이지에 올리면 begin_page 헤더가 80.00x60.00 |
| `m-equation-160x120` | 160×120 | 1 | equation 를 160x120 페이지에 올리면 begin_page 헤더가 160.00x120.00 |
| `m-equation-240x180` | 240×180 | 1 | equation 를 240x180 페이지에 올리면 begin_page 헤더가 240.00x180.00 |
| `m-equation-320x240` | 320×240 | 1 | equation 를 320x240 페이지에 올리면 begin_page 헤더가 320.00x240.00 |
| `m-equation-480x360` | 480×360 | 1 | equation 를 480x360 페이지에 올리면 begin_page 헤더가 480.00x360.00 |
| `m-equation-640x480` | 640×480 | 1 | equation 를 640x480 페이지에 올리면 begin_page 헤더가 640.00x480.00 |
| `m-equation-80x60` | 80×60 | 1 | equation 를 80x60 페이지에 올리면 begin_page 헤더가 80.00x60.00 |
| `m-footnoteMarker-160x120` | 160×120 | 1 | footnoteMarker 를 160x120 페이지에 올리면 begin_page 헤더가 160.00x120.00 |
| `m-footnoteMarker-240x180` | 240×180 | 1 | footnoteMarker 를 240x180 페이지에 올리면 begin_page 헤더가 240.00x180.00 |
| `m-footnoteMarker-320x240` | 320×240 | 1 | footnoteMarker 를 320x240 페이지에 올리면 begin_page 헤더가 320.00x240.00 |
| `m-footnoteMarker-480x360` | 480×360 | 1 | footnoteMarker 를 480x360 페이지에 올리면 begin_page 헤더가 480.00x360.00 |
| `m-footnoteMarker-640x480` | 640×480 | 1 | footnoteMarker 를 640x480 페이지에 올리면 begin_page 헤더가 640.00x480.00 |
| `m-footnoteMarker-80x60` | 80×60 | 1 | footnoteMarker 를 80x60 페이지에 올리면 begin_page 헤더가 80.00x60.00 |
| `m-formObject-160x120` | 160×120 | 1 | formObject 를 160x120 페이지에 올리면 begin_page 헤더가 160.00x120.00 |
| `m-formObject-240x180` | 240×180 | 1 | formObject 를 240x180 페이지에 올리면 begin_page 헤더가 240.00x180.00 |
| `m-formObject-320x240` | 320×240 | 1 | formObject 를 320x240 페이지에 올리면 begin_page 헤더가 320.00x240.00 |
| `m-formObject-480x360` | 480×360 | 1 | formObject 를 480x360 페이지에 올리면 begin_page 헤더가 480.00x360.00 |
| `m-formObject-640x480` | 640×480 | 1 | formObject 를 640x480 페이지에 올리면 begin_page 헤더가 640.00x480.00 |
| `m-formObject-80x60` | 80×60 | 1 | formObject 를 80x60 페이지에 올리면 begin_page 헤더가 80.00x60.00 |
| `m-image-160x120` | 160×120 | 1 | image 를 160x120 페이지에 올리면 begin_page 헤더가 160.00x120.00 |
| `m-image-240x180` | 240×180 | 1 | image 를 240x180 페이지에 올리면 begin_page 헤더가 240.00x180.00 |
| `m-image-320x240` | 320×240 | 1 | image 를 320x240 페이지에 올리면 begin_page 헤더가 320.00x240.00 |
| `m-image-480x360` | 480×360 | 1 | image 를 480x360 페이지에 올리면 begin_page 헤더가 480.00x360.00 |
| `m-image-640x480` | 640×480 | 1 | image 를 640x480 페이지에 올리면 begin_page 헤더가 640.00x480.00 |
| `m-image-80x60` | 80×60 | 1 | image 를 80x60 페이지에 올리면 begin_page 헤더가 80.00x60.00 |
| `m-line-160x120` | 160×120 | 1 | line 를 160x120 페이지에 올리면 begin_page 헤더가 160.00x120.00 |
| `m-line-240x180` | 240×180 | 1 | line 를 240x180 페이지에 올리면 begin_page 헤더가 240.00x180.00 |
| `m-line-320x240` | 320×240 | 1 | line 를 320x240 페이지에 올리면 begin_page 헤더가 320.00x240.00 |
| `m-line-480x360` | 480×360 | 1 | line 를 480x360 페이지에 올리면 begin_page 헤더가 480.00x360.00 |
| `m-line-640x480` | 640×480 | 1 | line 를 640x480 페이지에 올리면 begin_page 헤더가 640.00x480.00 |
| `m-line-80x60` | 80×60 | 1 | line 를 80x60 페이지에 올리면 begin_page 헤더가 80.00x60.00 |
| `m-pageBackground-160x120` | 160×120 | 1 | pageBackground 를 160x120 페이지에 올리면 begin_page 헤더가 160.00x120.00 |
| `m-pageBackground-240x180` | 240×180 | 1 | pageBackground 를 240x180 페이지에 올리면 begin_page 헤더가 240.00x180.00 |
| `m-pageBackground-320x240` | 320×240 | 1 | pageBackground 를 320x240 페이지에 올리면 begin_page 헤더가 320.00x240.00 |
| `m-pageBackground-480x360` | 480×360 | 1 | pageBackground 를 480x360 페이지에 올리면 begin_page 헤더가 480.00x360.00 |
| `m-pageBackground-640x480` | 640×480 | 1 | pageBackground 를 640x480 페이지에 올리면 begin_page 헤더가 640.00x480.00 |
| `m-pageBackground-80x60` | 80×60 | 1 | pageBackground 를 80x60 페이지에 올리면 begin_page 헤더가 80.00x60.00 |
| `m-path-160x120` | 160×120 | 1 | path 를 160x120 페이지에 올리면 begin_page 헤더가 160.00x120.00 |
| `m-path-240x180` | 240×180 | 1 | path 를 240x180 페이지에 올리면 begin_page 헤더가 240.00x180.00 |
| `m-path-320x240` | 320×240 | 1 | path 를 320x240 페이지에 올리면 begin_page 헤더가 320.00x240.00 |
| `m-path-480x360` | 480×360 | 1 | path 를 480x360 페이지에 올리면 begin_page 헤더가 480.00x360.00 |
| `m-path-640x480` | 640×480 | 1 | path 를 640x480 페이지에 올리면 begin_page 헤더가 640.00x480.00 |
| `m-path-80x60` | 80×60 | 1 | path 를 80x60 페이지에 올리면 begin_page 헤더가 80.00x60.00 |
| `m-placeholder-160x120` | 160×120 | 1 | placeholder 를 160x120 페이지에 올리면 begin_page 헤더가 160.00x120.00 |
| `m-placeholder-240x180` | 240×180 | 1 | placeholder 를 240x180 페이지에 올리면 begin_page 헤더가 240.00x180.00 |
| `m-placeholder-320x240` | 320×240 | 1 | placeholder 를 320x240 페이지에 올리면 begin_page 헤더가 320.00x240.00 |
| `m-placeholder-480x360` | 480×360 | 1 | placeholder 를 480x360 페이지에 올리면 begin_page 헤더가 480.00x360.00 |
| `m-placeholder-640x480` | 640×480 | 1 | placeholder 를 640x480 페이지에 올리면 begin_page 헤더가 640.00x480.00 |
| `m-placeholder-80x60` | 80×60 | 1 | placeholder 를 80x60 페이지에 올리면 begin_page 헤더가 80.00x60.00 |
| `m-rawSvg-160x120` | 160×120 | 1 | rawSvg 를 160x120 페이지에 올리면 begin_page 헤더가 160.00x120.00 |
| `m-rawSvg-240x180` | 240×180 | 1 | rawSvg 를 240x180 페이지에 올리면 begin_page 헤더가 240.00x180.00 |
| `m-rawSvg-320x240` | 320×240 | 1 | rawSvg 를 320x240 페이지에 올리면 begin_page 헤더가 320.00x240.00 |
| `m-rawSvg-480x360` | 480×360 | 1 | rawSvg 를 480x360 페이지에 올리면 begin_page 헤더가 480.00x360.00 |
| `m-rawSvg-640x480` | 640×480 | 1 | rawSvg 를 640x480 페이지에 올리면 begin_page 헤더가 640.00x480.00 |
| `m-rawSvg-80x60` | 80×60 | 1 | rawSvg 를 80x60 페이지에 올리면 begin_page 헤더가 80.00x60.00 |
| `m-rectangle-160x120` | 160×120 | 1 | rectangle 를 160x120 페이지에 올리면 begin_page 헤더가 160.00x120.00 |
| `m-rectangle-240x180` | 240×180 | 1 | rectangle 를 240x180 페이지에 올리면 begin_page 헤더가 240.00x180.00 |
| `m-rectangle-320x240` | 320×240 | 1 | rectangle 를 320x240 페이지에 올리면 begin_page 헤더가 320.00x240.00 |
| `m-rectangle-480x360` | 480×360 | 1 | rectangle 를 480x360 페이지에 올리면 begin_page 헤더가 480.00x360.00 |
| `m-rectangle-640x480` | 640×480 | 1 | rectangle 를 640x480 페이지에 올리면 begin_page 헤더가 640.00x480.00 |
| `m-rectangle-80x60` | 80×60 | 1 | rectangle 를 80x60 페이지에 올리면 begin_page 헤더가 80.00x60.00 |
| `m-tabLeader-160x120` | 160×120 | 1 | tabLeader 를 160x120 페이지에 올리면 begin_page 헤더가 160.00x120.00 |
| `m-tabLeader-240x180` | 240×180 | 1 | tabLeader 를 240x180 페이지에 올리면 begin_page 헤더가 240.00x180.00 |
| `m-tabLeader-320x240` | 320×240 | 1 | tabLeader 를 320x240 페이지에 올리면 begin_page 헤더가 320.00x240.00 |
| `m-tabLeader-480x360` | 480×360 | 1 | tabLeader 를 480x360 페이지에 올리면 begin_page 헤더가 480.00x360.00 |
| `m-tabLeader-640x480` | 640×480 | 1 | tabLeader 를 640x480 페이지에 올리면 begin_page 헤더가 640.00x480.00 |
| `m-tabLeader-80x60` | 80×60 | 1 | tabLeader 를 80x60 페이지에 올리면 begin_page 헤더가 80.00x60.00 |
| `m-textControlMark-160x120` | 160×120 | 1 | textControlMark 를 160x120 페이지에 올리면 begin_page 헤더가 160.00x120.00 |
| `m-textControlMark-240x180` | 240×180 | 1 | textControlMark 를 240x180 페이지에 올리면 begin_page 헤더가 240.00x180.00 |
| `m-textControlMark-320x240` | 320×240 | 1 | textControlMark 를 320x240 페이지에 올리면 begin_page 헤더가 320.00x240.00 |
| `m-textControlMark-480x360` | 480×360 | 1 | textControlMark 를 480x360 페이지에 올리면 begin_page 헤더가 480.00x360.00 |
| `m-textControlMark-640x480` | 640×480 | 1 | textControlMark 를 640x480 페이지에 올리면 begin_page 헤더가 640.00x480.00 |
| `m-textControlMark-80x60` | 80×60 | 1 | textControlMark 를 80x60 페이지에 올리면 begin_page 헤더가 80.00x60.00 |
| `m-textDecoration-160x120` | 160×120 | 1 | textDecoration 를 160x120 페이지에 올리면 begin_page 헤더가 160.00x120.00 |
| `m-textDecoration-240x180` | 240×180 | 1 | textDecoration 를 240x180 페이지에 올리면 begin_page 헤더가 240.00x180.00 |
| `m-textDecoration-320x240` | 320×240 | 1 | textDecoration 를 320x240 페이지에 올리면 begin_page 헤더가 320.00x240.00 |
| `m-textDecoration-480x360` | 480×360 | 1 | textDecoration 를 480x360 페이지에 올리면 begin_page 헤더가 480.00x360.00 |
| `m-textDecoration-640x480` | 640×480 | 1 | textDecoration 를 640x480 페이지에 올리면 begin_page 헤더가 640.00x480.00 |
| `m-textDecoration-80x60` | 80×60 | 1 | textDecoration 를 80x60 페이지에 올리면 begin_page 헤더가 80.00x60.00 |
| `m-textRun-160x120` | 160×120 | 1 | textRun 를 160x120 페이지에 올리면 begin_page 헤더가 160.00x120.00 |
| `m-textRun-240x180` | 240×180 | 1 | textRun 를 240x180 페이지에 올리면 begin_page 헤더가 240.00x180.00 |
| `m-textRun-320x240` | 320×240 | 1 | textRun 를 320x240 페이지에 올리면 begin_page 헤더가 320.00x240.00 |
| `m-textRun-480x360` | 480×360 | 1 | textRun 를 480x360 페이지에 올리면 begin_page 헤더가 480.00x360.00 |
| `m-textRun-640x480` | 640×480 | 1 | textRun 를 640x480 페이지에 올리면 begin_page 헤더가 640.00x480.00 |
| `m-textRun-80x60` | 80×60 | 1 | textRun 를 80x60 페이지에 올리면 begin_page 헤더가 80.00x60.00 |
| `s000-empty` | 400×300 | 0 | 빈 페이지도 begin/end 경계를 남긴다 |
| `s001-background` | 400×300 | 1 | 배경만 있으면 Background plane 한 줄 |
| `s002-rect` | 400×300 | 1 | 사각형 하나 |
| `s003-line` | 400×300 | 1 | 수평선 하나 |
| `s004-reorder` | 400×300 | 3 | 트리 순서가 뒤바뀌어도 배경이 먼저 재생된다 |
| `s005-text` | 400×300 | 1 | 벡터 텍스트 정직성 문자열 |
| `s006-gradient-rect` | 400×300 | 1 | 그라디언트 사각형 |
| `s007-image` | 400×300 | 1 | 1x1 PNG 이미지 |
| `s100-pageBackground` | 400×300 | 1 | pageBackground 단독 장면 |
| `s101-textRun` | 400×300 | 1 | textRun 단독 장면 |
| `s102-charOverlap` | 400×300 | 1 | charOverlap 단독 장면 |
| `s103-textControlMark` | 400×300 | 1 | textControlMark 단독 장면 |
| `s104-tabLeader` | 400×300 | 1 | tabLeader 단독 장면 |
| `s105-textDecoration` | 400×300 | 1 | textDecoration 단독 장면 |
| `s106-footnoteMarker` | 400×300 | 1 | footnoteMarker 단독 장면 |
| `s107-line` | 400×300 | 1 | line 단독 장면 |
| `s108-rectangle` | 400×300 | 1 | rectangle 단독 장면 |
| `s109-ellipse` | 400×300 | 1 | ellipse 단독 장면 |
| `s110-path` | 400×300 | 1 | path 단독 장면 |
| `s111-image` | 400×300 | 1 | image 단독 장면 |
| `s112-equation` | 400×300 | 1 | equation 단독 장면 |
| `s113-formObject` | 400×300 | 1 | formObject 단독 장면 |
| `s114-placeholder` | 400×300 | 1 | placeholder 단독 장면 |
| `s115-rawSvg` | 400×300 | 1 | rawSvg 단독 장면 |
| `s200-size-1x1` | 1×1 | 1 | 페이지 치수 1x1 가 begin_page 에 그대로 찍힌다 |
| `s201-size-10x10` | 10×10 | 1 | 페이지 치수 10x10 가 begin_page 에 그대로 찍힌다 |
| `s202-size-40x30` | 40×30 | 1 | 페이지 치수 40x30 가 begin_page 에 그대로 찍힌다 |
| `s203-size-96x96` | 96×96 | 1 | 페이지 치수 96x96 가 begin_page 에 그대로 찍힌다 |
| `s204-size-200x150` | 200×150 | 1 | 페이지 치수 200x150 가 begin_page 에 그대로 찍힌다 |
| `s205-size-400x300` | 400×300 | 1 | 페이지 치수 400x300 가 begin_page 에 그대로 찍힌다 |
| `s206-size-595x842` | 595×842 | 1 | 페이지 치수 595x842 가 begin_page 에 그대로 찍힌다 |
| `s207-size-800x600` | 800×600 | 1 | 페이지 치수 800x600 가 begin_page 에 그대로 찍힌다 |
| `s208-size-1024x768` | 1024×768 | 1 | 페이지 치수 1024x768 가 begin_page 에 그대로 찍힌다 |
| `s209-size-1280x720` | 1280×720 | 1 | 페이지 치수 1280x720 가 begin_page 에 그대로 찍힌다 |
| `s300-rect-grid` | 400×300 | 20 | 20칸 사각형 격자 |
| `s301-text-then-bg` | 400×300 | 2 | 텍스트가 트리 앞에 있어도 배경이 먼저 |
| `s302-all-materializable` | 400×300 | 16 | 만들 수 있는 kind 전부 + 배경 재정렬 |
| `s400-empty-50x50` | 50×50 | 0 | 빈 50x50 페이지 |
| `s401-empty-100x200` | 100×200 | 0 | 빈 100x200 페이지 |
| `s402-empty-300x100` | 300×100 | 0 | 빈 300x100 페이지 |
| `s403-empty-777x333` | 777×333 | 0 | 빈 777x333 페이지 |
| `s500-line-stack` | 400×300 | 12 | 12개 수평선 |
| `s501-text-ladder` | 400×300 | 10 | 텍스트 10줄 |
| `s502-decorations` | 400×300 | 4 | 장식·탭·제어 표식 |
| `s503-chrome` | 400×300 | 5 | 양식·자리표시·수식·각주 |
| `s504-shapes` | 400×300 | 4 | 도형 가족 |
| `s505-offset` | 400×300 | 3 | 페이지 가장자리 bbox |
| `s506-zero-height-line` | 400×300 | 1 | 높이 0 선분은 유효하다 |
| `s507-tiny-60` | 400×300 | 60 | 60개 작은 사각형 |
| `s508-a4-zones` | 595×842 | 5 | A4 근사 머리/본문/바닥 |
| `s509-landscape` | 842×595 | 3 | 가로 페이지 |
| `s510-overlap-pair` | 400×300 | 2 | 글자겹침 쌍 |
| `s600-honesty-text` | 400×300 | 1 | 정직성 텍스트 프로브 |
| `s601-honesty-gradient` | 400×300 | 1 | 정직성 그라디언트 프로브 |
| `s602-honesty-image` | 400×300 | 1 | 정직성 이미지 프로브 |
| `s700-rect-x-0` | 400×300 | 1 | 사각형 x=0 |
| `s701-rect-x-1` | 400×300 | 1 | 사각형 x=1 |
| `s702-rect-x-7` | 400×300 | 1 | 사각형 x=7 |
| `s703-rect-x-13` | 400×300 | 1 | 사각형 x=13 |
| `s704-rect-x-50` | 400×300 | 1 | 사각형 x=50 |
| `s705-rect-x-99` | 400×300 | 1 | 사각형 x=99 |
| `s706-rect-x-150` | 400×300 | 1 | 사각형 x=150 |
| `s707-rect-x-200` | 400×300 | 1 | 사각형 x=200 |
| `s708-rect-x-250` | 400×300 | 1 | 사각형 x=250 |
| `s709-rect-x-300` | 400×300 | 1 | 사각형 x=300 |
| `s710-rect-x-350` | 400×300 | 1 | 사각형 x=350 |
| `s711-rect-x-389` | 400×300 | 1 | 사각형 x=389 |
| `s800-rect-y-0` | 400×300 | 1 | 사각형 y=0 |
| `s801-rect-y-1` | 400×300 | 1 | 사각형 y=1 |
| `s802-rect-y-7` | 400×300 | 1 | 사각형 y=7 |
| `s803-rect-y-13` | 400×300 | 1 | 사각형 y=13 |
| `s804-rect-y-50` | 400×300 | 1 | 사각형 y=50 |
| `s805-rect-y-99` | 400×300 | 1 | 사각형 y=99 |
| `s806-rect-y-150` | 400×300 | 1 | 사각형 y=150 |
| `s807-rect-y-200` | 400×300 | 1 | 사각형 y=200 |
| `s808-rect-y-250` | 400×300 | 1 | 사각형 y=250 |
| `s809-rect-y-289` | 400×300 | 1 | 사각형 y=289 |
| `s900-pair-rectangle-line` | 400×300 | 2 | flow 안 rectangle 다음 line 순서 유지 |
| `s901-pair-line-ellipse` | 400×300 | 2 | flow 안 line 다음 ellipse 순서 유지 |
| `s902-pair-ellipse-path` | 400×300 | 2 | flow 안 ellipse 다음 path 순서 유지 |
| `s903-pair-path-textRun` | 400×300 | 2 | flow 안 path 다음 textRun 순서 유지 |
| `s904-pair-textRun-image` | 400×300 | 2 | flow 안 textRun 다음 image 순서 유지 |
| `s905-pair-image-equation` | 400×300 | 2 | flow 안 image 다음 equation 순서 유지 |
| `s906-pair-equation-formObject` | 400×300 | 2 | flow 안 equation 다음 formObject 순서 유지 |
| `s907-pair-formObject-placeholder` | 400×300 | 2 | flow 안 formObject 다음 placeholder 순서 유지 |
| `s908-pair-placeholder-rawSvg` | 400×300 | 2 | flow 안 placeholder 다음 rawSvg 순서 유지 |
| `s909-pair-rawSvg-footnoteMarker` | 400×300 | 2 | flow 안 rawSvg 다음 footnoteMarker 순서 유지 |
| `s910-pair-footnoteMarker-tabLeader` | 400×300 | 2 | flow 안 footnoteMarker 다음 tabLeader 순서 유지 |
| `s911-pair-tabLeader-textDecoration` | 400×300 | 2 | flow 안 tabLeader 다음 textDecoration 순서 유지 |
| `s912-pair-textDecoration-charOverlap` | 400×300 | 2 | flow 안 textDecoration 다음 charOverlap 순서 유지 |
| `s913-pair-charOverlap-textControlMark` | 400×300 | 2 | flow 안 charOverlap 다음 textControlMark 순서 유지 |
| `s914-pair-textControlMark-rectangle` | 400×300 | 2 | flow 안 textControlMark 다음 rectangle 순서 유지 |

## JSON 필드

| 필드 | 의미 |
| --- | --- |
| `schema` | 지금 `1` |
| `id` | 안정 식별자 |
| `width` / `height` | 페이지 치수 px |
| `contract` | 이 장이 닫는 불변식 한 줄 |
| `ops[].kind` | 카탈로그 kind |
| `ops[].x,y,w,h` | bbox px |
| `ops[].text` | textRun 계열 문자열 |
| `ops[].gradient` | 그라디언트 채우기 |
| `ops[].image` | TINY_PNG 적재 |
| `expectedKinds` | plane 재정렬 후 kind 순서 |
| `expectedTrace` | TraceBackend `finish` 줄 |

## 기대 추적 형식

```
begin_page 400.00x300.00
  pageBackground bbox=0.00,0.00,400.00,300.00
  rectangle bbox=20.00,20.00,10.00,10.00
end_page ops=2
```

좌표는 항상 소수 2자리다. `f64` 기본 출력의 자릿수 흔들림을 없앤다.

## 상호 diff

같은 장면을 다섯 가족(null/trace/svg/png/skia)으로 재생해도 **추적 로그는 같다**.
다른 `OutputFamily` 끼리 PNG 바이트와 SVG 문자열을 맞대지 않는다.
없는 래스터는 skip 이 아니라, 타입은 있고 `finish` 가 빈 산출물이다.

## 관련

- 계약 표: [RenderBackend 계약 카탈로그](../manual/render_backend_contract_catalog.md)
- 설계 배경: [출력 백엔드 공통 계약](render_backend.md)
