use clap::Parser;
use comfy_table::Table;
use std::{env, ffi::OsStr, fs, io::Read, path::Path, process::exit};
use walkdir::WalkDir;

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
struct Config {
    ///What to search for.
    #[clap(short, long)]
    query: String,

    ///Where to search. (default: current directory)
    #[clap(short, long, default_value_t = env::current_dir().unwrap().to_string_lossy().to_string())]
    directory: String,

    ///Search in content in files
    #[clap(short, default_value_t = false)]
    contentsearch: bool,

    ///Search in path names in directories
    #[clap(short, default_value_t = false)]
    pathnamesearch: bool,

    ///Seach in file names.
    #[clap(short, default_value_t = false)]
    filenamesearch: bool,
}

fn main() {
    // Use clap to parse input arguments and assign argument to variables.
    let args = Config::parse();
    let dir = args.directory;
    let query = args.query;
    let mut contentsearch = args.contentsearch;
    let mut pathnamesearch = args.pathnamesearch;
    let mut filenamesearch = args.filenamesearch;

    // Make sure the entered directory to search in acurally exists.
    let path = Path::new(&dir);
    if !path.exists() {
        println!("Folder is missing");
        exit(1);
    }

    // If none of the flaggs are set, enable all flaggs.
    if !contentsearch && !pathnamesearch && !filenamesearch {
        contentsearch = true;
        pathnamesearch = true;
        filenamesearch = true;
    }

    // Determin what type of files to preform the search in.
    // let filetypes = vec!["rs", "txt", "sql"];
    //let filetypesban = vec!["bin", "o", "pdb", "rmeta", "rlib", "exe", "d"];
    //let banned_filetypes: Vec<str> = fs::File::open("banned_filetypes.txt");
    let binding = fs::read_to_string("banned_filetypes.txt").unwrap();
    let filetypesban: Vec<&str> = binding.split("\r\n").collect();

    // Use comfy_table to present result in a neat looking table.
    let mut table = Table::new();
    table.set_header(vec!["Type", "Match", "Path"]);

    // Vector to collect matching result and to prevent duplicates.
    let mut dirpathresult: Vec<String> = Vec::new();

    // One way to only get folders or files to iterate over instead of everything.
    /*
    for object in WalkDir::new(path)
    .into_iter()
    .filter_entry(move |e| files.includes_entry(e))
    .filter_map(|e| e.ok())
    .filter(|e| e.file_type().is_file()) // Gives only files.
    .filter(|e| e.file_type().is_dir()) // Gives only folders.
    */

    // Use WalkDir to genreate main object loop to iterate through.
    for object in WalkDir::new(path) {
        let file = object.as_ref().unwrap().path();

        // Search all parts of a path if we are looking for a directory.
        if pathnamesearch && file.is_dir() {
            // Use an empty string to save a unique kind of path-key which will be used to identify duplicates.
            let mut dirpathbuf = String::new();

            // Create a collection of string slices from the full path. Ex: "C:\Windows" -> ["C:", "\", "Windows"]
            // Check each component from the path if it matches our query.
            let dirpath: Vec<_> = file
                .components()
                .map(|comp| comp.as_os_str().to_string_lossy())
                .collect();
            for component in &dirpath {
                if component.contains(&query) {
                    // Save the path component to a string.
                    dirpathbuf.push_str(&component);

                    // When the combination of path components does not exists in the result vector
                    // add the path combination to the vector and save the directory information to result table.
                    if !dirpathresult.contains(&dirpathbuf) {
                        dirpathresult.push(dirpathbuf.clone());
                        table.add_row(vec![
                            "Directory",
                            &component,
                            file.as_os_str().to_str().unwrap(),
                        ]);
                    }
                }
            }
        }

        if file.is_file() {
            let filename = file.file_name().unwrap().to_string_lossy();

            // When a match is found in the filename of a file, add the files information to result table.
            if filenamesearch && filename.contains(&query) {
                table.add_row(vec![
                    "Filename",
                    &filename,
                    file.as_os_str().to_str().unwrap(),
                ]);
            }

            // Use a vector with banned filetypes to filter away undesired matches in content of files.
            if contentsearch
                && !filetypesban
                    .iter()
                    .any(|&x| x == file.extension().unwrap_or(OsStr::new("").as_ref()))
            {
                // Prevusly used the function "read_to_string()" but resulted in error the file used UTF-16 encoding.
                //let content = fs::read_to_string(file).unwrap();
                // Below code is used to convert from UTF-16 to UTF-8 with risk of lost charcters.
                let mut contents = fs::File::open(&file).unwrap();
                let mut buf = vec![];
                contents.read_to_end(&mut buf).unwrap();
                let content = String::from_utf8_lossy(&buf);

                // Need a row number to tell from which line in the file a match was found.
                let mut rownumber = 0;

                for row in content.lines() {
                    rownumber += 1;
                    if row.contains(&query) {
                        // Use variable to concatenate and format matched file and linenumber for result table.
                        let mut matchandlineno = String::new();
                        matchandlineno.push_str(
                            file.file_name()
                                .unwrap_or(OsStr::new("?"))
                                .to_str()
                                .unwrap(),
                        );
                        matchandlineno.push_str(", LineNo: ");
                        matchandlineno.push_str(&rownumber.to_string());

                        // Save matched file information to result table.
                        table.add_row(vec![
                            "Content",
                            matchandlineno.as_str(),
                            file.as_os_str().to_str().unwrap(),
                        ]);
                    }
                }
            }
        }
    }

    // No match have been identified and result table is empty (with exception of header row on index 0).
    if table.row(1).is_none() {
        println!("No match.");
    }

    // Match have been found. Index 0 is header row.
    if table.row(1).is_some() {
        println!("{table}");
    }
}
