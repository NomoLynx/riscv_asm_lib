use super::super::label_offset::*;

pub trait ToMarkdown {
    /// Convert the object to a Markdown string representation.
    fn to_markdown(&self) -> String;
}

impl ToMarkdown for LabelOffsetTableEntry {
    fn to_markdown(&self) -> String {
        format!("| {} | 0x{:X} | {} | {} |", 
                self.get_label().get_real_name(), 
                self.get_offset(), 
                self.get_sequence_number(), 
                self.get_related_label().get_real_name())
    }
}

impl ToMarkdown for LabelTable<'_> {
    fn to_markdown(&self) -> String {
        let mut markdown = String::new();
        markdown.push_str("| Label | Offset | Sequence Number | Related Label |\n");
        markdown.push_str("|-------|--------|------------------|----------------|\n");
        
        for entry in self.entries() {
            markdown.push_str(& entry.to_markdown());
            markdown.push('\n');
        }
        
        markdown
    }
}