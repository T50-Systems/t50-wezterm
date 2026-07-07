impl Face {
    pub fn family_name(&self) -> String {
        unsafe {
            if (*self.face).family_name.is_null() {
                "".to_string()
            } else {
                let c = CStr::from_ptr((*self.face).family_name);
                c.to_string_lossy().to_string()
            }
        }
    }

    pub fn style_name(&self) -> String {
        unsafe {
            if (*self.face).style_name.is_null() {
                "".to_string()
            } else {
                let c = CStr::from_ptr((*self.face).style_name);
                c.to_string_lossy().to_string()
            }
        }
    }

    pub fn postscript_name(&self) -> String {
        unsafe {
            let c = FT_Get_Postscript_Name(self.face);
            if c.is_null() {
                "".to_string()
            } else {
                let c = CStr::from_ptr(c);
                c.to_string_lossy().to_string()
            }
        }
    }

    pub fn variations(&self) -> anyhow::Result<Vec<ParsedFont>> {
        let mut mm = std::ptr::null_mut();

        unsafe {
            ft_result(FT_Get_MM_Var(self.face, &mut mm), ()).context("FT_Get_MM_Var")?;

            let mut res = vec![];
            let num_styles = (*mm).num_namedstyles;
            for i in 1..=num_styles {
                FT_Set_Named_Instance(self.face, i);
                let source = FontDataHandle {
                    source: self.source.source.clone(),
                    index: self.source.index,
                    variation: i,
                    origin: self.source.origin.clone(),
                    coverage: self.source.coverage.clone(),
                };
                res.push(ParsedFont::from_face(&self, source)?);
            }

            FT_Done_MM_Var(self.lib, mm);
            FT_Set_Named_Instance(self.face, 0);

            log::debug!("Variations: {:#?}", res);

            Ok(res)
        }
    }

    pub fn get_glyph_name(&self, glyph_index: u32) -> Option<String> {
        let mut buf = [0u8; 128];
        let res = unsafe {
            FT_Get_Glyph_Name(
                self.face,
                glyph_index,
                buf.as_mut_ptr() as *mut _,
                buf.len() as _,
            )
        };
        if res != 0 {
            None
        } else {
            Some(String::from_utf8_lossy(&buf).into_owned())
        }
    }

    pub fn get_sfnt_names(&self) -> HashMap<u32, Vec<NameRecord>> {
        let num_names = unsafe { FT_Get_Sfnt_Name_Count(self.face) };

        let mut names = HashMap::new();

        let mut sfnt_name = FT_SfntName {
            platform_id: 0,
            encoding_id: 0,
            language_id: 0,
            name_id: 0,
            string: std::ptr::null_mut(),
            string_len: 0,
        };

        for i in 0..num_names {
            if unsafe { FT_Get_Sfnt_Name(self.face, i, &mut sfnt_name) } != 0 {
                continue;
            }

            if sfnt_name.string.is_null() {
                continue;
            }

            if !matches!(
                sfnt_name.name_id as u32,
                TT_NAME_ID_TYPOGRAPHIC_FAMILY
                    | TT_NAME_ID_TYPOGRAPHIC_SUBFAMILY
                    | TT_NAME_ID_FONT_FAMILY
                    | TT_NAME_ID_FONT_SUBFAMILY
                    | TT_NAME_ID_PS_NAME
            ) {
                continue;
            }

            let bytes = unsafe {
                from_raw_parts(sfnt_name.string as *const u8, sfnt_name.string_len as usize)
            };

            let encoding = match (sfnt_name.platform_id as u32, sfnt_name.encoding_id as u32) {
                (TT_PLATFORM_MACINTOSH, TT_MAC_ID_JAPANESE)
                | (TT_PLATFORM_MICROSOFT, TT_MS_ID_SJIS) => encoding_rs::SHIFT_JIS,
                (TT_PLATFORM_MACINTOSH, TT_MAC_ID_SIMPLIFIED_CHINESE)
                | (TT_PLATFORM_MICROSOFT, TT_MS_ID_PRC) => encoding_rs::GBK,
                (TT_PLATFORM_MACINTOSH, TT_MAC_ID_TRADITIONAL_CHINESE)
                | (TT_PLATFORM_MICROSOFT, TT_MS_ID_BIG_5) => encoding_rs::BIG5,
                (
                    TT_PLATFORM_MICROSOFT,
                    TT_MS_ID_UCS_4 | TT_MS_ID_UNICODE_CS | TT_MS_ID_SYMBOL_CS,
                ) => encoding_rs::UTF_16BE,
                (TT_PLATFORM_MICROSOFT, TT_MS_ID_WANSUNG) => encoding_rs::EUC_KR,
                (TT_PLATFORM_APPLE_UNICODE | TT_PLATFORM_ISO, _) => encoding_rs::UTF_16BE,
                (TT_PLATFORM_MACINTOSH, TT_MAC_ID_ROMAN) => encoding_rs::MACINTOSH,
                _ => {
                    log::trace!(
                        "Skipping name_id={} because platform_id={} encoding_id={}",
                        sfnt_name.name_id,
                        sfnt_name.platform_id,
                        sfnt_name.encoding_id
                    );
                    continue;
                }
            };

            let (name, _) = encoding.decode_with_bom_removal(bytes);

            names
                .entry(sfnt_name.name_id as u32)
                .or_insert_with(Vec::new)
                .push(NameRecord {
                    platform_id: sfnt_name.platform_id,
                    encoding_id: sfnt_name.encoding_id,
                    name_id: sfnt_name.name_id,
                    language_id: sfnt_name.language_id,
                    name: name.to_string(),
                });
        }
        names
    }

    pub fn get_os2_table(&self) -> Option<&TT_OS2> {
        unsafe {
            let os2: *const TT_OS2 = FT_Get_Sfnt_Table(self.face, FT_Sfnt_Tag::FT_SFNT_OS2) as _;
            if os2.is_null() {
                None
            } else {
                Some(&*os2)
            }
        }
    }

    /// Returns the cap_height/units_per_EM ratio if known
    pub fn cap_height(&self) -> Option<f64> {
        unsafe {
            let os2 = self.get_os2_table()?;
            let units_per_em = (*self.face).units_per_EM;
            if units_per_em == 0 || os2.sCapHeight == 0 {
                return None;
            }
            Some(os2.sCapHeight as f64 / units_per_em as f64)
        }
    }

    pub fn weight_and_width(&self) -> (u16, u16) {
        let (mut weight, mut width) = self
            .get_os2_table()
            .map(|os2| (os2.usWeightClass as f64, os2.usWidthClass as f64))
            .unwrap_or((400., 5.));

        unsafe {
            let index = (*self.face).face_index;
            let variation = index >> 16;
            if variation > 0 {
                let vidx = (variation - 1) as usize;

                let mut mm = std::ptr::null_mut();

                ft_result(FT_Get_MM_Var(self.face, &mut mm), ())
                    .context("FT_Get_MM_Var")
                    .unwrap();
                {
                    let mm = &*mm;

                    let styles = from_raw_parts(mm.namedstyle, mm.num_namedstyles as usize);
                    let instance = &styles[vidx];
                    let axes = from_raw_parts(mm.axis, mm.num_axis as usize);

                    for (i, axis) in axes.iter().enumerate() {
                        let coords = from_raw_parts(instance.coords, mm.num_axis as usize);
                        let value = coords[i].to_num::<f64>();
                        let default_value = axis.def.to_num::<f64>();
                        let scale = if default_value != 0. {
                            value / default_value
                        } else {
                            1.
                        };

                        if axis.tag == ft_make_tag(b'w', b'g', b'h', b't') {
                            weight = weight * scale;
                        }

                        if axis.tag == ft_make_tag(b'w', b'd', b't', b'h') {
                            width = width * scale;
                        }
                    }
                }

                FT_Done_MM_Var(self.lib, mm);
            }
        }

        (weight.round() as u16, width.round() as u16)
    }

    pub fn italic(&self) -> bool {
        unsafe { ((*self.face).style_flags & FT_STYLE_FLAG_ITALIC as FT_Long) != 0 }
    }

    pub fn compute_coverage(&self) -> RangeSet<u32> {
        if let Some(coverage) = self.source.coverage.as_ref() {
            return coverage.clone();
        }
        let mut coverage = RangeSet::new();

        for encoding in &[
            FT_Encoding::FT_ENCODING_UNICODE,
            FT_Encoding::FT_ENCODING_MS_SYMBOL,
        ] {
            if unsafe { FT_Select_Charmap(self.face, *encoding) } != 0 {
                continue;
            }

            let mut glyph = 0;
            let mut ucs4 = unsafe { FT_Get_First_Char(self.face, &mut glyph) };
            if glyph == 0 {
                break;
            }
            let mut ucs4_range_start = ucs4;
            loop {
                let ucs4_new = unsafe { FT_Get_Next_Char(self.face, ucs4, &mut glyph) };
                if glyph == 0 {
                    break;
                }
                if ucs4_new - ucs4 != 1 {
                    coverage.add_range_unchecked(ucs4_range_start as u32..(ucs4 + 1) as u32);
                    ucs4_range_start = ucs4_new;
                }
                ucs4 = ucs4_new;
            }
            coverage.add_range_unchecked(ucs4_range_start as u32..(ucs4 + 1) as u32);

            if *encoding == FT_Encoding::FT_ENCODING_MS_SYMBOL {
                // Fontconfig duplicates F000..F0FF to 0000..00FF
                for ucs4 in 0xf00..0xf100 {
                    if coverage.contains(ucs4) {
                        coverage.add(ucs4 as u32 - 0xf000);
                    }
                }
            }
        }
        coverage
    }

    /// Returns the bitmap strike sizes in this font
    pub fn pixel_sizes(&self) -> Vec<u16> {
        let sizes = unsafe {
            let rec = &(*self.face);
            from_raw_parts(rec.available_sizes, rec.num_fixed_sizes as usize)
        };
        sizes
            .iter()
            .filter_map(|info| {
                if info.height > 0 {
                    Some(info.height as u16)
                } else {
                    None
                }
            })
            .collect()
    }
}
