mod raw {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

lazy_static! {
    static ref BINDING_VERSION_STRING: String = match (raw::GIT_COMMIT_HASH, raw::GIT_DIRTY) {
        (Some(hash), Some(dirty)) => format!("MMTk PyPy {} ({}{})", raw::PKG_VERSION, hash.split_at(7).0, if dirty { "-dirty" } else { "" }),
        (Some(hash), None) => format!("MMTk PyPy {} ({}{})", raw::PKG_VERSION, hash.split_at(7).0, "-?"),
        _ => format!("MMTk PyPy {}", raw::PKG_VERSION),
    };
    static ref MMTK_PYPY_FULL_VERSION_STRING: String = format!("{}, using {}", *BINDING_VERSION_STRING, *mmtk::build_info::MMTK_FULL_BUILD_INFO);

    pub static ref MMTK_PYPY_FULL_VERSION: &'static str = &MMTK_PYPY_FULL_VERSION_STRING;
}
