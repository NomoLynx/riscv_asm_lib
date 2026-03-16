pub trait ToMarkdownTableRow {
    fn get_markdown_header(&self) -> String;
    fn to_markdown(&self) -> String;
}
