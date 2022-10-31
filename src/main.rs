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

    //Search in content in files
    #[clap(short, default_value_t = false)]
    contentsearch: bool,

    //Search in path names in directories
    #[clap(short, default_value_t = false)]
    pathnamesearch: bool,

    //Seach in filenames.
    #[clap(short, default_value_t = false)]
    filenamesearch: bool,
}

fn main() {
    let args = Config::parse();
    let dir = args.directory;
    let query = args.query;
    let mut contentsearch = args.contentsearch;
    let mut pathnamesearch = args.pathnamesearch;
    let mut filenamesearch = args.filenamesearch;

    let path = Path::new(&dir);
    if !path.exists() {
        println!("Folder is missing");
        exit(1);
    }

    if !contentsearch && !pathnamesearch && !filenamesearch {
        contentsearch = true;
        pathnamesearch = true;
        filenamesearch = true;
    }

    // println!("Search for: {}", query);
    // println!("In directory: {:?}", path);
    //println!("{}", str::repeat("x", 15));

    let filetypes = vec!["rs", "txt", "sql"];
    let mut table = Table::new();

    // table
    //     .set_header(vec!["Header1", "Header2", "Header3"])
    //     .add_row(vec![
    //         "This is a text",
    //         "This is another text",
    //         "This is the third text",
    //     ])
    //     .add_row(vec![
    //         "This is another text",
    //         "Now\nadd some\nmulti line stuff",
    //         "This is awesome",
    //     ]);

    let mut dirpathresult: Vec<String> = Vec::new();

    for object in WalkDir::new(path)
    //.into_iter()
    //.filter_entry(move |e| files.includes_entry(e))
    //.filter_map(|e| e.ok())
    //.filter(|e| e.file_type().is_file())
    //.filter(|e| e.file_type().is_dir())
    {
        let file = object.as_ref().unwrap().path();

        if file.is_dir() && pathnamesearch {
            let mut dirpathbuf = String::new();
            let dirpath: Vec<_> = file
                .components()
                .map(|comp| comp.as_os_str().to_string_lossy())
                .collect();
            for component in &dirpath {
                if component.contains(&query) {
                    dirpathbuf.push_str(&component);
                    if !dirpathresult.contains(&dirpathbuf) {
                        dirpathresult.push(dirpathbuf.clone());
                        // println!("file: {:?} - dirpathbuf: {}",file.as_os_str().to_str(),dirpathbuf);
                        table.add_row(vec![
                            "Directory",
                            &component,
                            file.as_os_str().to_str().unwrap(),
                        ]);
                    }
                    //break;
                }
            }
            //println!("dirpathbuf: {}", dirpathbuf)
        }

        if file.is_file() {
            let filename = file.file_name().unwrap().to_string_lossy();
            if filename.contains(&query) && filenamesearch {
                //println!("filename: {:?}", filename);
                table.add_row(vec![
                    "Filename",
                    &filename,
                    file.as_os_str().to_str().unwrap(),
                ]);
            }

            if filetypes
                .iter()
                .any(|&x| x == file.extension().unwrap_or(OsStr::new("").as_ref()))
                && contentsearch
            {
                //println!("{}", object.unwrap().path().display());
                //println!("{:?}", file);
                //let content = fs::read_to_string(file).unwrap();
                let mut contents = fs::File::open(&file).unwrap();
                let mut buf = vec![];
                contents.read_to_end(&mut buf).unwrap();
                let content = String::from_utf8_lossy(&buf);
                let mut rownumber = 0;
                for row in content.lines() {
                    rownumber += 1;
                    if row.contains(&query) {
                        // println!(
                        //     "{:?}, LineNo:{}, {}",
                        //     file.file_name()
                        //         .unwrap_or(OsStr::new("?"))
                        //         .to_string_lossy(),
                        //     &rownumber,
                        //     row
                        // );
                        let mut matchandlineno = String::new();
                        matchandlineno.push_str(
                            file.file_name()
                                .unwrap_or(OsStr::new("?"))
                                .to_str()
                                .unwrap(),
                        );
                        matchandlineno.push_str(", LineNo: ");
                        matchandlineno.push_str(&rownumber.to_string());
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

    //println!("{:?}", table.row(0));
    if table.row(0).is_none() {
        println!("No match.");
    }

    if table.row(0).is_some() {
        table.set_header(vec!["Type", "Match", "Path"]);
        println!("{table}");
    }
}
