use subtype::subtype_path_buf;

subtype_path_buf!(
    pub struct GitRepoDir(PathBuf);
);

impl GitRepoDir {}
