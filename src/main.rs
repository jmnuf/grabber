use std::process::ExitCode;
use std::env;
use std::path::{Path, PathBuf};

#[derive(Default)]
struct Grabber {
    search_type: GrabberSearchTypes,
    recursive: bool,
    numbered: bool,
    verbose: bool,
}

enum GrabberSearchTypes {
    FileNames,
    FContents,
}
impl Default for GrabberSearchTypes {
    fn default() -> Self { Self::FContents }
}

struct FileSearchResult {
    file_path: String,
    position: (usize, usize), // Line Number + Column Number
    context: String,
}
impl std::fmt::Display for FileSearchResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
	write!(f, "{}:{}:{}: {}", self.file_path, &self.position.0, &self.position.1, self.context)
    }
}

fn main() -> ExitCode {
    let mut args: Vec<String> = env::args().collect();
    let program_name = args.remove(0);
    let current_dir:PathBuf = env::current_dir().expect("Failed to read current directory");

    let mut configs_count = 0;
    let config_flags:Vec<char> = args.iter().take_while(|x| {
	if x.starts_with("-") {
	    configs_count += 1;
	    true
	} else {
	    false
	}
    }).flat_map(|x| {
	let mut ys = vec![];
	let x = &x[1..].to_string();
	for c in x.chars() {
	    ys.push(c);
	}
	ys
    }).collect();
    let mut args:Vec<_> = args.iter().skip(configs_count).collect();

    let mut grabber = Grabber::default();
    let mut verbose = false;
    for arg in config_flags.iter() {
	// TODO: Consider an enum that holds this shtuff
	match arg {
	    'r' => grabber.recursive = true,
	    'n' => grabber.numbered = true,
	    'F' => grabber.search_type = GrabberSearchTypes::FileNames,
	    'C' => grabber.search_type = GrabberSearchTypes::FContents,
	    'h' => {
		Grabber::print_usage();
		return ExitCode::SUCCESS;
	    },
	    'v' => verbose = true,
	    _ => {
		eprintln!("Unknown option: `-{arg}`");
		Grabber::print_usage();
		return ExitCode::FAILURE;
	    }
	}
    }
    grabber.verbose = verbose;

    if grabber.verbose {
	println!("Program: {program_name}");
	println!("Current_Directory: {current_dir:?}");
	for cfg in config_flags.iter() {
	    println!("Cfg: {cfg}");
	}
	println!("Search(peek): {n:?}", n=args.first());
    }
    let search_string = match args.first() {
	None => {
	    eprintln!("Didn't provide a search string! Search string is required");
	    Grabber::print_short_usage();
	    return ExitCode::SUCCESS;
	},
	Some(_) => args.remove(0).trim(),
    };
    if grabber.verbose {
	println!("SearchTerm: {search_string}");
	for arg in args.iter() {
	    println!("GivenDirectory: {arg}");
	}
    }
    let mut paths:Vec<PathBuf> = Vec::new();
    for d in args.iter() {
	let d = if cfg!(target_os="windows") {
	    d.replace("/", "\\")
	} else { d.to_string() };
	let p = Path::new(&d).to_path_buf();
	if !p.is_dir() {
	    eprintln!("Path `{}`: is not a valid directory", p.display());
	    return ExitCode::SUCCESS;
	}
	paths.push(p);
    }
    if paths.is_empty() {
	let p = if cfg!(target_os = "windows") {
	    Path::new(".\\")
	} else {
	    Path::new("./")
	}.to_path_buf();
	paths.push(p);
	if verbose {
	    println!("Default_Path: `./`");
	}
    }

    // TODO: Would like to do searching through Regex
    match grabber.search_type {
	GrabberSearchTypes::FileNames => {
	    let mut successes = 0;
	    for p in paths.into_iter() {
		let result = grabber.search_file_names_in_dir(search_string, &p);
		match result {
		    Ok(results) => {
			successes += 1;
			for x in results.iter() {
			    let x = if cfg!(target_os="windows") {
				x.replace("\\", "/")
			    } else { x.to_string() };
			    println!("{x}");
			}
		    },
		    Err(e) => {
			eprintln!("Failed to read files in directory `{}`: {e}", p.display());
		    },
		};
	    }
	    
	    if successes == 0 {
		ExitCode::FAILURE
	    } else {
		ExitCode::SUCCESS
	    }
	},
	GrabberSearchTypes::FContents => {
	    let mut successes = 0;
	    for p in paths.into_iter() {
		let result = grabber.search_file_contents_in_dir(search_string, &p);
		match result {
		    Ok(results) => {
			successes += 1;
			for x in results.iter() {
			    let x = if cfg!(target_os="windows") {
				x.replace("\\", "/")
			    } else { x.to_string() };
			    println!("{x}");
			}
		    },
		    Err(e) => {
			eprintln!("Failed to read files in directory `{}`: {e}", p.display());
		    },
		};
	    }
	    
	    if successes == 0 {
		ExitCode::FAILURE
	    } else {
		ExitCode::SUCCESS
	    }
	},
    }
}

impl Grabber {
    fn print_short_usage() {
	println!("grabber [-flag1, -flag2, ..] <search-string> [dir1, dir2, ..]");
    }
    fn print_usage() {
	Self::print_short_usage();
	println!("    -h            Display this help message");
	println!("    -v            Show the config settings before searching");
	println!("    -r            Search child directories");
	println!("    -n            Display the line and column number where match was found");
	println!("    -F            Search for match in the file names");
	println!("    -C            Search for match in the file contents");
    }

    fn read_dir(dir: &PathBuf) -> std::io::Result<Vec<PathBuf>> {
	// TODO: One failing DirEntry shouldn't kill the whole dir off, probably
	let mut entries = std::fs::read_dir(dir)?
	    .map(|x| {
		x.map(|y| y.path())
	    })
	    .collect::<std::io::Result<Vec<_>>>()?;
	entries.sort();
	return Ok(entries);
    }

    fn search_file_names_in_dir(&self, search: &str, dir: &PathBuf) -> Result<Vec<String>, std::io::Error> {
	let mut matches:Vec<String> = Vec::new();
	let entries = Self::read_dir(dir)?;
	for path in entries {
	    if path.is_dir() && self.recursive {
		if let Some(file_name) = path.file_name() {
		    // Ignore dot directories
		    let file_name = format!("{}", file_name.to_string_lossy());
		    if file_name.starts_with('.') {
			continue;
		    }
		}
		let sub_results = self.search_file_names_in_dir(search, &path.to_path_buf())?;
		for name in sub_results.into_iter() {
		    matches.push(name);
		}
	    } else if path.is_file() || path.is_symlink() {
		let file_name = format!("{}", path.display());
		if file_name.contains(search) {
		    matches.push(file_name);
		}
	    }
	}
	return Ok(matches);
    }

    fn search_file_contents_in_dir(&self, search: &str, dir: &PathBuf) -> Result<Vec<String>, std::io::Error> {
	use std::io::{BufReader, BufRead};
	use std::fs::File;
	
	let mut matches:Vec<String> = Vec::new();
	let entries = Self::read_dir(dir)?;
	for path in entries {
	    if path.is_dir() && self.recursive {
		if let Some(file_name) = path.file_name() {
		    // Ignore dot directories
		    let file_name = format!("{}", file_name.to_string_lossy());
		    if file_name.starts_with('.') {
			continue;
		    }
		}
		let sub_results = self.search_file_contents_in_dir(search, &path.to_path_buf())?;
		for m in sub_results.into_iter() {
		    matches.push(m);
		}
	    } else if path.is_file() {
		let f  = match File::open(&path) {
		    Ok(f) => f,
		    Err(err) => {
			if self.verbose {
			    eprintln!("Failed to open file {}: `{}`", path.display(), err);
			}
			continue;
		    }
		};
		let br = BufReader::new(f);
		let mut data = FileSearchResult {
		    file_path: format!("{}", path.display()),
		    position: (0, 0), // Line Number + Column Number
		    context: String::new(),
		};
		for (i, l) in br.lines().enumerate() {
		    let l = match l {
			Err(e) => {
			    if self.verbose {
				eprintln!("Failed to read line: {e}");
			    }
			    break;
			},
			Ok(l) => l,
		    };
		    data.position.0 = i + 1;
		    data.context = l.clone();
		    // Is that MFing Jay Z, the american rapper?!
		    if let Some(jay_z) = l.find(search) {
			data.position.1 = jay_z + 1;
			let m = format!("{data}");
			matches.push(m);
		    }
		}
	    }
	}
	return Ok(matches)
    }
}

