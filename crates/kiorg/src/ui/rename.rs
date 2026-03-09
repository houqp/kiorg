/// State for an in-progress inline rename.
pub struct Rename {
    pub original_index: usize,
    pub original_name: String,
    pub new_name: String,
}
