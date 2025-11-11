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
                result.push_str("\t");
            }
            result.push_str(&row.elements.join(" "));
            result.push('\n');
        }
        result
    }

    pub fn optimize(&mut self) {
        // 同じレジスタにpush/popが連続する場合は削除する最適化
        // self.rowsを直接操作するため、逆順で走査する
        let mut i = self.rows.len();
        while i > 1 {
            i -= 1;
            if self.rows[i - 1].elements.len() == 2
                && self.rows[i].elements.len() == 2
                && self.rows[i - 1].elements[0] == "push"
                && self.rows[i].elements[0] == "pop"
                && self.rows[i - 1].elements[1] == self.rows[i].elements[1]
            {
                self.rows.remove(i);
                self.rows.remove(i - 1);
                i -= 1; // 連続している場合を考慮してインデックスを調整
            }
        }
    }
}
