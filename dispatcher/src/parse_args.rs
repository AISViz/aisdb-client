use std::path::Path;

pub struct AppArgs {
    pub dbpath: std::path::PathBuf,
    pub files: Vec<std::path::PathBuf>,
    pub rawdata_dir: Option<std::path::PathBuf>,
    pub source: String,
    //pub start: usize,
    //pub end: usize,
}

pub fn parse_path(s: &std::ffi::OsStr) -> Result<std::path::PathBuf, &'static str> {
    Ok(s.into())
}

/// collect --dbpath and --rawdata_dir args from command line
pub fn parse_args() -> Result<AppArgs, pico_args::Error> {
    let mut pargs = pico_args::Arguments::from_env();

    if pargs.contains(["-h", "--help"]) || pargs.clone().finish().is_empty() {
        print!("{}", HELP);
        std::process::exit(0);
    }

    let args = AppArgs {
        dbpath: pargs
            .opt_value_from_str("--dbpath")
            .unwrap()
            .unwrap_or_else(|| Path::new(":memory:").to_path_buf()),
        rawdata_dir: pargs
            .opt_value_from_os_str("--rawdata_dir", parse_path)
            .unwrap(),
        files: pargs.values_from_os_str("--file", parse_path).unwrap(),
        source: pargs.value_from_str("--source").unwrap(),
        /*
        start: pargs
            .opt_value_from_fn("--start", str::parse)
            .unwrap()
            .unwrap_or(0),
        end: pargs
            .opt_value_from_fn("--end", str::parse)
            .unwrap()
            .unwrap_or(usize::MAX),
        */
    };

    let remaining = pargs.finish();
    if !remaining.is_empty() {
        eprintln!("unused args {:?}", remaining);
    }
    Ok(args)
}
