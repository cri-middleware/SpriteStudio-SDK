use notify_debouncer_mini::{
    new_debouncer,
    notify::RecursiveMode,
};
use std::{path::Path, sync::mpsc::channel, time::Duration};
use std::sync::{Arc, atomic::{AtomicBool, Ordering}, mpsc::RecvTimeoutError};

pub fn watch_sspj_project(
    sspj: &Path, 
    config: &crate::converter::converter::ConverterConfig,
    should_stop: Arc<AtomicBool> 
) -> std::result::Result<(), crate::converter::converter_error::ConverterError> {
    if !(sspj.exists() && sspj.extension().unwrap_or(std::ffi::OsStr::new("")) == "sspj") {
        return Err(crate::converter::converter_error::ConverterError::IoError(std::io::Error::new(std::io::ErrorKind::NotFound, "Invalid SSPJ file path")));
    }

    let Some(parent_dir) = sspj.parent() else {
        return Err(crate::converter::converter_error::ConverterError::IoError(std::io::Error::new(std::io::ErrorKind::NotFound, "Invalid SSPJ parent directory")));
    };

    let (tx, rx) = channel();
    let Ok(mut debouncer) = new_debouncer(Duration::from_secs(1), tx) else {
        return Err(crate::converter::converter_error::ConverterError::IoError(std::io::Error::new(std::io::ErrorKind::NotFound, "Failed to create debouncer")));
    };
    let ret = debouncer
        .watcher()
        .watch(parent_dir, RecursiveMode::Recursive);
    if ret.is_err() {
        return Err(crate::converter::converter_error::ConverterError::IoError(std::io::Error::new(std::io::ErrorKind::NotFound, "Failed to watch directory")));
    };

    loop {
        if should_stop.load(Ordering::Relaxed) {
            // println!("Stop requested.");
            break;
        }

        match rx.recv_timeout(Duration::from_millis(1000)) {
            Ok(res) => {
                match res {
                    Ok(events) => {
                        let ret = events.iter().find(|event| is_sspj_or_ssxe_files(&event.path));
                        if let Some(_) = ret {
                            // println!("{:?} changed", event.path);
                            let r = crate::converter::converter::convert_sspj_to_ssab_files(sspj, &config);
                            match r {
                                Ok(_) => println!("convert success"),
                                Err(e) => return Err(e)
                            }
                        }
                    },
                    Err(e) => println!("watch error: {:?}", e),
                }
            },
            Err(RecvTimeoutError::Timeout) => {
                continue;
            },
            Err(RecvTimeoutError::Disconnected) => {
                break;
            }
        }
    }

    Ok(())
}

fn is_sspj_or_ssxe_files(path: &Path) -> bool {
    let ext = path.extension().unwrap_or(std::ffi::OsStr::new(""));
    ext == "sspj" || ext == "ssce" || ext == "ssae" || ext == "ssee" || ext == "ssse"
}
