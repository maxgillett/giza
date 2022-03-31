use winterfell::ProverError;

#[derive(Debug)]
pub enum ExecutionError {
    ProverError(ProverError),
}
