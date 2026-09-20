//! Comandos de la CLI.
//!
//! Cada comando vive en su propio módulo y expone una función `run`
//! con una firma específica según el comando.

pub mod index;
pub mod init;
pub mod search;
pub mod stats;
