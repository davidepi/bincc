#![allow(clippy::upper_case_acronyms)] // I hate this >:(
#![allow(clippy::comparison_chain)]
// this is ugly and completely unreadable
//EDIT 2021/05/28 this lint would've prevented me a bug :( ----^
#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod analysis;
pub mod disasm;
