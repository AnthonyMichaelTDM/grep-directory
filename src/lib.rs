use std::error::Error;  //allows for some better errors
use std::fs;            //the library that will allow us to parse files
use std::path::{Path, PathBuf};    //the library that will allow us to get more info about files and directories      

const VALID_OPTIONS: [&str; 9] = [
    "-c", "--case-insensitive",
    //"-f", "--filter",
    "-r", "--recursive",
    "-v", "--verbose",
    "-h", "--help", "help",
];
pub struct Config {
    pub query: String,
    pub path: String,
    pub case_sensitive: bool,
    pub filter: bool,
    pub filter_for: Vec<String>,
    pub recurse: bool,
    pub verbose: bool,
    pub help: bool,
}
impl Config {
    pub fn new(args: &[String]) -> Result<Config, Box<dyn Error>> {
        //DATA
        let mut config: Config = Config { query: String::new(), path: String::new(), case_sensitive: false, filter: false, filter_for: Vec::new(), recurse: false, verbose: false, help: false };
        let options: Vec<String>;
        let path:String;
        let query:String;

        //parse args
        match Config::parse_arguments(&args) {
            Ok( (a,b,c) ) => (options, path, query) = (a,b,c),
            Err(err) => return Err(err),
        }

        //ensure everything is valid
        //throw error if any options aren't valid
        if options.iter().any(|o| !VALID_OPTIONS.contains(&o.as_str())) {
            return Err("One or more invalid options.".into());
        }
        //throw error if path doesn't exist
        if !Path::new(&path).exists() {
            return Err("Invalid path.".into());
        }

        //assign path and query
        config.path = path; 
        config.query = query; 
        //modify config based on options
        options.iter().for_each(|option| {
            match option.as_str() {
                "-c" | "--case-insensitive" => config.case_sensitive = true,
                "-r" | "--recursive" => config.recurse = true,
                "-v" | "--verbose" => config.verbose = true,
                "-h" | "--help" => config.help = true,
                _ => {},
            }
        });

        //return
        Ok(config)
    }

    fn parse_arguments(args: &[String]) -> Result<(Vec<String>,String,String),Box<dyn Error>> { //query, 
        //DATA
        let options: Vec<String> = args.into_iter().filter(|s| s.starts_with("-")).map(|s| s.clone()).collect();
        let mut args_iter = args[0..].into_iter().filter(|s| !s.starts_with("-")).map(|s| s.clone());
        args_iter.next(); //skip first argument
        let path:String = args_iter.next().unwrap_or(String::new()).clone();
        let query:String = args_iter.collect::<String>().clone();

        //error handling
        if path.is_empty() {
            return Err("No/invalid path given".into());
        } 
        if query.is_empty() {
            return Err("No/invalid query given".into());
        }

        //return
        return Ok((options,path,query));
    }
}

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    //DATA
    let paths_to_grep: Vec<PathBuf>;
    let base_path = Path::new(&config.path);

    //if user asked for help, give instructions
    if config.help {
        help();
        return Ok(());
    }

    //fill paths_to_grep based on what config.path points to, and the value of config.recurse
    if !base_path.is_dir() { //it's a file
        paths_to_grep = vec![PathBuf::from(base_path)];
    }
    else if config.recurse { //it's a directory, recurse
        paths_to_grep = list_files_recurse(base_path);
    }
    else { //it's a directory, don't recurse
        paths_to_grep = list_files(base_path);
    }

    //look through all paths_to_grep
    println!("Files containing query: ");
    paths_to_grep.iter().for_each(|path| {
        //DATA
        let contains_query:bool;
        let path_as_string:String = path.to_string_lossy().to_string();
        
        //find out if the file contains the query
        if config.case_sensitive {
            contains_query = search(&config.query, &path).unwrap_or_else(|err| {
                if config.verbose {eprintln!("Error searching {:?}: {}",path_as_string, err);}
                false
            });
        } else {
            contains_query = search_case_insensitive(&config.query, &path).unwrap_or_else(|err| {
                if config.verbose {eprintln!("Error searching {:?}: {}",path_as_string, err);}
                false
            });
        }

        //if it does, print the file name
        if contains_query {
            println!("\t{}",path_as_string);
        }
    });

    return Ok(());
}

pub fn help() {
    println!("                              grep-directory.exe");
    println!("                              By Anthony Rubick\n");
    println!("search through all files in a directory for a given string\n");

    println!("USAGE:\n\tgrep-directory.exe [OPTIONS]... [PATH] \"[QUERY]\"\n");

    println!("OPTIONS:");
    println!("\t-c\t--case-insensitive\t\t\tis query case sensitive (default: yes)");
    //println!("\t-f\t--filter <EXTENSIONS>...\t\tComma separated list of extensions, will only count lines of files with these extensions");
    println!("\t-r,\t--recursive\t\t\t\tSearch through subdirectories");
    println!("\t-v,\t--verbose\t\t\t\tinclude all error messages in output");
    println!("\t-h,\t-help\t\t\t\t\tPrints help information\n");
    
    println!("PATH:\n\tPath to search in, first argument without a '-'\n");
    
    println!("QUERY:\n\tString to search for, all the stuff after the path\n\twrap in \"'s if it contains spaces\n");

}

pub fn search<'a> (query: &'a str, path: &'a Path) -> Result<bool,Box<dyn Error>> {
    //DATA
    let contents:String;
    
    //read file
    match fs::read_to_string(path) {
        Ok(val) => contents = val,
        Err(e) => return Err(e.into()),
    }

    //parse contents for query, case sensitive
    //return true if found, false otherwise
    return Ok(contents.contains(&query));
}

pub fn search_case_insensitive<'a> (query: &'a str, path: &'a Path) -> Result<bool,Box<dyn Error>> {
    //DATA
    let contents:String;
    
    //read file
    match fs::read_to_string(path) {
        Ok(val) => contents = val,
        Err(e) => return Err(e.into()),
    }

    //parse contents for query, case sensitive
    //return true if found, false otherwise
    return Ok(contents.to_ascii_lowercase().contains(&query.to_ascii_lowercase()));
}

/**
 * returns a vector containing paths to all files in path and subdirectories of path
 */
fn list_files_recurse(path: &Path) -> Vec<PathBuf> {
    let mut vec = Vec::new();
    _list_files_recurse(&mut vec,&path);
    vec
}
fn _list_files_recurse(vec: &mut Vec<PathBuf>, path: &Path) {
    if path.is_dir() {
        let paths = fs::read_dir(&path).unwrap();
        for path_result in paths {
            let full_path = path_result.unwrap().path();
            if full_path.is_dir() {
                _list_files_recurse(vec, &full_path);
            } else {
                vec.push(full_path);
            }
        }
    }
}
/**
 * returns a vector containing paths to all files in path, but not subdirectories of path
 */
fn list_files(path: &Path) -> Vec<PathBuf> {
    let mut vec = Vec::new();
    if path.is_dir() {
        let paths = fs::read_dir(&path).unwrap();
        for path_results in paths {
            let full_path = path_results.unwrap().path();
            if !full_path.is_dir() {
                vec.push(full_path);
            }
        }
    }
    return vec;
}
