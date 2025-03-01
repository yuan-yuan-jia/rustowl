use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub enum Error {
    SyntaxError,
    UnknownError,
    LocalDeclareNotFound,
    LocalIsNotUserVariable,
}

/// location in source code
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
#[serde(transparent)]
pub struct Loc(u32);
impl Loc {
    pub fn new(source: &str, byte_pos: u32, offset: u32) -> Self {
        let byte_pos = if byte_pos < offset {
            0
        } else {
            byte_pos - offset
        };
        for (i, (byte, _)) in source.char_indices().enumerate() {
            if byte_pos <= byte as u32 {
                return Self(i as u32);
            }
        }
        Self(0)
    }
}

/// represents range in source code
#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub struct Range {
    pub from: Loc,
    pub until: Loc,
}
impl Range {
    pub fn new(from: Loc, until: Loc) -> Self {
        Self { from, until }
    }
    /*
    pub fn from_source_info(body: &Body<'_>, source_info: SourceInfo) -> Self {
        let scope = Range::from(body.source_scopes.get(source_info.scope).unwrap().span);
        let wide = Range::from(source_info.span);
        Range::new(
            Loc::from(wide.from - scope.from),
            Loc::from(wide.until - scope.from),
        )
    }
    */
}

/// variable in MIR
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum MirVariable {
    User {
        index: usize,
        live: Range,
        dead: Range,
    },
    Other {
        index: usize,
        live: Range,
        dead: Range,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(transparent)]
pub struct MirVariables(HashMap<usize, MirVariable>);
impl MirVariables {
    pub fn new() -> Self {
        Self(HashMap::new())
    }
    pub fn push(&mut self, var: MirVariable) {
        match &var {
            MirVariable::User { index, .. } => {
                if self.0.get(index).is_none() {
                    self.0.insert(*index, var);
                }
            }
            MirVariable::Other { index, .. } => {
                if self.0.get(index).is_none() {
                    self.0.insert(*index, var);
                }
            }
        }
    }
    pub fn to_vec(self) -> Vec<MirVariable> {
        self.0.into_values().collect()
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum Item {
    Function { span: Range, mir: Function },
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct File {
    pub items: Vec<Function>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(transparent)]
pub struct Workspace(pub HashMap<String, File>);
impl Workspace {
    pub fn merge(mut self, other: Self) -> Self {
        let Workspace(files) = other;
        for (file, mir) in files {
            if let Some(insert) = self.0.get_mut(&file) {
                insert.items.extend_from_slice(&mir.items);
            } else {
                self.0.insert(file, mir);
            }
        }
        self
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum MirRval {
    Move {
        target_local_index: usize,
        range: Range,
    },
    Borrow {
        target_local_index: usize,
        range: Range,
        mutable: bool,
        outlive: Option<Range>,
    },
}

/// statement in MIR
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum MirStatement {
    StorageLive {
        target_local_index: usize,
        range: Range,
    },
    StorageDead {
        target_local_index: usize,
        range: Range,
    },
    Assign {
        target_local_index: usize,
        range: Range,
        rval: Option<MirRval>,
    },
}
/// terminator in MIR
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum MirTerminator {
    Drop {
        local_index: usize,
        range: Range,
    },
    Call {
        destination_local_index: usize,
        fn_span: Range,
    },
    Other,
}
/// basic block in MIR
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MirBasicBlock {
    pub statements: Vec<MirStatement>,
    pub terminator: Option<MirTerminator>,
}

/// declared variable in MIR
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum MirDecl {
    User {
        local_index: usize,
        name: String,
        span: Range,
        ty: String,
        lives: Vec<Range>,
        drop: bool,
        drop_range: Vec<Range>,
        must_live_at: Vec<Range>,
    },
    Other {
        local_index: usize,
        ty: String,
        lives: Vec<Range>,
        drop: bool,
        drop_range: Vec<Range>,
        must_live_at: Vec<Range>,
    },
}
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Function {
    pub basic_blocks: Vec<MirBasicBlock>,
    pub decls: Vec<MirDecl>,
}
