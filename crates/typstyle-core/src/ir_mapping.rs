use std::collections::HashMap;

use crate::ast_mapping::SpanMapping;

#[derive(Debug, Clone)]
pub struct IrAtomSource {
    pub atom_id: u32,
    pub src_start: usize,
    pub src_end: usize,
}

#[derive(Debug, Clone)]
pub struct IrAtomOutput {
    pub atom_id: u32,
    pub out_start: usize,
    pub out_end: usize,
}

#[derive(Debug, Default)]
pub struct IrAtomRecorder {
    next_atom_id: u32,
    src_atoms: Vec<IrAtomSource>,
}

impl IrAtomRecorder {
    pub fn reset(&mut self) {
        self.next_atom_id = 0;
        self.src_atoms.clear();
    }

    pub fn alloc_atom(&mut self, src_start: usize, src_end: usize) -> u32 {
        let atom_id = self.next_atom_id;
        self.next_atom_id = self.next_atom_id.saturating_add(1);
        self.src_atoms.push(IrAtomSource {
            atom_id,
            src_start,
            src_end,
        });
        atom_id
    }

    pub fn snapshot_sources(&self) -> Vec<IrAtomSource> {
        self.src_atoms.clone()
    }
}

pub fn join_ir_mappings(
    src_atoms: &[IrAtomSource],
    out_atoms: &[IrAtomOutput],
) -> Vec<SpanMapping> {
    let src_by_id: HashMap<u32, (&usize, &usize)> = src_atoms
        .iter()
        .map(|s| (s.atom_id, (&s.src_start, &s.src_end)))
        .collect();

    let mut mapping: Vec<SpanMapping> = out_atoms
        .iter()
        .filter_map(|out| {
            let (src_start, src_end) = src_by_id.get(&out.atom_id)?;
            Some(SpanMapping {
                src_start: **src_start,
                src_end: **src_end,
                out_start: out.out_start,
                out_end: out.out_end,
            })
        })
        .collect();

    mapping.sort_by(|a, b| {
        a.src_start
            .cmp(&b.src_start)
            .then(a.src_end.cmp(&b.src_end))
            .then(a.out_start.cmp(&b.out_start))
            .then(a.out_end.cmp(&b.out_end))
    });

    mapping
}

pub fn remap_offsets_for_indent_change(
    text: &str,
    from_indent: usize,
    to_indent: usize,
    ranges: &mut [IrAtomOutput],
) -> String {
    let converted = crate::utils::change_indent(text, from_indent, to_indent);
    let map = build_indent_offset_map(text, from_indent, to_indent);
    let max = map.len().saturating_sub(1);

    for r in ranges.iter_mut() {
        r.out_start = map[r.out_start.min(max)];
        r.out_end = map[r.out_end.min(max)];
    }

    converted
}

fn build_indent_offset_map(text: &str, from_indent: usize, to_indent: usize) -> Vec<usize> {
    if text.is_empty() || from_indent == 0 || from_indent == to_indent {
        return (0..=text.len()).collect();
    }

    let mut map = vec![0usize; text.len() + 1];
    let mut old_pos = 0usize;
    let mut new_pos = 0usize;
    let mut first = true;

    for line in text.lines() {
        if !first {
            // `lines()` splits on `\n`; preserve newline 1:1.
            map[old_pos] = new_pos;
            old_pos += 1;
            new_pos += 1;
        }
        first = false;

        let trimmed = line.trim_start();
        if trimmed.is_empty() {
            for i in 0..line.len() {
                map[old_pos + i] = new_pos;
            }
            old_pos += line.len();
            continue;
        }

        let leading_spaces = line.len() - trimmed.len();
        let indent_level = leading_spaces / from_indent;
        let new_leading = indent_level * to_indent;

        for i in 0..leading_spaces {
            map[old_pos + i] = if i < new_leading {
                new_pos + i
            } else {
                new_pos + new_leading
            };
        }
        old_pos += leading_spaces;
        new_pos += new_leading;

        for i in 0..trimmed.len() {
            map[old_pos + i] = new_pos + i;
        }
        old_pos += trimmed.len();
        new_pos += trimmed.len();
    }

    for item in map.iter_mut().take(text.len() + 1).skip(old_pos) {
        *item = new_pos;
    }

    map
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn join_keeps_one_to_many_matches() {
        let src_atoms = vec![IrAtomSource {
            atom_id: 7,
            src_start: 10,
            src_end: 14,
        }];
        let out_atoms = vec![
            IrAtomOutput {
                atom_id: 7,
                out_start: 100,
                out_end: 104,
            },
            IrAtomOutput {
                atom_id: 7,
                out_start: 200,
                out_end: 204,
            },
        ];

        let mapping = join_ir_mappings(&src_atoms, &out_atoms);
        assert_eq!(mapping.len(), 2);
        assert_eq!(mapping[0].src_start, 10);
        assert_eq!(mapping[0].src_end, 14);
        assert_eq!(mapping[0].out_start, 100);
        assert_eq!(mapping[0].out_end, 104);
        assert_eq!(mapping[1].src_start, 10);
        assert_eq!(mapping[1].src_end, 14);
        assert_eq!(mapping[1].out_start, 200);
        assert_eq!(mapping[1].out_end, 204);
    }

    #[test]
    fn remap_offsets_handles_unicode_text_after_indent_change() {
        let ir_4 = "Root [\n    \"你好\",\n]";
        let token = "\"你好\"";
        let start_4 = ir_4.find(token).expect("token in 4-space IR");
        let mut out_atoms = vec![IrAtomOutput {
            atom_id: 1,
            out_start: start_4,
            out_end: start_4 + token.len(),
        }];

        let ir_2 = remap_offsets_for_indent_change(ir_4, 4, 2, &mut out_atoms);
        let start_2 = ir_2.find(token).expect("token in 2-space IR");

        assert_eq!(out_atoms[0].out_start, start_2);
        assert_eq!(out_atoms[0].out_end, start_2 + token.len());
        assert_eq!(&ir_2[out_atoms[0].out_start..out_atoms[0].out_end], token);
    }
}
