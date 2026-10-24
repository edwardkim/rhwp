//! 브라우저의 Unicode 문자·조판 계약을 보존하는 SVG 부분 글꼴.

use std::collections::HashSet;

use read_fonts::{
    collections::IntSet,
    types::{GlyphId, Tag},
    FontRef, TableProvider,
};

pub(super) fn subset(data: &[u8], chars: &HashSet<char>) -> Result<Vec<u8>, String> {
    let original = ttf_parser::Face::parse(data, 0).map_err(|error| error.to_string())?;
    let font = FontRef::new(data).map_err(|error| error.to_string())?;
    if !original.is_subsetting_allowed() {
        return Err("원 프로그램의 부분집합 금지 표시는 전체 face로 보존함".into());
    }
    // skera의 현재 지원 경계다. CFF/CFF2의 glyph ID를 바꾸면서 원본 윤곽선을
    // 그대로 붙이거나 필요한 bitmap을 제거하지 않고, 호출자가 전체 face를 공급한다.
    if font.glyf().is_err() {
        return Err("TrueType 윤곽선이 아닌 face는 전체 프로그램을 보존함".into());
    }
    // 아직 재작성하지 못하는 glyph 참조 테이블은 삭제하거나 옛 glyph ID로
    // 복사하지 않는다. 특히 legacy kern과 수식·SVG 글리프의 원 조판을 보존한다.
    for tag in [
        b"kern", b"kerx", b"mort", b"morx", b"MATH", b"SVG ", b"Feat", b"Glat", b"Gloc", b"Silf",
        b"Sill", b"trak", b"just",
    ] {
        if font.data_for_tag(Tag::new(tag)).is_some() {
            return Err(
                "부분집합으로 재작성하지 못하는 조판 테이블은 전체 프로그램을 보존함".into(),
            );
        }
    }
    for ch in chars {
        if let Some(glyph) = original.glyph_index(*ch) {
            if original.glyph_bounding_box(glyph).is_none()
                && original.glyph_raster_image(glyph, u16::MAX).is_some()
            {
                return Err("사용하는 bitmap 글리프는 전체 프로그램을 보존함".into());
            }
        }
    }
    if !chars.iter().any(|ch| original.glyph_index(*ch).is_some()) {
        return Err("요청 문자에 대응하는 glyph가 없는 face".into());
    }
    // VS는 독립 cmap glyph가 없어도 format14 closure의 입력이다. 원 요청의
    // Unicode를 전부 전달하며 실제 일반/변형 glyph 선택은 subsetter가 해소한다.
    let unicodes: IntSet<u32> = chars.iter().map(|ch| u32::from(*ch)).collect();
    // IntSet은 입력 HashSet의 순서와 무관한 문자·glyph 순서를 제공한다.
    // GSUB closure·GPOS·힌팅·이름을 보존해 실제 웹 조판이 원 프로그램을 소비한다.
    let plan = skera::Plan::new(
        &IntSet::<GlyphId>::empty(),
        &unicodes,
        &font,
        skera::SubsetFlags::SUBSET_FLAGS_NOTDEF_OUTLINE
            | skera::SubsetFlags::SUBSET_FLAGS_GLYPH_NAMES,
        &skera::DEFAULT_DROP_TABLES.iter().copied().collect(),
        &IntSet::all(),
        &IntSet::all(),
        &IntSet::all(),
        &IntSet::all(),
    );
    let subset = skera::subset_font(&font, &plan).map_err(|error| error.to_string())?;
    let selected = ttf_parser::Face::parse(&subset, 0).map_err(|error| error.to_string())?;
    if selected.units_per_em() != original.units_per_em() {
        return Err("부분 글꼴의 em 변경".into());
    }
    for ch in chars {
        if let Some(source) = original.glyph_index(*ch) {
            let target = selected.glyph_index(*ch).ok_or_else(|| {
                format!("부분 글꼴의 Unicode 매핑 누락: U+{:04X}", u32::from(*ch))
            })?;
            if original.glyph_hor_advance(source) != selected.glyph_hor_advance(target) {
                return Err("부분 글꼴의 원본 전진폭 변경".into());
            }
        }
    }
    Ok(subset)
}
