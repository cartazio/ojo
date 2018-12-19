use diff::LineDiff;

use crate::storage::File;
use crate::{NodeId, PatchId};

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(transparent)]
pub struct Changes {
    pub changes: Vec<Change>,
}

// While iterating through the diff, we need to remember what the previous line was and where it
// came from: either there wasn't one, or it came from one of the two files.
enum LastLine<'a> {
    Start,
    File1(&'a NodeId),
    File2(&'a NodeId),
}

impl<'a> LastLine<'a> {
    fn either(&self) -> Option<&NodeId> {
        match *self {
            LastLine::File1(i) => Some(i),
            LastLine::File2(i) => Some(i),
            LastLine::Start => None,
        }
    }
}

impl Changes {
    pub fn from_diff(file1: &File, file2: &File, diff: &[LineDiff]) -> Changes {
        let mut changes = Vec::new();
        let mut last = LastLine::Start;
        for d in diff {
            match *d {
                LineDiff::New(i) => {
                    let id = file2.node_id(i);
                    changes.push(Change::NewNode {
                        id: id.clone(),
                        contents: file2.line(i).to_owned(),
                    });

                    // We are adding a new line, so we need to connect it to whatever line came
                    // before it, no matter where it came from.
                    if let Some(last_id) = last.either() {
                        changes.push(Change::NewEdge {
                            src: last_id.clone(),
                            dst: id.clone(),
                        });
                    }
                    last = LastLine::File2(id);
                }
                LineDiff::Keep(i, _) => {
                    let id = file1.node_id(i);

                    // If the last line came from the new file, we need to hook it up to this line.
                    if let LastLine::File2(last_id) = last {
                        changes.push(Change::NewEdge {
                            src: last_id.clone(),
                            dst: id.clone(),
                        });
                    }
                    last = LastLine::File1(id);
                }
                LineDiff::Delete(i) => {
                    let id = file1.node_id(i);
                    changes.push(Change::DeleteNode { id: id.clone() });
                }
            }
        }
        Changes { changes }
    }

    pub fn set_patch_id(&mut self, new_id: &PatchId) {
        for ch in &mut self.changes {
            ch.set_patch_id(new_id);
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum Change {
    NewNode { id: NodeId, contents: Vec<u8> },
    DeleteNode { id: NodeId },
    NewEdge { src: NodeId, dst: NodeId },
}

impl Change {
    fn set_patch_id(&mut self, new_id: &PatchId) {
        match *self {
            Change::NewNode { ref mut id, .. } => {
                id.set_patch_id(new_id);
            }
            Change::NewEdge {
                ref mut src,
                ref mut dst,
            } => {
                src.set_patch_id(new_id);
                dst.set_patch_id(new_id);
            }
            Change::DeleteNode { ref mut id } => {
                id.set_patch_id(new_id);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Change::*;
    use super::Changes;
    use crate::storage::File;
    use crate::NodeId;
    use diff::LineDiff::*;

    #[test]
    fn from_diff_empty_first() {
        let file1 = File::from_bytes(b"");
        let file2 = File::from_bytes(b"something");
        let diff = vec![New(0)];

        let expected = vec![NewNode {
            id: NodeId::cur(0),
            contents: b"something".to_vec(),
        }];
        assert_eq!(Changes::from_diff(&file1, &file2, &diff).changes, expected);
    }
}
