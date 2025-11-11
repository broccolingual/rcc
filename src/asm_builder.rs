#[derive(Clone, Debug)]
struct Row {
    indent: bool,
    elements: Vec<String>,
}

#[derive(Debug)]
pub struct AsmBuilder {
    rows: Vec<Row>,
}

impl AsmBuilder {
    pub fn new() -> Self {
        AsmBuilder { rows: Vec::new() }
    }

    pub fn add_row(&mut self, row: &str, indent: bool) {
        let elements = row
            .split_whitespace()
            .map(String::from)
            .collect::<Vec<String>>();
        self.rows.push(Row { indent, elements });
    }

    pub fn build(&self) -> String {
        let mut result = String::new();
        for row in &self.rows {
            if row.indent {
                result.push_str("  ");
            }
            result.push_str(&row.elements.join(" "));
            result.push('\n');
        }
        result
    }
}
