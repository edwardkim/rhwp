// SVG test reader, not a layout oracle. Existing assertions retain their own
// independent expectations. A cropped image is painted inside a nested SVG
// viewport; its intrinsic width/height are source pixels, not page geometry.
pub fn image_frames(svg: &str) -> Vec<[f64; 4]> {
    let xml = roxmltree::Document::parse(svg).expect("valid rendered SVG");
    xml.descendants()
        .filter(|n| n.has_tag_name("image"))
        .filter_map(|image| {
            let viewport = image.ancestors().skip(1).find(|n| {
                n.has_tag_name("svg")
                    && n.attribute("viewBox").is_some()
                    && ["x", "y", "width", "height"]
                        .iter()
                        .all(|key| n.attribute(*key).is_some())
            });
            let frame = viewport.unwrap_or(image);
            let attr = |key| frame.attribute(key)?.parse::<f64>().ok();
            Some([attr("x")?, attr("y")?, attr("width")?, attr("height")?])
        })
        .collect()
}
