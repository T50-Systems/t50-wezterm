/// Our own stream implementation.
/// This is present because we cannot guarantee to be able to convert
/// Path -> c-string on Windows systems, but also because we've seen
/// mysterious errors about not being able to open a resource.
/// The intent is to avoid a potential problem and to help reveal
/// more context on problems opening files as/when that happens.
struct FreeTypeStream {
    stream: FT_StreamRec_,
    backing: StreamBacking,
    name: String,
}

#[allow(dead_code)]
enum StreamBacking {
    File(BufReader<File>),
    Map(Mmap),
    Static(&'static [u8]),
    Memory(Arc<Box<[u8]>>),
}

impl FreeTypeStream {
    pub fn from_source(source: &FontDataSource) -> anyhow::Result<FT_Stream> {
        let (backing, base, len) = match source {
            FontDataSource::OnDisk(path) => return Self::open_path(path),
            FontDataSource::BuiltIn { data, .. } => {
                let base = data.as_ptr();
                let len = data.len();
                (StreamBacking::Static(data), base, len)
            }
            FontDataSource::Memory { data, .. } => {
                let base = data.as_ptr();
                let len = data.len();
                (StreamBacking::Memory(Arc::clone(data)), base, len)
            }
        };

        let name = source.name_or_path_str().to_string();

        if len > c_ulong::MAX as usize {
            anyhow::bail!("{} is too large to pass to freetype! (len={})", name, len);
        }

        let stream = Box::new(Self {
            stream: FT_StreamRec_ {
                base: base as *mut _,
                size: len as c_ulong,
                pos: 0,
                descriptor: FT_StreamDesc_ {
                    pointer: ptr::null_mut(),
                },
                pathname: FT_StreamDesc_ {
                    pointer: ptr::null_mut(),
                },
                read: None,
                close: Some(Self::close),
                memory: ptr::null_mut(),
                cursor: ptr::null_mut(),
                limit: ptr::null_mut(),
            },
            backing,
            name,
        });
        let stream = Box::into_raw(stream);
        unsafe {
            (*stream).stream.descriptor.pointer = stream as *mut _;
            Ok(&mut (*stream).stream)
        }
    }

    fn open_path(p: &Path) -> anyhow::Result<FT_Stream> {
        let file = File::open(p).with_context(|| format!("opening file {}", p.display()))?;

        let meta = file
            .metadata()
            .with_context(|| format!("querying metadata for {}", p.display()))?;

        if !meta.is_file() {
            anyhow::bail!("{} is not a file", p.display());
        }

        let len = meta.len();
        if len as usize > c_ulong::MAX as usize {
            anyhow::bail!(
                "{} is too large to pass to freetype! (len={})",
                p.display(),
                len
            );
        }

        let (backing, base) = match unsafe { MmapOptions::new().map(&file) } {
            Ok(map) => {
                let base = map.as_ptr() as *mut _;
                (StreamBacking::Map(map), base)
            }
            Err(err) => {
                log::warn!(
                    "Unable to memory map {}: {}, will use regular file IO instead",
                    p.display(),
                    err
                );
                (StreamBacking::File(BufReader::new(file)), ptr::null_mut())
            }
        };

        let stream = Box::new(Self {
            stream: FT_StreamRec_ {
                base,
                size: len as c_ulong,
                pos: 0,
                descriptor: FT_StreamDesc_ {
                    pointer: ptr::null_mut(),
                },
                pathname: FT_StreamDesc_ {
                    pointer: ptr::null_mut(),
                },
                read: if base.is_null() {
                    Some(Self::read)
                } else {
                    // when backing is mmap, a null read routine causes
                    // freetype to simply resolve data from `base`
                    None
                },
                close: Some(Self::close),
                memory: ptr::null_mut(),
                cursor: ptr::null_mut(),
                limit: ptr::null_mut(),
            },
            backing,
            name: p.to_string_lossy().to_string(),
        });
        let stream = Box::into_raw(stream);
        unsafe {
            (*stream).stream.descriptor.pointer = stream as *mut _;
            Ok(&mut (*stream).stream)
        }
    }

    /// Called by freetype when it wants to read data from the file
    unsafe extern "C" fn read(
        stream: FT_Stream,
        offset: c_ulong,
        buffer: *mut c_uchar,
        count: c_ulong,
    ) -> c_ulong {
        if count == 0 {
            return 0;
        }

        let myself = &mut *((*stream).descriptor.pointer as *mut Self);
        match &mut myself.backing {
            StreamBacking::Map(_) | StreamBacking::Static(_) | StreamBacking::Memory(_) => {
                log::error!("read called on memory data {} !?", myself.name);
                0
            }
            StreamBacking::File(file) => {
                if let Err(err) = file.seek(SeekFrom::Start(offset.into())) {
                    log::error!(
                        "failed to seek {} to offset {}: {:#}",
                        myself.name,
                        offset,
                        err
                    );
                    return 0;
                }

                let buf = std::slice::from_raw_parts_mut(buffer, count as usize);
                match file.read(buf) {
                    Ok(len) => len as c_ulong,
                    Err(err) => {
                        log::error!(
                            "failed to read {} bytes @ offset {} of {}: {:#}",
                            count,
                            offset,
                            myself.name,
                            err
                        );
                        0
                    }
                }
            }
        }
    }

    /// Called by freetype when the stream is closed
    unsafe extern "C" fn close(stream: FT_Stream) {
        let myself = Box::from_raw((*stream).descriptor.pointer as *mut Self);
        drop(myself);
    }
}
