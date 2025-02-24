use std::{
    collections::BTreeSet,
    ffi::OsStr,
    fs::File,
    io::{BufReader, Cursor, Read, Write},
    path::PathBuf,
};

use anyhow::{Context, Result};
use bzip2::read::BzDecoder;
use clap::Parser;
use convlog::tenhou_xml_to_mjai;
use flate2::{write::GzEncoder, Compression};
use glob::glob;
use indicatif::{ParallelProgressIterator, ProgressBar, ProgressStyle};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use serde_json as json;
use sqlite::{self, State};

#[derive(Debug, Parser)]
struct Arguments {
    pub sqlite_db: PathBuf,
    pub out_folder: PathBuf,
    #[arg(long)]
    pub skip: Option<i64>,
    #[arg(long)]
    pub overwrite: bool,
    #[arg(long)]
    pub log_id_file: Option<PathBuf>,
}
fn main() -> Result<()> {
    let Arguments {
        sqlite_db,
        out_folder,
        skip,
        mut overwrite,
        log_id_file,
    } = Arguments::parse();

    let offset = skip.unwrap_or(0);

    let connection = sqlite::open(sqlite_db)?;

    let mut xml_logs: Vec<(String, Vec<u8>)> = Vec::new();

    if let Some(filename) = log_id_file {
        overwrite = true;

        let file = File::open(filename)?;
        let mut buf_reader = BufReader::new(file);
        let mut contents = String::new();
        buf_reader.read_to_string(&mut contents)?;

        let log_ids_to_read = contents.split("\n");

        for log_id in log_ids_to_read {
            let query = "SELECT log_id, log_content FROM logs WHERE log_id = :id";
            let mut statement = connection.prepare(query)?;
            statement.bind((":id", log_id))?;
            statement.next()?;
            xml_logs.push((statement.read("log_id")?, statement.read("log_content")?));
        }
    } else {
        let query = "SELECT log_id, log_content FROM logs LIMIT -1 OFFSET ?";
        let mut statement = connection.prepare(query)?;
        statement.bind((1, offset))?;

        print!("Reading from DB...");
        while let Ok(State::Row) = statement.next() {
            xml_logs.push((statement.read("log_id")?, statement.read("log_content")?));
        }
        println!(" DB read.");
    }

    let total_count = xml_logs.len();

    if !overwrite {
        let files_done_glob = glob(&format!("{}/*.json.gz", out_folder.display()))?.map(|path| {
            path.unwrap()
                .file_name()
                .and_then(OsStr::to_str)
                .map(|name| name.replace(".json.gz", ""))
        });
        let files_done = BTreeSet::from_iter(files_done_glob);

        xml_logs.retain(|(log_id, _)| !files_done.contains(&Some(log_id.to_owned())));

        println!("Skipping {} converted logs.", total_count - xml_logs.len());
    }

    let bar_style = ProgressStyle::with_template(
        "[{elapsed_precise}/{eta_precise}] {wide_bar} {pos:>7}/{len:7} [{per_sec}/s]",
    )?;
    let bar = ProgressBar::new(xml_logs.len() as u64).with_style(bar_style);

    xml_logs
        .par_iter()
        .progress_with(bar)
        .try_for_each(|(log_id, log_content_bz)| {
            convert_log(log_id, log_content_bz, out_folder.clone())
        })?;

    println!("Converted {} logs.", xml_logs.len());

    Ok(())
}

fn convert_log(log_id: &String, log_content_bz: &Vec<u8>, out_folder: PathBuf) -> Result<()> {
    let log_content_bz: Cursor<Vec<u8>> = Cursor::new(log_content_bz.to_owned());
    let mut decompressor = BzDecoder::new(log_content_bz);
    let mut log_content = String::new();
    decompressor.read_to_string(&mut log_content)?;

    let events = tenhou_xml_to_mjai(&log_content).with_context(|| {
        format!(
            "failed to parse tenhou.net/0 log {}:\n {}",
            log_id, &log_content
        )
    })?;

    let mut w = Vec::new();

    for event in &events {
        let to_write = json::to_string(event).context("failed to serialize")?;
        writeln!(w, "{to_write}").with_context(|| format!("failed to write to vector"))?;
    }

    let mjai_out = out_folder.join(format!("{}.json.gz", log_id));
    let file_w: Box<dyn Write> = {
        let mjai_out_file = File::create(&mjai_out)
            .with_context(|| format!("failed to create mjai out file {:}", mjai_out.display()))?;
        Box::from(mjai_out_file)
    };

    let mut gz_enc = GzEncoder::new(file_w, Compression::default());
    gz_enc
        .write_all(&w)
        .with_context(|| format!("failed to write to mjai out file {}", mjai_out.display()))?;
    gz_enc
        .finish()
        .with_context(|| format!("failed to finish"))?;

    Ok(())
}
