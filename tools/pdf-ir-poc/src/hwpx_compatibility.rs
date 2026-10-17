//! POC-only adapter for generated zero-margin text boxes. No product serializer changes.
use quick_xml::{
    events::{BytesStart, Event},
    Reader, Writer,
};
use std::io::{Cursor, Read, Write};
use zip::{write::SimpleFileOptions, System, ZipArchive, ZipWriter};

pub fn explicit_zero_text_margins(bytes: &[u8]) -> Result<(Vec<u8>, usize), String> {
    let mut source = ZipArchive::new(Cursor::new(bytes)).map_err(|e| e.to_string())?;
    let mut target = ZipWriter::new(Cursor::new(Vec::new()));
    let mut total = 0;
    for i in 0..source.len() {
        let mut file = source.by_index(i).map_err(|e| e.to_string())?;
        let name = file.name().map_err(|e| e.to_string())?.into_owned();
        if name.starts_with("Contents/section") && name.ends_with(".xml") {
            let mut xml = Vec::new();
            file.read_to_end(&mut xml).map_err(|e| e.to_string())?;
            let (xml, count) = rewrite_section(&xml)?;
            total += count;
            let options = SimpleFileOptions::default()
                .compression_method(file.compression())
                .last_modified_time(file.last_modified().unwrap_or_default())
                .system(System::Unix);
            target
                .start_file(&name, options)
                .map_err(|e| e.to_string())?;
            target.write_all(&xml).map_err(|e| e.to_string())?;
        } else {
            target.raw_copy_file(file).map_err(|e| e.to_string())?;
        }
    }
    Ok((
        target.finish().map_err(|e| e.to_string())?.into_inner(),
        total,
    ))
}

fn rewrite_section(xml: &[u8]) -> Result<(Vec<u8>, usize), String> {
    let mut reader = Reader::from_reader(xml);
    let mut writer = Writer::new(Vec::new());
    let mut margins = Vec::new();
    let mut total = 0;
    loop {
        let event = reader.read_event().map_err(|e| e.to_string())?;
        match &event {
            Event::Start(tag) if tag.local_name().as_ref() == "drawText" => margins.push(false),
            Event::Start(tag) | Event::Empty(tag) if tag.local_name().as_ref() == "textMargin" => {
                if let Some(present) = margins.last_mut() {
                    *present = true;
                }
            }
            Event::End(tag) if tag.local_name().as_ref() == "drawText" => {
                if !margins.pop().ok_or("invalid drawText nesting")? {
                    let mut margin = BytesStart::new("hp:textMargin");
                    for key in ["left", "right", "top", "bottom"] {
                        margin.push_attribute((key, "0"));
                    }
                    writer
                        .write_event(Event::Empty(margin))
                        .map_err(|e| e.to_string())?;
                    total += 1;
                }
            }
            Event::DocType(_) => return Err("unexpected DTD in generated section".into()),
            Event::Eof => break,
            _ => {}
        }
        writer.write_event(event).map_err(|e| e.to_string())?;
    }
    if !margins.is_empty() {
        return Err("unterminated drawText".into());
    }
    Ok((writer.into_inner(), total))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_margin_is_explicit_and_existing_margin_is_preserved() {
        let input=br#"<s xmlns:hp="test"><hp:drawText><hp:subList/><hp:textMargin left="283" right="283" top="283" bottom="283"/></hp:drawText><hp:drawText><hp:subList/></hp:drawText></s>"#;
        let (output, count) = rewrite_section(input).unwrap();
        assert_eq!(count, 1);
        let mut reader = Reader::from_reader(output.as_slice());
        let mut left = Vec::new();
        loop {
            match reader.read_event().unwrap() {
                Event::Empty(tag) if tag.local_name().as_ref() == "textMargin" => {
                    let a = tag.try_get_attribute("left").unwrap().unwrap();
                    left.push(a.value.into_owned());
                }
                Event::Eof => break,
                _ => {}
            }
        }
        assert_eq!(left, vec!["283".to_string(), "0".to_string()]);
        let (again, count) = rewrite_section(&output).unwrap();
        assert_eq!(count, 0);
        assert_eq!(again, output);
    }

    #[test]
    fn package_order_compression_resources_and_text_are_preserved() {
        let mut zip = ZipWriter::new(Cursor::new(Vec::new()));
        let opts = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
        zip.start_file("mimetype", opts).unwrap();
        zip.write_all(b"application/hwp+zip").unwrap();
        zip.start_file("BinData/image.png", opts).unwrap();
        zip.write_all(&[0, 1, 2, 3]).unwrap();
        zip.start_file("Contents/section0.xml", opts).unwrap();
        zip.write_all(br#"<s xmlns:hp="test"><hp:drawText><hp:subList><hp:t>A &amp; B</hp:t></hp:subList></hp:drawText></s>"#).unwrap();
        let input = zip.finish().unwrap().into_inner();
        let (output, count) = explicit_zero_text_margins(&input).unwrap();
        assert_eq!(count, 1);
        let mut loaded = ZipArchive::new(Cursor::new(output)).unwrap();
        assert_eq!(loaded.by_index(0).unwrap().name().unwrap(), "mimetype");
        assert_eq!(
            loaded.by_index(0).unwrap().compression(),
            zip::CompressionMethod::Stored
        );
        let mut data = Vec::new();
        loaded
            .by_name("BinData/image.png")
            .unwrap()
            .read_to_end(&mut data)
            .unwrap();
        assert_eq!(data, vec![0, 1, 2, 3]);
        let mut xml = String::new();
        loaded
            .by_name("Contents/section0.xml")
            .unwrap()
            .read_to_string(&mut xml)
            .unwrap();
        assert!(xml.contains("A &amp; B"));
    }
}
