use std::time::SystemTime;
use std::path::{Path, PathBuf};
use std::fs;
use std::io::{self, Read, Cursor};

use tiny_http::Response;

pub fn format_system_time_to_string(st: SystemTime) -> String {
    let duration_since_epoch = st.duration_since(SystemTime::UNIX_EPOCH).unwrap();
    let secs_since_epoch = duration_since_epoch.as_secs();

    let year = 1970 + (secs_since_epoch / (60 * 60 * 24 * 365));
    let month = 1 + ((secs_since_epoch / (60 * 60 * 24 * 30)) % 12);
    let day = 1 + ((secs_since_epoch / (60 * 60 * 24)) % 30);  // Simplified, months are treated as if all had 30 days
    let hour = (secs_since_epoch / (60 * 60)) % 24;
    let min = (secs_since_epoch / 60) % 60;
    let sec = secs_since_epoch % 60;

    format!("{:04}-{:02}-{:02}T{:02}:{:02}:{:02}", year, month, day, hour, min, sec)
}

pub fn files_sorted_by_date<P: AsRef<Path>>(path: P) -> io::Result<Vec<(PathBuf, SystemTime)>> {
    let mut entries: Vec<_> = fs::read_dir(path)?
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.is_file() {
                Some((path, entry.metadata().ok()?.modified().unwrap_or(SystemTime::UNIX_EPOCH)))
            } else {
                None
            }
        })
        .collect();

    // Sort entries based on their last modified time
    entries.sort_by_key(|&(_, time)| time.clone());

    Ok(entries)
}

pub fn yield_video_file(post_content: String) -> Response<Cursor<Vec<u8>>> {
    // validate that path in post_content exists
    let path = Path::new(&post_content);
    if !path.exists() {
        return Response::from_string("Path does not exist").with_status_code(400);
    }

    // validate that path is .mkv video file
    let extension = path.extension().unwrap();
    if extension != "mkv" {
        return Response::from_string("Path is not a .mkv video file").with_status_code(400);
    }

    let mut file = fs::File::open(path).unwrap();
    let mut contents = Vec::new();
    file.read_to_end(&mut contents).unwrap();
    let header = tiny_http::Header::from_bytes(&b"Content-Type"[..], &b"video/x-matroska"[..]).unwrap();

    return Response::from_data(contents).with_header(header).with_status_code(200);

}