use super::super::{bytes_to_hex, code_section::CodeSection, data_section::DataSection, elf_file::ElfFile};
use super::super::super::elf_section::NoteSection;
use super::super::*;
use super::super::super::elf_section::*;

pub trait ToMarkdown {
    fn to_markdown(&self) -> String;
}

impl ToMarkdown for DataSection {
    fn to_markdown(&self) -> String {
        let mut r = format!("## Data Section\n\n");
        r.push_str(& format!("```bin\n{}\n```", self.to_hex()));
        r.push_str("\n");
        r
    }
    
}

impl ToMarkdown for Vec<DataSection> {
    fn to_markdown(&self) -> String {
        let mut r = format!("## Data Section\n\n");
        for (i, section) in self.iter().enumerate() {
            r.push_str(& format!("```bin\n{}\n```", section.to_hex()));
            if i < self.len() - 1 {
                r.push_str("\n");
            }
        }
        r.push_str("\n");
        r
    }
}

impl ToMarkdown for CodeSection {
    fn to_markdown(&self) -> String {
        let mut r = format!("## Text Section\n\n");
        r.push_str(& format!("```bin\n{}\n```", self.to_hex()));
        r.push_str("\n");
        r
    }
}

impl ToMarkdown for Vec<CodeSection> {
    fn to_markdown(&self) -> String {
        let mut r = format!("## Text Section\n\n");
        for (i, section) in self.iter().enumerate() {
            r.push_str(& format!("```bin\n{}\n```", section.to_hex()));
            if i < self.len() - 1 {
                r.push_str("\n\n");
            }
        }
        r.push_str("\n");
        r
    }
}

impl ToMarkdown for Elf64Header {
    fn to_markdown(&self) -> String {
        let mut r = format!("## ELF Header\n\n");
        r.push_str(& format!("```bin\n{}\n```", self.to_hex()));
        r.push_str("\n");
        r
    }
}

impl ToMarkdown for ProgramHeader {
    fn to_markdown(&self) -> String {
        let mut r = format!("## Program Header\n\n");
        r.push_str(& format!("```bin\n{}\n```", self.to_hex()));
        r.push_str("\n");
        r
    }
}

impl ToMarkdown for Vec<ProgramHeader> {
    fn to_markdown(&self) -> String {
        let mut r = format!("## Program Headers\n\n");
        for (i, header) in self.iter().enumerate() {
            r.push_str(& format!("```bin\n{}\n```", header.to_hex()));
            if i < self.len() - 1 {
                r.push_str("\n\n");
            }
        }

        r.push_str("\n");
        r
    }
}

impl ToMarkdown for NoteSection {
    fn to_markdown(&self) -> String {
        let mut r = format!("## Note Section\n\n");
        r.push_str(& format!("```bin\n{}\n```", self.to_hex()));
        r.push_str("\n");
        r
    }
}

impl ToMarkdown for Vec<u8> {
    fn to_markdown(&self) -> String {
        let mut r = String::new();
        r.push_str(& format!("```bin\n{}\n```", bytes_to_hex(self)));
        r.push_str("\n");
        r
    }
}


impl ToMarkdown for ElfFile {
    fn to_markdown(&self) -> String {
        let mut r = format!("# ELF file\n");
        r.push_str("\n");

        //serialize elf header
        r.push_str(& self.header.to_markdown());
        r.push_str("\n");

        // serialize program header
        r.push_str(& self.program_headers.to_markdown());
        r.push_str("\n");

        r.push_str("## Segments\n\n");
        // serialize sections
        for section in &self.segments {
            r.push_str(& section.to_markdown());
            r.push_str("\n");
        }

        r
    }
}