#[derive(Default)]
pub(crate) struct XmlWriter {
    xml: String,
}

impl XmlWriter {
    pub(crate) fn declaration(&mut self) {
        self.xml
            .push_str("<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"no\"?>");
    }

    pub(crate) fn open(&mut self, name: &str, attributes: &[(&str, String)]) {
        self.xml.push('<');
        self.xml.push_str(name);
        self.attributes(attributes);
        self.xml.push('>');
    }

    pub(crate) fn empty(&mut self, name: &str, attributes: &[(&str, String)]) {
        self.xml.push('<');
        self.xml.push_str(name);
        self.attributes(attributes);
        self.xml.push_str("/>");
    }

    pub(crate) fn close(&mut self, name: &str) {
        self.xml.push_str("</");
        self.xml.push_str(name);
        self.xml.push('>');
    }

    pub(crate) fn text(&mut self, value: &str) {
        push_escaped(&mut self.xml, value, false);
    }

    pub(crate) fn raw(&mut self, value: &str) {
        self.xml.push_str(value);
    }

    pub(crate) fn finish(self) -> Vec<u8> {
        self.xml.into_bytes()
    }

    fn attributes(&mut self, attributes: &[(&str, String)]) {
        for (name, value) in attributes {
            self.xml.push(' ');
            self.xml.push_str(name);
            self.xml.push_str("=\"");
            push_escaped(&mut self.xml, value, true);
            self.xml.push('"');
        }
    }
}

fn push_escaped(output: &mut String, value: &str, attribute: bool) {
    for character in value.chars() {
        match character {
            '&' => output.push_str("&amp;"),
            '<' => output.push_str("&lt;"),
            '>' => output.push_str("&gt;"),
            '"' if attribute => output.push_str("&quot;"),
            '\'' if attribute => output.push_str("&apos;"),
            _ => output.push(character),
        }
    }
}
