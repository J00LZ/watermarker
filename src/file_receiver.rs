use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
    path::PathBuf,
    sync::mpsc::{channel, Receiver, TryRecvError},
    thread,
};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum FileReceiverSource {
    SourceImages,
    Watermark,
    DestinationFolder,
}

impl Display for FileReceiverSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FileReceiverSource::SourceImages => write!(f, "Source images"),
            FileReceiverSource::Watermark => write!(f, "Watermark"),
            FileReceiverSource::DestinationFolder => write!(f, "Destination folder"),
        }
    }
}

impl FileReceiverSource {
    pub fn extensions(&self) -> &[(&str, &[&str])] {
        match self {
            FileReceiverSource::SourceImages | FileReceiverSource::Watermark => {
                &[("Image", &["jpg", "jpeg", "png", "bmp"])]
            }
            FileReceiverSource::DestinationFolder => &[],
        }
    }
}

#[derive(Debug, Default)]
pub struct FileReceievers {
    receivers: HashMap<FileReceiverSource, FileReceiver>,
    received: HashSet<FileReceiverSource>,
}

impl FileReceievers {
    pub fn new_receiver(&mut self, source: FileReceiverSource) {
        println!("Adding new receiver: {:?}", source);
        self.received.remove(&source);
        let extensions = source.extensions();
        self.receivers
            .insert(source, FileReceiver::recv_or_save(extensions, source));
    }

    pub fn get_receiver(&self, source: FileReceiverSource) -> Option<&FileReceiver> {
        self.receivers.get(&source)
    }

    pub fn receive_all(&mut self) {
        for (p, receiver) in self.receivers.iter_mut() {
            if self.received.contains(p) {
                continue;
            }
            if !matches!(receiver.try_recv(), FileReceiverResult::Waiting) {
                self.received.insert(*p);
            }
        }
    }
}

#[derive(Debug)]
pub struct FileReceiver {
    receiver: Receiver<Vec<PathBuf>>,
    file: Option<Vec<PathBuf>>,
    has_received: bool,
}

pub enum FileReceiverResult<'p> {
    File(&'p [PathBuf]),
    NoFile,
    Waiting,
}

impl FileReceiver {
    fn recv_or_save(extensions: &[(&str, &[&str])], source: FileReceiverSource) -> Self {
        let (s, receiver) = channel();
        let mut dialog = rfd::FileDialog::new();
        for (name, exts) in extensions {
            dialog = dialog.add_filter(*name, exts);
        }
        thread::spawn(move || {
            let r = match source {
                FileReceiverSource::SourceImages => dialog.pick_files(),
                FileReceiverSource::Watermark => dialog.pick_file().map(|f| vec![f]),
                FileReceiverSource::DestinationFolder => dialog.pick_folder().map(|f| vec![f]),
            };
            if let Some(res) = r {
                s.send(res).unwrap();
            }
        });
        Self {
            receiver,
            file: None,
            has_received: false,
        }
    }

    pub fn get_file(&self) -> FileReceiverResult {
        if self.has_received {
            if let Some(f) = &self.file {
                FileReceiverResult::File(f)
            } else {
                FileReceiverResult::NoFile
            }
        } else {
            FileReceiverResult::Waiting
        }
    }

    fn try_recv(&mut self) -> FileReceiverResult {
        if self.has_received {
            return if let Some(f) = &self.file {
                FileReceiverResult::File(f)
            } else {
                FileReceiverResult::NoFile
            };
        }

        match self.receiver.try_recv() {
            Ok(p) => {
                self.has_received = true;
                self.file = Some(p);
                self.get_file()
            }
            Err(TryRecvError::Empty) => FileReceiverResult::Waiting,
            Err(TryRecvError::Disconnected) => {
                self.has_received = true;
                FileReceiverResult::NoFile
            }
        }
    }
}
